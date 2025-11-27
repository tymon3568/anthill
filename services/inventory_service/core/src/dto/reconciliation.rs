//! Stock Reconciliation Data Transfer Objects
//!
//! This module contains request and response structures for reconciliation operations.

use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use super::stock_take::StockAdjustment;

use crate::domains::inventory::reconciliation::{
    CycleType, ReconciliationStatus, StockReconciliation, StockReconciliationItem,
};

/// Request to create a new reconciliation session
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateReconciliationRequest {
    /// Reconciliation name
    #[validate(length(
        min = 1,
        max = 255,
        message = "Name must be between 1 and 255 characters"
    ))]
    pub name: String,
    /// Optional description
    #[validate(length(max = 1000, message = "Description cannot exceed 1000 characters"))]
    pub description: Option<String>,
    /// Cycle counting type
    pub cycle_type: CycleType,
    /// Warehouse to reconcile (optional for full reconciliation)
    pub warehouse_id: Option<Uuid>,
    /// Location filter for location-based reconciliation
    pub location_filter: Option<serde_json::Value>,
    /// Product filter for category/ABC-based reconciliation
    pub product_filter: Option<serde_json::Value>,
    /// Optional notes
    pub notes: Option<String>,
}

/// Response for reconciliation creation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateReconciliationResponse {
    /// The created reconciliation
    pub reconciliation: StockReconciliation,
}

/// Request to record counts for reconciliation items
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CountReconciliationRequest {
    /// List of counted items
    #[validate(length(min = 1, message = "At least one item must be counted"))]
    pub items: Vec<ReconciliationCountItem>,
}

/// Individual count item
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReconciliationCountItem {
    /// Product being counted
    pub product_id: Uuid,
    /// Warehouse
    pub warehouse_id: Uuid,
    /// Specific location (optional)
    pub location_id: Option<Uuid>,
    /// Actual counted quantity
    #[validate(range(min = 0, message = "Quantity cannot be negative"))]
    pub counted_quantity: i64,
    /// Unit cost for valuation (optional)
    pub unit_cost: Option<f64>,
    /// Optional notes for this count
    pub notes: Option<String>,
}

/// Response for count submission
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CountReconciliationResponse {
    /// Updated reconciliation items
    pub items: Vec<StockReconciliationItem>,
}

/// Request to finalize a reconciliation (no body needed, just the ID in path)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct FinalizeReconciliationRequest {}

/// Response for reconciliation finalization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct FinalizeReconciliationResponse {
    /// The finalized reconciliation
    pub reconciliation: StockReconciliation,
    /// Generated stock adjustments (if any)
    pub adjustments: Vec<StockAdjustment>,
}

// StockAdjustment is imported from stock_take module for consistency

/// Request to approve a reconciliation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ApproveReconciliationRequest {
    /// Approval notes
    pub notes: Option<String>,
}

/// Response for reconciliation approval
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ApproveReconciliationResponse {
    /// The approved reconciliation
    pub reconciliation: StockReconciliation,
}

/// Query parameters for listing reconciliations
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(IntoParams))]
pub struct ReconciliationListQuery {
    /// Filter by warehouse
    pub warehouse_id: Option<Uuid>,
    /// Filter by status
    pub status: Option<ReconciliationStatus>,
    /// Filter by cycle type
    pub cycle_type: Option<CycleType>,
    /// Page number (1-based)
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    pub page: Option<u32>,
    /// Items per page
    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    pub limit: Option<u32>,
}

/// Query parameters for reconciliation analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(IntoParams))]
pub struct ReconciliationAnalyticsQuery {
    /// Warehouse ID filter
    pub warehouse_id: Option<Uuid>,
}

/// Response for reconciliation list
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReconciliationListResponse {
    /// List of reconciliations
    pub reconciliations: Vec<StockReconciliation>,
    /// Pagination info
    pub pagination: PaginationInfo,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PaginationInfo {
    /// Current page
    pub page: u32,
    /// Items per page
    pub limit: u32,
    /// Total items
    pub total: u64,
    /// Total pages
    pub total_pages: u32,
}

/// Response for getting a single reconciliation with items
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReconciliationDetailResponse {
    /// The reconciliation
    pub reconciliation: StockReconciliation,
    /// Associated items
    pub items: Vec<StockReconciliationItem>,
}

/// Reconciliation analytics/summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReconciliationAnalyticsResponse {
    /// Total reconciliations
    pub total_reconciliations: i64,
    /// Completed reconciliations
    pub completed_reconciliations: i64,
    /// Average variance percentage
    pub average_variance_percentage: Option<f64>,
    /// Total variance value
    pub total_variance_value: Option<f64>,
    /// Items with high variance (>5%)
    pub high_variance_items: i64,
    /// Reconciliation accuracy rate
    pub accuracy_rate: Option<f64>,
}

/// Variance analysis for a specific reconciliation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct VarianceAnalysisResponse {
    /// Reconciliation details
    pub reconciliation: StockReconciliation,
    /// Items grouped by variance range
    pub variance_ranges: Vec<VarianceRange>,
    /// Top 10 items with highest variance
    pub top_variance_items: Vec<StockReconciliationItem>,
}

/// Variance range grouping
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct VarianceRange {
    /// Range description (e.g., "0-1%", "1-5%", ">5%")
    pub range: String,
    /// Number of items in this range
    pub count: i64,
    /// Total variance value for items in this range
    pub total_variance_value: Option<f64>,
}
