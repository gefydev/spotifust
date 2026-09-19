use crate::error::AppError;
use rspotify::{
    AuthCodePkceSpotify, Credentials, OAuth,
    clients::{BaseClient, OAuthClient},
    scopes,
};

const CLIENT_ID: &str = "6b2bd6e25f5e49e1853788e7b705522f"; // Needs to be a valid client id or from env
const REDIRECT_URI: &str = "spotifust://callback";

fn get_spotify_client() -> AuthCodePkceSpotify {
    let client_id = std::env::var("SPOTIFY_CLIENT_ID").unwrap_or_else(|_| CLIENT_ID.to_string());

    let creds = Credentials::new_pkce(&client_id);
    let oauth = OAuth {
        redirect_uri: REDIRECT_URI.to_string(),
        scopes: scopes!(
            "user-read-playback-state",
            "user-modify-playback-state",
            "user-read-currently-playing",
            "streaming",
            "app-remote-control",
            "playlist-read-private",
            "playlist-read-collaborative",
            "user-library-read",
            "user-top-read",
            "playlist-modify-public",
            "playlist-modify-private",
            "user-library-modify",
            "user-follow-read",
            "user-follow-modify"
        ),
        ..Default::default()
    };

    AuthCodePkceSpotify::new(creds, oauth)
}

/// Saves the refresh token to the OS keychain via `keyring`.
#[allow(clippy::missing_errors_doc)]
pub fn save_refresh_token_to_keyring(refresh_token: &str) -> Result<(), AppError> {
    let entry = keyring::Entry::new("spotifust", "spotify_refresh_token")
        .map_err(|e| AppError::Auth(format!("Keyring error: {e}")))?;
    entry
        .set_password(refresh_token)
        .map_err(|e| AppError::Auth(format!("Failed to save token to keyring: {e}")))?;
    Ok(())
}

/// Retrieves the refresh token from the OS keychain via `keyring`.
#[allow(clippy::missing_errors_doc)]
pub fn get_refresh_token_from_keyring() -> Result<String, AppError> {
    let entry = keyring::Entry::new("spotifust", "spotify_refresh_token")
        .map_err(|e| AppError::Auth(format!("Keyring error: {e}")))?;
    entry
        .get_password()
        .map_err(|_| AppError::Auth("No token in keyring".to_string()))
}

/// Deletes the refresh token from the OS keychain via `keyring`.
#[allow(clippy::missing_errors_doc)]
pub fn delete_refresh_token_from_keyring() -> Result<(), AppError> {
    let entry = keyring::Entry::new("spotifust", "spotify_refresh_token")
        .map_err(|e| AppError::Auth(format!("Keyring error: {e}")))?;
    let _ = entry.delete_credential();
    Ok(())
}

#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub async fn do_login_flow() -> Result<AuthCodePkceSpotify, AppError> {
    let mut spotify = get_spotify_client();
    let url = spotify
        .get_authorize_url(None)
        .map_err(|e| AppError::Auth(format!("Failed to generate auth url: {e}")))?;

    open::that(&url).map_err(|e| AppError::Auth(format!("Failed to open browser: {e}")))?;

    // Wait for the temp file to be created by the interceptor instance
    let temp_dir = std::env::temp_dir();
    let auth_file = temp_dir.join("spotifust_auth.txt");

    // Ensure it doesn't exist from a previous run
    let _ = std::fs::remove_file(&auth_file);

    let mut attempts = 0;
    let url_with_code = loop {
        if attempts > 60 {
            // 2 minutes timeout (assuming 2s intervals)
            return Err(AppError::Auth("Timeout waiting for login".into()));
        }
        if let Ok(content) = std::fs::read_to_string(&auth_file) {
            let _ = std::fs::remove_file(&auth_file);
            break content;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        attempts += 1;
    };

    let code = spotify
        .parse_response_code(&url_with_code)
        .ok_or_else(|| AppError::Auth("Could not parse auth code from URL".to_string()))?;

    spotify
        .request_token(&code)
        .await
        .map_err(|e| AppError::Auth(format!("Failed to request token: {e}")))?;

    let token_mutex = spotify.get_token();
    let token_guard = token_mutex
        .lock()
        .await
        .map_err(|e| AppError::Auth(format!("Failed to lock token mutex: {e:?}")))?;
    let token = token_guard
        .clone()
        .ok_or_else(|| AppError::Auth("No token obtained".to_string()))?;

    if let Some(refresh_token) = &token.refresh_token {
        save_refresh_token_to_keyring(refresh_token)?;
    }

    Ok(spotify)
}

#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub async fn check_existing_login() -> Result<AuthCodePkceSpotify, AppError> {
    let refresh_token = get_refresh_token_from_keyring()?;

    let spotify = get_spotify_client();

    // We construct a mock token just with the refresh token so rspotify can refresh it
    let token = rspotify::model::Token {
        refresh_token: Some(refresh_token),
        ..Default::default()
    };

    let token_mutex = spotify.get_token();
    *token_mutex
        .lock()
        .await
        .map_err(|e| AppError::Auth(format!("Failed to lock token mutex: {e:?}")))? = Some(token);

    // Force a refresh to verify it works and get an access token
    spotify
        .refresh_token()
        .await
        .map_err(|e| AppError::Auth(format!("Failed to refresh token: {e}")))?;

    Ok(spotify)
}

/// Refreshes the Spotify client's OAuth token silently if expired or missing.
#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub async fn refresh_token_if_expired(spotify: &AuthCodePkceSpotify) -> Result<(), AppError> {
    let token_mutex = spotify.get_token();
    let token_guard = token_mutex
        .lock()
        .await
        .map_err(|e| AppError::Auth(format!("Failed to lock token mutex: {e:?}")))?;

    let is_expired = if let Some(token) = token_guard.as_ref() {
        token.is_expired()
    } else {
        true
    };

    drop(token_guard);

    if is_expired {
        // If token has refresh_token, refresh directly. Otherwise load from keyring.
        let has_refresh = {
            let mutex = spotify.get_token();
            let guard = mutex
                .lock()
                .await
                .map_err(|e| AppError::Auth(format!("Lock error: {e:?}")))?;
            guard.as_ref().and_then(|t| t.refresh_token.clone())
        };

        if has_refresh.is_none() {
            let refresh_token = get_refresh_token_from_keyring()?;
            let mock_token = rspotify::model::Token {
                refresh_token: Some(refresh_token),
                ..Default::default()
            };
            let mutex = spotify.get_token();
            *mutex
                .lock()
                .await
                .map_err(|e| AppError::Auth(format!("Lock error: {e:?}")))? = Some(mock_token);
        }

        spotify
            .refresh_token()
            .await
            .map_err(|e| AppError::Auth(format!("Failed to silently refresh token: {e}")))?;
    }

    Ok(())
}

#[must_use]
pub fn map_rspotify_error(e: rspotify::ClientError) -> AppError {
    match e {
        rspotify::ClientError::Http(http_err) => match *http_err {
            rspotify::http::HttpError::StatusCode(ref resp)
                if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS =>
            {
                let secs = resp
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(2);
                AppError::RateLimited(secs)
            }
            rspotify::http::HttpError::StatusCode(ref resp)
                if resp.status() == reqwest::StatusCode::UNAUTHORIZED =>
            {
                AppError::Auth("Unauthorized 401".to_string())
            }
            other => AppError::Network(other.to_string()),
        },
        rspotify::ClientError::InvalidToken => AppError::Auth("Invalid token".to_string()),
        other => {
            let err_str = other.to_string();
            if err_str.contains("429") {
                AppError::RateLimited(2)
            } else {
                AppError::Network(err_str)
            }
        }
    }
}

#[allow(clippy::missing_errors_doc)]
pub async fn with_auto_reauth<F, Fut, T>(spotify: &AuthCodePkceSpotify, f: F) -> Result<T, AppError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    let mut attempts = 0;
    loop {
        match f().await {
            Ok(val) => return Ok(val),
            Err(AppError::RateLimited(secs)) if attempts < 3 => {
                attempts += 1;
                tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
            }
            Err(AppError::Auth(_) | AppError::Network(_)) if attempts < 1 => {
                attempts += 1;
                if refresh_token_if_expired(spotify).await.is_ok() {
                    continue;
                }
                return Err(AppError::Auth(
                    "Session expired. Please log in again.".to_string(),
                ));
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyring_service_and_account_name() {
        let res = keyring::Entry::new("spotifust", "spotify_refresh_token");
        let _ = res;
    }

    #[test]
    fn test_keyring_helper_functions_exist() {
        let _ = get_refresh_token_from_keyring();
    }

    #[tokio::test]
    async fn test_with_auto_reauth_success_path() {
        let spotify = get_spotify_client();
        let res = with_auto_reauth(&spotify, || async { Ok::<i32, AppError>(42) }).await;
        assert_eq!(res.unwrap(), 42);
    }

    #[test]
    fn test_map_rspotify_error_invalid_token() {
        let err = rspotify::ClientError::InvalidToken;
        let mapped = map_rspotify_error(err);
        assert!(matches!(mapped, AppError::Auth(_)));
    }

    #[tokio::test]
    async fn test_with_auto_reauth_rate_limit_retry() {
        let spotify = get_spotify_client();
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c = counter.clone();
        let res = with_auto_reauth(&spotify, || {
            let count = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            async move {
                if count == 0 {
                    Err(AppError::RateLimited(0))
                } else {
                    Ok(99)
                }
            }
        })
        .await;
        assert_eq!(res.unwrap(), 99);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}
