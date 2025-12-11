//! PostgreSQL implementation of event repository
//!
//! This module provides concrete implementation of EventRepository trait
//! for writing events to the transactional outbox table.

use async_trait::async_trait;
use serde_json::Value;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use inventory_service_core::repositories::event::EventRepository;
use shared_error::AppError;

/// PostgreSQL implementation of EventRepository
pub struct EventRepositoryImpl {
    pool: PgPool,
}

impl EventRepositoryImpl {
    /// Create a new EventRepositoryImpl with the given database connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventRepository for EventRepositoryImpl {
    /// Insert an event into the outbox table
    async fn insert_event(
        &self,
        tenant_id: Uuid,
        event_type: &str,
        event_data: Value,
    ) -> Result<Uuid, AppError> {
        let event_id = Uuid::now_v7();

        sqlx::query!(
            r#"
            INSERT INTO event_outbox (
                id, tenant_id, event_type, event_data, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, 'pending', NOW(), NOW())
            "#,
            event_id,
            tenant_id,
            event_type,
            event_data
        )
        .execute(&self.pool)
        .await?;

        Ok(event_id)
    }

    /// Insert an event into the outbox table within an existing transaction
    async fn insert_event_in_tx(
        &self,
        tenant_id: Uuid,
        event_type: &str,
        event_data: Value,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Uuid, AppError> {
        let event_id = Uuid::now_v7();

        sqlx::query!(
            r#"
            INSERT INTO event_outbox (
                id, tenant_id, event_type, event_data, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, 'pending', NOW(), NOW())
            "#,
            event_id,
            tenant_id,
            event_type,
            event_data
        )
        .execute(tx)
        .await?;

        Ok(event_id)
    }
}
