use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("HTTP error {0}: {1}")]
    Http(reqwest::StatusCode, String),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error("Invalid token")]
    Jwt()
}
