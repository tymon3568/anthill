use crate::domains::auth::dto::admin_dto::{AdminCreateUserReq, AdminCreateUserResp};
use crate::domains::auth::dto::auth_dto::{
    AuthResp, LoginReq, RefreshReq, RegisterReq, UserInfo, UserListResp,
};
use async_trait::async_trait;
use shared_error::AppError;
use uuid::Uuid;

/// Auth service trait
///
/// Defines the business logic interface for authentication operations.
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Register a new user
    async fn register(
        &self,
        req: RegisterReq,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResp, AppError>;

    /// Login user
    async fn login(
        &self,
        req: LoginReq,
        tenant_identifier: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResp, AppError>;

    /// Refresh access token
    async fn refresh_token(
        &self,
        req: RefreshReq,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResp, AppError>;

    /// Logout user by revoking refresh token session
    async fn logout(&self, refresh_token: &str) -> Result<(), AppError>;

    /// List users (paginated, tenant-scoped, with optional filtering)
    async fn list_users(
        &self,
        tenant_id: Uuid,
        page: i32,
        page_size: i32,
        role: Option<String>,
        status: Option<String>,
    ) -> Result<UserListResp, AppError>;

    /// Get user info by ID
    async fn get_user(&self, user_id: Uuid, tenant_id: Uuid) -> Result<UserInfo, AppError>;

    /// Cleanup stale sessions (admin operation)
    /// Revokes sessions with expired refresh tokens
    async fn cleanup_stale_sessions(&self) -> Result<u64, AppError>;

    /// Admin creates a new user in their tenant
    ///
    /// Creates a user with the specified role (default: "user").
    /// The user is always created in the admin's tenant (from JWT).
    /// Password is hashed with bcrypt before storage.
    ///
    /// # Arguments
    /// * `admin_tenant_id` - Tenant ID from the admin's JWT (enforces tenant isolation)
    /// * `req` - Admin create user request with email, password, optional full_name and role
    ///
    /// # Returns
    /// * `AdminCreateUserResp` with created user details (no sensitive data)
    ///
    /// # Errors
    /// * `AppError::UserAlreadyExists` - Email already exists in tenant
    /// * `AppError::ValidationError` - Invalid email, weak password, or invalid role
    /// * `AppError::Forbidden` - Cannot create user with "owner" role
    async fn admin_create_user(
        &self,
        admin_tenant_id: Uuid,
        req: AdminCreateUserReq,
    ) -> Result<AdminCreateUserResp, AppError>;
}
