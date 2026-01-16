use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Password Reset DTOs
// ============================================================================

/// Request to initiate password reset (forgot-password)
///
/// This endpoint always returns success to prevent email enumeration.
/// If the email exists, a reset email will be sent.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ForgotPasswordReq {
    /// Email address to send reset link to
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "user@example.com")]
    pub email: String,
}

/// Request to reset password with token
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordReq {
    /// Reset token from email link
    #[validate(length(min = 32, max = 128, message = "Invalid token format"))]
    #[schema(example = "abc123def456...")]
    pub token: String,

    /// New password (must meet strength requirements)
    #[validate(length(min = 8, max = 128, message = "Password must be 8-128 characters"))]
    #[schema(example = "NewSecurePassword123!")]
    pub new_password: String,

    /// Confirm new password (must match new_password)
    #[validate(length(min = 8, max = 128))]
    #[schema(example = "NewSecurePassword123!")]
    pub confirm_password: String,
}

/// Request to validate token without resetting (optional: check before showing form)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ValidateResetTokenReq {
    /// Reset token to validate
    #[validate(length(min = 32, max = 128))]
    #[schema(example = "abc123def456...")]
    pub token: String,
}

/// Response for forgot-password request
///
/// Always returns success to prevent email enumeration.
#[derive(Debug, Serialize, ToSchema)]
pub struct ForgotPasswordResp {
    /// Message (always indicates success)
    pub message: String,

    /// Email address (masked for security)
    pub email_masked: String,
}

/// Response for successful password reset
#[derive(Debug, Serialize, ToSchema)]
pub struct ResetPasswordResp {
    /// User ID whose password was reset
    pub user_id: Uuid,

    /// Success message
    pub message: String,

    /// Number of sessions invalidated
    pub sessions_invalidated: u64,
}

/// Response for token validation
#[derive(Debug, Serialize, ToSchema)]
pub struct ValidateResetTokenResp {
    /// Whether the token is valid
    pub valid: bool,

    /// User email (masked) if token is valid
    pub email_masked: Option<String>,

    /// Expiration time (ISO 8601)
    pub expires_at: Option<String>,

    /// Error message if invalid
    pub error: Option<String>,
}

/// Helper function to mask email for display
/// e.g., "user@example.com" -> "u***@e*****.com"
pub fn mask_email(email: &str) -> String {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return "***@***.***".to_string();
    }

    let local = parts[0];
    let domain = parts[1];

    let masked_local = if local.len() <= 1 {
        "*".to_string()
    } else {
        format!("{}***", &local[..1])
    };

    let domain_parts: Vec<&str> = domain.split('.').collect();
    let masked_domain = if domain_parts.len() >= 2 {
        let name = domain_parts[0];
        let ext = domain_parts[domain_parts.len() - 1];
        let masked_name = if name.len() <= 1 {
            "*".to_string()
        } else {
            format!("{}*****", &name[..1])
        };
        format!("{}.{}", masked_name, ext)
    } else {
        "*****".to_string()
    };

    format!("{}@{}", masked_local, masked_domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("user@example.com"), "u***@e*****.com");
        assert_eq!(mask_email("test@domain.org"), "t***@d*****.org");
    }

    #[test]
    fn test_mask_email_short() {
        // Single char local/domain gets masked completely for privacy
        assert_eq!(mask_email("a@b.co"), "*@*.co");
        assert_eq!(mask_email("a@b.c"), "*@*.c");
    }

    #[test]
    fn test_mask_email_invalid() {
        assert_eq!(mask_email("invalid"), "***@***.***");
        assert_eq!(mask_email(""), "***@***.***");
    }
}
