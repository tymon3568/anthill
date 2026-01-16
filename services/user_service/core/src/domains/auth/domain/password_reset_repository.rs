use super::model::{PasswordResetAudit, PasswordResetToken};
use async_trait::async_trait;
use shared_error::AppError;
use uuid::Uuid;

/// Password Reset Repository trait
///
/// Defines the interface for password reset token data access operations.
/// Implements security best practices: tokens are hashed, single-use, and time-limited.
#[async_trait]
pub trait PasswordResetRepository: Send + Sync {
    /// Create a new password reset token
    ///
    /// Stores the hashed token with metadata. The plaintext token should only exist
    /// in memory and be sent via email.
    async fn create(&self, token: &PasswordResetToken) -> Result<PasswordResetToken, AppError>;

    /// Find password reset token by hash
    ///
    /// Used to validate a reset token from the user's email link.
    /// Returns None if token doesn't exist or has been used.
    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<PasswordResetToken>, AppError>;

    /// Find pending (unused, not expired) token for a user
    ///
    /// Used to check if user already has an active reset token.
    async fn find_pending_by_user_id(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Option<PasswordResetToken>, AppError>;

    /// Mark token as used
    ///
    /// Called after successful password reset. Sets used_at timestamp
    /// to enforce single-use.
    async fn mark_as_used(
        &self,
        token_id: Uuid,
        reset_from_ip: Option<String>,
        reset_from_user_agent: Option<String>,
    ) -> Result<(), AppError>;

    /// Count recent tokens for a user (for rate limiting)
    ///
    /// Returns the number of reset tokens created in the last N minutes.
    async fn count_recent_tokens_for_user(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        since_minutes: i64,
    ) -> Result<i64, AppError>;

    /// Delete expired tokens (cleanup job)
    ///
    /// Removes tokens that have expired and can no longer be used.
    /// Returns the number of deleted tokens.
    async fn delete_expired(&self) -> Result<u64, AppError>;

    /// Invalidate all pending tokens for a user
    ///
    /// Called when a new reset is requested to invalidate old tokens,
    /// or when password is successfully reset.
    async fn invalidate_all_for_user(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<u64, AppError>;

    // =========================================================================
    // Audit log methods
    // =========================================================================

    /// Log a password reset event
    ///
    /// Records all reset-related events for security audit trail.
    async fn log_audit_event(&self, audit: &PasswordResetAudit) -> Result<(), AppError>;

    /// Query audit events for an email (rate limiting support)
    ///
    /// Returns count of events of a specific type in the last N minutes.
    async fn count_audit_events_for_email(
        &self,
        email: &str,
        event_type: &str,
        since_minutes: i64,
    ) -> Result<i64, AppError>;
}
