//! Account lockout functionality for failed login attempts

use crate::limiter::{RateLimitError, RateLimiter};
use crate::memory_limiter::InMemoryRateLimiter;
use crate::middleware::SharedRateLimiter;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

/// Account lockout status
#[derive(Debug, Clone)]
pub struct LockoutStatus {
    /// Whether the account is currently locked
    pub is_locked: bool,
    /// Number of failed attempts
    pub failed_attempts: u32,
    /// Remaining lockout time in seconds (if locked)
    pub remaining_seconds: Option<u64>,
    /// Progressive delay to apply (in milliseconds)
    pub delay_ms: Option<u64>,
}

impl LockoutStatus {
    /// Create an unlocked status
    pub fn unlocked(failed_attempts: u32, delay_ms: Option<u64>) -> Self {
        Self {
            is_locked: false,
            failed_attempts,
            remaining_seconds: None,
            delay_ms,
        }
    }

    /// Create a locked status
    pub fn locked(failed_attempts: u32, remaining_seconds: u64) -> Self {
        Self {
            is_locked: true,
            failed_attempts,
            remaining_seconds: Some(remaining_seconds),
            delay_ms: None,
        }
    }
}

/// Account lockout manager
#[derive(Clone)]
pub struct AccountLockout {
    /// The underlying rate limiter
    limiter: Arc<SharedRateLimiter>,
    /// Maximum failed attempts before lockout
    threshold: u32,
    /// Lockout duration in seconds
    lockout_duration: u64,
    /// Enable progressive delays
    progressive_delays: bool,
}

impl AccountLockout {
    /// Create a new account lockout manager
    pub fn new(limiter: Arc<SharedRateLimiter>, threshold: u32, lockout_duration: u64) -> Self {
        Self {
            limiter,
            threshold,
            lockout_duration,
            progressive_delays: true,
        }
    }

    /// Create with in-memory limiter for testing
    pub fn in_memory(threshold: u32, lockout_duration: u64) -> Self {
        Self {
            limiter: Arc::new(SharedRateLimiter::InMemory(InMemoryRateLimiter::new())),
            threshold,
            lockout_duration,
            progressive_delays: true,
        }
    }

    /// Disable progressive delays
    pub fn without_progressive_delays(mut self) -> Self {
        self.progressive_delays = false;
        self
    }

    /// Build the lockout key for a user
    fn lockout_key(user_id: &str) -> String {
        format!("lockout:user:{}", user_id)
    }

    /// Build the failed attempts key for a user
    fn failed_key(user_id: &str) -> String {
        format!("failed_login:user:{}", user_id)
    }

    /// Calculate progressive delay based on failed attempts
    fn calculate_delay(&self, attempts: u32) -> Option<u64> {
        if !self.progressive_delays || attempts == 0 {
            return None;
        }

        // Progressive delay: 1s, 2s, 4s, 8s, 16s (capped at 16 seconds)
        // attempts: 1->1s, 2->2s, 3->4s, 4->8s, 5+->16s
        let delay_secs = 1u64 << (attempts.min(5) - 1);
        Some(delay_secs * 1000) // Return in milliseconds
    }

    /// Check if an account is locked
    pub async fn check_lockout(&self, user_id: &str) -> Result<LockoutStatus, RateLimitError> {
        let lockout_key = Self::lockout_key(user_id);
        let failed_key = Self::failed_key(user_id);

        // Check lockout status
        let lockout_count = match &*self.limiter {
            SharedRateLimiter::Redis(l) => l.get_count(&lockout_key).await?,
            SharedRateLimiter::InMemory(l) => l.get_count(&lockout_key).await?,
        };

        if lockout_count > 0 {
            // Account is locked - get actual remaining TTL
            let remaining = match &*self.limiter {
                SharedRateLimiter::Redis(l) => l.get_ttl(&lockout_key).await?,
                SharedRateLimiter::InMemory(l) => l.get_ttl(&lockout_key).await?,
            };
            // Use actual TTL if available, otherwise fall back to full duration
            let remaining_seconds = if remaining > 0 {
                remaining
            } else {
                self.lockout_duration
            };
            return Ok(LockoutStatus::locked(self.threshold, remaining_seconds));
        }

        // Get failed attempt count
        let failed_count = match &*self.limiter {
            SharedRateLimiter::Redis(l) => l.get_count(&failed_key).await?,
            SharedRateLimiter::InMemory(l) => l.get_count(&failed_key).await?,
        };

        let delay = self.calculate_delay(failed_count);
        Ok(LockoutStatus::unlocked(failed_count, delay))
    }

    /// Record a failed login attempt
    pub async fn record_failed_attempt(
        &self,
        user_id: &str,
    ) -> Result<LockoutStatus, RateLimitError> {
        let failed_key = Self::failed_key(user_id);

        // Increment failed attempts
        let result = self
            .limiter
            .check(&failed_key, self.threshold + 1, Duration::from_secs(self.lockout_duration))
            .await?;

        let failed_count = self.threshold + 1 - result.remaining;

        // Check if we've hit the threshold
        if failed_count >= self.threshold {
            // Lock the account
            let lockout_key = Self::lockout_key(user_id);
            self.limiter
                .check(&lockout_key, 1, Duration::from_secs(self.lockout_duration))
                .await?;

            info!("Account {} locked after {} failed attempts", user_id, failed_count);

            return Ok(LockoutStatus::locked(failed_count, self.lockout_duration));
        }

        let delay = self.calculate_delay(failed_count);
        warn!(
            "Failed login attempt {} for user {}, delay: {:?}ms",
            failed_count, user_id, delay
        );

        Ok(LockoutStatus::unlocked(failed_count, delay))
    }

    /// Record a successful login (reset failed attempts)
    pub async fn record_success(&self, user_id: &str) -> Result<(), RateLimitError> {
        let failed_key = Self::failed_key(user_id);
        let lockout_key = Self::lockout_key(user_id);

        self.limiter.reset(&failed_key).await?;
        self.limiter.reset(&lockout_key).await?;

        Ok(())
    }

    /// Manually unlock an account (admin action)
    pub async fn unlock_account(&self, user_id: &str) -> Result<(), RateLimitError> {
        self.record_success(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lockout_after_threshold() {
        let lockout = AccountLockout::in_memory(3, 60);
        let user_id = "test-user-1";

        // First two attempts should not lock
        for i in 1..=2 {
            let status = lockout.record_failed_attempt(user_id).await.unwrap();
            assert!(!status.is_locked, "Attempt {} should not lock", i);
            assert_eq!(status.failed_attempts, i);
        }

        // Third attempt should lock
        let status = lockout.record_failed_attempt(user_id).await.unwrap();
        assert!(status.is_locked, "Third attempt should lock");
        assert_eq!(status.failed_attempts, 3);
    }

    #[tokio::test]
    async fn test_check_lockout() {
        let lockout = AccountLockout::in_memory(2, 60);
        let user_id = "test-user-2";

        // Initially not locked
        let status = lockout.check_lockout(user_id).await.unwrap();
        assert!(!status.is_locked);
        assert_eq!(status.failed_attempts, 0);

        // Record failures to lock
        lockout.record_failed_attempt(user_id).await.unwrap();
        lockout.record_failed_attempt(user_id).await.unwrap();

        // Should now be locked
        let status = lockout.check_lockout(user_id).await.unwrap();
        assert!(status.is_locked);
    }

    #[tokio::test]
    async fn test_unlock_account() {
        let lockout = AccountLockout::in_memory(2, 60);
        let user_id = "test-user-3";

        // Lock the account
        lockout.record_failed_attempt(user_id).await.unwrap();
        lockout.record_failed_attempt(user_id).await.unwrap();

        let status = lockout.check_lockout(user_id).await.unwrap();
        assert!(status.is_locked);

        // Unlock
        lockout.unlock_account(user_id).await.unwrap();

        // Should be unlocked
        let status = lockout.check_lockout(user_id).await.unwrap();
        assert!(!status.is_locked);
        assert_eq!(status.failed_attempts, 0);
    }

    #[tokio::test]
    async fn test_progressive_delays() {
        let lockout = AccountLockout::in_memory(10, 60);
        let user_id = "test-user-4";

        // 1st attempt: 1s delay
        let status = lockout.record_failed_attempt(user_id).await.unwrap();
        assert_eq!(status.delay_ms, Some(1000));

        // 2nd attempt: 2s delay
        let status = lockout.record_failed_attempt(user_id).await.unwrap();
        assert_eq!(status.delay_ms, Some(2000));

        // 3rd attempt: 4s delay
        let status = lockout.record_failed_attempt(user_id).await.unwrap();
        assert_eq!(status.delay_ms, Some(4000));

        // 4th attempt: 8s delay
        let status = lockout.record_failed_attempt(user_id).await.unwrap();
        assert_eq!(status.delay_ms, Some(8000));

        // 5th+ attempts: capped at 16s
        let status = lockout.record_failed_attempt(user_id).await.unwrap();
        assert_eq!(status.delay_ms, Some(16000));
    }

    #[tokio::test]
    async fn test_success_resets_attempts() {
        let lockout = AccountLockout::in_memory(5, 60);
        let user_id = "test-user-5";

        // Record some failures
        lockout.record_failed_attempt(user_id).await.unwrap();
        lockout.record_failed_attempt(user_id).await.unwrap();

        let status = lockout.check_lockout(user_id).await.unwrap();
        assert_eq!(status.failed_attempts, 2);

        // Successful login
        lockout.record_success(user_id).await.unwrap();

        // Should be reset
        let status = lockout.check_lockout(user_id).await.unwrap();
        assert_eq!(status.failed_attempts, 0);
    }
}
