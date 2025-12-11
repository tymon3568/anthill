//! Repository trait for event outbox operations
//!
//! This module contains trait definitions for writing events to the transactional outbox.

use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use shared_error::AppError;

/// Repository trait for event outbox operations
#[async_trait]
pub trait EventRepository: Send + Sync {
    /// Insert an event into the outbox table
    /// This should be called within the same database transaction as the business logic
    async fn insert_event(
        &self,
        tenant_id: Uuid,
        event_type: &str,
        event_data: Value,
    ) -> Result<Uuid, AppError>;

    /// Insert an event into the outbox table within an existing transaction
    /// This ensures the event is published atomically with the business logic
    async fn insert_event_in_tx(
        &self,
        tenant_id: Uuid,
        event_type: &str,
        event_data: Value,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Uuid, AppError>;
}
