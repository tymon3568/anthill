//! Data Transfer Objects for Delivery Order operations

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request to pick items for a delivery order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickItemsRequest {
    pub items: Vec<PickItemRequest>,
}

/// Individual item to pick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickItemRequest {
    pub delivery_item_id: Uuid,
    pub picked_quantity: i64,
}

/// Response for pick operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickItemsResponse {
    pub delivery_id: Uuid,
    pub status: String,
    pub picked_items_count: usize,
    pub total_picked_quantity: i64,
}
