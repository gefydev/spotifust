pub mod album;
pub mod artist;
pub mod auth;
pub mod cache;
pub mod local_files;
pub mod lyrics;
pub mod playlist;
pub mod search;
pub mod tracks;
pub mod updater;
pub mod user;

/// Picks the thumbnail-sized image (closest to 300px wide) from a Spotify image list.
/// Thumbnails are used for track-level artwork to cut download size and decode cost
/// vs. always taking the largest (640px) image.
#[must_use]
pub fn pick_thumb_image(images: &[rspotify::model::Image]) -> Option<String> {
    let mut best: Option<(&rspotify::model::Image, i64)> = None;
    for img in images {
        if let Some(w) = img.width {
            if w > 0 {
                let dist = (i64::from(w) - 300).abs();
                if best.is_none_or(|(_, best_dist)| dist < best_dist) {
                    best = Some((img, dist));
                }
            }
        }
    }
    best.map(|(img, _)| img.url.clone())
        .or_else(|| images.first().map(|img| img.url.clone()))
}
