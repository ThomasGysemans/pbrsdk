use thiserror::Error;

/// Describes an error that occurred while calling PocketBase.
#[derive(Debug, Error)]
pub enum ApiError {
    /// A traditional HTTP error.
    #[error("HTTP error {0}: {1}")]
    Http(reqwest::StatusCode, String),

    /// An unexpected error from the Reqwest client itself.
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// Un unexpected error that was triggered by an invalid JWT token.
    /// Will happen only if the JWT is corrupted, not if it expired.
    #[error("Invalid token")]
    Jwt()
}
