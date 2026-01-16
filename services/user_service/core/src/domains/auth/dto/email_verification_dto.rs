use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Email Verification DTOs
// ============================================================================

/// Request to resend verification email
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResendVerificationReq {
    /// Email address to resend verification to
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "user@example.com")]
    pub email: String,
}

/// Request to verify email with token
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct VerifyEmailReq {
    /// Verification token from email link
    #[validate(length(min = 32, max = 128))]
    #[schema(example = "abc123def456...")]
    pub token: String,
}

/// Response for successful email verification
#[derive(Debug, Serialize, ToSchema)]
pub struct VerifyEmailResp {
    pub user_id: Uuid,
    pub email: String,
    pub verified: bool,
    pub message: String,
}

/// Response for resend verification email
#[derive(Debug, Serialize, ToSchema)]
pub struct ResendVerificationResp {
    pub email: String,
    pub message: String,
    pub retry_after_seconds: Option<u64>, // If rate limited
}
