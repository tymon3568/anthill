use async_trait::async_trait;
use shared_error::AppError;
use sqlx::PgPool;
use user_service_core::domains::auth::domain::email_verification_repository::EmailVerificationRepository;
use user_service_core::domains::auth::domain::model::EmailVerificationToken;
use uuid::Uuid;

/// PostgreSQL implementation of EmailVerificationRepository
pub struct PgEmailVerificationRepository {
    pool: PgPool,
}

impl PgEmailVerificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EmailVerificationRepository for PgEmailVerificationRepository {
    async fn create(
        &self,
        token: &EmailVerificationToken,
    ) -> Result<EmailVerificationToken, AppError> {
        let created = sqlx::query_as::<_, EmailVerificationToken>(
            r#"
            INSERT INTO email_verification_tokens (
                token_id, user_id, tenant_id, token_hash, email,
                expires_at, created_at, verification_attempts
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(token.token_id)
        .bind(token.user_id)
        .bind(token.tenant_id)
        .bind(&token.token_hash)
        .bind(&token.email)
        .bind(token.expires_at)
        .bind(token.created_at)
        .bind(token.verification_attempts)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<EmailVerificationToken>, AppError> {
        let token = sqlx::query_as::<_, EmailVerificationToken>(
            r#"
            SELECT * FROM email_verification_tokens
            WHERE token_hash = $1
            AND verified_at IS NULL
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
    ) -> Result<Option<EmailVerificationToken>, AppError> {
        let token = sqlx::query_as::<_, EmailVerificationToken>(
            r#"
            SELECT * FROM email_verification_tokens
            WHERE user_id = $1
            AND tenant_id = $2
            AND verified_at IS NULL
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

    async fn mark_as_verified(
        &self,
        token_id: Uuid,
        verified_from_ip: Option<String>,
        verified_from_user_agent: Option<String>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            UPDATE email_verification_tokens
            SET verified_at = NOW(),
                verified_from_ip = $2::inet,
                verified_from_user_agent = $3
            WHERE token_id = $1
            "#,
        )
        .bind(token_id)
        .bind(verified_from_ip)
        .bind(verified_from_user_agent)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Verification token not found".to_string()));
        }

        Ok(())
    }

    async fn increment_attempts(&self, token_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE email_verification_tokens
            SET verification_attempts = verification_attempts + 1,
                last_attempt_at = NOW()
            WHERE token_id = $1
            "#,
        )
        .bind(token_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM email_verification_tokens
            WHERE expires_at < NOW()
            AND verified_at IS NULL
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn count_recent_tokens_for_email(
        &self,
        email: &str,
        tenant_id: Uuid,
        since_minutes: i64,
    ) -> Result<i64, AppError> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM email_verification_tokens
            WHERE email = $1
            AND tenant_id = $2
            AND created_at > NOW() - INTERVAL '1 minute' * $3
            AND verified_at IS NULL
            "#,
        )
        .bind(email)
        .bind(tenant_id)
        .bind(since_minutes)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn invalidate_all_for_user(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE email_verification_tokens
            SET verified_at = NOW(),
                verified_from_ip = '0.0.0.0',
                verified_from_user_agent = 'auto-invalidated'
            WHERE user_id = $1
            AND tenant_id = $2
            AND verified_at IS NULL
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
