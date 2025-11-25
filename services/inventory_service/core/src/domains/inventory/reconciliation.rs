//! Stock Reconciliation domain entities
//!
//! This module defines the domain entities for stock reconciliations and reconciliation items.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a stock reconciliation session
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StockReconciliation {
    /// Primary key
    pub reconciliation_id: Uuid,
    /// Tenant isolation
    pub tenant_id: Uuid,
    /// Auto-generated reconciliation number
    pub reconciliation_number: String,
    /// Reconciliation name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Reconciliation status
    pub status: ReconciliationStatus,
    /// Cycle counting type
    pub cycle_type: CycleType,
    /// Warehouse being reconciled (optional for full reconciliation)
    pub warehouse_id: Option<Uuid>,
    /// Location filter for location-based reconciliation
    pub location_filter: Option<serde_json::Value>,
    /// Product filter for category/ABC-based reconciliation
    pub product_filter: Option<serde_json::Value>,
    /// Total items in this reconciliation
    pub total_items: i32,
    /// Number of items counted
    pub counted_items: i32,
    /// Total variance across all items
    pub total_variance: i64,
    /// User who created the reconciliation
    pub created_by: Uuid,
    /// When reconciliation was created
    pub created_at: DateTime<Utc>,
    /// When reconciliation was last updated
    pub updated_at: DateTime<Utc>,
    /// When counting started
    pub started_at: Option<DateTime<Utc>>,
    /// When reconciliation was completed
    pub completed_at: Option<DateTime<Utc>>,
    /// User who approved the reconciliation
    pub approved_by: Option<Uuid>,
    /// When reconciliation was approved
    pub approved_at: Option<DateTime<Utc>>,
    /// Additional notes
    pub notes: Option<String>,
}

/// Reconciliation status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum ReconciliationStatus {
    Draft,
    InProgress,
    Completed,
    Cancelled,
}

impl fmt::Display for ReconciliationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReconciliationStatus::Draft => write!(f, "draft"),
            ReconciliationStatus::InProgress => write!(f, "in_progress"),
            ReconciliationStatus::Completed => write!(f, "completed"),
            ReconciliationStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Cycle counting type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum CycleType {
    Full,
    AbcA,
    AbcB,
    AbcC,
    Location,
    Random,
}

impl fmt::Display for CycleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CycleType::Full => write!(f, "full"),
            CycleType::AbcA => write!(f, "abc_a"),
            CycleType::AbcB => write!(f, "abc_b"),
            CycleType::AbcC => write!(f, "abc_c"),
            CycleType::Location => write!(f, "location"),
            CycleType::Random => write!(f, "random"),
        }
    }
}

/// Represents an item in a stock reconciliation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StockReconciliationItem {
    /// Tenant isolation
    pub tenant_id: Uuid,
    /// Parent reconciliation
    pub reconciliation_id: Uuid,
    /// Product being reconciled
    pub product_id: Uuid,
    /// Warehouse
    pub warehouse_id: Uuid,
    /// Specific location within warehouse
    pub location_id: Option<Uuid>,
    /// Expected quantity from system
    pub expected_quantity: i64,
    /// Actual counted quantity
    pub counted_quantity: Option<i64>,
    /// Variance (counted - expected)
    pub variance: Option<i64>,
    /// Variance percentage
    pub variance_percentage: Option<f64>,
    /// Unit cost for valuation
    pub unit_cost: Option<f64>,
    /// Variance value (variance * unit_cost)
    pub variance_value: Option<f64>,
    /// Notes for this item
    pub notes: Option<String>,
    /// User who performed the count
    pub counted_by: Option<Uuid>,
    /// When the count was performed
    pub counted_at: Option<DateTime<Utc>>,
    /// When this item was created
    pub created_at: DateTime<Utc>,
    /// When this item was last updated
    pub updated_at: DateTime<Utc>,
}
