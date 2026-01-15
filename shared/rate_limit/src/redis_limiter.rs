//! Redis-based rate limiter implementation using sliding window

use crate::limiter::{RateLimitError, RateLimitResult, RateLimiter};
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error};

/// Redis-based rate limiter using sliding window log algorithm
///
/// Uses Redis sorted sets to implement accurate sliding window rate limiting.
/// Suitable for distributed deployments where multiple instances need to share
/// rate limit state.
#[derive(Clone)]
pub struct RedisRateLimiter {
    /// Redis connection manager
    connection: Arc<RwLock<Option<ConnectionManager>>>,
    /// Redis URL for reconnection
    #[allow(dead_code)]
    redis_url: String,
    /// Key prefix for all rate limit keys
    key_prefix: String,
}

impl RedisRateLimiter {
    /// Create a new Redis rate limiter
    pub async fn new(redis_url: &str) -> Result<Self, RateLimitError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        let connection = ConnectionManager::new(client)
            .await
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        Ok(Self {
            connection: Arc::new(RwLock::new(Some(connection))),
            redis_url: redis_url.to_string(),
            key_prefix: "rl".to_string(), // Short prefix to save memory
        })
    }

    /// Create with custom key prefix
    pub async fn with_prefix(redis_url: &str, prefix: &str) -> Result<Self, RateLimitError> {
        let mut limiter = Self::new(redis_url).await?;
        limiter.key_prefix = prefix.to_string();
        Ok(limiter)
    }

    /// Get current timestamp in milliseconds for finer granularity
    fn now_millis() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Get current timestamp in seconds
    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Build the full Redis key
    fn build_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    /// Try to reconnect to Redis
    #[allow(dead_code)]
    async fn reconnect(&self) -> Result<(), RateLimitError> {
        let client = redis::Client::open(self.redis_url.as_str())
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        let connection = ConnectionManager::new(client)
            .await
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        let mut conn_guard = self.connection.write().await;
        *conn_guard = Some(connection);
        Ok(())
    }
}

#[async_trait]
impl RateLimiter for RedisRateLimiter {
    async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: u32,
        window: Duration,
    ) -> Result<RateLimitResult, RateLimitError> {
        let full_key = self.build_key(key);
        let now = Self::now_millis();
        let window_millis = window.as_millis() as u64;
        let window_start = now.saturating_sub(window_millis);
        let reset_at = Self::now_secs() + window.as_secs();

        let conn_guard = self.connection.read().await;
        let mut conn = conn_guard
            .clone()
            .ok_or_else(|| RateLimitError::RedisError("No connection".to_string()))?;
        drop(conn_guard);

        // Use a Lua script for atomic sliding window operation
        let script = redis::Script::new(
            r#"
            local key = KEYS[1]
            local now = tonumber(ARGV[1])
            local window_start = tonumber(ARGV[2])
            local max_requests = tonumber(ARGV[3])
            local window_seconds = tonumber(ARGV[4])

            -- Remove old entries
            redis.call('ZREMRANGEBYSCORE', key, '-inf', window_start)

            -- Count current entries
            local count = redis.call('ZCARD', key)

            if count >= max_requests then
                return {0, count, 0}  -- Denied
            else
                -- Add new entry with current timestamp as both score and member
                redis.call('ZADD', key, now, now)
                redis.call('EXPIRE', key, window_seconds)
                return {1, count + 1, max_requests - count - 1}  -- Allowed
            end
            "#,
        );

        let result: Vec<i64> = script
            .key(&full_key)
            .arg(now)
            .arg(window_start)
            .arg(max_requests)
            .arg(window.as_secs())
            .invoke_async(&mut conn)
            .await
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        if result[0] == 1 {
            Ok(RateLimitResult::allowed(max_requests, result[2] as u32, reset_at))
        } else {
            debug!("Rate limit exceeded for key {}: {} requests", key, result[1]);
            Ok(RateLimitResult::denied(max_requests, reset_at))
        }
    }

    async fn reset(&self, key: &str) -> Result<(), RateLimitError> {
        let full_key = self.build_key(key);

        let conn_guard = self.connection.read().await;
        let mut conn = conn_guard
            .clone()
            .ok_or_else(|| RateLimitError::RedisError("No connection".to_string()))?;
        drop(conn_guard);

        conn.del::<_, ()>(&full_key)
            .await
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        Ok(())
    }

    async fn get_count(&self, key: &str) -> Result<u32, RateLimitError> {
        let full_key = self.build_key(key);
        let now = Self::now_millis();
        let window_start = now.saturating_sub(3600 * 1000); // 1 hour default

        let conn_guard = self.connection.read().await;
        let mut conn = conn_guard
            .clone()
            .ok_or_else(|| RateLimitError::RedisError("No connection".to_string()))?;
        drop(conn_guard);

        // Remove old and count
        let _: () = conn
            .zrembyscore(&full_key, "-inf", window_start)
            .await
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        let count: u32 = conn
            .zcard(&full_key)
            .await
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        Ok(count)
    }

    async fn is_healthy(&self) -> bool {
        let conn_guard = self.connection.read().await;
        if let Some(mut conn) = conn_guard.clone() {
            drop(conn_guard);
            match redis::cmd("PING").query_async::<String>(&mut conn).await {
                Ok(response) => response == "PONG",
                Err(e) => {
                    error!("Redis health check failed: {}", e);
                    false
                },
            }
        } else {
            false
        }
    }
}

impl std::fmt::Debug for RedisRateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisRateLimiter")
            .field("redis_url", &"[REDACTED]")
            .field("key_prefix", &self.key_prefix)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests require a running Redis instance
    // Run with: REDIS_URL=redis://localhost:6379 cargo test --features redis-tests

    #[tokio::test]
    #[ignore = "Requires running Redis instance"]
    async fn test_redis_rate_limiting() {
        let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());
        let limiter = RedisRateLimiter::new(&redis_url).await.unwrap();
        let key = format!("test:redis:{}", uuid::Uuid::new_v4());
        let window = Duration::from_secs(60);

        // Reset first
        limiter.reset(&key).await.unwrap();

        // First 3 requests should be allowed
        for i in 0..3 {
            let result = limiter.check_rate_limit(&key, 3, window).await.unwrap();
            assert!(result.allowed, "Request {} should be allowed", i + 1);
        }

        // 4th request should be denied
        let result = limiter.check_rate_limit(&key, 3, window).await.unwrap();
        assert!(!result.allowed, "4th request should be denied");

        // Cleanup
        limiter.reset(&key).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "Requires running Redis instance"]
    async fn test_redis_health_check() {
        let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());
        let limiter = RedisRateLimiter::new(&redis_url).await.unwrap();
        assert!(limiter.is_healthy().await);
    }
}
