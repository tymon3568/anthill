//! Service traits for delivery order operations
//!
//! This module contains trait definitions for Delivery Order business logic operations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::delivery::{PickItemsRequest, PickItemsResponse};
use shared_error::AppError;

/// Service trait for delivery order operations
#[async_trait]
pub trait DeliveryService: Send + Sync {
    /// Pick items for a delivery order
    async fn pick_items(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
        user_id: Uuid,
        request: PickItemsRequest,
    ) -> Result<PickItemsResponse, AppError>;
}
