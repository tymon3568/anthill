//! Outbox worker for reliable event publishing
//!
//! This module contains the background worker that polls the event_outbox table
//! and publishes events to NATS.

use async_nats::Client;
use serde_json::Value;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};
use uuid::Uuid;

use shared_error::AppError;

/// Configuration for the outbox worker
#[derive(Debug, Clone)]
pub struct OutboxWorkerConfig {
    /// How often to poll for new events (in seconds)
    pub poll_interval_seconds: u64,
    /// Maximum number of events to process in one batch
    pub batch_size: i32,
    /// Maximum number of retry attempts per event
    pub max_retries: i32,
    /// NATS subject prefix for events
    pub nats_subject_prefix: String,
}

impl Default for OutboxWorkerConfig {
    fn default() -> Self {
        Self {
            poll_interval_seconds: 5,
            batch_size: 50,
            max_retries: 3,
            nats_subject_prefix: "inventory.events".to_string(),
        }
    }
}

/// Start the outbox worker
pub async fn start_outbox_worker(
    pool: PgPool,
    nats_client: Client,
    config: OutboxWorkerConfig,
) -> Result<(), AppError> {
    info!("Starting outbox worker with config: {:?}", config);

    let mut interval = time::interval(Duration::from_secs(config.poll_interval_seconds));

    loop {
        interval.tick().await;

        if let Err(e) = process_pending_events(&pool, &nats_client, &config).await {
            error!("Error processing pending events: {}", e);
        }
    }
}

/// Process pending events from the outbox
async fn process_pending_events(
    pool: &PgPool,
    nats_client: &Client,
    config: &OutboxWorkerConfig,
) -> Result<(), AppError> {
    // Fetch pending events ordered by creation time
    let events = sqlx::query_as!(
        EventRow,
        r#"
        SELECT id, tenant_id, event_type, event_data as "event_data: _", retry_count
        FROM event_outbox
        WHERE status = 'pending'
        ORDER BY created_at ASC
        LIMIT $1
        FOR UPDATE SKIP LOCKED
        "#,
        config.batch_size
    )
    .fetch_all(pool)
    .await?;

    if events.is_empty() {
        return Ok(());
    }

    info!("Processing {} pending events", events.len());

    for event in events {
        if let Err(e) = process_event(pool, nats_client, config, &event).await {
            error!("Failed to process event {}: {}", event.id, e);
        }
    }

    Ok(())
}

/// Process a single event
async fn process_event(
    pool: &PgPool,
    nats_client: &Client,
    config: &OutboxWorkerConfig,
    event: &EventRow,
) -> Result<(), AppError> {
    let subject = format!("{}.{}", config.nats_subject_prefix, event.event_type);

    // Publish to NATS
    let event_bytes = serde_json::to_vec(&event.event_data)?;
    match nats_client.publish(&subject, event_bytes.into()).await {
        Ok(_) => {
            // Mark as published
            sqlx::query!(
                r#"
                UPDATE event_outbox
                SET status = 'published', published_at = NOW(), updated_at = NOW()
                WHERE id = $1
                "#,
                event.id
            )
            .execute(pool)
            .await?;

            info!("Published event {} to subject {}", event.id, subject);
        },
        Err(e) => {
            // Increment retry count
            let new_retry_count = event.retry_count + 1;

            if new_retry_count >= config.max_retries {
                // Mark as failed
                sqlx::query!(
                    r#"
                    UPDATE event_outbox
                    SET status = 'failed', retry_count = $2, error_message = $3, updated_at = NOW()
                    WHERE id = $1
                    "#,
                    event.id,
                    new_retry_count,
                    format!("Failed to publish after {} retries: {}", config.max_retries, e)
                )
                .execute(pool)
                .await?;

                error!(
                    "Event {} failed permanently after {} retries",
                    event.id, config.max_retries
                );
            } else {
                // Update retry count
                sqlx::query!(
                    r#"
                    UPDATE event_outbox
                    SET retry_count = $2, error_message = $3, updated_at = NOW()
                    WHERE id = $1
                    "#,
                    event.id,
                    new_retry_count,
                    format!("Publish attempt {} failed: {}", new_retry_count, e)
                )
                .execute(pool)
                .await?;

                warn!("Event {} publish failed, retry count: {}", event.id, new_retry_count);
            }
        },
    }

    Ok(())
}

/// Struct to represent event row from database
#[derive(sqlx::FromRow)]
struct EventRow {
    id: Uuid,
    tenant_id: Uuid,
    event_type: String,
    event_data: Value,
    retry_count: i32,
}
