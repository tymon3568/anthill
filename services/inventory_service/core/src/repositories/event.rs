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
    async fn insert_event(
        &self,
        tenant_id: Uuid,
        event_type: &str,
        event_data: Value,
    ) -> Result<Uuid, AppError>;
}
