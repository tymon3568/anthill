use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

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
///
/// ## Tenant Bootstrap Behavior
///
/// When registering, the system automatically assigns roles based on tenant state:
/// - **New tenant** (name doesn't exist): User becomes `owner` with full privileges
/// - **Existing tenant** (name matches by slug): User joins with default `user` role
#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct RegisterReq {
    /// Email address (must be unique within the tenant)
    #[validate(email)]
    #[schema(example = "user@example.com")]
    pub email: String,

    /// Password (min 8 characters). Recommended: include uppercase, lowercase, and number for stronger security.
    #[validate(length(min = 8))]
    #[schema(example = "SecurePass123!", min_length = 8)]
    pub password: String,

    /// Full name of the user
    #[validate(length(min = 1))]
    #[schema(example = "John Doe")]
    pub full_name: String,

    /// Tenant name - creates new tenant if doesn't exist, joins if exists.
    /// The tenant slug is derived from this name (e.g., "Acme Corp" → "acme-corp").
    /// If creating a new tenant, the registering user becomes the tenant owner.
    #[schema(example = "Acme Corp")]
    pub tenant_name: Option<String>,
}

/// Login request
#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct LoginReq {
    /// Email address
    #[validate(email)]
    #[schema(example = "user@example.com")]
    pub email: String,

    /// Password
    #[validate(length(min = 1))]
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

/// User information returned in auth responses
///
/// The `role` field indicates the user's effective role:
/// - `owner`: Tenant creator with full management privileges
/// - `admin`: Administrative access within the tenant
/// - `user`: Standard user with limited access
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct UserInfo {
    /// User ID (UUID v7)
    pub id: Uuid,

    /// Email address
    #[schema(example = "user@example.com")]
    pub email: String,

    /// Full name (optional)
    #[schema(example = "John Doe")]
    pub full_name: Option<String>,

    /// Tenant ID this user belongs to
    pub tenant_id: Uuid,

    /// User role (owner/admin/user). Assigned automatically during registration:
    /// - 'owner' if user created a new tenant
    /// - 'user' if user joined an existing tenant
    /// Note: This is the primary role from the users table. See `roles` for all assigned roles from Casbin.
    #[schema(example = "owner")]
    pub role: String,

    /// All roles assigned to this user from Casbin RBAC system.
    /// This includes all roles the user has been assigned via the admin role management API.
    #[serde(default)]
    #[schema(example = json!(["user", "admin"]))]
    pub roles: Vec<String>,

    /// User status (active/suspended)
    #[schema(example = "active")]
    pub status: String,

    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Refresh token request
#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct RefreshReq {
    /// Refresh token
    #[validate(length(min = 1))]
    pub refresh_token: String,
}

/// Optional refresh token request body (for httpOnly cookie auth)
/// When using httpOnly cookies, the body can be empty as the token is read from cookies
#[derive(Serialize, Deserialize, ToSchema, Default)]
pub struct OptionalRefreshReq {
    /// Refresh token (optional when using httpOnly cookies)
    pub refresh_token: Option<String>,
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

/// Query parameters for tenant slug check
#[derive(Debug, Deserialize, ToSchema)]
pub struct CheckTenantSlugQuery {
    /// The tenant slug to check (URL-friendly version of tenant name)
    #[schema(example = "acme-corp")]
    pub slug: String,
}

/// Response for tenant slug availability check
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CheckTenantSlugResp {
    /// The normalized slug that was checked
    #[schema(example = "acme-corp")]
    pub slug: String,

    /// Whether the slug is available (no existing tenant with this slug)
    #[schema(example = true)]
    pub available: bool,

    /// If not available, the name of the existing tenant
    #[schema(example = "Acme Corp")]
    pub existing_tenant_name: Option<String>,
}
