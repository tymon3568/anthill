//! Reports Data Transfer Objects
//!
//! This module contains request and response DTOs for inventory reports,
//! following the 3-crate pattern with zero infrastructure dependencies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Stock Aging Report DTOs
// ============================================================================

/// Aging basis determines how the age of stock is calculated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum AgingBasis {
    /// Age since last inbound movement (receipt/GRN)
    #[default]
    LastInbound,
    /// Age since last movement of any type
    LastMovement,
}

impl std::fmt::Display for AgingBasis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgingBasis::LastInbound => write!(f, "last_inbound"),
            AgingBasis::LastMovement => write!(f, "last_movement"),
        }
    }
}

/// Predefined age buckets for stock aging analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum AgeBucketPreset {
    /// Default preset: 0-30, 31-60, 61-90, 91-180, 181-365, 365+
    #[default]
    Default,
    /// Monthly preset: 0-30, 31-60, 61-90, 91-120, 121-150, 151-180, 180+
    Monthly,
    /// Quarterly preset: 0-90, 91-180, 181-270, 271-365, 365+
    Quarterly,
}

/// Represents an age bucket with range boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AgeBucket {
    /// Bucket label (e.g., "0-30 days")
    pub label: String,
    /// Minimum days (inclusive)
    pub min_days: i32,
    /// Maximum days (exclusive), None means unlimited
    pub max_days: Option<i32>,
}

impl AgeBucketPreset {
    /// Get the age buckets for this preset
    pub fn buckets(&self) -> Vec<AgeBucket> {
        match self {
            AgeBucketPreset::Default => vec![
                AgeBucket {
                    label: "0-30 days".to_string(),
                    min_days: 0,
                    max_days: Some(31),
                },
                AgeBucket {
                    label: "31-60 days".to_string(),
                    min_days: 31,
                    max_days: Some(61),
                },
                AgeBucket {
                    label: "61-90 days".to_string(),
                    min_days: 61,
                    max_days: Some(91),
                },
                AgeBucket {
                    label: "91-180 days".to_string(),
                    min_days: 91,
                    max_days: Some(181),
                },
                AgeBucket {
                    label: "181-365 days".to_string(),
                    min_days: 181,
                    max_days: Some(366),
                },
                AgeBucket {
                    label: "366+ days".to_string(),
                    min_days: 366,
                    max_days: None,
                },
            ],
            AgeBucketPreset::Monthly => vec![
                AgeBucket {
                    label: "0-30 days".to_string(),
                    min_days: 0,
                    max_days: Some(31),
                },
                AgeBucket {
                    label: "31-60 days".to_string(),
                    min_days: 31,
                    max_days: Some(61),
                },
                AgeBucket {
                    label: "61-90 days".to_string(),
                    min_days: 61,
                    max_days: Some(91),
                },
                AgeBucket {
                    label: "91-120 days".to_string(),
                    min_days: 91,
                    max_days: Some(121),
                },
                AgeBucket {
                    label: "121-150 days".to_string(),
                    min_days: 121,
                    max_days: Some(151),
                },
                AgeBucket {
                    label: "151-180 days".to_string(),
                    min_days: 151,
                    max_days: Some(181),
                },
                AgeBucket {
                    label: "181+ days".to_string(),
                    min_days: 181,
                    max_days: None,
                },
            ],
            AgeBucketPreset::Quarterly => vec![
                AgeBucket {
                    label: "0-90 days".to_string(),
                    min_days: 0,
                    max_days: Some(91),
                },
                AgeBucket {
                    label: "91-180 days".to_string(),
                    min_days: 91,
                    max_days: Some(181),
                },
                AgeBucket {
                    label: "181-270 days".to_string(),
                    min_days: 181,
                    max_days: Some(271),
                },
                AgeBucket {
                    label: "271-365 days".to_string(),
                    min_days: 271,
                    max_days: Some(366),
                },
                AgeBucket {
                    label: "366+ days".to_string(),
                    min_days: 366,
                    max_days: None,
                },
            ],
        }
    }
}

/// Query parameters for stock aging report
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(IntoParams, ToSchema))]
pub struct StockAgingReportQuery {
    /// Warehouse ID filter (optional)
    pub warehouse_id: Option<Uuid>,
    /// Location ID filter - implies subtree (optional)
    pub location_id: Option<Uuid>,
    /// Aging basis (default: last_inbound)
    #[serde(default)]
    pub aging_basis: AgingBasis,
    /// As-of timestamp for point-in-time analysis (default: now)
    pub as_of: Option<DateTime<Utc>>,
    /// Bucket preset (default: default)
    #[serde(default)]
    pub bucket_preset: AgeBucketPreset,
    /// Product ID filter (optional)
    pub product_id: Option<Uuid>,
    /// Variant ID filter (optional)
    pub variant_id: Option<Uuid>,
    /// Category ID filter (optional)
    pub category_id: Option<Uuid>,
    /// Include lot-level detail (default: false)
    #[serde(default)]
    pub include_lots: bool,
    /// Page number for pagination (1-based)
    pub page: Option<u32>,
    /// Items per page (default: 50, max: 100)
    pub limit: Option<u32>,
}

/// A single row in the stock aging report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StockAgingReportRow {
    /// Product ID
    pub product_id: Uuid,
    /// Product SKU
    pub product_sku: String,
    /// Product name
    pub product_name: String,
    /// Variant ID (if applicable)
    pub variant_id: Option<Uuid>,
    /// Warehouse ID
    pub warehouse_id: Uuid,
    /// Warehouse name
    pub warehouse_name: String,
    /// Location ID (if location-level)
    pub location_id: Option<Uuid>,
    /// Location name
    pub location_name: Option<String>,
    /// Lot ID (if include_lots is true)
    pub lot_id: Option<Uuid>,
    /// Lot number
    pub lot_number: Option<String>,
    /// Quantity on hand
    pub qty_on_hand: i64,
    /// Basis timestamp used for aging calculation
    pub basis_timestamp: Option<DateTime<Utc>>,
    /// Days since basis timestamp
    pub age_days: i32,
    /// Age bucket label
    pub age_bucket: String,
    /// Inventory value in cents (if valuation available)
    pub value_cents: Option<i64>,
}

/// Response for stock aging report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StockAgingReportResponse {
    /// Report rows
    pub rows: Vec<StockAgingReportRow>,
    /// As-of timestamp used for this report
    pub as_of: DateTime<Utc>,
    /// Aging basis used
    pub aging_basis: AgingBasis,
    /// Bucket definitions used
    pub buckets: Vec<AgeBucket>,
    /// Total row count (for pagination)
    pub total_count: u64,
    /// Current page
    pub page: u32,
    /// Page size
    pub page_size: u32,
}

// ============================================================================
// Inventory Turnover Report DTOs
// ============================================================================

/// Grouping options for turnover report
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum TurnoverGroupBy {
    /// Group by product
    #[default]
    Product,
    /// Group by category
    Category,
    /// Group by warehouse
    Warehouse,
}

impl std::fmt::Display for TurnoverGroupBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TurnoverGroupBy::Product => write!(f, "product"),
            TurnoverGroupBy::Category => write!(f, "category"),
            TurnoverGroupBy::Warehouse => write!(f, "warehouse"),
        }
    }
}

/// Query parameters for inventory turnover report
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(IntoParams, ToSchema))]
pub struct TurnoverReportQuery {
    /// Start date (required)
    pub from: DateTime<Utc>,
    /// End date (required)
    pub to: DateTime<Utc>,
    /// Warehouse ID filter (optional)
    pub warehouse_id: Option<Uuid>,
    /// Location ID filter (optional)
    pub location_id: Option<Uuid>,
    /// Product ID filter (optional)
    pub product_id: Option<Uuid>,
    /// Category ID filter (optional)
    pub category_id: Option<Uuid>,
    /// Grouping option (default: product)
    #[serde(default)]
    pub group_by: TurnoverGroupBy,
    /// Page number for pagination (1-based)
    pub page: Option<u32>,
    /// Items per page (default: 50, max: 100)
    pub limit: Option<u32>,
}

impl TurnoverReportQuery {
    /// Get the number of days in the query period
    /// Returns 0 if from is after to (invalid range)
    pub fn period_days(&self) -> i64 {
        (self.to - self.from).num_days().max(0)
    }
}

/// A single row in the turnover report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct TurnoverReportRow {
    /// Group identifier (product_id, category_id, or warehouse_id)
    pub group_id: Uuid,
    /// Group name (product name, category name, or warehouse name)
    pub group_name: String,
    /// Opening inventory value in cents
    pub opening_inventory_value_cents: i64,
    /// Closing inventory value in cents
    pub closing_inventory_value_cents: i64,
    /// Average inventory value in cents: (opening + closing) / 2
    pub avg_inventory_value_cents: i64,
    /// Cost of goods sold / consumption value in cents
    pub cogs_value_cents: i64,
    /// Turnover ratio: cogs / avg_inventory_value
    /// Returns 0.0 if avg_inventory_value is 0
    pub turnover_ratio: f64,
    /// Days inventory outstanding: period_days / turnover_ratio
    /// Returns None if turnover_ratio is 0
    pub days_inventory_outstanding: Option<f64>,
    /// Opening quantity
    pub opening_qty: i64,
    /// Closing quantity
    pub closing_qty: i64,
    /// Quantity sold/consumed in period
    pub qty_consumed: i64,
}

/// Response for inventory turnover report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct TurnoverReportResponse {
    /// Report rows
    pub rows: Vec<TurnoverReportRow>,
    /// Period start date
    pub from: DateTime<Utc>,
    /// Period end date
    pub to: DateTime<Utc>,
    /// Number of days in period
    pub period_days: i64,
    /// Grouping used
    pub group_by: TurnoverGroupBy,
    /// Total row count (for pagination)
    pub total_count: u64,
    /// Current page
    pub page: u32,
    /// Page size
    pub page_size: u32,
}

// ============================================================================
// Pure Domain Logic (no infrastructure dependencies)
// ============================================================================

/// Calculate turnover ratio from COGS and average inventory value
/// Returns 0.0 if average inventory value is zero or negative
pub fn calculate_turnover_ratio(cogs_cents: i64, avg_inventory_cents: i64) -> f64 {
    if avg_inventory_cents <= 0 {
        return 0.0;
    }
    cogs_cents as f64 / avg_inventory_cents as f64
}

/// Calculate Days Inventory Outstanding (DIO) from turnover ratio
/// Returns None if turnover ratio is zero
pub fn calculate_dio(turnover_ratio: f64, period_days: i64) -> Option<f64> {
    if turnover_ratio <= 0.0 {
        return None;
    }
    Some(period_days as f64 / turnover_ratio)
}

/// Calculate average inventory value from opening and closing values
/// Uses i128 intermediate to prevent overflow for large monetary amounts
pub fn calculate_avg_inventory(opening_cents: i64, closing_cents: i64) -> i64 {
    let sum = opening_cents as i128 + closing_cents as i128;
    (sum / 2) as i64
}

/// Determine age bucket label for a given age in days
pub fn get_age_bucket_label(age_days: i32, buckets: &[AgeBucket]) -> String {
    for bucket in buckets {
        let in_range =
            age_days >= bucket.min_days && bucket.max_days.is_none_or(|max| age_days < max);
        if in_range {
            return bucket.label.clone();
        }
    }
    // Fallback - should not happen with proper bucket config
    "Unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turnover_ratio_normal() {
        let ratio = calculate_turnover_ratio(100_000, 50_000);
        assert!((ratio - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_turnover_ratio_zero_inventory() {
        let ratio = calculate_turnover_ratio(100_000, 0);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_turnover_ratio_negative_inventory() {
        let ratio = calculate_turnover_ratio(100_000, -50_000);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_dio_normal() {
        let dio = calculate_dio(2.0, 90);
        assert!(dio.is_some());
        assert!((dio.unwrap() - 45.0).abs() < 0.001);
    }

    #[test]
    fn test_dio_zero_turnover() {
        let dio = calculate_dio(0.0, 90);
        assert!(dio.is_none());
    }

    #[test]
    fn test_avg_inventory() {
        let avg = calculate_avg_inventory(100_000, 200_000);
        assert_eq!(avg, 150_000);
    }

    #[test]
    fn test_age_bucket_default_preset() {
        let buckets = AgeBucketPreset::Default.buckets();

        assert_eq!(get_age_bucket_label(0, &buckets), "0-30 days");
        assert_eq!(get_age_bucket_label(30, &buckets), "0-30 days");
        assert_eq!(get_age_bucket_label(31, &buckets), "31-60 days");
        assert_eq!(get_age_bucket_label(60, &buckets), "31-60 days");
        assert_eq!(get_age_bucket_label(61, &buckets), "61-90 days");
        assert_eq!(get_age_bucket_label(91, &buckets), "91-180 days");
        assert_eq!(get_age_bucket_label(181, &buckets), "181-365 days");
        assert_eq!(get_age_bucket_label(366, &buckets), "366+ days");
        assert_eq!(get_age_bucket_label(1000, &buckets), "366+ days");
    }

    #[test]
    fn test_period_days() {
        use chrono::TimeZone;
        let query = TurnoverReportQuery {
            from: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            to: Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap(),
            warehouse_id: None,
            location_id: None,
            product_id: None,
            category_id: None,
            group_by: TurnoverGroupBy::Product,
            page: None,
            limit: None,
        };
        assert_eq!(query.period_days(), 91);
    }
}
