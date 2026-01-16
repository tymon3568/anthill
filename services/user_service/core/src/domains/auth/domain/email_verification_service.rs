use crate::domains::auth::dto::email_verification_dto::{ResendVerificationResp, VerifyEmailResp};
use async_trait::async_trait;
use shared_error::AppError;
use uuid::Uuid;

/// Service trait for email verification operations
///
/// Handles verification token generation, email sending, and verification confirmation.
/// All operations are tenant-scoped for multi-tenant isolation.
#[async_trait]
pub trait EmailVerificationService: Send + Sync {
    /// Create and send a verification email for a newly registered user
    ///
    /// Called automatically during user registration to send verification email.
    /// Creates a verification token, stores the hash, and sends email with the plaintext token.
    ///
    /// # Arguments
    /// * `user_id` - User who needs to verify their email
    /// * `tenant_id` - Tenant the user belongs to
    /// * `email` - Email address to verify and send the verification link to
    ///
    /// # Returns
    /// * `Ok(())` - Verification email sent successfully
    /// * `Err(AppError)` - Failed to create token or send email
    async fn send_verification_email(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        email: &str,
    ) -> Result<(), AppError>;

    /// Verify an email using the verification token
    ///
    /// Validates the token, marks the email as verified, and updates the user record.
    /// Token is invalidated after successful verification (single-use).
    ///
    /// # Arguments
    /// * `token` - Plaintext verification token from email link
    /// * `verified_from_ip` - Optional IP address of the verifying client
    /// * `verified_from_user_agent` - Optional User-Agent of the verifying client
    ///
    /// # Returns
    /// * `Ok(VerifyEmailResp)` - Email verified successfully
    /// * `Err(AppError::NotFound)` - Token not found or expired
    /// * `Err(AppError::ValidationError)` - Token already used
    async fn verify_email(
        &self,
        token: &str,
        verified_from_ip: Option<String>,
        verified_from_user_agent: Option<String>,
    ) -> Result<VerifyEmailResp, AppError>;

    /// Resend verification email to user
    ///
    /// Rate-limited to prevent email spam. Creates a new token and invalidates
    /// any existing pending tokens for the user.
    ///
    /// # Arguments
    /// * `email` - Email address to resend verification to
    /// * `tenant_id` - Tenant context (optional, looks up from email if not provided)
    ///
    /// # Returns
    /// * `Ok(ResendVerificationResp)` - Email resent successfully or rate limited info
    /// * `Err(AppError::NotFound)` - User not found
    /// * `Err(AppError::ValidationError)` - Email already verified
    async fn resend_verification_email(
        &self,
        email: &str,
        tenant_id: Option<Uuid>,
    ) -> Result<ResendVerificationResp, AppError>;

    /// Check if a user's email is verified
    ///
    /// # Arguments
    /// * `user_id` - User ID to check
    /// * `tenant_id` - Tenant the user belongs to
    ///
    /// # Returns
    /// * `Ok(true)` - Email is verified
    /// * `Ok(false)` - Email is not verified
    async fn is_email_verified(&self, user_id: Uuid, tenant_id: Uuid) -> Result<bool, AppError>;

    /// Cleanup expired verification tokens
    ///
    /// Should be called periodically to remove old tokens from the database.
    ///
    /// # Returns
    /// * Number of tokens deleted
    async fn cleanup_expired_tokens(&self) -> Result<u64, AppError>;
}
