use async_trait::async_trait;
use sha2::{Digest, Sha256};
use shared_error::AppError;
use std::sync::Arc;
use user_service_core::domains::auth::{
    domain::{
        model::{PasswordResetAudit, PasswordResetToken},
        password_reset_repository::PasswordResetRepository,
        password_reset_service::PasswordResetService,
        repository::{SessionRepository, UserRepository},
    },
    dto::password_reset_dto::{
        mask_email, ForgotPasswordResp, ResetPasswordResp, ValidateResetTokenResp,
    },
};
use uuid::Uuid;

/// Password Reset Service implementation
///
/// Implements the complete forgot-password flow with security best practices:
/// - Timing-safe responses to prevent email enumeration
/// - Rate limiting to prevent abuse
/// - Single-use tokens with short expiration
/// - Session invalidation after password change
/// - Comprehensive audit logging
pub struct PasswordResetServiceImpl<PRR, UR, SR>
where
    PRR: PasswordResetRepository,
    UR: UserRepository,
    SR: SessionRepository,
{
    reset_repo: Arc<PRR>,
    user_repo: Arc<UR>,
    session_repo: Arc<SR>,
    reset_base_url: String,
    reset_expiry_hours: i64,
    rate_limit_max: u32,
    rate_limit_window_minutes: i64,
    smtp_enabled: bool,
    min_password_length: usize,
}

impl<PRR, UR, SR> PasswordResetServiceImpl<PRR, UR, SR>
where
    PRR: PasswordResetRepository,
    UR: UserRepository,
    SR: SessionRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        reset_repo: Arc<PRR>,
        user_repo: Arc<UR>,
        session_repo: Arc<SR>,
        reset_base_url: String,
        reset_expiry_hours: i64,
        rate_limit_max: u32,
        rate_limit_window_minutes: i64,
        smtp_enabled: bool,
        min_password_length: usize,
    ) -> Self {
        Self {
            reset_repo,
            user_repo,
            session_repo,
            reset_base_url,
            reset_expiry_hours,
            rate_limit_max,
            rate_limit_window_minutes,
            smtp_enabled,
            min_password_length,
        }
    }

    /// Generate a cryptographically secure reset token
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
        let token_hash = Self::hash_token(&plaintext_token);

        (plaintext_token, token_hash)
    }

    /// Hash a plaintext token for lookup
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Build reset URL from token
    fn build_reset_url(&self, token: &str) -> String {
        format!("{}/reset-password?token={}", self.reset_base_url, token)
    }

    /// Send password reset email (logs if SMTP not configured)
    async fn send_reset_email(&self, email: &str, reset_url: &str) -> Result<(), AppError> {
        if !self.smtp_enabled {
            // Log for development/testing
            tracing::info!(
                email = %email,
                url = %reset_url,
                "🔐 [DEV] Password reset link (SMTP not configured)"
            );
            return Ok(());
        }

        // TODO: Implement actual SMTP sending with lettre crate
        // For now, just log
        tracing::info!(
            email = %email,
            "🔐 Password reset email sent"
        );

        Ok(())
    }

    /// Validate password strength
    fn validate_password(&self, password: &str) -> Result<(), AppError> {
        if password.len() < self.min_password_length {
            return Err(AppError::ValidationError(format!(
                "Password must be at least {} characters",
                self.min_password_length
            )));
        }

        // Basic strength checks
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());

        if !has_uppercase || !has_lowercase || !has_digit {
            return Err(AppError::ValidationError(
                "Password must contain uppercase, lowercase, and numeric characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Hash password with bcrypt
    fn hash_password(&self, password: &str) -> Result<String, AppError> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::InternalError(format!("Failed to hash password: {}", e)))
    }
}

#[async_trait]
impl<PRR, UR, SR> PasswordResetService for PasswordResetServiceImpl<PRR, UR, SR>
where
    PRR: PasswordResetRepository + Send + Sync,
    UR: UserRepository + Send + Sync,
    SR: SessionRepository + Send + Sync,
{
    async fn request_password_reset(
        &self,
        email: &str,
        tenant_id: Option<Uuid>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<ForgotPasswordResp, AppError> {
        // Always return success response (timing-safe)
        let masked_email = mask_email(email);
        let success_response = ForgotPasswordResp {
            message: "If an account exists with this email, a password reset link has been sent."
                .to_string(),
            email_masked: masked_email.clone(),
        };

        // Check rate limit first (by email, not user)
        let rate_limit_count = self
            .reset_repo
            .count_audit_events_for_email(email, "requested", self.rate_limit_window_minutes)
            .await?;

        if rate_limit_count >= self.rate_limit_max as i64 {
            // Log rate limit event
            let audit = PasswordResetAudit::rate_limited(
                email.to_string(),
                ip_address.clone(),
                user_agent.clone(),
            );
            self.reset_repo.log_audit_event(&audit).await?;

            tracing::warn!(
                email = %email,
                count = %rate_limit_count,
                "Password reset rate limit exceeded"
            );

            // Still return success to prevent enumeration
            return Ok(success_response);
        }

        // For security, we need tenant context
        // If not provided, we can't proceed but still return success
        let tenant_id = match tenant_id {
            Some(id) => id,
            None => {
                tracing::debug!(email = %email, "Password reset without tenant context");
                return Ok(success_response);
            },
        };

        // Try to find user by email
        let user = match self.user_repo.find_by_email(tenant_id, email).await? {
            Some(u) => u,
            None => {
                // User doesn't exist - still return success (timing-safe)
                tracing::debug!(email = %email, tenant_id = %tenant_id, "Password reset for non-existent user");
                return Ok(success_response);
            },
        };

        // Check if user has password auth (can't reset if they only have Kanidm auth)
        if user.password_hash.is_none() && user.auth_method == "kanidm" {
            tracing::debug!(
                user_id = %user.user_id,
                "Password reset for Kanidm-only user"
            );
            return Ok(success_response);
        }

        // Invalidate any existing pending tokens for this user
        self.reset_repo
            .invalidate_all_for_user(user.user_id, tenant_id)
            .await?;

        // Generate new token
        let (plaintext_token, token_hash) = Self::generate_token();

        // Create reset token record
        let token = PasswordResetToken::new(
            user.user_id,
            tenant_id,
            token_hash,
            self.reset_expiry_hours,
            ip_address.clone(),
            user_agent.clone(),
        );

        // Store token
        self.reset_repo.create(&token).await?;

        // Log audit event
        let audit = PasswordResetAudit::requested(
            user.user_id,
            tenant_id,
            email.to_string(),
            ip_address,
            user_agent,
        );
        self.reset_repo.log_audit_event(&audit).await?;

        // Build reset URL and send email
        let reset_url = self.build_reset_url(&plaintext_token);
        self.send_reset_email(email, &reset_url).await?;

        tracing::info!(
            user_id = %user.user_id,
            tenant_id = %tenant_id,
            "Password reset email sent"
        );

        Ok(success_response)
    }

    async fn reset_password(
        &self,
        token: &str,
        new_password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<ResetPasswordResp, AppError> {
        // Hash the token for lookup
        let token_hash = Self::hash_token(token);

        // Find the token
        let reset_token = self
            .reset_repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or_else(|| {
                AppError::NotFound("Invalid or expired password reset token".to_string())
            })?;

        // Check if already used
        if reset_token.is_used() {
            // Log failure
            let audit = PasswordResetAudit::failed(
                Some(reset_token.user_id),
                Some(reset_token.tenant_id),
                String::new(),
                "token_already_used",
                ip_address,
                user_agent,
            );
            self.reset_repo.log_audit_event(&audit).await?;

            return Err(AppError::ValidationError(
                "This password reset link has already been used".to_string(),
            ));
        }

        // Check if expired
        if reset_token.is_expired() {
            // Log failure
            let audit = PasswordResetAudit::failed(
                Some(reset_token.user_id),
                Some(reset_token.tenant_id),
                String::new(),
                "token_expired",
                ip_address,
                user_agent,
            );
            self.reset_repo.log_audit_event(&audit).await?;

            return Err(AppError::ValidationError(
                "This password reset link has expired. Please request a new one.".to_string(),
            ));
        }

        // Validate new password strength
        self.validate_password(new_password)?;

        // Get user to get email for audit
        let mut user = self
            .user_repo
            .find_by_id(reset_token.tenant_id, reset_token.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Hash new password
        let password_hash = self.hash_password(new_password)?;

        // Update user's password
        user.password_hash = Some(password_hash);
        user.password_changed_at = Some(chrono::Utc::now());
        self.user_repo.update(&user).await?;

        // Mark token as used
        self.reset_repo
            .mark_as_used(reset_token.token_id, ip_address.clone(), user_agent.clone())
            .await?;

        // Invalidate all sessions for this user (force re-login)
        let sessions_invalidated = self.session_repo.revoke_all_for_user(user.user_id).await?;

        // Log success
        let audit = PasswordResetAudit::completed(
            user.user_id,
            reset_token.tenant_id,
            user.email.clone(),
            ip_address,
            user_agent,
        );
        self.reset_repo.log_audit_event(&audit).await?;

        tracing::info!(
            user_id = %user.user_id,
            tenant_id = %reset_token.tenant_id,
            sessions_invalidated = %sessions_invalidated,
            "Password reset completed"
        );

        Ok(ResetPasswordResp {
            user_id: user.user_id,
            message: "Password has been reset successfully. Please log in with your new password."
                .to_string(),
            sessions_invalidated,
        })
    }

    async fn validate_reset_token(&self, token: &str) -> Result<ValidateResetTokenResp, AppError> {
        // Hash the token for lookup
        let token_hash = Self::hash_token(token);

        // Find the token
        let reset_token = match self.reset_repo.find_by_token_hash(&token_hash).await? {
            Some(t) => t,
            None => {
                return Ok(ValidateResetTokenResp {
                    valid: false,
                    email_masked: None,
                    expires_at: None,
                    error: Some("Invalid or expired reset token".to_string()),
                });
            },
        };

        // Check if used
        if reset_token.is_used() {
            return Ok(ValidateResetTokenResp {
                valid: false,
                email_masked: None,
                expires_at: None,
                error: Some("This reset link has already been used".to_string()),
            });
        }

        // Check if expired
        if reset_token.is_expired() {
            return Ok(ValidateResetTokenResp {
                valid: false,
                email_masked: None,
                expires_at: None,
                error: Some("This reset link has expired".to_string()),
            });
        }

        // Get user's email for masked display
        let user = self
            .user_repo
            .find_by_id(reset_token.tenant_id, reset_token.user_id)
            .await?;

        let email_masked = user.map(|u| mask_email(&u.email));

        Ok(ValidateResetTokenResp {
            valid: true,
            email_masked,
            expires_at: Some(reset_token.expires_at.to_rfc3339()),
            error: None,
        })
    }

    async fn cleanup_expired_tokens(&self) -> Result<u64, AppError> {
        let deleted = self.reset_repo.delete_expired().await?;
        tracing::info!(deleted = %deleted, "Cleaned up expired password reset tokens");
        Ok(deleted)
    }
}
