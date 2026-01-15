//! In-memory rate limiter implementation using sliding window

use crate::limiter::{RateLimitError, RateLimitResult, RateLimiter};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Entry for tracking rate limit data
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// Timestamps of requests within the window
    timestamps: Vec<u64>,
    /// Window start time for cleanup
    window_start: u64,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            timestamps: Vec::new(),
            window_start: 0,
        }
    }

    /// Clean up old timestamps outside the window
    fn cleanup(&mut self, window_start: u64) {
        self.timestamps.retain(|&ts| ts >= window_start);
        self.window_start = window_start;
    }
}

/// In-memory rate limiter using sliding window log algorithm
///
/// This implementation is suitable for single-instance deployments or testing.
/// For distributed systems, use `RedisRateLimiter` instead.
#[derive(Debug)]
pub struct InMemoryRateLimiter {
    /// Store for rate limit entries
    store: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    /// Maximum entries before cleanup
    max_entries: usize,
}

impl InMemoryRateLimiter {
    /// Create a new in-memory rate limiter
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            max_entries: 10_000, // Default max entries
        }
    }

    /// Create with custom max entries limit
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            max_entries,
        }
    }

    /// Get current timestamp in seconds
    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Cleanup old entries to prevent memory bloat
    #[allow(dead_code)]
    async fn cleanup_old_entries(&self) {
        let mut store = self.store.write().await;
        if store.len() > self.max_entries {
            let now = Self::now_secs();
            // Remove entries that haven't been accessed recently (1 hour)
            store.retain(|_, entry| now - entry.window_start < 3600);
        }
    }
}

impl Default for InMemoryRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RateLimiter for InMemoryRateLimiter {
    async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: u32,
        window: Duration,
    ) -> Result<RateLimitResult, RateLimitError> {
        let now = Self::now_secs();
        let window_secs = window.as_secs();
        let window_start = now.saturating_sub(window_secs);
        let reset_at = now + window_secs;

        let mut store = self.store.write().await;

        let entry = store
            .entry(key.to_string())
            .or_insert_with(RateLimitEntry::new);

        // Cleanup old timestamps
        entry.cleanup(window_start);

        let current_count = entry.timestamps.len() as u32;

        if current_count >= max_requests {
            // Rate limit exceeded
            Ok(RateLimitResult::denied(max_requests, reset_at))
        } else {
            // Add current timestamp
            entry.timestamps.push(now);
            let remaining = max_requests.saturating_sub(current_count + 1);
            Ok(RateLimitResult::allowed(max_requests, remaining, reset_at))
        }
    }

    async fn reset(&self, key: &str) -> Result<(), RateLimitError> {
        let mut store = self.store.write().await;
        store.remove(key);
        Ok(())
    }

    async fn get_count(&self, key: &str) -> Result<u32, RateLimitError> {
        let now = Self::now_secs();
        let store = self.store.read().await;

        match store.get(key) {
            Some(entry) => {
                // Count only entries within a reasonable window (1 hour default)
                let window_start = now.saturating_sub(3600);
                let count = entry
                    .timestamps
                    .iter()
                    .filter(|&&ts| ts >= window_start)
                    .count();
                Ok(count as u32)
            },
            None => Ok(0),
        }
    }

    async fn is_healthy(&self) -> bool {
        true // In-memory is always healthy
    }
}

impl Clone for InMemoryRateLimiter {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
            max_entries: self.max_entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_rate_limiting() {
        let limiter = InMemoryRateLimiter::new();
        let key = "test:user:1";
        let window = Duration::from_secs(60);

        // First 3 requests should be allowed
        for i in 0..3 {
            let result = limiter.check_rate_limit(key, 3, window).await.unwrap();
            assert!(result.allowed, "Request {} should be allowed", i + 1);
            assert_eq!(result.remaining, 2 - i);
        }

        // 4th request should be denied
        let result = limiter.check_rate_limit(key, 3, window).await.unwrap();
        assert!(!result.allowed, "4th request should be denied");
        assert_eq!(result.remaining, 0);
    }

    #[tokio::test]
    async fn test_reset() {
        let limiter = InMemoryRateLimiter::new();
        let key = "test:reset";
        let window = Duration::from_secs(60);

        // Use up the limit
        for _ in 0..3 {
            limiter.check_rate_limit(key, 3, window).await.unwrap();
        }

        // Should be denied
        let result = limiter.check_rate_limit(key, 3, window).await.unwrap();
        assert!(!result.allowed);

        // Reset
        limiter.reset(key).await.unwrap();

        // Should be allowed again
        let result = limiter.check_rate_limit(key, 3, window).await.unwrap();
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_get_count() {
        let limiter = InMemoryRateLimiter::new();
        let key = "test:count";
        let window = Duration::from_secs(60);

        assert_eq!(limiter.get_count(key).await.unwrap(), 0);

        limiter.check_rate_limit(key, 10, window).await.unwrap();
        assert_eq!(limiter.get_count(key).await.unwrap(), 1);

        limiter.check_rate_limit(key, 10, window).await.unwrap();
        assert_eq!(limiter.get_count(key).await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_different_keys_independent() {
        let limiter = InMemoryRateLimiter::new();
        let window = Duration::from_secs(60);

        // Use up limit for key1
        for _ in 0..3 {
            limiter.check_rate_limit("key1", 3, window).await.unwrap();
        }

        // key2 should still be allowed
        let result = limiter.check_rate_limit("key2", 3, window).await.unwrap();
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_is_healthy() {
        let limiter = InMemoryRateLimiter::new();
        assert!(limiter.is_healthy().await);
    }
}
