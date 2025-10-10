use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use user_service_core::domains::auth::domain::{
    model::Session,
    repository::SessionRepository,
};
use shared_error::AppError;

/// PostgreSQL implementation of SessionRepository
pub struct PgSessionRepository {
    pool: PgPool,
}

impl PgSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for PgSessionRepository {
    async fn create(&self, session: &Session) -> Result<Session, AppError> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (
                session_id, user_id, tenant_id,
                access_token_hash, refresh_token_hash,
                ip_address, user_agent, device_info,
                access_token_expires_at, refresh_token_expires_at,
                revoked, created_at, last_used_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#
        )
        .bind(session.session_id)
        .bind(session.user_id)
        .bind(session.tenant_id)
        .bind(&session.access_token_hash)
        .bind(&session.refresh_token_hash)
        .bind(&session.ip_address)
        .bind(&session.user_agent)
        .bind(&session.device_info)
        .bind(session.access_token_expires_at)
        .bind(session.refresh_token_expires_at)
        .bind(session.revoked)
        .bind(session.created_at)
        .bind(session.last_used_at)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(session)
    }
    
    async fn find_by_refresh_token(&self, token_hash: &str) -> Result<Option<Session>, AppError> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            SELECT * FROM sessions
            WHERE refresh_token_hash = $1
              AND NOT revoked
              AND refresh_token_expires_at > NOW()
            "#
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(session)
    }
    
    async fn revoke(&self, session_id: Uuid, reason: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE sessions
            SET revoked = TRUE,
                revoked_at = NOW(),
                revoked_reason = $2
            WHERE session_id = $1
            "#
        )
        .bind(session_id)
        .bind(reason)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE sessions
            SET revoked = TRUE,
                revoked_at = NOW(),
                revoked_reason = 'logout_all'
            WHERE user_id = $1 AND NOT revoked
            "#
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected())
    }
    
    async fn update_last_used(&self, session_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE sessions
            SET last_used_at = NOW()
            WHERE session_id = $1
            "#
        )
        .bind(session_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn delete_expired(&self) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM sessions
            WHERE refresh_token_expires_at < NOW()
               OR (revoked = TRUE AND revoked_at < NOW() - INTERVAL '30 days')
            "#
        )
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected())
    }
}
