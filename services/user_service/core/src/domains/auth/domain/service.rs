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

    /// Internal method to delete a user (for compensating transactions)
    ///
    /// This is used to roll back user creation when subsequent operations fail
    /// (e.g., Casbin policy creation failure). Performs a hard delete.
    ///
    /// # Arguments
    /// * `user_id` - ID of the user to delete
    /// * `tenant_id` - Tenant ID (for tenant isolation verification)
    ///
    /// # Returns
    /// * `bool` - true if user was deleted, false if not found
    async fn internal_delete_user(&self, user_id: Uuid, tenant_id: Uuid) -> Result<bool, AppError>;

    /// Suspend a user (admin operation)
    ///
    /// Sets user status to "suspended" and revokes all active sessions.
    /// Protects tenant owner from being suspended by non-owners.
    ///
    /// # Arguments
    /// * `admin_tenant_id` - Tenant ID from admin's JWT (for isolation)
    /// * `admin_user_id` - Admin's user ID (for owner protection check)
    /// * `target_user_id` - User to suspend
    /// * `reason` - Optional reason for suspension
    ///
    /// # Errors
    /// * `AppError::NotFound` - User not found in tenant
    /// * `AppError::Forbidden` - Attempting to suspend tenant owner
    async fn admin_suspend_user(
        &self,
        admin_tenant_id: Uuid,
        admin_user_id: Uuid,
        target_user_id: Uuid,
        reason: Option<String>,
    ) -> Result<crate::domains::auth::dto::admin_dto::SuspendUserResp, AppError>;

    /// Unsuspend a user (admin operation)
    ///
    /// Sets user status back to "active".
    ///
    /// # Arguments
    /// * `admin_tenant_id` - Tenant ID from admin's JWT
    /// * `target_user_id` - User to unsuspend
    async fn admin_unsuspend_user(
        &self,
        admin_tenant_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<crate::domains::auth::dto::admin_dto::UnsuspendUserResp, AppError>;

    /// Soft delete a user (admin operation)
    ///
    /// Sets deleted_at timestamp and revokes all sessions.
    /// Protects tenant owner from deletion.
    ///
    /// # Arguments
    /// * `admin_tenant_id` - Tenant ID from admin's JWT
    /// * `admin_user_id` - Admin's user ID (for owner protection)
    /// * `target_user_id` - User to delete
    async fn admin_delete_user(
        &self,
        admin_tenant_id: Uuid,
        admin_user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<crate::domains::auth::dto::admin_dto::DeleteUserResp, AppError>;

    /// Reset user password (admin operation)
    ///
    /// Updates password hash and optionally revokes all sessions.
    /// Updates password_changed_at timestamp.
    ///
    /// # Arguments
    /// * `admin_tenant_id` - Tenant ID from admin's JWT
    /// * `target_user_id` - User whose password to reset
    /// * `new_password` - New password (will be hashed)
    /// * `force_logout` - Whether to revoke all sessions
    async fn admin_reset_password(
        &self,
        admin_tenant_id: Uuid,
        target_user_id: Uuid,
        new_password: String,
        force_logout: bool,
    ) -> Result<crate::domains::auth::dto::admin_dto::AdminResetPasswordResp, AppError>;
}
