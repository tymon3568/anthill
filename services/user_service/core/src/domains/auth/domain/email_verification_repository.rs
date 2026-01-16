use super::model::EmailVerificationToken;
use async_trait::async_trait;
use shared_error::AppError;
use uuid::Uuid;

/// Email Verification Repository trait
///
/// Defines the interface for email verification token data access operations.
#[async_trait]
pub trait EmailVerificationRepository: Send + Sync {
    /// Create a new verification token
    async fn create(
        &self,
        token: &EmailVerificationToken,
    ) -> Result<EmailVerificationToken, AppError>;

    /// Find verification token by hash
    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<EmailVerificationToken>, AppError>;

    /// Find pending (unverified) token by user ID
    async fn find_pending_by_user_id(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Option<EmailVerificationToken>, AppError>;

    /// Mark token as verified
    async fn mark_as_verified(
        &self,
        token_id: Uuid,
        verified_from_ip: Option<String>,
        verified_from_user_agent: Option<String>,
    ) -> Result<(), AppError>;

    /// Increment verification attempt counter
    async fn increment_attempts(&self, token_id: Uuid) -> Result<(), AppError>;

    /// Delete expired tokens (cleanup job)
    async fn delete_expired(&self) -> Result<u64, AppError>;

    /// Count recent unverified tokens for email (for rate limiting resends)
    async fn count_recent_tokens_for_email(
        &self,
        email: &str,
        tenant_id: Uuid,
        since_minutes: i64,
    ) -> Result<i64, AppError>;

    /// Invalidate all pending tokens for a user (when they verify or change email)
    async fn invalidate_all_for_user(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<u64, AppError>;
}
