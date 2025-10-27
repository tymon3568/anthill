use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    // Database errors
    Database(sqlx::Error),
    
    // Authentication errors
    Unauthorized(String), // With message
    InvalidCredentials,
    TokenExpired,
    InvalidToken,
    
    // Validation errors
    ValidationError(String),
    
    // Business logic errors
    UserAlreadyExists,
    UserNotFound,
    TenantNotFound,
    NotFound(String), // Generic not found with custom message
    Forbidden(String), // Forbidden access with custom message
    
    // Internal errors
    InternalServerError(String),
    InternalError(String), // Alias for InternalServerError
    ConfigError(String),
    DatabaseError(String), // String-based database error
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "Database error: {}", e),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::InvalidCredentials => write!(f, "Invalid credentials"),
            AppError::TokenExpired => write!(f, "Token expired"),
            AppError::InvalidToken => write!(f, "Invalid token"),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::UserAlreadyExists => write!(f, "User already exists"),
            AppError::UserNotFound => write!(f, "User not found"),
            AppError::TenantNotFound => write!(f, "Tenant not found"),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            AppError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, error_code) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string(), "DATABASE_ERROR")
            }
            AppError::Unauthorized(ref msg) => (StatusCode::UNAUTHORIZED, msg.clone(), "UNAUTHORIZED"),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string(), "INVALID_CREDENTIALS"),
            AppError::TokenExpired => (StatusCode::UNAUTHORIZED, self.to_string(), "TOKEN_EXPIRED"),
            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string(), "INVALID_TOKEN"),
            AppError::ValidationError(ref msg) => (StatusCode::BAD_REQUEST, msg.clone(), "VALIDATION_ERROR"),
            AppError::UserAlreadyExists => (StatusCode::CONFLICT, self.to_string(), "USER_EXISTS"),
            AppError::UserNotFound => (StatusCode::NOT_FOUND, self.to_string(), "USER_NOT_FOUND"),
            AppError::TenantNotFound => (StatusCode::NOT_FOUND, self.to_string(), "TENANT_NOT_FOUND"),
            AppError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.clone(), "NOT_FOUND"),
            AppError::Forbidden(ref msg) => (StatusCode::FORBIDDEN, msg.clone(), "FORBIDDEN"),
            AppError::InternalServerError(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string(), "INTERNAL_ERROR")
            }
            AppError::InternalError(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string(), "INTERNAL_ERROR")
            }
            AppError::ConfigError(ref msg) => {
                tracing::error!("Config error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error".to_string(), "CONFIG_ERROR")
            }
            AppError::DatabaseError(ref msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string(), "DATABASE_ERROR")
            }
        };

        let body = Json(json!({
            "error": error_message,
            "code": error_code,
        }));

        (status, body).into_response()
    }
}

// From implementations for common error types
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<std::env::VarError> for AppError {
    fn from(err: std::env::VarError) -> Self {
        AppError::ConfigError(err.to_string())
    }
}
