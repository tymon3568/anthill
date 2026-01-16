use async_trait::async_trait;
use shared_error::AppError;
use sqlx::PgPool;
use user_service_core::domains::auth::domain::model::{PasswordResetAudit, PasswordResetToken};
use user_service_core::domains::auth::domain::password_reset_repository::PasswordResetRepository;
use uuid::Uuid;

/// PostgreSQL implementation of PasswordResetRepository
pub struct PgPasswordResetRepository {
    pool: PgPool,
}

impl PgPasswordResetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PasswordResetRepository for PgPasswordResetRepository {
    async fn create(&self, token: &PasswordResetToken) -> Result<PasswordResetToken, AppError> {
        let created = sqlx::query_as::<_, PasswordResetToken>(
            r#"
            INSERT INTO password_reset_tokens (
                token_id, user_id, tenant_id, token_hash,
                expires_at, created_at, ip_address, user_agent
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::inet, $8)
            RETURNING
                token_id, user_id, tenant_id, token_hash,
                expires_at, created_at, used_at,
                ip_address::text, user_agent,
                reset_from_ip::text, reset_from_user_agent
            "#,
        )
        .bind(token.token_id)
        .bind(token.user_id)
        .bind(token.tenant_id)
        .bind(&token.token_hash)
        .bind(token.expires_at)
        .bind(token.created_at)
        .bind(&token.ip_address)
        .bind(&token.user_agent)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<PasswordResetToken>, AppError> {
        let token = sqlx::query_as::<_, PasswordResetToken>(
            r#"
            SELECT
                token_id, user_id, tenant_id, token_hash,
                expires_at, created_at, used_at,
                ip_address::text, user_agent,
                reset_from_ip::text, reset_from_user_agent
            FROM password_reset_tokens
            WHERE token_hash = $1
            AND used_at IS NULL
            AND expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(token)
    }

    async fn find_pending_by_user_id(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Option<PasswordResetToken>, AppError> {
        let token = sqlx::query_as::<_, PasswordResetToken>(
            r#"
            SELECT
                token_id, user_id, tenant_id, token_hash,
                expires_at, created_at, used_at,
                ip_address::text, user_agent,
                reset_from_ip::text, reset_from_user_agent
            FROM password_reset_tokens
            WHERE user_id = $1
            AND tenant_id = $2
            AND used_at IS NULL
            AND expires_at > NOW()
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(token)
    }

    async fn mark_as_used(
        &self,
        token_id: Uuid,
        reset_from_ip: Option<String>,
        reset_from_user_agent: Option<String>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            UPDATE password_reset_tokens
            SET used_at = NOW(),
                reset_from_ip = $2::inet,
                reset_from_user_agent = $3
            WHERE token_id = $1
            AND used_at IS NULL
            "#,
        )
        .bind(token_id)
        .bind(reset_from_ip)
        .bind(reset_from_user_agent)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                "Password reset token not found or already used".to_string(),
            ));
        }

        Ok(())
    }

    async fn count_recent_tokens_for_user(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        since_minutes: i64,
    ) -> Result<i64, AppError> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM password_reset_tokens
            WHERE user_id = $1
            AND tenant_id = $2
            AND created_at > NOW() - INTERVAL '1 minute' * $3
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(since_minutes)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn delete_expired(&self) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM password_reset_tokens
            WHERE expires_at < NOW()
            AND used_at IS NULL
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn invalidate_all_for_user(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE password_reset_tokens
            SET used_at = NOW(),
                reset_from_ip = '0.0.0.0'::inet,
                reset_from_user_agent = 'auto-invalidated'
            WHERE user_id = $1
            AND tenant_id = $2
            AND used_at IS NULL
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    // =========================================================================
    // Audit log methods
    // =========================================================================

    async fn log_audit_event(&self, audit: &PasswordResetAudit) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO password_reset_audit (
                audit_id, user_id, tenant_id, email,
                event_type, ip_address, user_agent, failure_reason, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6::inet, $7, $8, $9)
            "#,
        )
        .bind(audit.audit_id)
        .bind(audit.user_id)
        .bind(audit.tenant_id)
        .bind(&audit.email)
        .bind(&audit.event_type)
        .bind(&audit.ip_address)
        .bind(&audit.user_agent)
        .bind(&audit.failure_reason)
        .bind(audit.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn count_audit_events_for_email(
        &self,
        email: &str,
        event_type: &str,
        since_minutes: i64,
    ) -> Result<i64, AppError> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM password_reset_audit
            WHERE email = $1
            AND event_type = $2
            AND created_at > NOW() - INTERVAL '1 minute' * $3
            "#,
        )
        .bind(email)
        .bind(event_type)
        .bind(since_minutes)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}
