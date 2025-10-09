use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Standard error response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResp {
    /// Error message
    #[schema(example = "Invalid credentials")]
    pub error: String,
    
    /// Error code for client-side handling
    #[schema(example = "AUTH_FAILED")]
    pub code: Option<String>,
}

/// Health check response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResp {
    /// Service status
    #[schema(example = "ok")]
    pub status: String,
    
    /// Service version
    #[schema(example = "0.1.0")]
    pub version: String,
    
    /// Current timestamp
    pub timestamp: DateTime<Utc>,
}

/// User registration request
#[derive(Serialize, Deserialize, ToSchema)]
pub struct RegisterReq {
    /// Email address
    #[schema(example = "user@example.com")]
    pub email: String,
    
    /// Password (min 8 characters)
    #[schema(example = "SecurePass123!", min_length = 8)]
    pub password: String,
    
    /// Full name
    #[schema(example = "John Doe")]
    pub full_name: String,
    
    /// Tenant name (for new tenant creation)
    #[schema(example = "Acme Corp")]
    pub tenant_name: Option<String>,
}

/// Login request
#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginReq {
    /// Email address
    #[schema(example = "user@example.com")]
    pub email: String,
    
    /// Password
    #[schema(example = "SecurePass123!")]
    pub password: String,
}

/// Authentication response with JWT tokens
#[derive(Serialize, Deserialize, ToSchema)]
pub struct AuthResp {
    /// Access token (JWT, expires in 15 minutes)
    pub access_token: String,
    
    /// Refresh token (expires in 7 days)
    pub refresh_token: String,
    
    /// Token type
    #[schema(example = "Bearer")]
    pub token_type: String,
    
    /// Token expiration time in seconds
    #[schema(example = 900)]
    pub expires_in: i64,
    
    /// User information
    pub user: UserInfo,
}

/// User information
#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserInfo {
    /// User ID
    pub id: Uuid,
    
    /// Email address
    #[schema(example = "user@example.com")]
    pub email: String,
    
    /// Full name
    #[schema(example = "John Doe")]
    pub full_name: String,
    
    /// Tenant ID
    pub tenant_id: Uuid,
    
    /// User role
    #[schema(example = "admin")]
    pub role: String,
    
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Refresh token request
#[derive(Serialize, Deserialize, ToSchema)]
pub struct RefreshReq {
    /// Refresh token
    pub refresh_token: String,
}

/// List of users (paginated)
#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserListResp {
    /// List of users
    pub users: Vec<UserInfo>,
    
    /// Total count
    pub total: i64,
    
    /// Current page
    #[schema(example = 1)]
    pub page: i32,
    
    /// Page size
    #[schema(example = 20)]
    pub page_size: i32,
}
