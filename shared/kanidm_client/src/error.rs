use thiserror::Error;

#[derive(Debug, Error)]
pub enum KanidmError {
    #[error("OAuth2 error: {0}")]
    OAuth2Error(String),

    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Invalid JWT token: {0}")]
    InvalidToken(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token signature")]
    InvalidSignature,

    #[error("Missing required claim: {0}")]
    MissingClaim(String),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("JWT decode error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Invalid URL: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Kanidm API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("User not found in Kanidm")]
    UserNotFound,

    #[error("Group not found in Kanidm")]
    GroupNotFound,

    #[error("No tenant groups found for user")]
    NoTenantGroups,
}

pub type Result<T> = std::result::Result<T, KanidmError>;
