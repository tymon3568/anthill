//! AuthZ Version Repository Trait
//!
//! Defines the interface for authorization versioning operations.
//! Used for immediate-effect permission invalidation.

use async_trait::async_trait;
use shared_error::AppError;
use uuid::Uuid;

/// AuthZ version data for a tenant
#[derive(Debug, Clone)]
pub struct TenantAuthzVersion {
    pub tenant_id: Uuid,
    pub version: i64,
}

/// AuthZ version data for a user
#[derive(Debug, Clone)]
pub struct UserAuthzVersion {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub version: i64,
}

/// AuthZ Version Repository Trait
///
/// Provides read/write access to authorization versions for tenants and users.
/// Implementations should use Redis as the fast path with PostgreSQL as the source of truth.
///
/// ## Version Semantics
/// - Tenant version: Bumped when role definitions or policies change for the tenant
/// - User version: Bumped when a specific user's role assignment or security state changes
///
/// ## Caching Strategy
/// - Redis hit: Return cached version immediately
/// - Redis miss: Query PostgreSQL, warm Redis cache, return value
/// - Redis error: Fall back to PostgreSQL with timeout, log degradation
#[async_trait]
pub trait AuthzVersionRepository: Send + Sync {
    // === Read Operations ===

    /// Get the current authorization version for a tenant.
    ///
    /// Returns version from cache (Redis) if available, otherwise falls back to database.
    async fn get_tenant_version(&self, tenant_id: Uuid) -> Result<i64, AppError>;

    /// Get the current authorization version for a user.
    ///
    /// Returns version from cache (Redis) if available, otherwise falls back to database.
    async fn get_user_version(&self, user_id: Uuid) -> Result<i64, AppError>;

    /// Get both tenant and user versions in a single call.
    ///
    /// Optimized for middleware use where both versions need to be checked.
    async fn get_versions(&self, tenant_id: Uuid, user_id: Uuid) -> Result<(i64, i64), AppError>;

    // === Write Operations ===

    /// Bump the authorization version for a tenant.
    ///
    /// This should be called when:
    /// - Role definitions change
    /// - Policy rules are added/removed/modified
    /// - Tenant-wide security settings change
    ///
    /// Returns the new version after increment.
    async fn bump_tenant_version(&self, tenant_id: Uuid) -> Result<i64, AppError>;

    /// Bump the authorization version for a user.
    ///
    /// This should be called when:
    /// - User's role assignment changes
    /// - User is suspended/unsuspended
    /// - User's password is reset (security-sensitive)
    /// - User's account is reactivated
    ///
    /// Returns the new version after increment.
    async fn bump_user_version(&self, user_id: Uuid) -> Result<i64, AppError>;

    // === Cache Management ===

    /// Warm the cache for a tenant (load from DB into Redis).
    ///
    /// Used proactively after version bumps or when cache misses occur.
    async fn warm_tenant_cache(&self, tenant_id: Uuid) -> Result<(), AppError>;

    /// Warm the cache for a user (load from DB into Redis).
    ///
    /// Used proactively after version bumps or when cache misses occur.
    async fn warm_user_cache(&self, user_id: Uuid) -> Result<(), AppError>;

    /// Invalidate the cached version for a tenant.
    ///
    /// Forces next read to go to database.
    async fn invalidate_tenant_cache(&self, tenant_id: Uuid) -> Result<(), AppError>;

    /// Invalidate the cached version for a user.
    ///
    /// Forces next read to go to database.
    async fn invalidate_user_cache(&self, user_id: Uuid) -> Result<(), AppError>;
}
