use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use shared_error::AppError;
use user_service_core::domains::auth::domain::{
    invitation_repository::InvitationRepository,
    model::{Invitation, InvitationStatus},
};

/// PostgreSQL implementation of InvitationRepository
pub struct PgInvitationRepository {
    pool: PgPool,
}

impl PgInvitationRepository {
    /// Create a new PgInvitationRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InvitationRepository for PgInvitationRepository {
    async fn create(&self, invitation: &Invitation) -> Result<Invitation, AppError> {
        let invitation = sqlx::query_as::<_, Invitation>(
            r#"
            INSERT INTO user_invitations (
                invitation_id, tenant_id, token_hash, email, invited_role,
                invited_by_user_id, invited_from_ip, invited_from_user_agent,
                expires_at, custom_message, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING
                invitation_id, tenant_id, token_hash, email, invited_role,
                invited_by_user_id, status, expires_at, accepted_at,
                accepted_user_id, invited_from_ip, invited_from_user_agent,
                accepted_from_ip, accepted_from_user_agent, accept_attempts,
                last_attempt_at, custom_message, metadata, created_at, updated_at,
                deleted_at
            "#,
        )
        .bind(invitation.invitation_id)
        .bind(invitation.tenant_id)
        .bind(&invitation.token_hash)
        .bind(&invitation.email)
        .bind(&invitation.invited_role)
        .bind(invitation.invited_by_user_id)
        .bind(&invitation.invited_from_ip)
        .bind(&invitation.invited_from_user_agent)
        .bind(invitation.expires_at)
        .bind(&invitation.custom_message)
        .bind(&invitation.metadata)
        .bind(invitation.created_at)
        .bind(invitation.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create invitation: {}", e)))?;

        Ok(invitation)
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<Option<Invitation>, AppError> {
        let invitation = sqlx::query_as::<_, Invitation>(
            r#"
            SELECT
                invitation_id, tenant_id, token_hash, email, invited_role,
                invited_by_user_id, status, expires_at, accepted_at,
                accepted_user_id, invited_from_ip, invited_from_user_agent,
                accepted_from_ip, accepted_from_user_agent, accept_attempts,
                last_attempt_at, custom_message, metadata, created_at, updated_at,
                deleted_at
            FROM user_invitations
            WHERE tenant_id = $1 AND invitation_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(invitation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find invitation by ID: {}", e)))?;

        Ok(invitation)
    }

    /// WARNING: This method does NOT filter by tenant_id for security reasons.
    /// It is intended for public token acceptance flows where tenant context
    /// is not yet available. Callers MUST validate the returned Invitation's
    /// tenant_id against the expected tenant before proceeding.
    async fn find_pending_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Invitation>, AppError> {
        let invitation = sqlx::query_as::<_, Invitation>(
            r#"
            SELECT
                invitation_id, tenant_id, token_hash, email, invited_role,
                invited_by_user_id, status, expires_at, accepted_at,
                accepted_user_id, invited_from_ip, invited_from_user_agent,
                accepted_from_ip, accepted_from_user_agent, accept_attempts,
                last_attempt_at, custom_message, metadata, created_at, updated_at,
                deleted_at
            FROM user_invitations
            WHERE token_hash = $1 AND status = $2 AND expires_at > NOW() AND deleted_at IS NULL
            "#,
        )
        .bind(token_hash)
        .bind(InvitationStatus::Pending)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!(
                "Failed to find pending invitation by token hash: {}",
                e
            ))
        })?;

        Ok(invitation)
    }

    async fn find_pending_by_tenant_and_email(
        &self,
        tenant_id: Uuid,
        email: &str,
    ) -> Result<Option<Invitation>, AppError> {
        let invitation = sqlx::query_as::<_, Invitation>(
            r#"
            SELECT
                invitation_id, tenant_id, token_hash, email, invited_role,
                invited_by_user_id, status, expires_at, accepted_at,
                accepted_user_id, invited_from_ip, invited_from_user_agent,
                accepted_from_ip, accepted_from_user_agent, accept_attempts,
                last_attempt_at, custom_message, metadata, created_at, updated_at,
                deleted_at
            FROM user_invitations
            WHERE tenant_id = $1 AND email = $2 AND status = $3 AND expires_at > NOW() AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(email)
        .bind(InvitationStatus::Pending)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!(
                "Failed to find pending invitation by tenant and email: {}",
                e
            ))
        })?;

        Ok(invitation)
    }

    async fn list_by_tenant(
        &self,
        tenant_id: Uuid,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Invitation>, AppError> {
        let query = if let Some(status_filter) = status {
            sqlx::query_as::<_, Invitation>(
                r#"
                SELECT
                    invitation_id, tenant_id, token_hash, email, invited_role,
                    invited_by_user_id, status, expires_at, accepted_at,
                    accepted_user_id, invited_from_ip, invited_from_user_agent,
                    accepted_from_ip, accepted_from_user_agent, accept_attempts,
                    last_attempt_at, custom_message, metadata, created_at, updated_at,
                    deleted_at
                FROM user_invitations
                WHERE tenant_id = $1 AND status = $2 AND deleted_at IS NULL
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(tenant_id)
            .bind(status_filter)
            .bind(limit)
            .bind(offset)
        } else {
            sqlx::query_as::<_, Invitation>(
                r#"
                SELECT
                    invitation_id, tenant_id, token_hash, email, invited_role,
                    invited_by_user_id, status, expires_at, accepted_at,
                    accepted_user_id, invited_from_ip, invited_from_user_agent,
                    accepted_from_ip, accepted_from_user_agent, accept_attempts,
                    last_attempt_at, custom_message, metadata, created_at, updated_at,
                    deleted_at
                FROM user_invitations
                WHERE tenant_id = $1 AND deleted_at IS NULL
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(tenant_id)
            .bind(limit)
            .bind(offset)
        };

        let invitations = query.fetch_all(&self.pool).await.map_err(|e| {
            AppError::DatabaseError(format!("Failed to list invitations by tenant: {}", e))
        })?;

        Ok(invitations)
    }

    async fn count_by_tenant(
        &self,
        tenant_id: Uuid,
        status: Option<&str>,
    ) -> Result<i64, AppError> {
        let query = if let Some(status_filter) = status {
            sqlx::query(
                r#"
                SELECT COUNT(*) as count
                FROM user_invitations
                WHERE tenant_id = $1 AND status = $2 AND deleted_at IS NULL
                "#,
            )
            .bind(tenant_id)
            .bind(status_filter)
        } else {
            sqlx::query(
                r#"
                SELECT COUNT(*) as count
                FROM user_invitations
                WHERE tenant_id = $1 AND deleted_at IS NULL
                "#,
            )
            .bind(tenant_id)
        };

        let row = query.fetch_one(&self.pool).await.map_err(|e| {
            AppError::DatabaseError(format!("Failed to count invitations by tenant: {}", e))
        })?;

        let count: i64 = row.try_get("count").map_err(|e| {
            AppError::DatabaseError(format!("Failed to read invitation count: {}", e))
        })?;
        Ok(count)
    }

    async fn update_status(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
        status: InvitationStatus,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE user_invitations
            SET status = $1, updated_at = NOW()
            WHERE tenant_id = $2 AND invitation_id = $3 AND deleted_at IS NULL
            "#,
        )
        .bind(status)
        .bind(tenant_id)
        .bind(invitation_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update invitation status: {}", e))
        })?;

        Ok(())
    }

    async fn update_for_resend(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
        new_token_hash: &str,
        new_expires_at: chrono::DateTime<Utc>,
        invited_from_ip: Option<&str>,
        invited_from_user_agent: Option<&str>,
    ) -> Result<Invitation, AppError> {
        let invitation = sqlx::query_as::<_, Invitation>(
            r#"
            UPDATE user_invitations
            SET token_hash = $1,
                expires_at = $2,
                accept_attempts = 0,
                last_attempt_at = NULL,
                invited_from_ip = COALESCE($3, invited_from_ip),
                invited_from_user_agent = COALESCE($4, invited_from_user_agent),
                updated_at = NOW()
            WHERE tenant_id = $5 AND invitation_id = $6 AND status = $7 AND deleted_at IS NULL
            RETURNING
                invitation_id, tenant_id, token_hash, email, invited_role,
                invited_by_user_id, status, expires_at, accepted_at,
                accepted_user_id, invited_from_ip, invited_from_user_agent,
                accepted_from_ip, accepted_from_user_agent, accept_attempts,
                last_attempt_at, custom_message, metadata, created_at, updated_at,
                deleted_at
            "#,
        )
        .bind(new_token_hash)
        .bind(new_expires_at)
        .bind(invited_from_ip)
        .bind(invited_from_user_agent)
        .bind(tenant_id)
        .bind(invitation_id)
        .bind(InvitationStatus::Pending)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update invitation for resend: {}", e))
        })?;

        match invitation {
            Some(inv) => Ok(inv),
            None => Err(AppError::NotFound("Invitation not found or not pending".to_string())),
        }
    }

    async fn mark_accepted(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
        accepted_user_id: Uuid,
        accepted_from_ip: Option<&str>,
        accepted_from_user_agent: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE user_invitations
            SET status = $1,
                accepted_at = NOW(),
                accepted_user_id = $2,
                accepted_from_ip = $3,
                accepted_from_user_agent = $4,
                updated_at = NOW()
            WHERE tenant_id = $5 AND invitation_id = $6 AND status = $7 AND deleted_at IS NULL
            "#,
        )
        .bind(InvitationStatus::Accepted)
        .bind(accepted_user_id)
        .bind(accepted_from_ip)
        .bind(accepted_from_user_agent)
        .bind(tenant_id)
        .bind(invitation_id)
        .bind(InvitationStatus::Pending)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to mark invitation as accepted: {}", e))
        })?;

        Ok(())
    }

    async fn mark_expired(&self, tenant_id: Uuid, invitation_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE user_invitations
            SET status = $1, updated_at = NOW()
            WHERE tenant_id = $2 AND invitation_id = $3 AND status = $4 AND deleted_at IS NULL
            "#,
        )
        .bind(InvitationStatus::Expired)
        .bind(tenant_id)
        .bind(invitation_id)
        .bind(InvitationStatus::Pending)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to mark invitation as expired: {}", e))
        })?;

        Ok(())
    }

    async fn revoke(&self, tenant_id: Uuid, invitation_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE user_invitations
            SET status = $1, updated_at = NOW()
            WHERE tenant_id = $2 AND invitation_id = $3 AND status = $4 AND deleted_at IS NULL
            "#,
        )
        .bind(InvitationStatus::Revoked)
        .bind(tenant_id)
        .bind(invitation_id)
        .bind(InvitationStatus::Pending)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to revoke invitation: {}", e)))?;

        Ok(())
    }

    async fn increment_accept_attempts(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE user_invitations
            SET accept_attempts = accept_attempts + 1,
                last_attempt_at = NOW(),
                updated_at = NOW()
            WHERE tenant_id = $1 AND invitation_id = $2 AND status = $3 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(invitation_id)
        .bind(InvitationStatus::Pending)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to increment accept attempts: {}", e))
        })?;

        Ok(())
    }

    async fn check_and_increment_accept_attempts(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
        max_attempts: i32,
    ) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE user_invitations
            SET accept_attempts = accept_attempts + 1, last_attempt_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND invitation_id = $2 AND status = $3 AND accept_attempts < $4 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(invitation_id)
        .bind(InvitationStatus::Pending)
        .bind(max_attempts)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to check and increment accept attempts: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn soft_delete(&self, tenant_id: Uuid, invitation_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE user_invitations
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND invitation_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(invitation_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to soft delete invitation: {}", e)))?;

        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<i64, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE user_invitations
            SET status = $1, updated_at = NOW()
            WHERE status = $2 AND expires_at < NOW() AND deleted_at IS NULL
            "#,
        )
        .bind(InvitationStatus::Expired)
        .bind(InvitationStatus::Pending)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to cleanup expired invitations: {}", e))
        })?;

        Ok(result.rows_affected() as i64)
    }

    async fn hard_delete_old(&self, before_date: DateTime<Utc>) -> Result<i64, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_invitations
            WHERE deleted_at IS NOT NULL AND deleted_at < $1
            "#,
        )
        .bind(before_date)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to hard delete old invitations: {}", e))
        })?;

        Ok(result.rows_affected() as i64)
    }
}
