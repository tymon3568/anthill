//! Stock Levels DTOs
//!
//! Data transfer objects for stock level queries and responses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[cfg(feature = "openapi")]
use utoipa::{IntoParams, ToSchema};

use super::common::PaginationInfo;

/// Query parameters for listing stock levels
#[derive(Debug, Clone, Deserialize, Default, Validate)]
#[cfg_attr(feature = "openapi", derive(IntoParams))]
#[serde(rename_all = "camelCase")]
pub struct StockLevelListQuery {
    /// Filter by warehouse ID
    pub warehouse_id: Option<Uuid>,
    /// Filter by product ID
    pub product_id: Option<Uuid>,
    /// Search by product name or SKU
    pub search: Option<String>,
    /// Filter for low stock items only (below reorder point)
    pub low_stock_only: Option<bool>,
    /// Filter for out of stock items only (available_quantity = 0)
    pub out_of_stock_only: Option<bool>,
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: i32,
    /// Items per page
    #[serde(default = "default_page_size")]
    pub page_size: i32,
    /// Sort by field
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    /// Sort direction (asc/desc)
    #[serde(default = "default_sort_dir")]
    pub sort_dir: String,
}

fn default_page() -> i32 {
    1
}
fn default_page_size() -> i32 {
    20
}
fn default_sort_by() -> String {
    "product_name".to_string()
}
fn default_sort_dir() -> String {
    "asc".to_string()
}

/// Stock level response with product and warehouse details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct StockLevelResponse {
    /// Inventory level ID
    pub inventory_id: Uuid,
    /// Tenant ID
    pub tenant_id: Uuid,
    /// Product ID
    pub product_id: Uuid,
    /// Product SKU
    pub product_sku: String,
    /// Product name
    pub product_name: String,
    /// Warehouse ID
    pub warehouse_id: Uuid,
    /// Warehouse code
    pub warehouse_code: String,
    /// Warehouse name
    pub warehouse_name: String,
    /// Available quantity (can be sold/used)
    pub available_quantity: i64,
    /// Reserved quantity (held for orders)
    pub reserved_quantity: i64,
    /// Total quantity (available + reserved)
    pub total_quantity: i64,
    /// Stock status based on quantity
    pub status: StockStatus,
    /// Reorder point (if configured)
    pub reorder_point: Option<i64>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Stock status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum StockStatus {
    /// Quantity > 0 and above reorder point
    InStock,
    /// Quantity > 0 but below reorder point
    LowStock,
    /// Available quantity = 0
    OutOfStock,
}

impl StockStatus {
    pub fn from_quantities(available: i64, reorder_point: Option<i64>) -> Self {
        if available <= 0 {
            StockStatus::OutOfStock
        } else if let Some(rp) = reorder_point {
            if available <= rp {
                StockStatus::LowStock
            } else {
                StockStatus::InStock
            }
        } else {
            StockStatus::InStock
        }
    }
}

/// Summary statistics for stock levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct StockLevelSummary {
    /// Total number of products with stock records
    pub total_products: i64,
    /// Total available quantity across all products
    pub total_available_quantity: i64,
    /// Total reserved quantity across all products
    pub total_reserved_quantity: i64,
    /// Number of products with low stock
    pub low_stock_count: i64,
    /// Number of products that are out of stock
    pub out_of_stock_count: i64,
}

/// Paginated list of stock levels with summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct StockLevelListResponse {
    /// List of stock levels
    pub items: Vec<StockLevelResponse>,
    /// Pagination info
    pub pagination: PaginationInfo,
    /// Summary statistics
    pub summary: StockLevelSummary,
}
