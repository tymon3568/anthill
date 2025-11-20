//! NATS client wrapper for event-driven communication
//!
//! This module provides a high-level wrapper around the async-nats client
//! with connection management, automatic reconnection, and event serialization.

use async_nats::{Client, ConnectOptions};
use futures_util::stream::StreamExt;
use serde::{de::DeserializeOwned, Serialize};
use shared_error::AppError;
use std::time::Duration;
use tokio::sync::OnceCell;
use tracing::{error, info, warn};

use crate::events::EventEnvelope;

/// NATS client wrapper with connection management
#[derive(Clone)]
pub struct NatsClient {
    client: Client,
}

impl NatsClient {
    /// Connect to NATS server with automatic reconnection
    pub async fn connect(nats_url: &str) -> Result<Self, AppError> {
        info!("Connecting to NATS at {}", nats_url);

        let client = ConnectOptions::new()
            .retry_on_initial_connect()
            .reconnect_delay_callback(|attempt| {
                let delay = Duration::from_millis(2u64.pow(attempt.min(6) as u32) * 100);
                warn!("Reconnecting to NATS in {:?}", delay);
                delay
            })
            .connect(nats_url)
            .await
            .map_err(|e| {
                error!("Failed to connect to NATS: {}", e);
                AppError::InternalError(format!("NATS connection failed: {}", e))
            })?;

        info!("Successfully connected to NATS");
        Ok(Self { client })
    }

    /// Publish an event to a NATS subject
    pub async fn publish_event<T: Serialize>(
        &self,
        subject: &str,
        event: EventEnvelope<T>,
    ) -> Result<(), AppError> {
        let subject = subject.to_string();
        let payload = serde_json::to_vec(&event).map_err(|e| {
            error!("Failed to serialize event: {}", e);
            AppError::InternalError(format!("Event serialization failed: {}", e))
        })?;

        self.client
            .publish(subject.clone(), payload.into())
            .await
            .map_err(|e| {
                error!("Failed to publish event to {}: {}", subject, e);
                AppError::InternalError(format!("NATS publish failed: {}", e))
            })?;

        info!("Published event {} to subject {}", event.event_type, subject);
        Ok(())
    }

    /// Subscribe to a NATS subject and process events with a handler function
    ///
    /// This spawns a background task to handle incoming messages.
    /// The handler function receives the deserialized event data.
    pub async fn subscribe_event<T, F, Fut>(
        &self,
        subject: &str,
        mut handler: F,
    ) -> Result<(), AppError>
    where
        T: DeserializeOwned + Send + 'static,
        F: FnMut(EventEnvelope<T>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let subject = subject.to_string();
        let mut subscriber = self.client.subscribe(subject.clone()).await.map_err(|e| {
            error!("Failed to subscribe to {}: {}", subject, e);
            AppError::InternalError(format!("NATS subscribe failed: {}", e))
        })?;

        info!("Subscribed to subject {}", subject);

        // Spawn background task to handle messages
        tokio::spawn(async move {
            while let Some(message) = subscriber.next().await {
                match serde_json::from_slice::<EventEnvelope<T>>(&message.payload) {
                    Ok(event) => {
                        info!("Received event {} on subject {}", event.event_type, subject);
                        handler(event).await;
                    },
                    Err(e) => {
                        error!("Failed to deserialize event from {}: {}", subject, e);
                        // Continue processing other messages
                    },
                }
            }
            warn!("Subscriber for {} has ended", subject);
        });

        Ok(())
    }

    /// Get the underlying NATS client (for advanced usage)
    pub fn inner(&self) -> &Client {
        &self.client
    }
}

/// Global NATS client instance (lazy initialization)
static NATS_CLIENT: OnceCell<NatsClient> = OnceCell::const_new();

/// Initialize the global NATS client
pub async fn init_nats_client(nats_url: &str) -> Result<(), AppError> {
    let client = NatsClient::connect(nats_url).await?;
    NATS_CLIENT
        .set(client)
        .map_err(|_| AppError::InternalError("NATS client already initialized".to_string()))?;
    Ok(())
}

/// Get the global NATS client instance
pub async fn get_nats_client() -> Result<&'static NatsClient, AppError> {
    NATS_CLIENT
        .get()
        .ok_or_else(|| AppError::InternalError("NATS client not initialized".to_string()))
}
