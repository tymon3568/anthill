use async_nats::Subscriber;
use serde::{de::DeserializeOwned, Serialize};

use shared_error::AppError;

use crate::events::EventEnvelope;

#[derive(Clone)]
pub struct NatsClient {
    client: async_nats::Client,
}

impl NatsClient {
    pub async fn connect(url: &str) -> Result<Self, AppError> {
        let client = async_nats::connect(url)
            .await
            .map_err(|e| AppError::InternalError(format!("NATS connection failed: {}", e)))?;
        Ok(Self { client })
    }

    pub async fn publish_event<T: Serialize>(
        &self,
        subject: String,
        event: &EventEnvelope<T>,
    ) -> Result<(), AppError> {
        let json = serde_json::to_string(event)
            .map_err(|e| AppError::InternalError(format!("Serialization failed: {}", e)))?;
        self.client
            .publish(subject, json.into())
            .await
            .map_err(|e| AppError::InternalError(format!("Publish failed: {}", e)))?;
        Ok(())
    }

    pub async fn subscribe_event<T: DeserializeOwned + Send + 'static>(
        &self,
        subject: String,
    ) -> Result<Subscriber, AppError> {
        let subscriber = self
            .client
            .subscribe(subject)
            .await
            .map_err(|e| AppError::InternalError(format!("Subscribe failed: {}", e)))?;
        Ok(subscriber)
    }
}

static NATS_CLIENT: once_cell::sync::OnceCell<NatsClient> = once_cell::sync::OnceCell::new();

pub async fn init_nats_client(url: &str) -> Result<(), AppError> {
    let client = NatsClient::connect(url).await?;
    NATS_CLIENT
        .set(client)
        .map_err(|_| AppError::InternalError("NATS client already initialized".to_string()))?;
    Ok(())
}

pub fn get_nats_client() -> Result<&'static NatsClient, AppError> {
    NATS_CLIENT
        .get()
        .ok_or_else(|| AppError::InternalError("NATS client not initialized".to_string()))
}
