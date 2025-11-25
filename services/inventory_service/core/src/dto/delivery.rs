//! Data Transfer Objects for Delivery Order operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

/// Request to pick items for a delivery order
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PickItemsRequest {
    pub items: Vec<PickItemRequest>,
}

/// Individual item to pick
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PickItemRequest {
    pub delivery_item_id: Uuid,
    pub picked_quantity: i64,
}

/// Response for pick operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PickItemsResponse {
    pub delivery_id: Uuid,
    pub status: String,
    pub picked_items_count: usize,
    pub total_picked_quantity: i64,
}

/// Request to pack items for a delivery order
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PackItemsRequest {
    pub notes: Option<String>,
}

/// Response for pack operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PackItemsResponse {
    pub delivery_id: Uuid,
    pub status: String,
    pub packed_at: DateTime<Utc>,
}

/// Request to ship a delivery order
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ShipItemsRequest {
    pub tracking_number: Option<String>,
    pub carrier: Option<String>,
    pub shipping_cost: Option<i64>,
    pub notes: Option<String>,
}

/// Response for ship operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ShipItemsResponse {
    pub delivery_id: Uuid,
    pub status: String,
    pub shipped_at: DateTime<Utc>,
    pub stock_moves_created: usize,
    pub total_cogs: i64,
}
