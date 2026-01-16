//! Redis-backed AuthZ Version Repository Implementation
//!
//! Provides a hybrid storage approach:
//! - Redis for fast path (hot path for middleware checks)
//! - PostgreSQL as source of truth (fallback and version bumping)

use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use shared_error::AppError;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use user_service_core::domains::auth::domain::authz_version_repository::AuthzVersionRepository;
use uuid::Uuid;

/// Redis key prefix for tenant authorization versions
const TENANT_VERSION_PREFIX: &str = "authz:tenant";

/// Redis key prefix for user authorization versions
const USER_VERSION_PREFIX: &str = "authz:user";

/// TTL for cached versions in seconds (1 hour)
const VERSION_CACHE_TTL_SECS: u64 = 3600;

/// Timeout for Redis operations in milliseconds
const REDIS_TIMEOUT_MS: u64 = 100;

/// Redis-backed AuthZ Version Repository
///
/// Implements the hybrid versioning strategy:
/// - Tenant-level version: invalidates all users in a tenant when role/policies change
/// - User-level version: invalidates only a specific user when their role/status changes
#[derive(Clone)]
pub struct RedisAuthzVersionRepository {
    /// PostgreSQL connection pool (source of truth)
    pool: PgPool,
    /// Redis connection manager (cache)
    redis: Arc<RwLock<Option<ConnectionManager>>>,
    /// Redis URL for reconnection attempts
    #[allow(dead_code)]
    redis_url: String,
}

impl RedisAuthzVersionRepository {
    /// Create a new RedisAuthzVersionRepository
    ///
    /// # Arguments
    /// * `pool` - PostgreSQL connection pool
    /// * `redis_url` - Redis connection URL (e.g., "redis://localhost:6379")
    ///
    /// # Returns
    /// A new repository instance. Redis connection failures are logged but don't prevent creation.
    pub async fn new(pool: PgPool, redis_url: &str) -> Self {
        let redis = match Self::connect_redis(redis_url).await {
            Ok(conn) => {
                info!("AuthZ version store: Redis connected successfully");
                Arc::new(RwLock::new(Some(conn)))
            },
            Err(e) => {
                warn!(
                    "AuthZ version store: Redis connection failed, operating in DB-only mode: {}",
                    e
                );
                Arc::new(RwLock::new(None))
            },
        };

        Self {
            pool,
            redis,
            redis_url: redis_url.to_string(),
        }
    }

    /// Create a repository without Redis (PostgreSQL only)
    ///
    /// Useful for testing or when Redis is not available.
    pub fn new_without_redis(pool: PgPool) -> Self {
        Self {
            pool,
            redis: Arc::new(RwLock::new(None)),
            redis_url: String::new(),
        }
    }

    /// Connect to Redis
    async fn connect_redis(redis_url: &str) -> Result<ConnectionManager, AppError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AppError::InternalError(format!("Redis client error: {}", e)))?;

        ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis connection error: {}", e)))
    }

    /// Build Redis key for tenant version
    fn tenant_key(tenant_id: Uuid) -> String {
        format!("{}:{}:v", TENANT_VERSION_PREFIX, tenant_id)
    }

    /// Build Redis key for user version
    fn user_key(user_id: Uuid) -> String {
        format!("{}:{}:v", USER_VERSION_PREFIX, user_id)
    }

    /// Get value from Redis with timeout
    async fn redis_get(&self, key: &str) -> Option<i64> {
        let conn_guard = self.redis.read().await;
        let mut conn = conn_guard.clone()?;
        drop(conn_guard);

        let result = tokio::time::timeout(
            Duration::from_millis(REDIS_TIMEOUT_MS),
            conn.get::<_, Option<i64>>(key),
        )
        .await;

        match result {
            Ok(Ok(value)) => value,
            Ok(Err(e)) => {
                debug!("Redis GET error for {}: {}", key, e);
                None
            },
            Err(_) => {
                warn!("Redis GET timeout for {}", key);
                None
            },
        }
    }

    /// Set value in Redis with TTL
    async fn redis_set(&self, key: &str, value: i64) -> bool {
        let conn_guard = self.redis.read().await;
        let Some(mut conn) = conn_guard.clone() else {
            return false;
        };
        drop(conn_guard);

        let result = tokio::time::timeout(
            Duration::from_millis(REDIS_TIMEOUT_MS),
            conn.set_ex::<_, _, ()>(key, value, VERSION_CACHE_TTL_SECS),
        )
        .await;

        match result {
            Ok(Ok(_)) => true,
            Ok(Err(e)) => {
                debug!("Redis SET error for {}: {}", key, e);
                false
            },
            Err(_) => {
                warn!("Redis SET timeout for {}", key);
                false
            },
        }
    }

    /// Delete key from Redis
    async fn redis_del(&self, key: &str) -> bool {
        let conn_guard = self.redis.read().await;
        let Some(mut conn) = conn_guard.clone() else {
            return false;
        };
        drop(conn_guard);

        let result =
            tokio::time::timeout(Duration::from_millis(REDIS_TIMEOUT_MS), conn.del::<_, ()>(key))
                .await;

        match result {
            Ok(Ok(_)) => true,
            Ok(Err(e)) => {
                debug!("Redis DEL error for {}: {}", key, e);
                false
            },
            Err(_) => {
                warn!("Redis DEL timeout for {}", key);
                false
            },
        }
    }

    /// Get tenant version from database
    async fn db_get_tenant_version(&self, tenant_id: Uuid) -> Result<i64, AppError> {
        let row = sqlx::query_scalar::<_, i64>(
            "SELECT authz_version FROM tenants WHERE tenant_id = $1 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| AppError::TenantNotFound)
    }

    /// Get user version from database
    async fn db_get_user_version(&self, user_id: Uuid) -> Result<i64, AppError> {
        let row = sqlx::query_scalar::<_, i64>(
            "SELECT authz_version FROM users WHERE user_id = $1 AND deleted_at IS NULL",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| AppError::UserNotFound)
    }

    /// Bump tenant version in database and return new version
    async fn db_bump_tenant_version(&self, tenant_id: Uuid) -> Result<i64, AppError> {
        let new_version = sqlx::query_scalar::<_, i64>(
            r#"
            UPDATE tenants
            SET authz_version = authz_version + 1, updated_at = NOW()
            WHERE tenant_id = $1 AND deleted_at IS NULL
            RETURNING authz_version
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        new_version.ok_or_else(|| AppError::TenantNotFound)
    }

    /// Bump user version in database and return new version
    async fn db_bump_user_version(&self, user_id: Uuid) -> Result<i64, AppError> {
        let new_version = sqlx::query_scalar::<_, i64>(
            r#"
            UPDATE users
            SET authz_version = authz_version + 1, updated_at = NOW()
            WHERE user_id = $1 AND deleted_at IS NULL
            RETURNING authz_version
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        new_version.ok_or_else(|| AppError::UserNotFound)
    }
}

#[async_trait]
impl AuthzVersionRepository for RedisAuthzVersionRepository {
    async fn get_tenant_version(&self, tenant_id: Uuid) -> Result<i64, AppError> {
        let key = Self::tenant_key(tenant_id);

        // Try Redis first
        if let Some(version) = self.redis_get(&key).await {
            debug!("AuthZ tenant version cache HIT: tenant={}, version={}", tenant_id, version);
            return Ok(version);
        }

        // Cache miss - get from DB
        debug!("AuthZ tenant version cache MISS: tenant={}", tenant_id);
        let version = self.db_get_tenant_version(tenant_id).await?;

        // Warm cache
        if self.redis_set(&key, version).await {
            debug!("AuthZ tenant version cache WARM: tenant={}, version={}", tenant_id, version);
        }

        Ok(version)
    }

    async fn get_user_version(&self, user_id: Uuid) -> Result<i64, AppError> {
        let key = Self::user_key(user_id);

        // Try Redis first
        if let Some(version) = self.redis_get(&key).await {
            debug!("AuthZ user version cache HIT: user={}, version={}", user_id, version);
            return Ok(version);
        }

        // Cache miss - get from DB
        debug!("AuthZ user version cache MISS: user={}", user_id);
        let version = self.db_get_user_version(user_id).await?;

        // Warm cache
        if self.redis_set(&key, version).await {
            debug!("AuthZ user version cache WARM: user={}, version={}", user_id, version);
        }

        Ok(version)
    }

    async fn get_versions(&self, tenant_id: Uuid, user_id: Uuid) -> Result<(i64, i64), AppError> {
        // Run both lookups concurrently
        let (tenant_version, user_version) =
            tokio::join!(self.get_tenant_version(tenant_id), self.get_user_version(user_id));

        Ok((tenant_version?, user_version?))
    }

    async fn bump_tenant_version(&self, tenant_id: Uuid) -> Result<i64, AppError> {
        // Bump in database (source of truth)
        let new_version = self.db_bump_tenant_version(tenant_id).await?;

        info!("AuthZ tenant version BUMPED: tenant={}, new_version={}", tenant_id, new_version);

        // Update cache
        let key = Self::tenant_key(tenant_id);
        if self.redis_set(&key, new_version).await {
            debug!("AuthZ tenant version cache UPDATED: tenant={}", tenant_id);
        } else {
            // If cache update fails, invalidate to force DB lookup next time
            self.redis_del(&key).await;
            warn!("AuthZ tenant version cache UPDATE FAILED, invalidated: tenant={}", tenant_id);
        }

        Ok(new_version)
    }

    async fn bump_user_version(&self, user_id: Uuid) -> Result<i64, AppError> {
        // Bump in database (source of truth)
        let new_version = self.db_bump_user_version(user_id).await?;

        info!("AuthZ user version BUMPED: user={}, new_version={}", user_id, new_version);

        // Update cache
        let key = Self::user_key(user_id);
        if self.redis_set(&key, new_version).await {
            debug!("AuthZ user version cache UPDATED: user={}", user_id);
        } else {
            // If cache update fails, invalidate to force DB lookup next time
            self.redis_del(&key).await;
            warn!("AuthZ user version cache UPDATE FAILED, invalidated: user={}", user_id);
        }

        Ok(new_version)
    }

    async fn warm_tenant_cache(&self, tenant_id: Uuid) -> Result<(), AppError> {
        let version = self.db_get_tenant_version(tenant_id).await?;
        let key = Self::tenant_key(tenant_id);

        if self.redis_set(&key, version).await {
            debug!("AuthZ tenant cache WARMED: tenant={}, version={}", tenant_id, version);
        } else {
            warn!("AuthZ tenant cache WARM FAILED: tenant={}", tenant_id);
        }

        Ok(())
    }

    async fn warm_user_cache(&self, user_id: Uuid) -> Result<(), AppError> {
        let version = self.db_get_user_version(user_id).await?;
        let key = Self::user_key(user_id);

        if self.redis_set(&key, version).await {
            debug!("AuthZ user cache WARMED: user={}, version={}", user_id, version);
        } else {
            warn!("AuthZ user cache WARM FAILED: user={}", user_id);
        }

        Ok(())
    }

    async fn invalidate_tenant_cache(&self, tenant_id: Uuid) -> Result<(), AppError> {
        let key = Self::tenant_key(tenant_id);
        self.redis_del(&key).await;
        debug!("AuthZ tenant cache INVALIDATED: tenant={}", tenant_id);
        Ok(())
    }

    async fn invalidate_user_cache(&self, user_id: Uuid) -> Result<(), AppError> {
        let key = Self::user_key(user_id);
        self.redis_del(&key).await;
        debug!("AuthZ user cache INVALIDATED: user={}", user_id);
        Ok(())
    }
}

impl std::fmt::Debug for RedisAuthzVersionRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisAuthzVersionRepository")
            .field("redis_url", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_key_format() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key = RedisAuthzVersionRepository::tenant_key(tenant_id);
        assert_eq!(key, "authz:tenant:550e8400-e29b-41d4-a716-446655440000:v");
    }

    #[test]
    fn test_user_key_format() {
        let user_id = Uuid::parse_str("660e8400-e29b-41d4-a716-446655440001").unwrap();
        let key = RedisAuthzVersionRepository::user_key(user_id);
        assert_eq!(key, "authz:user:660e8400-e29b-41d4-a716-446655440001:v");
    }
}
