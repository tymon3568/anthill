//! Reports Service Trait
//!
//! This module defines the service trait for inventory reporting operations.
//! No implementations here - pure interfaces following the 3-crate pattern.

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::reports::{
    StockAgingReportQuery, StockAgingReportResponse, TurnoverReportQuery, TurnoverReportResponse,
};
use shared_error::AppError;

/// Service trait for inventory reporting operations
///
/// Provides analytical reports for inventory management including:
/// - Stock aging analysis
/// - Inventory turnover metrics
///
/// All reports enforce tenant isolation and support filtering/pagination.
#[async_trait]
pub trait ReportsService: Send + Sync {
    /// Generate stock aging report
    ///
    /// Analyzes inventory by age based on last inbound or last movement dates.
    /// Groups stock into configurable age buckets.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `query` - Report parameters including filters, aging basis, and bucket preset
    ///
    /// # Returns
    /// Stock aging report with rows grouped by product/location and age bucket
    ///
    /// # Features
    /// - Supports two aging bases: `last_inbound` (GRN receipt) or `last_movement`
    /// - Configurable bucket presets (default, monthly, quarterly)
    /// - Optional lot-level detail
    /// - Point-in-time analysis via `as_of` parameter
    /// - Filters by warehouse, location, product, category
    async fn stock_aging_report(
        &self,
        tenant_id: Uuid,
        query: StockAgingReportQuery,
    ) -> Result<StockAgingReportResponse, AppError>;

    /// Generate inventory turnover report
    ///
    /// Calculates inventory efficiency metrics over a specified time period:
    /// - Turnover ratio: COGS / Average Inventory Value
    /// - Days Inventory Outstanding (DIO): Period Days / Turnover Ratio
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `query` - Report parameters including date range, filters, and grouping
    ///
    /// # Returns
    /// Turnover report with metrics per group (product, category, or warehouse)
    ///
    /// # Metrics Calculated
    /// - Opening inventory value (as of `from`)
    /// - Closing inventory value (as of `to`)
    /// - Average inventory value: (opening + closing) / 2
    /// - COGS/consumption value: sum of outgoing movement values in period
    /// - Turnover ratio: COGS / avg_inventory (0 if avg is zero)
    /// - DIO: period_days / turnover_ratio (None if turnover is zero)
    ///
    /// # Notes
    /// - All monetary values are in BIGINT cents
    /// - Division by zero is handled safely (returns 0 or None)
    async fn inventory_turnover_report(
        &self,
        tenant_id: Uuid,
        query: TurnoverReportQuery,
    ) -> Result<TurnoverReportResponse, AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    // Test that query structs can be constructed properly
    #[test]
    fn test_stock_aging_query_defaults() {
        use crate::dto::reports::{AgeBucketPreset, AgingBasis};

        let query = StockAgingReportQuery {
            warehouse_id: None,
            location_id: None,
            aging_basis: AgingBasis::default(),
            as_of: None,
            bucket_preset: AgeBucketPreset::default(),
            product_id: None,
            variant_id: None,
            category_id: None,
            include_lots: false,
            page: None,
            limit: None,
        };

        assert_eq!(query.aging_basis, AgingBasis::LastInbound);
        assert_eq!(query.bucket_preset, AgeBucketPreset::Default);
        assert!(!query.include_lots);
    }

    #[test]
    fn test_turnover_query_period_days() {
        use crate::dto::reports::TurnoverGroupBy;

        let query = TurnoverReportQuery {
            from: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            to: Utc.with_ymd_and_hms(2024, 3, 31, 23, 59, 59).unwrap(),
            warehouse_id: None,
            location_id: None,
            product_id: None,
            category_id: None,
            group_by: TurnoverGroupBy::Product,
            page: None,
            limit: None,
        };

        // Q1 2024: Jan 1 to Mar 31 = 90 days
        assert_eq!(query.period_days(), 89); // to - from in whole days
    }
}
