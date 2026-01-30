//! Transfer DTOs for API requests and responses
//!
//! This module defines the data transfer objects for stock transfer operations.

use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domains::inventory::transfer::{
    Transfer, TransferItem, TransferPriority, TransferStatus, TransferType,
};

/// Request to create a new transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateTransferRequest {
    /// Optional external reference number
    pub reference_number: Option<String>,
    /// Source warehouse ID
    pub source_warehouse_id: Uuid,
    /// Destination warehouse ID
    pub destination_warehouse_id: Uuid,
    /// Transfer type (defaults to manual)
    #[serde(default)]
    pub transfer_type: TransferType,
    /// Priority level (defaults to normal)
    #[serde(default)]
    pub priority: TransferPriority,
    /// Expected ship date
    pub expected_ship_date: Option<String>, // ISO 8601 date string
    /// Expected receive date
    pub expected_receive_date: Option<String>, // ISO 8601 date string
    /// Shipping method
    pub shipping_method: Option<String>,
    /// Additional notes
    pub notes: Option<String>,
    /// Reason for transfer
    pub reason: Option<String>,
    /// Transfer items
    pub items: Vec<CreateTransferItemRequest>,
}

/// Request to create a transfer item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateTransferItemRequest {
    /// Product ID
    pub product_id: Uuid,
    /// Quantity to transfer
    pub quantity: i64,
    /// Unit of measure ID (optional - will use product's default UoM if not provided)
    pub uom_id: Option<Uuid>,
    /// Unit cost in cents (optional)
    pub unit_cost: Option<i64>,
    /// Line number for ordering
    pub line_number: i32,
    /// Source zone within source warehouse (optional, for precise tracking)
    #[serde(default)]
    pub source_zone_id: Option<Uuid>,
    /// Source location/bin within source warehouse (optional)
    #[serde(default)]
    pub source_location_id: Option<Uuid>,
    /// Destination zone within destination warehouse (optional)
    #[serde(default)]
    pub destination_zone_id: Option<Uuid>,
    /// Destination location/bin within destination warehouse (optional)
    #[serde(default)]
    pub destination_location_id: Option<Uuid>,
    /// Additional notes
    pub notes: Option<String>,
}

/// Response for transfer creation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateTransferResponse {
    /// Created transfer ID
    pub transfer_id: Uuid,
    /// Generated transfer number
    pub transfer_number: String,
    /// Transfer status
    pub status: TransferStatus,
    /// Number of items created
    pub items_count: usize,
}

/// Request to confirm a transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ConfirmTransferRequest {
    /// Optional notes for confirmation
    pub notes: Option<String>,
}

/// Response for transfer confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ConfirmTransferResponse {
    /// Transfer ID
    pub transfer_id: Uuid,
    /// Updated status
    pub status: TransferStatus,
    /// Confirmation timestamp
    pub confirmed_at: String, // ISO 8601
}

/// Request to receive a transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReceiveTransferRequest {
    /// Optional notes for receipt
    pub notes: Option<String>,
}

/// Response for transfer receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReceiveTransferResponse {
    /// Transfer ID
    pub transfer_id: Uuid,
    /// Updated status
    pub status: TransferStatus,
    /// Receipt timestamp
    pub received_at: String, // ISO 8601
    /// Number of stock moves created
    pub stock_moves_created: usize,
}

/// Full transfer response with items
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct TransferResponse {
    /// Transfer details
    #[serde(flatten)]
    pub transfer: Transfer,
    /// Transfer items
    pub items: Vec<TransferItem>,
}

/// Transfer summary for listings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct TransferSummary {
    /// Transfer ID
    pub transfer_id: Uuid,
    /// Transfer number
    pub transfer_number: String,
    /// Status
    pub status: TransferStatus,
    /// Source warehouse name
    pub source_warehouse_name: String,
    /// Destination warehouse name
    pub destination_warehouse_name: String,
    /// Total quantity
    pub total_quantity: i64,
    /// Total value
    pub total_value: i64,
    /// Currency
    pub currency_code: String,
    /// Created at
    pub created_at: String, // ISO 8601
}

/// Parameters for listing transfers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ListTransfersParams {
    /// Filter by source warehouse
    pub source_warehouse_id: Option<Uuid>,
    /// Filter by destination warehouse
    pub destination_warehouse_id: Option<Uuid>,
    /// Filter by status
    pub status: Option<TransferStatus>,
    /// Page number (1-based)
    pub page: Option<i64>,
    /// Page size (default 20)
    pub page_size: Option<i64>,
}

/// Paginated list response for transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ListTransfersResponse {
    /// List of transfers
    pub items: Vec<Transfer>,
    /// Total count matching the filter
    pub total: i64,
    /// Current page (1-based)
    pub page: i64,
    /// Page size
    pub page_size: i64,
    /// Total pages
    pub total_pages: i64,
}

/// Request to cancel a transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CancelTransferRequest {
    /// Reason for cancellation
    pub reason: Option<String>,
}

/// Response for transfer cancellation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CancelTransferResponse {
    /// Transfer ID
    pub transfer_id: Uuid,
    /// Updated status
    pub status: TransferStatus,
    /// Cancellation timestamp
    pub cancelled_at: String, // ISO 8601
}
