use crate::domains::auth::dto::password_reset_dto::{
    ForgotPasswordResp, ResetPasswordResp, ValidateResetTokenResp,
};
use async_trait::async_trait;
use shared_error::AppError;
use uuid::Uuid;

/// Service trait for password reset operations
///
/// Implements the complete forgot-password flow with security best practices:
/// - Timing-safe responses to prevent email enumeration
/// - Rate limiting to prevent abuse
/// - Single-use tokens with short expiration
/// - Session invalidation after password change
/// - Comprehensive audit logging
#[async_trait]
pub trait PasswordResetService: Send + Sync {
    /// Request a password reset (forgot-password)
    ///
    /// Generates a reset token and sends email if the user exists.
    /// ALWAYS returns success to prevent email enumeration.
    /// Rate limited: max 3 requests per hour per email.
    ///
    /// # Arguments
    /// * `email` - Email address to send reset link to
    /// * `tenant_id` - Optional tenant ID for multi-tenant isolation
    /// * `ip_address` - Client IP address for audit logging
    /// * `user_agent` - Client user agent for audit logging
    ///
    /// # Returns
    /// * `Ok(ForgotPasswordResp)` - Always returns success (timing-safe)
    async fn request_password_reset(
        &self,
        email: &str,
        tenant_id: Option<Uuid>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<ForgotPasswordResp, AppError>;

    /// Reset password using token
    ///
    /// Validates the token, updates the password, and invalidates all sessions.
    /// Token is marked as used after successful reset (single-use).
    ///
    /// # Arguments
    /// * `token` - Plaintext reset token from email link
    /// * `new_password` - New password (will be validated for strength)
    /// * `ip_address` - Client IP address for audit logging
    /// * `user_agent` - Client user agent for audit logging
    ///
    /// # Returns
    /// * `Ok(ResetPasswordResp)` - Password reset successful
    /// * `Err(AppError::NotFound)` - Token not found or expired
    /// * `Err(AppError::ValidationError)` - Token already used or password too weak
    async fn reset_password(
        &self,
        token: &str,
        new_password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<ResetPasswordResp, AppError>;

    /// Validate a reset token without using it
    ///
    /// Used by frontend to check if token is valid before showing reset form.
    /// Does not consume the token.
    ///
    /// # Arguments
    /// * `token` - Plaintext reset token to validate
    ///
    /// # Returns
    /// * `Ok(ValidateResetTokenResp)` - Token validation result
    async fn validate_reset_token(&self, token: &str) -> Result<ValidateResetTokenResp, AppError>;

    /// Cleanup expired tokens
    ///
    /// Should be called periodically to remove old tokens from the database.
    /// Typically run as a background job.
    ///
    /// # Returns
    /// * Number of tokens deleted
    async fn cleanup_expired_tokens(&self) -> Result<u64, AppError>;
}
