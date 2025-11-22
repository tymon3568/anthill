//! Transfer DTOs for API requests and responses
//!
//! This module defines the data transfer objects for stock transfer operations.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domains::inventory::transfer::{
    Transfer, TransferItem, TransferPriority, TransferStatus, TransferType,
};

/// Request to create a new transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct CreateTransferItemRequest {
    /// Product ID
    pub product_id: Uuid,
    /// Quantity to transfer
    pub quantity: i64,
    /// Unit of measure ID
    pub uom_id: Uuid,
    /// Unit cost in cents (optional)
    pub unit_cost: Option<i64>,
    /// Line number for ordering
    pub line_number: i32,
    /// Additional notes
    pub notes: Option<String>,
}

/// Response for transfer creation
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct ConfirmTransferRequest {
    /// Optional notes for confirmation
    pub notes: Option<String>,
}

/// Response for transfer confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct ReceiveTransferRequest {
    /// Optional notes for receipt
    pub notes: Option<String>,
}

/// Response for transfer receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct TransferResponse {
    /// Transfer details
    #[serde(flatten)]
    pub transfer: Transfer,
    /// Transfer items
    pub items: Vec<TransferItem>,
}

/// Transfer summary for listings
#[derive(Debug, Clone, Serialize, Deserialize)]
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
