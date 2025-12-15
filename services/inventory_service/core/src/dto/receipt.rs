//! Data Transfer Objects for receipt operations
//!
//! This module contains request and response structures for Goods Receipt Note (GRN) operations.

use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::dto::PaginationInfo;

/// Request to create a new Goods Receipt Note
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReceiptCreateRequest {
    /// Warehouse where goods are being received
    pub warehouse_id: Uuid,

    /// Supplier providing the goods (optional for now)
    pub supplier_id: Option<Uuid>,

    /// External reference number (purchase order, etc.)
    #[validate(length(max = 100))]
    pub reference_number: Option<String>,

    /// Expected delivery date from supplier
    pub expected_delivery_date: Option<chrono::DateTime<chrono::Utc>>,

    /// Additional notes about the receipt
    #[validate(length(max = 1000))]
    pub notes: Option<String>,

    /// ISO 4217 currency code (e.g., "VND", "USD")
    #[validate(length(min = 3, max = 3, message = "Currency code must be 3 characters"))]
    pub currency_code: String,

    /// Line items being received
    #[validate(length(min = 1, message = "At least one receipt item is required"))]
    pub items: Vec<ReceiptItemCreateRequest>,
}

/// Request to create a receipt line item
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReceiptItemCreateRequest {
    /// Product being received
    pub product_id: Uuid,

    /// Expected quantity from purchase order
    #[validate(range(min = 0, message = "Expected quantity must be non-negative"))]
    pub expected_quantity: i64,

    /// Actual quantity received
    #[validate(range(min = 0, message = "Received quantity must be non-negative"))]
    pub received_quantity: i64,

    /// Cost per unit in smallest currency unit (cents/xu)
    #[validate(range(min = 0, message = "Unit cost must be non-negative if provided"))]
    pub unit_cost: Option<i64>,

    /// Unit of measure for quantities
    pub uom_id: Option<Uuid>,

    /// Lot number for batch tracking
    #[validate(length(max = 100))]
    pub lot_number: Option<String>,

    /// Array of serial numbers if tracking method is 'serial'
    pub serial_numbers: Option<serde_json::Value>,

    /// Expiry date for perishable goods
    pub expiry_date: Option<chrono::DateTime<chrono::Utc>>,

    /// Additional notes for this line item
    #[validate(length(max = 500))]
    pub notes: Option<String>,
}

/// Response containing created receipt details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReceiptResponse {
    /// Unique receipt identifier
    pub receipt_id: Uuid,

    /// Auto-generated receipt number (GRN-YYYY-XXXXX)
    pub receipt_number: String,

    /// Tenant identifier
    pub tenant_id: Uuid,

    /// Warehouse where goods are received
    pub warehouse_id: Uuid,

    /// Supplier providing the goods
    pub supplier_id: Option<Uuid>,

    /// External reference number
    pub reference_number: Option<String>,

    /// Current receipt status
    pub status: String,

    /// Date when the GRN was created
    pub receipt_date: chrono::DateTime<chrono::Utc>,

    /// Expected delivery date
    pub expected_delivery_date: Option<chrono::DateTime<chrono::Utc>>,

    /// Actual delivery date
    pub actual_delivery_date: Option<chrono::DateTime<chrono::Utc>>,

    /// Additional notes
    pub notes: Option<String>,

    /// User who created the GRN
    pub created_by: Uuid,

    /// Total quantity of all items
    pub total_quantity: Option<i64>,

    /// Total value in smallest currency unit
    pub total_value: Option<i64>,

    /// ISO 4217 currency code
    pub currency_code: String,

    /// Line items in the receipt
    pub items: Vec<ReceiptItemResponse>,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Response containing receipt line item details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReceiptItemResponse {
    /// Unique item identifier
    pub receipt_item_id: Uuid,

    /// Parent receipt identifier
    pub receipt_id: Uuid,

    /// Tenant identifier
    pub tenant_id: Uuid,

    /// Product being received
    pub product_id: Uuid,

    /// Expected quantity from purchase order
    pub expected_quantity: i64,

    /// Actual quantity received
    pub received_quantity: i64,

    /// Cost per unit in smallest currency unit
    pub unit_cost: Option<i64>,

    /// Calculated total cost for this line
    pub line_total: Option<i64>,

    /// Unit of measure
    pub uom_id: Option<Uuid>,

    /// Lot number for batch tracking
    pub lot_number: Option<String>,

    /// Serial numbers JSON
    pub serial_numbers: Option<serde_json::Value>,

    /// Expiry date for perishable goods
    pub expiry_date: Option<chrono::DateTime<chrono::Utc>>,

    /// Additional notes
    pub notes: Option<String>,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Query parameters for listing receipts
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct ReceiptListQuery {
    /// Page number (1-based)
    #[serde(default = "default_page")]
    #[validate(range(min = 1))]
    pub page: u32,

    /// Items per page
    #[serde(default = "default_page_size")]
    #[validate(range(min = 1, max = 100))]
    pub page_size: u32,

    /// Filter by warehouse
    pub warehouse_id: Option<Uuid>,

    /// Filter by supplier
    pub supplier_id: Option<Uuid>,

    /// Filter by status
    pub status: Option<String>,

    /// Filter by receipt number or reference number
    pub search: Option<String>,

    /// Filter receipts created after this date
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,

    /// Filter receipts created before this date
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

/// Paginated response for receipt listing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReceiptListResponse {
    /// List of receipts
    pub receipts: Vec<ReceiptSummaryResponse>,

    /// Pagination metadata
    pub pagination: PaginationInfo,
}

/// Summary response for receipt listing (without full items)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReceiptSummaryResponse {
    /// Unique receipt identifier
    pub receipt_id: Uuid,

    /// Auto-generated receipt number
    pub receipt_number: String,

    /// Warehouse identifier
    pub warehouse_id: Uuid,

    /// Supplier identifier
    pub supplier_id: Option<Uuid>,

    /// External reference number
    pub reference_number: Option<String>,

    /// Current status
    pub status: String,

    /// Receipt creation date
    pub receipt_date: chrono::DateTime<chrono::Utc>,

    /// Total quantity
    pub total_quantity: Option<i64>,

    /// Total value
    pub total_value: Option<i64>,

    /// Currency code
    pub currency_code: String,

    /// Number of line items
    pub item_count: i32,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Helper functions and constants
// ============================================================================

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}
