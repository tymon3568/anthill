# Task: Implement Authorization Decision Cache Layer with TTL

**Task ID:** `V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.14_implement_decision_cache_layer.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.2_Casbin_Authorization  
**Priority:** High  
**Status:** Todo  
**Assignee:**  
**Created Date:** 2026-01-04  
**Last Updated:** 2026-01-04  

## Context / Goal

Implement a **decision cache layer** for Casbin authorization to reduce CPU cost of repeated enforcement checks and improve tail latencies under high QPS.

Per **AUTHORIZATION_RBAC_STRATEGY.md**, even with Rust's fast in-process execution, authorization at scale is dominated by:
- Policy evaluation cost multiplied by QPS
- Lock contention in shared enforcer structures

The decision cache stores `allow/deny` results for a short time, keyed by:
- `(tenant_id, policy_version, subject, resource, action) -> allow/deny`

**TTL: 10-30 seconds recommended**

This ensures:
- Repeated identical authorization checks are O(1) lookups
- Policy changes invalidate cache via `policy_version` in the key
- Reduced enforcer contention under high QPS

## Requirements

### Functional
- Cache `enforce()` results with configurable TTL (default: 15 seconds)
- Cache key must include `policy_version` for safe invalidation
- Support both Redis (distributed) and in-memory (single instance) backends
- Provide cache hit/miss metrics
- Automatic cache warm-up is NOT required (lazy population on first access)

### Non-Functional
- **Performance**: Cache lookup < 1ms (p99)
- **Memory**: Bounded cache size with LRU eviction for in-memory backend
- **Correctness**: Version mismatch = cache miss (no stale decisions)
- **Resilience**: Redis failure falls back to direct `enforce()` (no cache)

### Architecture (3-crate pattern)
- `shared/auth/`: Add cache abstraction and integration with enforcer
- Keep cache logic separate from Casbin internals

### Error Handling
- No `unwrap()` / `expect()`
- Cache errors should be logged and bypassed, not propagated as request failures

## Design

### Cache Key Format
```
authz:decision:{tenant_id}:{policy_version}:{subject_hash}:{resource_hash}:{action}
```

Where:
- `subject_hash`: SHA256(user_id)[0:16] to keep keys compact
- `resource_hash`: SHA256(resource)[0:16]
- `policy_version`: from tenant's `authz_version`

### Cache Value
```rust
enum CachedDecision {
    Allow,
    Deny,
}
// Stored as: "1" for Allow, "0" for Deny
```

### Interface

```rust
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

pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}
```

### Integration with Enforcer

Create a wrapper function or modify existing `enforce()`:

```rust
pub async fn enforce_cached(
    cache: &dyn DecisionCache,
    enforcer: &SharedEnforcer,
    tenant_id: Uuid,
    policy_version: i64,
    subject: &str,
    resource: &str,
    action: &str,
) -> Result<bool, AppError> {
    // 1. Check cache
    if let Some(cached) = cache.get(&tenant_id, policy_version, subject, resource, action).await {
        return Ok(cached);
    }
    
    // 2. Cache miss - enforce and cache result
    let e = enforcer.read().await;
    let allowed = e.enforce((subject, tenant_id.to_string(), resource, action))
        .map_err(|e| AppError::InternalError(format!("Casbin error: {}", e)))?;
    drop(e);
    
    // 3. Store in cache (fire and forget, don't wait)
    cache.set(&tenant_id, policy_version, subject, resource, action, allowed).await;
    
    Ok(allowed)
}
```

## Specific Sub-tasks

- [ ] 1. Define cache abstraction in `shared/auth/`
  - [ ] 1.1. Create `DecisionCache` trait with get/set/invalidate methods
  - [ ] 1.2. Define `CacheStats` struct for observability
  - [ ] 1.3. Define cache key builder helper function
- [ ] 2. Implement in-memory cache backend
  - [ ] 2.1. Use `moka` or `quick_cache` crate for TTL + LRU support
  - [ ] 2.2. Configure max entries (default: 10,000)
  - [ ] 2.3. Configure TTL (default: 15 seconds)
  - [ ] 2.4. Implement atomic hit/miss counters
- [ ] 3. Implement Redis cache backend
  - [ ] 3.1. Use `redis` crate with connection pooling
  - [ ] 3.2. Implement SET with EX (expiry) for TTL
  - [ ] 3.3. Handle Redis connection errors gracefully
  - [ ] 3.4. Implement tenant invalidation via pattern delete (optional)
- [ ] 4. Create cache-aware enforce wrapper
  - [ ] 4.1. Create `enforce_cached()` function
  - [ ] 4.2. Ensure policy_version is passed from middleware context
  - [ ] 4.3. Add structured logging for cache hits/misses
- [ ] 5. Integrate with existing middleware
  - [ ] 5.1. Update `CasbinAuthLayer` to use cached enforcement
  - [ ] 5.2. Inject cache instance via `AuthzState` or extension
  - [ ] 5.3. Wire configuration (TTL, backend selection, max entries)
- [ ] 6. Add configuration support
  - [ ] 6.1. Add to `shared/config`: `decision_cache_ttl_seconds`, `decision_cache_backend`, `decision_cache_max_entries`
  - [ ] 6.2. Add `DECISION_CACHE_BACKEND` env var (redis/memory)
  - [ ] 6.3. Document defaults and tuning guidance
- [ ] 7. Testing
  - [ ] 7.1. Unit tests for cache key generation
  - [ ] 7.2. Unit tests for in-memory cache (hit/miss/TTL/LRU eviction)
  - [ ] 7.3. Integration test: cached vs uncached enforce behavior
  - [ ] 7.4. Test Redis fallback behavior on connection failure
- [ ] 8. Observability
  - [ ] 8.1. Add `tracing` spans for cache operations
  - [ ] 8.2. Export metrics: `authz_cache_hits_total`, `authz_cache_misses_total`
  - [ ] 8.3. Log cache statistics periodically (optional)

## Acceptance Criteria

- [ ] `DecisionCache` trait exists with in-memory and Redis implementations
- [ ] Cached enforcement returns same results as direct enforcement
- [ ] TTL correctly expires entries (verified by test)
- [ ] Policy version change causes cache miss (no stale decisions)
- [ ] Redis failure falls back to direct enforce (no request failure)
- [ ] Cache hit rate observable via `CacheStats`
- [ ] Performance: < 1ms p99 for cache lookups (in-memory)
- [ ] No `unwrap()`/`expect()` in production code
- [ ] Configuration via environment variables works
- [ ] Tests pass: `cargo test --workspace`

## Dependencies

- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.01_add_authz_versioning_schema.md` (Status: NeedsReview) - Provides `policy_version`
- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.02_implement_authz_version_store_with_redis.md` (Status: Todo) - Provides version lookup

## Recommended Crates

- **In-memory cache**: `moka` (battle-tested, TTL + LRU, async-friendly)
- **Redis**: `redis` with `deadpool-redis` for pooling
- **Hashing**: `sha2` for key hashing

## Configuration Defaults

```toml
# Default configuration
decision_cache_backend = "memory"  # or "redis"
decision_cache_ttl_seconds = 15
decision_cache_max_entries = 10000
decision_cache_redis_url = "redis://localhost:6379"
```

## Performance Expectations

| Metric | Target |
|--------|--------|
| Cache hit latency (memory) | < 100μs p99 |
| Cache hit latency (Redis) | < 2ms p99 |
| Cache miss overhead | < 50μs |
| Memory usage (10k entries) | < 10MB |
| Hit rate (steady state) | > 80% |

## Notes / Discussion

- **Why TTL-based + version in key**: Double protection. TTL provides bounded staleness. Version in key ensures immediate invalidation on policy change.
- **Why not just TTL**: Without version, there's a window where old decisions are served after policy update.
- **Why not just version**: Without TTL, cache can grow unbounded with old version entries.
- **Sensitive endpoints**: Per RBAC strategy, sensitive endpoints (user management, policy management) should bypass decision cache and use fresh enforcement. This can be handled by middleware route configuration.

## Related Documents

- `docs/AUTHORIZATION_RBAC_STRATEGY.md` - Performance Strategy section
- `shared/auth/src/enforcer.rs` - Current enforce logic
- `shared/auth/src/layer.rs` - Middleware integration point

## AI Agent Log

---
* 2026-01-04: Task created to implement decision cache layer per AUTHORIZATION_RBAC_STRATEGY.md requirements.
  - Addresses missing performance optimization for high QPS scenarios.
  - Integrates with policy versioning for safe cache invalidation.
