use super::model::{Session, Tenant, User};
use async_trait::async_trait;
use shared_error::AppError;
use uuid::Uuid;

/// User repository trait
///
/// Defines the interface for user data access operations.
/// Implementations must handle tenant isolation.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find user by email within a tenant
    async fn find_by_email(&self, tenant_id: Uuid, email: &str) -> Result<Option<User>, AppError>;

    /// Find user by ID within a tenant (only active users)
    async fn find_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<User>, AppError>;

    /// Find user by ID within a tenant (any status, for admin operations)
    /// This method should be used by admin handlers that need to operate on
    /// suspended or inactive users (e.g., unsuspend, role assignment)
    async fn find_by_id_any_status(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<User>, AppError>;

    /// Find multiple users by IDs within a tenant
    async fn find_by_ids(&self, tenant_id: Uuid, user_ids: &[Uuid]) -> Result<Vec<User>, AppError>;

    /// Create a new user
    async fn create(&self, user: &User) -> Result<User, AppError>;

    /// Update a user
    async fn update(&self, user: &User) -> Result<User, AppError>;

    /// List users with pagination and optional filtering (within a tenant)
    async fn list(
        &self,
        tenant_id: Uuid,
        page: i32,
        page_size: i32,
        role: Option<String>,
        status: Option<String>,
    ) -> Result<(Vec<User>, i64), AppError>;

    /// Check if email exists within a tenant
    async fn email_exists(&self, tenant_id: Uuid, email: &str) -> Result<bool, AppError>;

    /// Find user by email across all tenants (for password reset)
    /// Returns the first active user found with this email
    /// Note: This is only used for password reset flow where tenant context is unknown
    async fn find_by_email_global(&self, email: &str) -> Result<Option<User>, AppError>;

    /// Hard delete a user by ID (for compensating transactions)
    /// This permanently removes the user record - use with caution.
    /// Primarily used for rollback when subsequent operations fail after user creation.
    async fn hard_delete_by_id(&self, tenant_id: Uuid, user_id: Uuid) -> Result<bool, AppError>;
}

/// Tenant repository trait
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// Find tenant by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Tenant>, AppError>;

    /// Create a new tenant
    async fn create(&self, tenant: &Tenant) -> Result<Tenant, AppError>;

    /// Find tenant by name
    async fn find_by_name(&self, name: &str) -> Result<Option<Tenant>, AppError>;

    /// Find tenant by slug
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Tenant>, AppError>;

    /// Set the owner of a tenant
    /// Called during registration when a new tenant is created
    async fn set_owner(&self, tenant_id: Uuid, user_id: Uuid) -> Result<(), AppError>;
}

/// Session repository trait
///
/// Manages user sessions and JWT token tracking.
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Create a new session
    async fn create(&self, session: &Session) -> Result<Session, AppError>;

    /// Find session by refresh token hash
    async fn find_by_refresh_token(&self, token_hash: &str) -> Result<Option<Session>, AppError>;

    /// Revoke a session (logout)
    async fn revoke(&self, session_id: Uuid, reason: &str) -> Result<(), AppError>;

    /// Revoke all sessions for a user (force logout everywhere)
    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<u64, AppError>;

    /// Update last_used_at timestamp
    async fn update_last_used(&self, session_id: Uuid) -> Result<(), AppError>;

    /// Delete expired sessions (cleanup job)
    async fn delete_expired(&self) -> Result<u64, AppError>;
}
