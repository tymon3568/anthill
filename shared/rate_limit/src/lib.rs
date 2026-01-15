//! Shared Rate Limiting Library
//!
//! This crate provides rate limiting functionality for the Anthill platform,
//! including:
//!
//! - IP-based rate limiting for authentication endpoints
//! - User-based rate limiting for authenticated endpoints
//! - Account lockout after consecutive failed login attempts
//! - Progressive delays for failed attempts
//! - Redis backend for distributed rate limiting (with in-memory fallback)
//!
//! # Example
//!
//! ```rust,ignore
//! use shared_rate_limit::{RateLimitConfig, RateLimitState, RateLimitEndpoint, RateLimitLayer};
//!
//! // Create rate limit state from configuration
//! let config = RateLimitConfig::default();
//! let state = RateLimitState::from_config(config).await;
//!
//! // Apply rate limiting to a specific route
//! let app = Router::new()
//!     .route("/api/v1/auth/login", post(login))
//!     .layer(RateLimitLayer::new(state.clone(), RateLimitEndpoint::Login));
//! ```
//!
//! # Account Lockout
//!
//! ```rust,ignore
//! use shared_rate_limit::AccountLockout;
//!
//! let lockout = AccountLockout::new(rate_limiter, 10, 3600);
//!
//! // Check if account is locked
//! let status = lockout.check_lockout(user_id).await?;
//! if status.is_locked {
//!     return Err(AccountLocked { remaining: status.remaining_seconds });
//! }
//!
//! // Record failed attempt
//! let status = lockout.record_failed_attempt(user_id).await?;
//! if let Some(delay) = status.delay_ms {
//!     tokio::time::sleep(Duration::from_millis(delay)).await;
//! }
//!
//! // On successful login
//! lockout.record_success(user_id).await?;
//! ```

pub mod config;
pub mod limiter;
pub mod lockout;
pub mod memory_limiter;
pub mod middleware;
pub mod redis_limiter;

// Re-export main types
pub use config::{EndpointRules, RateLimitConfig, RateLimitRule};
pub use limiter::{KeyGenerator, RateLimitError, RateLimitResult, RateLimiter};
pub use lockout::{AccountLockout, LockoutStatus};
pub use memory_limiter::InMemoryRateLimiter;
pub use middleware::{
    RateLimitEndpoint, RateLimitExt, RateLimitLayer, RateLimitMiddleware, RateLimitState,
    SharedRateLimiter,
};
pub use redis_limiter::RedisRateLimiter;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_in_memory() {
        // Create configuration
        let config = RateLimitConfig {
            login_max_attempts: 5,
            login_window_seconds: 60,
            enabled: true,
            ..Default::default()
        };

        // Create state
        let state = RateLimitState::from_config(config).await;

        // Test rate limiting
        let ip = "192.168.1.100";
        for i in 0..5 {
            let result = state
                .check_endpoint(RateLimitEndpoint::Login, ip)
                .await
                .unwrap();
            assert!(result.allowed, "Request {} should be allowed", i + 1);
        }

        // 6th request should be denied
        let result = state
            .check_endpoint(RateLimitEndpoint::Login, ip)
            .await
            .unwrap();
        assert!(!result.allowed, "6th request should be denied");
    }

    #[tokio::test]
    async fn test_integration_different_endpoints() {
        let config = RateLimitConfig {
            login_max_attempts: 2,
            register_max_attempts: 1,
            ..Default::default()
        };

        let state = RateLimitState::from_config(config).await;
        let ip = "10.0.0.1";

        // Use up login limit
        state
            .check_endpoint(RateLimitEndpoint::Login, ip)
            .await
            .unwrap();
        state
            .check_endpoint(RateLimitEndpoint::Login, ip)
            .await
            .unwrap();

        // Login should be denied
        let result = state
            .check_endpoint(RateLimitEndpoint::Login, ip)
            .await
            .unwrap();
        assert!(!result.allowed);

        // Register should still work (different endpoint)
        let result = state
            .check_endpoint(RateLimitEndpoint::Register, ip)
            .await
            .unwrap();
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_lockout_integration() {
        let config = RateLimitConfig::default();
        let state = RateLimitState::from_config(config).await;

        let lockout = AccountLockout::new(
            state.limiter.clone(),
            3,  // Lock after 3 attempts
            60, // 60 second lockout
        );

        let user_id = "test-user-lockout";

        // Initial check - not locked
        let status = lockout.check_lockout(user_id).await.unwrap();
        assert!(!status.is_locked);

        // Record failures
        for _ in 0..3 {
            lockout.record_failed_attempt(user_id).await.unwrap();
        }

        // Should be locked now
        let status = lockout.check_lockout(user_id).await.unwrap();
        assert!(status.is_locked);

        // Unlock
        lockout.unlock_account(user_id).await.unwrap();

        // Should be unlocked
        let status = lockout.check_lockout(user_id).await.unwrap();
        assert!(!status.is_locked);
    }
}
