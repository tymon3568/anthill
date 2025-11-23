//! Stock Take Data Transfer Objects
//!
//! This module contains request and response structures for stock take operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::stock_take::{StockTake, StockTakeLine, StockTakeStatus};

/// Request to create a new stock take session
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateStockTakeRequest {
    /// Warehouse to perform stock take on
    pub warehouse_id: Uuid,
    /// Optional notes
    pub notes: Option<String>,
}

/// Response for stock take creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStockTakeResponse {
    /// The created stock take
    pub stock_take: StockTake,
}

/// Request to submit counted quantities for stock take lines
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CountStockTakeRequest {
    /// List of counted items
    #[validate(length(min = 1, message = "At least one item must be counted"))]
    pub items: Vec<CountItem>,
}

/// Individual count item
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CountItem {
    /// Product being counted
    pub product_id: Uuid,
    /// Actual counted quantity
    #[validate(range(min = 0, message = "Quantity cannot be negative"))]
    pub actual_quantity: i64,
    /// Optional notes for this count
    pub notes: Option<String>,
}

/// Response for count submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountStockTakeResponse {
    /// Updated stock take lines
    pub lines: Vec<StockTakeLine>,
}

/// Request to finalize a stock take (no body needed, just the ID in path)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeStockTakeRequest {}

/// Response for stock take finalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeStockTakeResponse {
    /// The finalized stock take
    pub stock_take: StockTake,
    /// Generated stock adjustments (if any)
    pub adjustments: Vec<StockAdjustment>,
}

/// Stock adjustment generated from stock take discrepancies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAdjustment {
    /// Adjustment ID
    pub adjustment_id: Uuid,
    /// Product adjusted
    pub product_id: Uuid,
    /// Warehouse
    pub warehouse_id: Uuid,
    /// Adjustment quantity (positive or negative)
    pub quantity: i64,
    /// Reason
    pub reason: String,
    /// Adjustment timestamp
    pub adjusted_at: DateTime<Utc>,
}

/// Query parameters for listing stock takes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockTakeListQuery {
    /// Filter by warehouse
    pub warehouse_id: Option<Uuid>,
    /// Filter by status
    pub status: Option<StockTakeStatus>,
    /// Page number (1-based)
    pub page: Option<u32>,
    /// Items per page
    pub limit: Option<u32>,
}

/// Response for stock take list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockTakeListResponse {
    /// List of stock takes
    pub stock_takes: Vec<StockTake>,
    /// Pagination info
    pub pagination: PaginationInfo,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Response for getting a single stock take with lines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockTakeDetailResponse {
    /// The stock take
    pub stock_take: StockTake,
    /// Associated lines
    pub lines: Vec<StockTakeLine>,
}
