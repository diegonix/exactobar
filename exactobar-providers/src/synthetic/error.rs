//! Synthetic.new-specific errors.

use thiserror::Error;

/// Synthetic.new-specific errors.
#[derive(Debug, Error)]
pub enum SyntheticError {
    /// API key not found in environment.
    #[error("API key not found (set SYNTHETIC_API_KEY env var)")]
    ApiKeyNotFound,

    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    HttpError(String),

    /// Parse error.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// API error.
    #[error("API error: {0}")]
    ApiError(String),

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
}

impl From<reqwest::Error> for SyntheticError {
    fn from(err: reqwest::Error) -> Self {
        SyntheticError::HttpError(err.to_string())
    }
}
