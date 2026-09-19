use crate::api::auth::{map_rspotify_error, with_auto_reauth};
use crate::error::AppError;
use rspotify::model::{ArtistId, Market};
use rspotify::prelude::Id;
use rspotify::{AuthCodePkceSpotify, clients::BaseClient};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtistTopTrack {
    pub id: String,
    pub title: String,
    pub album: String,
    pub duration_ms: u32,
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtistAlbum {
    pub id: String,
    pub name: String,
    pub image_url: Option<String>,
    pub release_date: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtistDetail {
    pub id: String,
    pub name: String,
    pub image_url: Option<String>,
    pub genres: Vec<String>,
    pub followers: u32,
    pub top_tracks: Vec<ArtistTopTrack>,
    pub albums: Vec<ArtistAlbum>,
}

/// Fetches detailed artist profile, top tracks, and discography (`/artists/{id}`).
#[allow(clippy::missing_errors_doc, deprecated)]
pub async fn fetch_artist_details(
    spotify: &AuthCodePkceSpotify,
    artist_id_str: &str,
) -> Result<ArtistDetail, AppError> {
    let aid = ArtistId::from_id(artist_id_str)
        .map_err(|e| AppError::Network(format!("Invalid artist ID '{artist_id_str}': {e}")))?;

    with_auto_reauth(spotify, || async {
        let full_artist = spotify
            .artist(aid.clone())
            .await
            .map_err(map_rspotify_error)?;

        let top_tracks_raw = spotify
            .artist_top_tracks(aid.clone(), Some(Market::FromToken))
            .await
            .map_err(map_rspotify_error)?;

        let mut top_tracks = Vec::new();
        for t in top_tracks_raw {
            let track_id = t.id.as_ref().map_or_else(String::new, ToString::to_string);
            let uri = t.id.as_ref().map_or_else(String::new, Id::uri);

            top_tracks.push(ArtistTopTrack {
                id: track_id,
                title: t.name,
                album: t.album.name,
                duration_ms: u32::try_from(t.duration.num_milliseconds()).unwrap_or(0),
                uri,
            });
        }

        let albums_page = spotify
            .artist_albums_manual(aid.clone(), None, None, Some(20), Some(0))
            .await
            .map_err(map_rspotify_error)?;

        let mut albums = Vec::new();
        for a in albums_page.items {
            let album_id = a.id.as_ref().map_or_else(String::new, ToString::to_string);
            let image_url = a.images.first().map(|img| img.url.clone());
            let release_date = a.release_date.unwrap_or_default();

            albums.push(ArtistAlbum {
                id: album_id,
                name: a.name,
                image_url,
                release_date,
            });
        }

        let image_url = full_artist.images.first().map(|img| img.url.clone());
        let followers = full_artist.followers.total;

        Ok(ArtistDetail {
            id: artist_id_str.to_string(),
            name: full_artist.name,
            image_url,
            genres: full_artist.genres,
            followers,
            top_tracks,
            albums,
        })
    })
    .await
}

/// Follows an artist for the authenticated user.
#[allow(clippy::missing_errors_doc)]
#[allow(deprecated)]
pub async fn follow_artist(spotify: &AuthCodePkceSpotify, artist_id: &str) -> Result<(), AppError> {
    use rspotify::clients::OAuthClient;

    let aid = ArtistId::from_id(artist_id)
        .map_err(|e| AppError::Network(format!("Invalid artist ID '{artist_id}': {e}")))?;

    with_auto_reauth(spotify, || async {
        spotify
            .user_follow_artists([aid.clone()])
            .await
            .map_err(map_rspotify_error)?;
        Ok(())
    })
    .await
}

/// Unfollows an artist for the authenticated user.
#[allow(clippy::missing_errors_doc)]
#[allow(deprecated)]
pub async fn unfollow_artist(
    spotify: &AuthCodePkceSpotify,
    artist_id: &str,
) -> Result<(), AppError> {
    use rspotify::clients::OAuthClient;

    let aid = ArtistId::from_id(artist_id)
        .map_err(|e| AppError::Network(format!("Invalid artist ID '{artist_id}': {e}")))?;

    with_auto_reauth(spotify, || async {
        spotify
            .user_unfollow_artists([aid.clone()])
            .await
            .map_err(map_rspotify_error)?;
        Ok(())
    })
    .await
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ArtistBio {
    pub artist_name: String,
    pub title: String,
    pub description: Option<String>,
    pub extract: String,
}

#[derive(serde::Deserialize)]
struct WikiSummaryResponse {
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    title: Option<String>,
    description: Option<String>,
    extract: Option<String>,
}

#[allow(clippy::missing_errors_doc)]
pub async fn fetch_artist_bio(artist_name: &str) -> Result<ArtistBio, AppError> {
    let trimmed = artist_name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Network("Empty artist name".to_string()));
    }

    let mut candidates = vec![
        trimmed.to_string(),
        format!("{trimmed} (band)"),
        format!("{trimmed} (musician)"),
    ];

    if let Some(primary) = trimmed.split(&[',', '&', '/'][..]).next() {
        let primary_trimmed = primary.trim();
        if !primary_trimmed.is_empty() && primary_trimmed != trimmed {
            candidates.push(primary_trimmed.to_string());
            candidates.push(format!("{primary_trimmed} (band)"));
            candidates.push(format!("{primary_trimmed} (musician)"));
        }
    }

    let client = reqwest::Client::new();

    for candidate in &candidates {
        let mut url = reqwest::Url::parse("https://en.wikipedia.org/api/rest_v1/page/summary/")
            .map_err(|e| AppError::Network(format!("Failed to parse wiki URL: {e}")))?;

        let path_candidate = candidate.replace(' ', "_");
        if let Ok(mut segments) = url.path_segments_mut() {
            segments.push(&path_candidate);
        }

        let resp = match client
            .get(url)
            .header(
                "User-Agent",
                "Spotifust/0.1.0 (https://github.com/gefydev/spotifust)",
            )
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => r,
            _ => continue,
        };

        if let Ok(body) = resp.json::<WikiSummaryResponse>().await {
            if body.r#type.as_deref() == Some("disambiguation") {
                continue;
            }
            if let Some(extract) = body.extract {
                let trimmed_extract = extract.trim();
                if !trimmed_extract.is_empty() {
                    return Ok(ArtistBio {
                        artist_name: trimmed.to_string(),
                        title: body.title.unwrap_or_else(|| candidate.clone()),
                        description: body.description,
                        extract: trimmed_extract.to_string(),
                    });
                }
            }
        }
    }

    Err(AppError::Network(format!(
        "No Wikipedia bio found for '{trimmed}'"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artist_bio_json_deserialization() {
        let json_data = r#"{
            "type": "standard",
            "title": "Daft Punk",
            "description": "French electronic music duo",
            "extract": "Daft Punk were a French electronic music duo formed in 1993 in Paris."
        }"#;
        let res: Result<WikiSummaryResponse, _> = serde_json::from_str(json_data);
        assert!(res.is_ok());
        let body = res.unwrap();
        assert_eq!(body.r#type.as_deref(), Some("standard"));
        assert_eq!(body.title.as_deref(), Some("Daft Punk"));
        assert_eq!(
            body.description.as_deref(),
            Some("French electronic music duo")
        );
        assert!(body.extract.is_some());
    }

    #[test]
    fn test_artist_bio_disambiguation_detection() {
        let json_data = r#"{
            "type": "disambiguation",
            "title": "Queen",
            "description": "Topics referred to by the same term"
        }"#;
        let res: Result<WikiSummaryResponse, _> = serde_json::from_str(json_data);
        assert!(res.is_ok());
        let body = res.unwrap();
        assert_eq!(body.r#type.as_deref(), Some("disambiguation"));
    }

    #[test]
    fn test_artist_detail_struct() {
        let ad = ArtistDetail {
            id: "art_1".to_string(),
            name: "GUNSHIP".to_string(),
            image_url: None,
            genres: vec!["synthwave".to_string(), "retrowave".to_string()],
            followers: 250_000,
            top_tracks: vec![ArtistTopTrack {
                id: "t_10".to_string(),
                title: "Tech Noir".to_string(),
                album: "GUNSHIP".to_string(),
                duration_ms: 297_000,
                uri: "spotify:track:t_10".to_string(),
            }],
            albums: vec![ArtistAlbum {
                id: "alb_10".to_string(),
                name: "GUNSHIP".to_string(),
                image_url: None,
                release_date: "2015-07-24".to_string(),
            }],
        };
        assert_eq!(ad.name, "GUNSHIP");
        assert_eq!(ad.genres.len(), 2);
        assert_eq!(ad.top_tracks[0].title, "Tech Noir");
    }
}
