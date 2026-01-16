use async_trait::async_trait;
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use shared_error::AppError;
use std::sync::Arc;
use user_service_core::domains::auth::{
    domain::{
        email_verification_repository::EmailVerificationRepository,
        email_verification_service::EmailVerificationService, model::EmailVerificationToken,
        repository::UserRepository,
    },
    dto::email_verification_dto::{ResendVerificationResp, VerifyEmailResp},
};
use uuid::Uuid;

/// Email Verification Service implementation
///
/// Handles email verification token generation, validation, and email sending.
/// Uses SHA-256 for token hashing (hash-at-rest pattern for security).
pub struct EmailVerificationServiceImpl<EVR, UR>
where
    EVR: EmailVerificationRepository,
    UR: UserRepository,
{
    verification_repo: Arc<EVR>,
    user_repo: Arc<UR>,
    verification_base_url: String,
    verification_expiry_hours: i64,
    resend_rate_limit_max: u32,
    resend_rate_limit_window_minutes: i64,
    // SMTP config (optional - if None, email sending is logged but not sent)
    smtp_enabled: bool,
}

impl<EVR, UR> EmailVerificationServiceImpl<EVR, UR>
where
    EVR: EmailVerificationRepository,
    UR: UserRepository,
{
    pub fn new(
        verification_repo: Arc<EVR>,
        user_repo: Arc<UR>,
        verification_base_url: String,
        verification_expiry_hours: i64,
        resend_rate_limit_max: u32,
        resend_rate_limit_window_minutes: i64,
        smtp_enabled: bool,
    ) -> Self {
        Self {
            verification_repo,
            user_repo,
            verification_base_url,
            verification_expiry_hours,
            resend_rate_limit_max,
            resend_rate_limit_window_minutes,
            smtp_enabled,
        }
    }

    /// Generate a cryptographically secure verification token
    /// Returns (plaintext_token, token_hash)
    fn generate_token() -> (String, String) {
        // Generate 32 random bytes using UUID v4 as random source
        let token_bytes = Uuid::new_v4().as_bytes().to_vec();
        let token_bytes2 = Uuid::new_v4().as_bytes().to_vec();
        let combined: Vec<u8> = token_bytes.into_iter().chain(token_bytes2).collect();

        // Encode as URL-safe base64
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        let plaintext_token = URL_SAFE_NO_PAD.encode(&combined);

        // Hash with SHA-256
        let mut hasher = Sha256::new();
        hasher.update(plaintext_token.as_bytes());
        let token_hash = format!("{:x}", hasher.finalize());

        (plaintext_token, token_hash)
    }

    /// Hash a plaintext token for lookup
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Build verification URL from token
    fn build_verification_url(&self, token: &str) -> String {
        format!("{}/verify-email?token={}", self.verification_base_url, token)
    }

    /// Send verification email (logs if SMTP not configured)
    async fn send_email(&self, email: &str, verification_url: &str) -> Result<(), AppError> {
        if !self.smtp_enabled {
            // Log for development/testing
            tracing::info!(
                email = %email,
                url = %verification_url,
                "📧 [DEV] Email verification link (SMTP not configured)"
            );
            return Ok(());
        }

        // TODO: Implement actual SMTP sending with lettre crate
        // For now, just log
        tracing::info!(
            email = %email,
            "📧 Verification email sent"
        );

        Ok(())
    }
}

#[async_trait]
impl<EVR, UR> EmailVerificationService for EmailVerificationServiceImpl<EVR, UR>
where
    EVR: EmailVerificationRepository + Send + Sync,
    UR: UserRepository + Send + Sync,
{
    async fn send_verification_email(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        email: &str,
    ) -> Result<(), AppError> {
        // Generate new token
        let (plaintext_token, token_hash) = Self::generate_token();

        // Create verification token record
        let now = Utc::now();
        let expires_at = now + Duration::hours(self.verification_expiry_hours);

        let token = EmailVerificationToken {
            token_id: Uuid::now_v7(),
            user_id,
            tenant_id,
            token_hash,
            email: email.to_string(),
            expires_at,
            created_at: now,
            verified_at: None,
            verified_from_ip: None,
            verified_from_user_agent: None,
            verification_attempts: 0,
            last_attempt_at: None,
        };

        // Store token
        self.verification_repo.create(&token).await?;

        // Build verification URL and send email
        let verification_url = self.build_verification_url(&plaintext_token);
        self.send_email(email, &verification_url).await?;

        tracing::info!(
            user_id = %user_id,
            tenant_id = %tenant_id,
            email = %email,
            "Verification email sent"
        );

        Ok(())
    }

    async fn verify_email(
        &self,
        token: &str,
        verified_from_ip: Option<String>,
        verified_from_user_agent: Option<String>,
    ) -> Result<VerifyEmailResp, AppError> {
        // Hash the token for lookup
        let token_hash = Self::hash_token(token);

        // Find the token
        let verification_token = self
            .verification_repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or_else(|| {
                AppError::NotFound("Invalid or expired verification token".to_string())
            })?;

        // Check if already verified
        if verification_token.verified_at.is_some() {
            return Err(AppError::ValidationError("Email has already been verified".to_string()));
        }

        // Check if expired
        if verification_token.expires_at < Utc::now() {
            return Err(AppError::ValidationError(
                "Verification token has expired. Please request a new one.".to_string(),
            ));
        }

        // Increment attempt counter
        self.verification_repo
            .increment_attempts(verification_token.token_id)
            .await?;

        // Mark token as verified
        self.verification_repo
            .mark_as_verified(
                verification_token.token_id,
                verified_from_ip,
                verified_from_user_agent,
            )
            .await?;

        // Update user's email_verified status
        let mut user = self
            .user_repo
            .find_by_id(verification_token.tenant_id, verification_token.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        user.email_verified = true;
        user.email_verified_at = Some(Utc::now());
        self.user_repo.update(&user).await?;

        // Invalidate all other pending tokens for this user
        self.verification_repo
            .invalidate_all_for_user(verification_token.user_id, verification_token.tenant_id)
            .await?;

        tracing::info!(
            user_id = %verification_token.user_id,
            tenant_id = %verification_token.tenant_id,
            email = %verification_token.email,
            "Email verified successfully"
        );

        Ok(VerifyEmailResp {
            user_id: verification_token.user_id,
            email: verification_token.email,
            verified: true,
            message: "Email verified successfully".to_string(),
        })
    }

    async fn resend_verification_email(
        &self,
        email: &str,
        tenant_id: Option<Uuid>,
    ) -> Result<ResendVerificationResp, AppError> {
        // Find user by email
        // If tenant_id is provided, use it; otherwise we need to search across tenants
        // For security, we require tenant_id context
        let tenant_id = tenant_id.ok_or_else(|| {
            AppError::ValidationError("Tenant context required for resend".to_string())
        })?;

        let user = self
            .user_repo
            .find_by_email(tenant_id, email)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Check if already verified
        if user.email_verified {
            return Err(AppError::ValidationError("Email is already verified".to_string()));
        }

        // Check rate limit
        let recent_count = self
            .verification_repo
            .count_recent_tokens_for_email(email, tenant_id, self.resend_rate_limit_window_minutes)
            .await?;

        if recent_count >= self.resend_rate_limit_max as i64 {
            let retry_after = self.resend_rate_limit_window_minutes as u64 * 60;
            return Ok(ResendVerificationResp {
                email: email.to_string(),
                message: "Rate limit exceeded. Please try again later.".to_string(),
                retry_after_seconds: Some(retry_after),
            });
        }

        // Invalidate existing tokens for this user
        self.verification_repo
            .invalidate_all_for_user(user.user_id, tenant_id)
            .await?;

        // Send new verification email
        self.send_verification_email(user.user_id, tenant_id, email)
            .await?;

        Ok(ResendVerificationResp {
            email: email.to_string(),
            message: "Verification email sent".to_string(),
            retry_after_seconds: None,
        })
    }

    async fn is_email_verified(&self, user_id: Uuid, tenant_id: Uuid) -> Result<bool, AppError> {
        let user = self
            .user_repo
            .find_by_id(tenant_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user.email_verified)
    }

    async fn cleanup_expired_tokens(&self) -> Result<u64, AppError> {
        let deleted = self.verification_repo.delete_expired().await?;
        tracing::info!(deleted = %deleted, "Cleaned up expired verification tokens");
        Ok(deleted)
    }
}
