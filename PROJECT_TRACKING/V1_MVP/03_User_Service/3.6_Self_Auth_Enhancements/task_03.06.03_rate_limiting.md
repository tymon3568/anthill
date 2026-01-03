# Task: Implement Rate Limiting for Authentication Endpoints

**Task ID:** V1_MVP/03_User_Service/3.6_Self_Auth_Enhancements/task_03.06.03_rate_limiting.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.6_Self_Auth_Enhancements
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2026-01-04
**Last Updated:** 2026-01-04

## Detailed Description:
Implement comprehensive rate limiting for authentication endpoints to prevent brute force attacks, credential stuffing, and abuse. Rate limiting is a critical security measure for production-ready authentication systems.

**Endpoints requiring rate limiting:**
- `POST /api/v1/auth/login` - Prevent brute force password attacks
- `POST /api/v1/auth/register` - Prevent mass account creation
- `POST /api/v1/auth/forgot-password` - Prevent email bombing
- `POST /api/v1/auth/resend-verification` - Prevent email spam
- `POST /api/v1/auth/refresh` - Prevent token abuse

**Rate limiting strategies:**
1. **IP-based limiting** - Limit requests per IP address
2. **User-based limiting** - Limit requests per user/email (for authenticated endpoints)
3. **Global limiting** - Overall API rate limits
4. **Sliding window** - More accurate than fixed window

**Response behavior:**
- Return `429 Too Many Requests` when limit exceeded
- Include `Retry-After` header with seconds until reset
- Include `X-RateLimit-*` headers for client awareness

## Specific Sub-tasks:
- [ ] 1. Infrastructure Setup
    - [ ] 1.1. Add `tower-governor` or implement custom rate limiter
    - [ ] 1.2. Configure Redis as rate limit store (with DB fallback)
    - [ ] 1.3. Create rate limit configuration in `shared/config`
- [ ] 2. Core Rate Limiting Layer
    - [ ] 2.1. Create `RateLimiter` trait abstraction
    - [ ] 2.2. Implement `RedisRateLimiter` for distributed rate limiting
    - [ ] 2.3. Implement `InMemoryRateLimiter` for single-instance/testing
    - [ ] 2.4. Create sliding window algorithm implementation
- [ ] 3. Axum Middleware Integration
    - [ ] 3.1. Create `RateLimitLayer` middleware
    - [ ] 3.2. Implement IP extraction (handle proxies, X-Forwarded-For)
    - [ ] 3.3. Implement per-route rate limit configuration
    - [ ] 3.4. Add rate limit headers to responses
- [ ] 4. Endpoint-Specific Limits
    - [ ] 4.1. Configure login endpoint (5 attempts per 15 min per IP)
    - [ ] 4.2. Configure register endpoint (3 per hour per IP)
    - [ ] 4.3. Configure forgot-password (3 per hour per email)
    - [ ] 4.4. Configure resend-verification (3 per hour per user)
    - [ ] 4.5. Configure refresh token (30 per hour per user)
- [ ] 5. Account Lockout Integration
    - [ ] 5.1. Track failed login attempts per user
    - [ ] 5.2. Implement progressive delays (1s, 2s, 4s, 8s...)
    - [ ] 5.3. Lock account after N consecutive failures (10 attempts)
    - [ ] 5.4. Create unlock mechanism (time-based + admin manual)
- [ ] 6. Monitoring & Alerting
    - [ ] 6.1. Log rate limit events with details
    - [ ] 6.2. Add metrics for rate limit hits
    - [ ] 6.3. Create alerts for suspicious patterns
- [ ] 7. Testing
    - [ ] 7.1. Unit tests for rate limiting algorithms
    - [ ] 7.2. Integration tests for middleware
    - [ ] 7.3. Load tests to verify limits work under pressure
    - [ ] 7.4. Test Redis failover to in-memory
- [ ] 8. Documentation
    - [ ] 8.1. Document rate limits in API docs
    - [ ] 8.2. Document configuration options
    - [ ] 8.3. Create runbook for handling rate limit issues

## Acceptance Criteria:
- [ ] Login endpoint limited to 5 attempts per 15 minutes per IP
- [ ] Register endpoint limited to 3 accounts per hour per IP
- [ ] Forgot-password limited to 3 requests per hour per email
- [ ] `429 Too Many Requests` returned when limit exceeded
- [ ] `Retry-After` header included in 429 responses
- [ ] `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset` headers present
- [ ] Rate limits work in distributed environment (via Redis)
- [ ] Graceful fallback when Redis unavailable
- [ ] Account lockout after 10 consecutive failed logins
- [ ] Rate limit configuration via environment variables
- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes

## Dependencies:
*   Task: `task_03.01.10_remove_kanidm_integration.md` (Status: InProgress_By_Claude)
*   Redis service available (optional, can use in-memory fallback)

## Related Documents:
*   `services/user_service/api/src/handlers.rs` - Auth handlers
*   `services/user_service/api/src/main.rs` - Route configuration
*   `shared/config/src/lib.rs` - Configuration module
*   `infra/docker_compose/docker-compose.yml` - Redis service

## Files to Create/Modify:
**New Files:**
- `shared/rate_limit/Cargo.toml` - New shared crate
- `shared/rate_limit/src/lib.rs` - Rate limiter exports
- `shared/rate_limit/src/limiter.rs` - RateLimiter trait
- `shared/rate_limit/src/redis_limiter.rs` - Redis implementation
- `shared/rate_limit/src/memory_limiter.rs` - In-memory implementation
- `shared/rate_limit/src/middleware.rs` - Axum middleware
- `shared/rate_limit/src/config.rs` - Rate limit configuration

**Modified Files:**
- `Cargo.toml` (workspace) - Add shared/rate_limit member
- `services/user_service/api/Cargo.toml` - Add rate_limit dependency
- `services/user_service/api/src/main.rs` - Apply rate limit middleware
- `shared/config/src/lib.rs` - Add rate limit config
- `.env.example` - Add rate limit variables

## Rate Limit Configuration:
```rust
// Example configuration structure
pub struct RateLimitConfig {
    /// Redis URL for distributed rate limiting
    pub redis_url: Option<String>,
    
    /// Login endpoint limits
    pub login_max_attempts: u32,        // default: 5
    pub login_window_seconds: u64,      // default: 900 (15 min)
    
    /// Register endpoint limits
    pub register_max_attempts: u32,     // default: 3
    pub register_window_seconds: u64,   // default: 3600 (1 hour)
    
    /// Forgot password limits
    pub forgot_password_max: u32,       // default: 3
    pub forgot_password_window: u64,    // default: 3600
    
    /// Account lockout settings
    pub lockout_threshold: u32,         // default: 10
    pub lockout_duration_seconds: u64,  // default: 3600
    
    /// Global API rate limit
    pub global_requests_per_second: u32, // default: 100
}
```

## Redis Schema:
```
# Rate limit keys
rate_limit:login:ip:{ip_hash}           -> count (TTL: window)
rate_limit:register:ip:{ip_hash}        -> count (TTL: window)
rate_limit:forgot:email:{email_hash}    -> count (TTL: window)
rate_limit:user:{user_id}:failed_login  -> count (TTL: window)

# Account lockout keys
lockout:user:{user_id}                  -> timestamp (TTL: lockout_duration)
```

## Response Headers:
```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
Retry-After: 300
X-RateLimit-Limit: 5
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1704312345

{
  "error": "rate_limit_exceeded",
  "message": "Too many requests. Please try again later.",
  "retry_after_seconds": 300
}
```

## Sliding Window Algorithm:
```rust
// Sliding window log algorithm
// More accurate than fixed window, prevents burst at window edges
//
// Key: rate_limit:{endpoint}:{identifier}
// Value: Sorted set of timestamps
//
// Algorithm:
// 1. Remove entries older than window
// 2. Count remaining entries
// 3. If count < limit, add current timestamp
// 4. Return remaining capacity

pub async fn check_rate_limit(
    key: &str,
    limit: u32,
    window_seconds: u64,
) -> Result<RateLimitResult, Error> {
    let now = SystemTime::now();
    let window_start = now - Duration::from_secs(window_seconds);
    
    // Redis MULTI/EXEC for atomic operation
    // ZREMRANGEBYSCORE key -inf {window_start}
    // ZCARD key
    // ZADD key {now} {now}
    // EXPIRE key {window_seconds}
}
```

## Notes / Discussion:
---
*   Consider using `tower-governor` crate for simpler integration
*   Alternative: `actix-limitation` patterns adapted for Axum
*   For MVP, start with IP-based limiting; add user-based later
*   Hash IP addresses before using as keys (privacy)
*   Consider geographic rate limiting for DDoS scenarios
*   Whitelist trusted IPs (internal services, monitoring)
*   Progressive delays (1s, 2s, 4s...) provide better UX than hard blocks
*   Log all rate limit events for security analysis
*   Consider implementing CAPTCHA trigger after N failed attempts

## Security Considerations:
- [ ] IP addresses hashed before storage
- [ ] Email addresses hashed in rate limit keys
- [ ] No timing side channels in rate limit checks
- [ ] Distributed locking for Redis operations
- [ ] Graceful degradation when Redis unavailable
- [ ] Rate limits apply before expensive operations
- [ ] Bypass prevention (multiple IPs, slow attacks)

## AI Agent Log:
---
*   2026-01-04 00:50: Task created as part of self-auth enhancement plan
    - Critical security feature for brute force protection
    - Enables safe deployment of password-based auth
    - Complements email verification and password reset features
