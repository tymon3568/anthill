//! Transfer domain entities
//!
//! This module defines the domain entities for stock transfers and transfer items.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a stock transfer between warehouses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Transfer {
    /// Primary key
    pub transfer_id: Uuid,
    /// Tenant isolation
    pub tenant_id: Uuid,
    /// Auto-generated transfer number
    pub transfer_number: String,
    /// Optional external reference
    pub reference_number: Option<String>,
    /// Source warehouse
    pub source_warehouse_id: Uuid,
    /// Destination warehouse
    pub destination_warehouse_id: Uuid,
    /// Transfer status
    pub status: TransferStatus,
    /// Transfer type
    pub transfer_type: TransferType,
    /// Priority level
    pub priority: TransferPriority,
    /// Transfer date
    pub transfer_date: DateTime<Utc>,
    /// Expected ship date
    pub expected_ship_date: Option<DateTime<Utc>>,
    /// Actual ship date
    pub actual_ship_date: Option<DateTime<Utc>>,
    /// Expected receive date
    pub expected_receive_date: Option<DateTime<Utc>>,
    /// Actual receive date
    pub actual_receive_date: Option<DateTime<Utc>>,
    /// Shipping method
    pub shipping_method: Option<String>,
    /// Shipping carrier
    pub carrier: Option<String>,
    /// Tracking number
    pub tracking_number: Option<String>,
    /// Shipping cost in cents
    pub shipping_cost: Option<i64>,
    /// Additional notes
    pub notes: Option<String>,
    /// Reason for transfer
    pub reason: Option<String>,
    /// User who created
    pub created_by: Uuid,
    /// User who updated
    pub updated_by: Option<Uuid>,
    /// User who approved
    pub approved_by: Option<Uuid>,
    /// Approval timestamp
    pub approved_at: Option<DateTime<Utc>>,
    /// Total quantity
    pub total_quantity: i64,
    /// Total value in cents
    pub total_value: i64,
    /// Currency code
    pub currency_code: String,
    /// Audit timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Soft delete
    pub deleted_at: Option<DateTime<Utc>>,
    /// User who deleted
    pub deleted_by: Option<Uuid>,
}

/// Transfer status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum TransferStatus {
    Draft,
    Confirmed,
    PartiallyPicked,
    Picked,
    PartiallyShipped,
    Shipped,
    Received,
    Cancelled,
}

/// Transfer type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum TransferType {
    #[default]
    Manual,
    AutoReplenishment,
    Emergency,
    Consolidation,
}

/// Transfer priority enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum TransferPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

/// Represents an item in a stock transfer
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct TransferItem {
    /// Primary key
    pub transfer_item_id: Uuid,
    /// Tenant isolation
    pub tenant_id: Uuid,
    /// Parent transfer
    pub transfer_id: Uuid,
    /// Product being transferred
    pub product_id: Uuid,
    /// Quantity to transfer
    pub quantity: i64,
    /// Unit of measure
    pub uom_id: Uuid,
    /// Unit cost in cents
    pub unit_cost: Option<i64>,
    /// Calculated line total
    pub line_total: i64,
    /// Line number for ordering
    pub line_number: i32,
    /// Additional notes
    pub notes: Option<String>,
    /// User who updated
    pub updated_by: Option<Uuid>,
    /// Audit timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Soft delete
    pub deleted_at: Option<DateTime<Utc>>,
    /// User who deleted
    pub deleted_by: Option<Uuid>,
}
