//! Rate limiter trait and types

use async_trait::async_trait;
use std::time::{Duration, SystemTime};

/// Result of a rate limit check
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Maximum number of requests allowed in the window
    pub limit: u32,
    /// Number of requests remaining in the current window
    pub remaining: u32,
    /// Unix timestamp when the rate limit resets
    pub reset_at: u64,
    /// Seconds until the rate limit resets
    pub retry_after: u64,
}

impl RateLimitResult {
    /// Create a new allowed result
    pub fn allowed(limit: u32, remaining: u32, reset_at: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let retry_after = reset_at.saturating_sub(now);

        Self {
            allowed: true,
            limit,
            remaining,
            reset_at,
            retry_after,
        }
    }

    /// Create a new denied result
    pub fn denied(limit: u32, reset_at: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let retry_after = reset_at.saturating_sub(now);

        Self {
            allowed: false,
            limit,
            remaining: 0,
            reset_at,
            retry_after,
        }
    }
}

/// Error types for rate limiting operations
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Redis connection error: {0}")]
    RedisError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Rate limiter trait for different implementations
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if a request is allowed and update the counter
    ///
    /// # Arguments
    /// * `key` - Unique identifier for the rate limit (e.g., IP hash, user ID)
    /// * `max_requests` - Maximum number of requests allowed
    /// * `window` - Time window for the rate limit
    ///
    /// # Returns
    /// Result containing the rate limit status
    async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: u32,
        window: Duration,
    ) -> Result<RateLimitResult, RateLimitError>;

    /// Reset the rate limit for a specific key
    async fn reset(&self, key: &str) -> Result<(), RateLimitError>;

    /// Get the current count for a key without incrementing
    async fn get_count(&self, key: &str) -> Result<u32, RateLimitError>;

    /// Get the remaining TTL (time-to-live) for a key in seconds
    /// Returns 0 if the key doesn't exist or has no TTL
    async fn get_ttl(&self, key: &str) -> Result<u64, RateLimitError>;

    /// Check if the limiter is healthy (e.g., Redis connection is alive)
    async fn is_healthy(&self) -> bool;
}

/// Key generator for rate limiting
pub struct KeyGenerator;

impl KeyGenerator {
    /// Generate a hashed key for an IP address
    pub fn ip_key(prefix: &str, ip: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(ip.as_bytes());
        let hash = hex::encode(hasher.finalize());
        format!("{}:{}", prefix, &hash[..16]) // Use first 16 chars of hash
    }

    /// Generate a hashed key for an email address
    pub fn email_key(prefix: &str, email: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(email.to_lowercase().as_bytes());
        let hash = hex::encode(hasher.finalize());
        format!("{}:{}", prefix, &hash[..16])
    }

    /// Generate a key for a user ID (no hashing needed for UUIDs)
    pub fn user_key(prefix: &str, user_id: &str) -> String {
        format!("{}:{}", prefix, user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_result_allowed() {
        let result = RateLimitResult::allowed(10, 5, 1704312345);
        assert!(result.allowed);
        assert_eq!(result.limit, 10);
        assert_eq!(result.remaining, 5);
    }

    #[test]
    fn test_rate_limit_result_denied() {
        let result = RateLimitResult::denied(10, 1704312345);
        assert!(!result.allowed);
        assert_eq!(result.limit, 10);
        assert_eq!(result.remaining, 0);
    }

    #[test]
    fn test_key_generator_ip() {
        let key1 = KeyGenerator::ip_key("rate_limit:login:ip", "192.168.1.1");
        let key2 = KeyGenerator::ip_key("rate_limit:login:ip", "192.168.1.1");
        let key3 = KeyGenerator::ip_key("rate_limit:login:ip", "192.168.1.2");

        assert_eq!(key1, key2); // Same IP should produce same key
        assert_ne!(key1, key3); // Different IP should produce different key
        assert!(key1.starts_with("rate_limit:login:ip:"));
    }

    #[test]
    fn test_key_generator_email() {
        let key1 = KeyGenerator::email_key("rate_limit:forgot", "User@Example.COM");
        let key2 = KeyGenerator::email_key("rate_limit:forgot", "user@example.com");

        assert_eq!(key1, key2); // Case-insensitive
    }

    #[test]
    fn test_key_generator_user() {
        let key =
            KeyGenerator::user_key("rate_limit:refresh", "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(key, "rate_limit:refresh:550e8400-e29b-41d4-a716-446655440000");
    }
}
