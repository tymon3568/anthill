use async_trait::async_trait;
use chrono::{Duration, Utc};
use shared_auth::enforcer::{add_role_for_user, SharedEnforcer};
use shared_error::AppError;
use tokio::task;
use tracing::{debug, info, warn};

use user_service_core::domains::auth::domain::{
    invitation_repository::InvitationRepository,
    invitation_service::InvitationService,
    model::{Invitation, InvitationStatus},
};
use user_service_core::domains::auth::utils::{
    invitation_utils::{generate_invite_token, hash_token},
    password_validator::validate_password_quick,
};
use user_service_core::domains::auth::{
    domain::{model::User, repository::UserRepository},
    dto::auth_dto::UserInfo,
};
use uuid::Uuid;

/// Implementation of InvitationService
pub struct InvitationServiceImpl<IR, UR>
where
    IR: InvitationRepository + Send + Sync,
    UR: UserRepository + Send + Sync,
{
    invitation_repo: IR,
    user_repo: UR,
    enforcer: SharedEnforcer,
    invitation_expiry_hours: i64,
    invitation_max_attempts: i32,
    invitation_max_per_admin_per_day: i32,
}

impl<IR, UR> InvitationServiceImpl<IR, UR>
where
    IR: InvitationRepository + Send + Sync,
    UR: UserRepository + Send + Sync,
{
    pub fn new(
        invitation_repo: IR,
        user_repo: UR,
        enforcer: SharedEnforcer,
        invitation_expiry_hours: i64,
        invitation_max_attempts: i32,
        invitation_max_per_admin_per_day: i32,
    ) -> Self {
        Self {
            invitation_repo,
            user_repo,
            enforcer,
            invitation_expiry_hours,
            invitation_max_attempts,
            invitation_max_per_admin_per_day,
        }
    }

    #[allow(dead_code)]
    fn user_to_user_info(&self, user: &User) -> UserInfo {
        UserInfo {
            id: user.user_id,
            email: user.email.clone(),
            full_name: user.full_name.clone(),
            tenant_id: user.tenant_id,
            role: user.role.clone(),
            created_at: user.created_at,
        }
    }
}

#[async_trait]
impl<IR, UR> InvitationService for InvitationServiceImpl<IR, UR>
where
    IR: InvitationRepository + Send + Sync,
    UR: UserRepository + Send + Sync,
{
    async fn create_invitation(
        &self,
        tenant_id: Uuid,
        email: &str,
        invited_role: &str,
        invited_by_user_id: Uuid,
        custom_message: Option<&str>,
        invited_from_ip: Option<&str>,
        invited_from_user_agent: Option<&str>,
    ) -> Result<(Invitation, String), AppError> {
        // Check daily invitation limit for admin
        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_end = today_start + Duration::days(1);
        let today_count = self
            .invitation_repo
            .count_created_by_user_today(
                invited_by_user_id,
                today_start.and_utc(),
                today_end.and_utc(),
            )
            .await?;
        if today_count >= self.invitation_max_per_admin_per_day as i64 {
            return Err(AppError::TooManyRequests(format!(
                "Daily invitation limit exceeded (max {} per day)",
                self.invitation_max_per_admin_per_day
            )));
        }

        // Check if pending invitation already exists for this email in tenant
        if let Some(_existing) = self
            .invitation_repo
            .find_pending_by_tenant_and_email(tenant_id, email)
            .await?
        {
            return Err(AppError::ValidationError(format!(
                "Pending invitation already exists for {} in this tenant",
                email
            )));
        }

        // Generate secure token
        let (plaintext_token, token_hash) = generate_invite_token();

        // Calculate expiry
        let expires_at = Utc::now() + Duration::hours(self.invitation_expiry_hours);

        // Create invitation
        let invitation = Invitation {
            invitation_id: Uuid::now_v7(),
            tenant_id,
            token_hash,
            email: email.to_string(),
            invited_role: invited_role.to_string(),
            invited_by_user_id,
            status: InvitationStatus::Pending,
            expires_at,
            accepted_at: None,
            accepted_user_id: None,
            invited_from_ip: invited_from_ip.map(|s| s.to_string()),
            invited_from_user_agent: invited_from_user_agent.map(|s| s.to_string()),
            accepted_from_ip: None,
            accepted_from_user_agent: None,
            accept_attempts: 0,
            last_attempt_at: None,
            custom_message: custom_message.map(|s| s.to_string()),
            metadata: sqlx::types::Json(serde_json::json!({})),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        // Save to database
        let saved_invitation = self.invitation_repo.create(&invitation).await?;

        // Audit log: invitation creation
        info!(
            invitation_id = %saved_invitation.invitation_id,
            tenant_id = %tenant_id,
            invited_by_user_id = %invited_by_user_id,
            invited_role = %invited_role,
            "invitation_created"
        );
        debug!(
            invitation_id = %saved_invitation.invitation_id,
            email = %email,
            invited_from_ip = ?invited_from_ip,
            "invitation_created_pii"
        );

        Ok((saved_invitation, plaintext_token))
    }

    async fn accept_invitation(
        &self,
        token: &str,
        password: &str,
        full_name: Option<&str>,
        accepted_from_ip: Option<&str>,
        accepted_from_user_agent: Option<&str>,
    ) -> Result<Invitation, AppError> {
        // Hash the token for lookup
        let token_hash = hash_token(token);

        // Find pending invitation
        let invitation = self
            .invitation_repo
            .find_pending_by_token_hash(&token_hash)
            .await?
            .ok_or_else(|| AppError::NotFound("Invalid or expired invitation".into()))?;

        // Check expiry first to avoid incrementing attempts on expired invitations
        if invitation.expires_at < Utc::now() {
            self.invitation_repo
                .mark_expired(invitation.tenant_id, invitation.invitation_id)
                .await?;
            info!(
                invitation_id = %invitation.invitation_id,
                tenant_id = %invitation.tenant_id,
                "invitation_expired_during_acceptance"
            );
            debug!(
                invitation_id = %invitation.invitation_id,
                email = %invitation.email,
                "invitation_expired_during_acceptance_pii"
            );
            return Err(AppError::Gone("Invitation has expired".into()));
        }

        // Atomically check and increment attempts (only for non-expired invitations)
        let increment_successful = self
            .invitation_repo
            .check_and_increment_accept_attempts(
                invitation.tenant_id,
                invitation.invitation_id,
                self.invitation_max_attempts,
            )
            .await?;

        if !increment_successful {
            warn!(
                invitation_id = %invitation.invitation_id,
                tenant_id = %invitation.tenant_id,
                "invitation_accept_too_many_attempts"
            );
            debug!(
                invitation_id = %invitation.invitation_id,
                email = %invitation.email,
                "invitation_accept_too_many_attempts_pii"
            );
            return Err(AppError::TooManyRequests("Too many acceptance attempts".into()));
        }

        // Check not already accepted
        if invitation.status != InvitationStatus::Pending {
            return Err(AppError::Conflict("Invitation already used".into()));
        }

        // Validate password strength
        let user_inputs = [
            invitation.email.as_str(),
            full_name.unwrap_or(""),
            &invitation.tenant_id.to_string(),
        ];
        validate_password_quick(password, &user_inputs)
            .map_err(|e| AppError::ValidationError(format!("Password validation failed: {}", e)))?;

        // Hash password (offload to blocking thread pool)
        let password = password.to_string(); // Clone for move into closure
        let password_hash =
            task::spawn_blocking(move || bcrypt::hash(&password, bcrypt::DEFAULT_COST))
                .await
                .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
                .map_err(|e| AppError::InternalError(format!("Failed to hash password: {}", e)))?;

        // Create user
        let user_id = Uuid::now_v7();
        let now = Utc::now();
        let user = User {
            user_id,
            tenant_id: invitation.tenant_id,
            email: invitation.email.clone(),
            password_hash: Some(password_hash),
            email_verified: true, // Invitation acceptance implies verification
            email_verified_at: Some(now),
            full_name: full_name.map(|s| s.to_string()),
            avatar_url: None,
            phone: None,
            role: invitation.invited_role.clone(),
            status: "active".to_string(),
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            password_changed_at: Some(now),
            kanidm_user_id: None,
            kanidm_synced_at: None,
            auth_method: "password".to_string(),
            migration_invited_at: None,
            migration_completed_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created_user = self.user_repo.create(&user).await?;

        // Mark invitation as accepted
        self.invitation_repo
            .mark_accepted(
                invitation.tenant_id,
                invitation.invitation_id,
                created_user.user_id,
                accepted_from_ip,
                accepted_from_user_agent,
            )
            .await?;

        // Add Casbin role
        let user_id_str = created_user.user_id.to_string();
        let tenant_id_str = invitation.tenant_id.to_string();
        add_role_for_user(&self.enforcer, &user_id_str, &invitation.invited_role, &tenant_id_str)
            .await
            .map_err(|e| {
                AppError::InternalError(format!("Failed to assign role to invited user: {}", e))
            })?;

        // Audit log: invitation acceptance
        info!(
            invitation_id = %invitation.invitation_id,
            tenant_id = %invitation.tenant_id,
            user_id = %created_user.user_id,
            "invitation_accepted"
        );
        debug!(
            invitation_id = %invitation.invitation_id,
            email = %invitation.email,
            accepted_from_ip = ?accepted_from_ip,
            accepted_from_user_agent = ?accepted_from_user_agent,
            "invitation_accepted_pii"
        );

        // Return updated invitation
        let mut updated_invitation = invitation;
        updated_invitation.status = InvitationStatus::Accepted;
        updated_invitation.accepted_at = Some(now);
        updated_invitation.accepted_user_id = Some(created_user.user_id);
        updated_invitation.accepted_from_ip = accepted_from_ip.map(|s| s.to_string());
        updated_invitation.accepted_from_user_agent =
            accepted_from_user_agent.map(|s| s.to_string());
        updated_invitation.updated_at = now;

        Ok(updated_invitation)
    }

    async fn get_invitation(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<Option<Invitation>, AppError> {
        self.invitation_repo
            .find_by_id(tenant_id, invitation_id)
            .await
    }

    async fn list_invitations(
        &self,
        tenant_id: Uuid,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Invitation>, AppError> {
        self.invitation_repo
            .list_by_tenant(tenant_id, status, limit, offset)
            .await
    }

    async fn count_invitations(
        &self,
        tenant_id: Uuid,
        status: Option<&str>,
    ) -> Result<i64, AppError> {
        self.invitation_repo
            .count_by_tenant(tenant_id, status)
            .await
    }

    async fn revoke_invitation(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<(), AppError> {
        // Use dedicated revoke method which enforces status = 'pending' check
        self.invitation_repo
            .revoke(tenant_id, invitation_id)
            .await?;

        // Audit log: invitation revocation (avoid extra DB read / TOCTOU)
        info!(
            invitation_id = %invitation_id,
            tenant_id = %tenant_id,
            "invitation_revoked"
        );

        Ok(())
    }

    async fn resend_invitation(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
        invited_from_ip: Option<&str>,
        invited_from_user_agent: Option<&str>,
    ) -> Result<(Invitation, String), AppError> {
        // Find invitation
        let invitation = self
            .invitation_repo
            .find_by_id(tenant_id, invitation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Invitation not found".into()))?;

        // Check if pending
        if invitation.status != InvitationStatus::Pending {
            return Err(AppError::Conflict("Can only resend pending invitations".into()));
        }

        // Generate new token
        let (new_plaintext_token, new_token_hash) = generate_invite_token();

        // Calculate new expiry
        let new_expires_at = Utc::now() + Duration::hours(self.invitation_expiry_hours);

        // Persist the updated invitation to the database
        let updated_invitation = self
            .invitation_repo
            .update_for_resend(
                tenant_id,
                invitation_id,
                &new_token_hash,
                new_expires_at,
                invited_from_ip,
                invited_from_user_agent,
            )
            .await?;

        // Audit log: invitation resend
        info!(
            invitation_id = %updated_invitation.invitation_id,
            tenant_id = %tenant_id,
            new_expires_at = %updated_invitation.expires_at,
            "invitation_resent"
        );
        debug!(
            invitation_id = %updated_invitation.invitation_id,
            email = %updated_invitation.email,
            "invitation_resent_pii"
        );

        Ok((updated_invitation, new_plaintext_token))
    }

    async fn cleanup_expired_invitations(&self) -> Result<i64, AppError> {
        let count = self.invitation_repo.cleanup_expired().await?;

        // Audit log: cleanup
        info!(
            expired_count = %count,
            "invitations_cleanup_completed"
        );

        Ok(count)
    }
}
