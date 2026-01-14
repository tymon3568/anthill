//! Simple in-memory rate limiter for invitation acceptance endpoint
//!
//! This is a temporary implementation until the full rate limiting infrastructure
//! from task_03.06.03_rate_limiting.md is implemented.
//!
//! Limits: 5 attempts per 15 minutes per IP address

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct InvitationRateLimitConfig {
    /// Maximum attempts per IP
    pub max_attempts: u32,
    /// Time window in seconds
    pub window_seconds: u64,
}

impl Default for InvitationRateLimitConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            window_seconds: 900, // 15 minutes
        }
    }
}

/// Rate limit result
#[derive(Debug)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Remaining attempts
    pub remaining: u32,
    /// Seconds until reset
    pub reset_in_seconds: u64,
}

/// Simple in-memory rate limiter
#[derive(Debug, Clone)]
pub struct InvitationRateLimiter {
    /// Configuration
    config: InvitationRateLimitConfig,
    /// IP -> list of attempt timestamps
    attempts: Arc<RwLock<HashMap<String, Vec<SystemTime>>>>,
}

impl InvitationRateLimiter {
    pub fn new(config: InvitationRateLimitConfig) -> Self {
        Self {
            config,
            attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if request from IP is allowed
    pub fn check_rate_limit(&self, ip: &str) -> RateLimitResult {
        let now = SystemTime::now();
        let window_start = now - Duration::from_secs(self.config.window_seconds);

        let mut attempts = self.attempts.write().unwrap();

        // Get or create entry for this IP
        let timestamps = attempts.entry(ip.to_string()).or_insert_with(Vec::new);

        // Remove timestamps outside the window
        timestamps.retain(|&t| t > window_start);

        let current_attempts = timestamps.len() as u32;

        if current_attempts >= self.config.max_attempts {
            // Calculate reset time (when oldest timestamp expires)
            let oldest_timestamp = timestamps.first().copied().unwrap_or(now);
            let reset_time = oldest_timestamp + Duration::from_secs(self.config.window_seconds);
            let reset_in_seconds = reset_time
                .duration_since(now)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();

            RateLimitResult {
                allowed: false,
                remaining: 0,
                reset_in_seconds,
            }
        } else {
            // Record this attempt
            timestamps.push(now);

            // Clean up very old entries periodically (every 100 attempts)
            if timestamps.len() % 100 == 0 {
                self.cleanup_old_entries(&mut attempts);
            }

            RateLimitResult {
                allowed: true,
                remaining: self.config.max_attempts - current_attempts - 1,
                reset_in_seconds: self.config.window_seconds,
            }
        }
    }

    /// Clean up entries that are completely outside any window
    fn cleanup_old_entries(&self, attempts: &mut HashMap<String, Vec<SystemTime>>) {
        let cutoff = SystemTime::now() - Duration::from_secs(self.config.window_seconds * 2);

        attempts.retain(|_, timestamps| {
            timestamps.retain(|&t| t > cutoff);
            !timestamps.is_empty()
        });
    }

    /// Get current attempt count for an IP (for testing)
    #[cfg(test)]
    pub fn get_attempt_count(&self, ip: &str) -> usize {
        let attempts = self.attempts.read().unwrap();
        attempts.get(ip).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for InvitationRateLimiter {
    fn default() -> Self {
        Self::new(InvitationRateLimitConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_rate_limit_allows_initial_requests() {
        let limiter = InvitationRateLimiter::default();
        let ip = "192.168.1.1";

        for i in 0..5 {
            let result = limiter.check_rate_limit(ip);
            assert!(result.allowed, "Request {} should be allowed", i);
            assert_eq!(result.remaining, 4 - i);
        }
    }

    #[test]
    fn test_rate_limit_blocks_after_limit() {
        let limiter = InvitationRateLimiter::default();
        let ip = "192.168.1.1";

        // Use up all attempts
        for _ in 0..5 {
            let result = limiter.check_rate_limit(ip);
            assert!(result.allowed);
        }

        // Next request should be blocked
        let result = limiter.check_rate_limit(ip);
        assert!(!result.allowed);
        assert_eq!(result.remaining, 0);
    }

    #[test]
    fn test_rate_limit_resets_after_window() {
        let config = InvitationRateLimitConfig {
            max_attempts: 2,
            window_seconds: 1, // 1 second for testing
        };
        let limiter = InvitationRateLimiter::new(config);
        let ip = "192.168.1.1";

        // Use up attempts
        for _ in 0..2 {
            let result = limiter.check_rate_limit(ip);
            assert!(result.allowed);
        }

        // Should be blocked
        let result = limiter.check_rate_limit(ip);
        assert!(!result.allowed);

        // Wait for window to expire
        thread::sleep(Duration::from_secs(2));

        // Should allow again
        let result = limiter.check_rate_limit(ip);
        assert!(result.allowed);
    }

    #[test]
    fn test_different_ips_isolated() {
        let limiter = InvitationRateLimiter::default();
        let ip1 = "192.168.1.1";
        let ip2 = "192.168.1.2";

        // IP1 uses up attempts
        for _ in 0..5 {
            let result = limiter.check_rate_limit(ip1);
            assert!(result.allowed);
        }

        // IP2 should still be allowed
        let result = limiter.check_rate_limit(ip2);
        assert!(result.allowed);
    }
}
