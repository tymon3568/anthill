//! Data Transfer Objects for RMA operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{RmaAction, RmaCondition, RmaStatus};

/// Request to create a new RMA
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateRmaRequest {
    pub customer_id: Uuid,
    pub original_delivery_id: Uuid,
    pub return_reason: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<CreateRmaItemRequest>,
}

/// Individual item in RMA creation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateRmaItemRequest {
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub quantity_returned: i64,
    pub condition: RmaCondition,
    pub action: RmaAction,
    pub unit_cost: Option<i64>,
    pub notes: Option<String>,
}

/// Response for RMA creation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateRmaResponse {
    pub rma_id: Uuid,
    pub rma_number: String,
    pub status: RmaStatus,
    pub created_at: DateTime<Utc>,
}

/// Request to approve or reject an RMA
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ApproveRmaRequest {
    pub approved: bool,
    pub notes: Option<String>,
}

/// Response for RMA approval
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ApproveRmaResponse {
    pub rma_id: Uuid,
    pub status: RmaStatus,
    pub approved_at: DateTime<Utc>,
}

/// Request to receive returned goods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReceiveRmaRequest {
    pub received_items: Vec<ReceiveRmaItemRequest>,
    pub notes: Option<String>,
}

/// Individual item in RMA receive
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReceiveRmaItemRequest {
    pub rma_item_id: Uuid,
    pub received_quantity: i64,
    pub condition: RmaCondition,
    pub notes: Option<String>,
}

/// Response for RMA receive
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReceiveRmaResponse {
    pub rma_id: Uuid,
    pub status: RmaStatus,
    pub received_at: DateTime<Utc>,
    pub stock_moves_created: usize,
}
