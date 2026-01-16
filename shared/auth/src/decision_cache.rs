//! Decision Cache Layer for Authorization
//!
//! Provides caching for Casbin authorization decisions to reduce
//! CPU cost and improve tail latencies under high QPS.
//!
//! The cache key includes policy_version to ensure immediate invalidation
//! when policies change.

use async_trait::async_trait;
use casbin::CoreApi;
use moka::future::Cache;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Cached decision result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachedDecision {
    Allow,
    Deny,
}

impl From<bool> for CachedDecision {
    fn from(allowed: bool) -> Self {
        if allowed {
            CachedDecision::Allow
        } else {
            CachedDecision::Deny
        }
    }
}

impl From<CachedDecision> for bool {
    fn from(decision: CachedDecision) -> Self {
        matches!(decision, CachedDecision::Allow)
    }
}

/// Cache statistics for observability
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

/// Decision cache trait for authorization caching
#[async_trait]
pub trait DecisionCache: Send + Sync {
    /// Get cached decision, returns None on miss
    async fn get(
        &self,
        tenant_id: &Uuid,
        policy_version: i64,
        subject: &str,
        resource: &str,
        action: &str,
    ) -> Option<bool>;

    /// Set decision with TTL
    async fn set(
        &self,
        tenant_id: &Uuid,
        policy_version: i64,
        subject: &str,
        resource: &str,
        action: &str,
        allowed: bool,
    );

    /// Invalidate all entries for a tenant (optional, for explicit flush)
    async fn invalidate_tenant(&self, tenant_id: &Uuid);

    /// Get cache statistics
    fn stats(&self) -> CacheStats;
}

/// Build cache key from components
///
/// Format: `authz:decision:{tenant_id}:{policy_version}:{subject_hash}:{resource_hash}:{action}`
///
/// Subject and resource are hashed (first 16 bytes of SHA256, hex-encoded)
/// to keep keys compact while maintaining low collision probability.
fn build_cache_key(
    tenant_id: &Uuid,
    policy_version: i64,
    subject: &str,
    resource: &str,
    action: &str,
) -> String {
    let subject_hash = hash_truncated(subject);
    let resource_hash = hash_truncated(resource);

    format!(
        "authz:decision:{}:{}:{}:{}:{}",
        tenant_id, policy_version, subject_hash, resource_hash, action
    )
}

/// Hash a string and return first 16 bytes as hex (32 chars)
fn hash_truncated(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    // Take first 16 bytes and hex-encode
    hex::encode(&result[..16])
}

/// In-memory decision cache using moka
pub struct InMemoryDecisionCache {
    cache: Cache<String, CachedDecision>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl InMemoryDecisionCache {
    /// Create a new in-memory cache with specified TTL and max entries
    pub fn new(ttl_seconds: u64, max_entries: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_entries)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();

        Self {
            cache,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// Create with default settings (TTL: 15s, max: 10,000 entries)
    pub fn with_defaults() -> Self {
        Self::new(15, 10_000)
    }
}

#[async_trait]
impl DecisionCache for InMemoryDecisionCache {
    async fn get(
        &self,
        tenant_id: &Uuid,
        policy_version: i64,
        subject: &str,
        resource: &str,
        action: &str,
    ) -> Option<bool> {
        let key = build_cache_key(tenant_id, policy_version, subject, resource, action);

        match self.cache.get(&key).await {
            Some(decision) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                tracing::trace!(
                    tenant_id = %tenant_id,
                    policy_version = policy_version,
                    subject = subject,
                    action = action,
                    decision = ?decision,
                    "Decision cache hit"
                );
                Some(decision.into())
            },
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                tracing::trace!(
                    tenant_id = %tenant_id,
                    policy_version = policy_version,
                    subject = subject,
                    action = action,
                    "Decision cache miss"
                );
                None
            },
        }
    }

    async fn set(
        &self,
        tenant_id: &Uuid,
        policy_version: i64,
        subject: &str,
        resource: &str,
        action: &str,
        allowed: bool,
    ) {
        let key = build_cache_key(tenant_id, policy_version, subject, resource, action);
        let decision = CachedDecision::from(allowed);

        self.cache.insert(key, decision).await;

        tracing::trace!(
            tenant_id = %tenant_id,
            policy_version = policy_version,
            subject = subject,
            action = action,
            allowed = allowed,
            "Decision cached"
        );
    }

    async fn invalidate_tenant(&self, tenant_id: &Uuid) {
        // For in-memory cache, we can't efficiently delete by prefix
        // The TTL + policy_version in key ensures staleness is bounded
        // For explicit invalidation, we'd need a secondary index
        tracing::debug!(
            tenant_id = %tenant_id,
            "Tenant invalidation requested (TTL-based eviction will handle this)"
        );
    }

    fn stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };

        CacheStats {
            hits,
            misses,
            hit_rate,
        }
    }
}

/// No-op cache implementation for when caching is disabled
pub struct NoOpDecisionCache;

#[async_trait]
impl DecisionCache for NoOpDecisionCache {
    async fn get(
        &self,
        _tenant_id: &Uuid,
        _policy_version: i64,
        _subject: &str,
        _resource: &str,
        _action: &str,
    ) -> Option<bool> {
        None
    }

    async fn set(
        &self,
        _tenant_id: &Uuid,
        _policy_version: i64,
        _subject: &str,
        _resource: &str,
        _action: &str,
        _allowed: bool,
    ) {
        // No-op
    }

    async fn invalidate_tenant(&self, _tenant_id: &Uuid) {
        // No-op
    }

    fn stats(&self) -> CacheStats {
        CacheStats {
            hits: 0,
            misses: 0,
            hit_rate: 0.0,
        }
    }
}

/// Cached enforcement wrapper
///
/// Checks cache first, falls back to enforcer on miss, and caches the result.
pub async fn enforce_cached(
    cache: Arc<dyn DecisionCache>,
    enforcer: &crate::enforcer::SharedEnforcer,
    tenant_id: Uuid,
    policy_version: i64,
    subject: &str,
    resource: &str,
    action: &str,
) -> Result<bool, shared_error::AppError> {
    // 1. Check cache
    if let Some(cached) = cache
        .get(&tenant_id, policy_version, subject, resource, action)
        .await
    {
        return Ok(cached);
    }

    // 2. Cache miss - enforce and cache result
    let e = enforcer.read().await;
    let allowed = e
        .enforce((subject, tenant_id.to_string(), resource, action))
        .map_err(|e| shared_error::AppError::InternalError(format!("Casbin error: {}", e)))?;
    drop(e);

    // 3. Store in cache (fire and forget - spawn background task to avoid blocking)
    let cache_clone = Arc::clone(&cache);
    let tenant_id_clone = tenant_id;
    let subject_clone = subject.to_string();
    let resource_clone = resource.to_string();
    let action_clone = action.to_string();
    tokio::spawn(async move {
        cache_clone
            .set(
                &tenant_id_clone,
                policy_version,
                &subject_clone,
                &resource_clone,
                &action_clone,
                allowed,
            )
            .await;
    });

    Ok(allowed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key = build_cache_key(&tenant_id, 1, "user123", "orders", "read");

        assert!(key.starts_with("authz:decision:550e8400-e29b-41d4-a716-446655440000:1:"));
        assert!(key.ends_with(":read"));
        // Key should be consistent
        let key2 = build_cache_key(&tenant_id, 1, "user123", "orders", "read");
        assert_eq!(key, key2);
    }

    #[test]
    fn test_different_versions_different_keys() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key1 = build_cache_key(&tenant_id, 1, "user123", "orders", "read");
        let key2 = build_cache_key(&tenant_id, 2, "user123", "orders", "read");

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_hash_truncated() {
        let hash1 = hash_truncated("user123");
        let hash2 = hash_truncated("user123");
        let hash3 = hash_truncated("user456");

        // Same input = same hash
        assert_eq!(hash1, hash2);
        // Different input = different hash
        assert_ne!(hash1, hash3);
        // 16 bytes = 32 hex chars
        assert_eq!(hash1.len(), 32);
    }

    #[tokio::test]
    async fn test_inmemory_cache_hit_miss() {
        let cache = InMemoryDecisionCache::new(60, 100);
        let tenant_id = Uuid::new_v4();

        // Miss on first access
        let result = cache.get(&tenant_id, 1, "user1", "orders", "read").await;
        assert!(result.is_none());

        // Set and hit
        cache
            .set(&tenant_id, 1, "user1", "orders", "read", true)
            .await;
        let result = cache.get(&tenant_id, 1, "user1", "orders", "read").await;
        assert_eq!(result, Some(true));

        // Different version = miss
        let result = cache.get(&tenant_id, 2, "user1", "orders", "read").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = InMemoryDecisionCache::new(60, 100);
        let tenant_id = Uuid::new_v4();

        // Initial stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        // Cause a miss
        cache.get(&tenant_id, 1, "user1", "orders", "read").await;
        let stats = cache.stats();
        assert_eq!(stats.misses, 1);

        // Set and hit
        cache
            .set(&tenant_id, 1, "user1", "orders", "read", true)
            .await;
        cache.get(&tenant_id, 1, "user1", "orders", "read").await;
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_cached_decision_conversion() {
        assert_eq!(CachedDecision::from(true), CachedDecision::Allow);
        assert_eq!(CachedDecision::from(false), CachedDecision::Deny);
        assert!(bool::from(CachedDecision::Allow));
        assert!(!bool::from(CachedDecision::Deny));
    }
}
