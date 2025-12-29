//! Reports Service Implementation
//!
//! PostgreSQL implementation of the ReportsService trait for inventory reports.
//! Follows the 3-crate pattern: api → infra → core → shared/*

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::dto::reports::{
    calculate_avg_inventory, calculate_dio, calculate_turnover_ratio, get_age_bucket_label,
    AgingBasis, StockAgingReportQuery, StockAgingReportResponse, StockAgingReportRow,
    TurnoverGroupBy, TurnoverReportQuery, TurnoverReportResponse, TurnoverReportRow,
};
use inventory_service_core::services::reports::ReportsService;
use shared_error::AppError;

/// PostgreSQL implementation of ReportsService
pub struct PgReportsService {
    pool: Arc<PgPool>,
}

impl PgReportsService {
    /// Create a new reports service instance
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// Helper struct for stock aging SQL results
#[derive(Debug, sqlx::FromRow)]
struct StockAgingRow {
    product_id: Uuid,
    product_sku: String,
    product_name: String,
    variant_id: Option<Uuid>,
    warehouse_id: Uuid,
    warehouse_name: String,
    location_id: Option<Uuid>,
    location_name: Option<String>,
    lot_id: Option<Uuid>,
    lot_number: Option<String>,
    qty_on_hand: i64,
    basis_timestamp: Option<DateTime<Utc>>,
    age_days: Option<i32>,
    value_cents: Option<i64>,
}

/// Helper struct for turnover SQL results
#[derive(Debug, sqlx::FromRow)]
struct TurnoverRow {
    group_id: Uuid,
    group_name: String,
    opening_qty: i64,
    closing_qty: i64,
    qty_consumed: i64,
    opening_value_cents: i64,
    closing_value_cents: i64,
    cogs_value_cents: i64,
}

#[async_trait]
impl ReportsService for PgReportsService {
    async fn stock_aging_report(
        &self,
        tenant_id: Uuid,
        query: StockAgingReportQuery,
    ) -> Result<StockAgingReportResponse, AppError> {
        let as_of = query.as_of.unwrap_or_else(Utc::now);
        let buckets = query.bucket_preset.buckets();
        let limit = query.limit.unwrap_or(50).min(100) as i64;
        let page = query.page.unwrap_or(1).max(1);
        let offset = ((page - 1) as i64) * limit;

        // Build the SQL query based on aging basis
        let sql = match query.aging_basis {
            AgingBasis::LastInbound => {
                r#"
                WITH current_stock AS (
                    SELECT
                        il.product_id,
                        il.warehouse_id,
                        il.location_id,
                        COALESCE(il.lot_serial_id, NULL) as lot_id,
                        SUM(il.available_quantity) as qty_on_hand
                    FROM inventory_levels il
                    WHERE il.tenant_id = $1
                      AND il.available_quantity > 0
                      AND il.deleted_at IS NULL
                      AND ($2::UUID IS NULL OR il.warehouse_id = $2)
                      AND ($3::UUID IS NULL OR il.location_id = $3)
                      AND ($4::UUID IS NULL OR il.product_id = $4)
                    GROUP BY il.product_id, il.warehouse_id, il.location_id, il.lot_serial_id
                ),
                last_inbound AS (
                    SELECT DISTINCT ON (sm.product_id, sm.destination_location_id)
                        sm.product_id,
                        sm.destination_location_id as warehouse_id,
                        sm.move_date as basis_timestamp
                    FROM stock_moves sm
                    WHERE sm.tenant_id = $1
                      AND sm.quantity > 0
                      AND sm.move_type IN ('receipt', 'grn', 'inbound', 'transfer_in')
                      AND sm.move_date <= $5
                    ORDER BY sm.product_id, sm.destination_location_id, sm.move_date DESC
                )
                SELECT
                    cs.product_id,
                    COALESCE(p.sku, '') as product_sku,
                    COALESCE(p.name, 'Unknown') as product_name,
                    NULL::UUID as variant_id,
                    cs.warehouse_id,
                    COALESCE(w.name, 'Unknown') as warehouse_name,
                    cs.location_id,
                    sl.name as location_name,
                    cs.lot_id,
                    ls.lot_number,
                    cs.qty_on_hand,
                    li.basis_timestamp,
                    EXTRACT(DAY FROM ($5::TIMESTAMPTZ - COALESCE(li.basis_timestamp, $5)))::INTEGER as age_days,
                    NULL::BIGINT as value_cents
                FROM current_stock cs
                LEFT JOIN products p ON cs.product_id = p.product_id AND p.tenant_id = $1
                LEFT JOIN warehouses w ON cs.warehouse_id = w.warehouse_id AND w.tenant_id = $1
                LEFT JOIN storage_locations sl ON cs.location_id = sl.location_id AND sl.tenant_id = $1
                LEFT JOIN lots_serial_numbers ls ON cs.lot_id = ls.lot_serial_id AND ls.tenant_id = $1
                LEFT JOIN last_inbound li ON cs.product_id = li.product_id AND cs.warehouse_id = li.warehouse_id
                WHERE ($6::UUID IS NULL OR p.category_id = $6)
                ORDER BY COALESCE(EXTRACT(DAY FROM ($5::TIMESTAMPTZ - li.basis_timestamp)), 9999) DESC, p.name
                LIMIT $7 OFFSET $8
                "#
            },
            AgingBasis::LastMovement => {
                r#"
                WITH current_stock AS (
                    SELECT
                        il.product_id,
                        il.warehouse_id,
                        il.location_id,
                        COALESCE(il.lot_serial_id, NULL) as lot_id,
                        SUM(il.available_quantity) as qty_on_hand
                    FROM inventory_levels il
                    WHERE il.tenant_id = $1
                      AND il.available_quantity > 0
                      AND il.deleted_at IS NULL
                      AND ($2::UUID IS NULL OR il.warehouse_id = $2)
                      AND ($3::UUID IS NULL OR il.location_id = $3)
                      AND ($4::UUID IS NULL OR il.product_id = $4)
                    GROUP BY il.product_id, il.warehouse_id, il.location_id, il.lot_serial_id
                ),
                last_movement AS (
                    SELECT DISTINCT ON (sm.product_id, COALESCE(sm.destination_location_id, sm.source_location_id))
                        sm.product_id,
                        COALESCE(sm.destination_location_id, sm.source_location_id) as warehouse_id,
                        sm.move_date as basis_timestamp
                    FROM stock_moves sm
                    WHERE sm.tenant_id = $1
                      AND sm.move_date <= $5
                    ORDER BY sm.product_id, COALESCE(sm.destination_location_id, sm.source_location_id), sm.move_date DESC
                )
                SELECT
                    cs.product_id,
                    COALESCE(p.sku, '') as product_sku,
                    COALESCE(p.name, 'Unknown') as product_name,
                    NULL::UUID as variant_id,
                    cs.warehouse_id,
                    COALESCE(w.name, 'Unknown') as warehouse_name,
                    cs.location_id,
                    sl.name as location_name,
                    cs.lot_id,
                    ls.lot_number,
                    cs.qty_on_hand,
                    lm.basis_timestamp,
                    EXTRACT(DAY FROM ($5::TIMESTAMPTZ - COALESCE(lm.basis_timestamp, $5)))::INTEGER as age_days,
                    NULL::BIGINT as value_cents
                FROM current_stock cs
                LEFT JOIN products p ON cs.product_id = p.product_id AND p.tenant_id = $1
                LEFT JOIN warehouses w ON cs.warehouse_id = w.warehouse_id AND w.tenant_id = $1
                LEFT JOIN storage_locations sl ON cs.location_id = sl.location_id AND sl.tenant_id = $1
                LEFT JOIN lots_serial_numbers ls ON cs.lot_id = ls.lot_serial_id AND ls.tenant_id = $1
                LEFT JOIN last_movement lm ON cs.product_id = lm.product_id AND cs.warehouse_id = lm.warehouse_id
                WHERE ($6::UUID IS NULL OR p.category_id = $6)
                ORDER BY COALESCE(EXTRACT(DAY FROM ($5::TIMESTAMPTZ - lm.basis_timestamp)), 9999) DESC, p.name
                LIMIT $7 OFFSET $8
                "#
            },
        };

        let rows = sqlx::query_as::<_, StockAgingRow>(sql)
            .bind(tenant_id)
            .bind(query.warehouse_id)
            .bind(query.location_id)
            .bind(query.product_id)
            .bind(as_of)
            .bind(query.category_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to fetch stock aging: {}", e)))?;

        // Count total for pagination
        let count_sql = r#"
            SELECT COUNT(DISTINCT (il.product_id, il.warehouse_id, il.location_id, il.lot_serial_id))
            FROM inventory_levels il
            LEFT JOIN products p ON il.product_id = p.product_id AND p.tenant_id = $1
            WHERE il.tenant_id = $1
              AND il.available_quantity > 0
              AND il.deleted_at IS NULL
              AND ($2::UUID IS NULL OR il.warehouse_id = $2)
              AND ($3::UUID IS NULL OR il.location_id = $3)
              AND ($4::UUID IS NULL OR il.product_id = $4)
              AND ($5::UUID IS NULL OR p.category_id = $5)
        "#;

        let total_count: (i64,) = sqlx::query_as(count_sql)
            .bind(tenant_id)
            .bind(query.warehouse_id)
            .bind(query.location_id)
            .bind(query.product_id)
            .bind(query.category_id)
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to count stock aging rows: {}", e))
            })?;

        // Convert SQL rows to response DTOs
        let report_rows: Vec<StockAgingReportRow> = rows
            .into_iter()
            .map(|row| {
                let age_days = row.age_days.unwrap_or(0);
                let age_bucket = get_age_bucket_label(age_days, &buckets);

                StockAgingReportRow {
                    product_id: row.product_id,
                    product_sku: row.product_sku,
                    product_name: row.product_name,
                    variant_id: row.variant_id,
                    warehouse_id: row.warehouse_id,
                    warehouse_name: row.warehouse_name,
                    location_id: row.location_id,
                    location_name: row.location_name,
                    lot_id: row.lot_id,
                    lot_number: row.lot_number,
                    qty_on_hand: row.qty_on_hand,
                    basis_timestamp: row.basis_timestamp,
                    age_days,
                    age_bucket,
                    value_cents: row.value_cents,
                }
            })
            .collect();

        Ok(StockAgingReportResponse {
            rows: report_rows,
            as_of,
            aging_basis: query.aging_basis,
            buckets,
            total_count: total_count.0 as u64,
            page,
            page_size: limit as u32,
        })
    }

    async fn inventory_turnover_report(
        &self,
        tenant_id: Uuid,
        query: TurnoverReportQuery,
    ) -> Result<TurnoverReportResponse, AppError> {
        let period_days = query.period_days();
        let limit = query.limit.unwrap_or(50).min(100) as i64;
        let page = query.page.unwrap_or(1).max(1);
        let offset = ((page - 1) as i64) * limit;

        // Build SQL based on grouping
        let sql = match query.group_by {
            TurnoverGroupBy::Product => {
                r#"
                WITH opening_inventory AS (
                    SELECT
                        sm.product_id as group_id,
                        SUM(CASE WHEN sm.destination_location_id IS NOT NULL THEN sm.quantity ELSE -sm.quantity END) as qty,
                        SUM(COALESCE(sm.total_cost, 0)) as value_cents
                    FROM stock_moves sm
                    WHERE sm.tenant_id = $1
                      AND sm.move_date < $2
                      AND ($5::UUID IS NULL OR sm.destination_location_id = $5 OR sm.source_location_id = $5)
                      AND ($6::UUID IS NULL OR sm.product_id = $6)
                    GROUP BY sm.product_id
                ),
                closing_inventory AS (
                    SELECT
                        sm.product_id as group_id,
                        SUM(CASE WHEN sm.destination_location_id IS NOT NULL THEN sm.quantity ELSE -sm.quantity END) as qty,
                        SUM(COALESCE(sm.total_cost, 0)) as value_cents
                    FROM stock_moves sm
                    WHERE sm.tenant_id = $1
                      AND sm.move_date <= $3
                      AND ($5::UUID IS NULL OR sm.destination_location_id = $5 OR sm.source_location_id = $5)
                      AND ($6::UUID IS NULL OR sm.product_id = $6)
                    GROUP BY sm.product_id
                ),
                period_consumption AS (
                    SELECT
                        sm.product_id as group_id,
                        SUM(sm.quantity) as qty_consumed,
                        SUM(ABS(COALESCE(sm.total_cost, 0))) as cogs_value_cents
                    FROM stock_moves sm
                    WHERE sm.tenant_id = $1
                      AND sm.move_date >= $2
                      AND sm.move_date <= $3
                      AND sm.quantity < 0
                      AND ($5::UUID IS NULL OR sm.source_location_id = $5)
                      AND ($6::UUID IS NULL OR sm.product_id = $6)
                    GROUP BY sm.product_id
                )
                SELECT
                    p.product_id as group_id,
                    p.name as group_name,
                    COALESCE(oi.qty, 0) as opening_qty,
                    COALESCE(ci.qty, 0) as closing_qty,
                    ABS(COALESCE(pc.qty_consumed, 0)) as qty_consumed,
                    COALESCE(oi.value_cents, 0) as opening_value_cents,
                    COALESCE(ci.value_cents, 0) as closing_value_cents,
                    COALESCE(pc.cogs_value_cents, 0) as cogs_value_cents
                FROM products p
                LEFT JOIN opening_inventory oi ON p.product_id = oi.group_id
                LEFT JOIN closing_inventory ci ON p.product_id = ci.group_id
                LEFT JOIN period_consumption pc ON p.product_id = pc.group_id
                WHERE p.tenant_id = $1
                  AND p.deleted_at IS NULL
                  AND ($6::UUID IS NULL OR p.product_id = $6)
                  AND ($7::UUID IS NULL OR p.category_id = $7)
                ORDER BY COALESCE(pc.cogs_value_cents, 0) DESC, p.name
                LIMIT $8 OFFSET $9
                "#
            },
            TurnoverGroupBy::Category => {
                r#"
                WITH opening_inventory AS (
                    SELECT
                        p.category_id as group_id,
                        SUM(CASE WHEN sm.destination_location_id IS NOT NULL THEN sm.quantity ELSE -sm.quantity END) as qty,
                        SUM(COALESCE(sm.total_cost, 0)) as value_cents
                    FROM stock_moves sm
                    JOIN products p ON sm.product_id = p.product_id AND p.tenant_id = $1
                    WHERE sm.tenant_id = $1
                      AND sm.move_date < $2
                      AND ($5::UUID IS NULL OR sm.destination_location_id = $5 OR sm.source_location_id = $5)
                      AND ($6::UUID IS NULL OR sm.product_id = $6)
                    GROUP BY p.category_id
                ),
                closing_inventory AS (
                    SELECT
                        p.category_id as group_id,
                        SUM(CASE WHEN sm.destination_location_id IS NOT NULL THEN sm.quantity ELSE -sm.quantity END) as qty,
                        SUM(COALESCE(sm.total_cost, 0)) as value_cents
                    FROM stock_moves sm
                    JOIN products p ON sm.product_id = p.product_id AND p.tenant_id = $1
                    WHERE sm.tenant_id = $1
                      AND sm.move_date <= $3
                      AND ($5::UUID IS NULL OR sm.destination_location_id = $5 OR sm.source_location_id = $5)
                      AND ($6::UUID IS NULL OR sm.product_id = $6)
                    GROUP BY p.category_id
                ),
                period_consumption AS (
                    SELECT
                        p.category_id as group_id,
                        SUM(sm.quantity) as qty_consumed,
                        SUM(ABS(COALESCE(sm.total_cost, 0))) as cogs_value_cents
                    FROM stock_moves sm
                    JOIN products p ON sm.product_id = p.product_id AND p.tenant_id = $1
                    WHERE sm.tenant_id = $1
                      AND sm.move_date >= $2
                      AND sm.move_date <= $3
                      AND sm.quantity < 0
                      AND ($5::UUID IS NULL OR sm.source_location_id = $5)
                      AND ($6::UUID IS NULL OR sm.product_id = $6)
                    GROUP BY p.category_id
                )
                SELECT
                    c.category_id as group_id,
                    c.name as group_name,
                    COALESCE(oi.qty, 0) as opening_qty,
                    COALESCE(ci.qty, 0) as closing_qty,
                    ABS(COALESCE(pc.qty_consumed, 0)) as qty_consumed,
                    COALESCE(oi.value_cents, 0) as opening_value_cents,
                    COALESCE(ci.value_cents, 0) as closing_value_cents,
                    COALESCE(pc.cogs_value_cents, 0) as cogs_value_cents
                FROM product_categories c
                LEFT JOIN opening_inventory oi ON c.category_id = oi.group_id
                LEFT JOIN closing_inventory ci ON c.category_id = ci.group_id
                LEFT JOIN period_consumption pc ON c.category_id = pc.group_id
                WHERE c.tenant_id = $1
                  AND c.deleted_at IS NULL
                  AND ($7::UUID IS NULL OR c.category_id = $7)
                ORDER BY COALESCE(pc.cogs_value_cents, 0) DESC, c.name
                LIMIT $8 OFFSET $9
                "#
            },
            TurnoverGroupBy::Warehouse => {
                r#"
                WITH opening_inventory AS (
                    SELECT
                        COALESCE(sm.destination_location_id, sm.source_location_id) as group_id,
                        SUM(CASE WHEN sm.destination_location_id IS NOT NULL THEN sm.quantity ELSE -sm.quantity END) as qty,
                        SUM(COALESCE(sm.total_cost, 0)) as value_cents
                    FROM stock_moves sm
                    WHERE sm.tenant_id = $1
                      AND sm.move_date < $2
                      AND ($5::UUID IS NULL OR sm.destination_location_id = $5 OR sm.source_location_id = $5)
                      AND ($6::UUID IS NULL OR sm.product_id = $6)
                    GROUP BY COALESCE(sm.destination_location_id, sm.source_location_id)
                ),
                closing_inventory AS (
                    SELECT
                        COALESCE(sm.destination_location_id, sm.source_location_id) as group_id,
                        SUM(CASE WHEN sm.destination_location_id IS NOT NULL THEN sm.quantity ELSE -sm.quantity END) as qty,
                        SUM(COALESCE(sm.total_cost, 0)) as value_cents
                    FROM stock_moves sm
                    WHERE sm.tenant_id = $1
                      AND sm.move_date <= $3
                      AND ($5::UUID IS NULL OR sm.destination_location_id = $5 OR sm.source_location_id = $5)
                      AND ($6::UUID IS NULL OR sm.product_id = $6)
                    GROUP BY COALESCE(sm.destination_location_id, sm.source_location_id)
                ),
                period_consumption AS (
                    SELECT
                        sm.source_location_id as group_id,
                        SUM(sm.quantity) as qty_consumed,
                        SUM(ABS(COALESCE(sm.total_cost, 0))) as cogs_value_cents
                    FROM stock_moves sm
                    WHERE sm.tenant_id = $1
                      AND sm.move_date >= $2
                      AND sm.move_date <= $3
                      AND sm.quantity < 0
                      AND ($5::UUID IS NULL OR sm.source_location_id = $5)
                      AND ($6::UUID IS NULL OR sm.product_id = $6)
                    GROUP BY sm.source_location_id
                )
                SELECT
                    w.warehouse_id as group_id,
                    w.name as group_name,
                    COALESCE(oi.qty, 0) as opening_qty,
                    COALESCE(ci.qty, 0) as closing_qty,
                    ABS(COALESCE(pc.qty_consumed, 0)) as qty_consumed,
                    COALESCE(oi.value_cents, 0) as opening_value_cents,
                    COALESCE(ci.value_cents, 0) as closing_value_cents,
                    COALESCE(pc.cogs_value_cents, 0) as cogs_value_cents
                FROM warehouses w
                LEFT JOIN opening_inventory oi ON w.warehouse_id = oi.group_id
                LEFT JOIN closing_inventory ci ON w.warehouse_id = ci.group_id
                LEFT JOIN period_consumption pc ON w.warehouse_id = pc.group_id
                WHERE w.tenant_id = $1
                  AND w.deleted_at IS NULL
                  AND ($5::UUID IS NULL OR w.warehouse_id = $5)
                ORDER BY COALESCE(pc.cogs_value_cents, 0) DESC, w.name
                LIMIT $8 OFFSET $9
                "#
            },
        };

        // Note: $4 is skipped in SQL queries, so we don't bind period_days
        // Binding order: $1=tenant_id, $2=from, $3=to, $5=warehouse_id, $6=product_id, $7=category_id, $8=limit, $9=offset
        // PostgreSQL allows gaps in parameter numbering when using explicit $N syntax
        let rows = sqlx::query_as::<_, TurnoverRow>(sql)
            .bind(tenant_id)       // $1
            .bind(query.from)      // $2
            .bind(query.to)        // $3
            .bind(Option::<i64>::None) // $4 placeholder (unused but needed for parameter alignment)
            .bind(query.warehouse_id)  // $5
            .bind(query.product_id)    // $6
            .bind(query.category_id)   // $7
            .bind(limit)           // $8
            .bind(offset)          // $9
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to fetch inventory turnover: {}", e))
            })?;

        // Count total for pagination - use separate binding logic per group_by variant
        let total_count: (i64,) = match query.group_by {
            TurnoverGroupBy::Product => {
                sqlx::query_as(
                    r#"SELECT COUNT(*) FROM products WHERE tenant_id = $1 AND deleted_at IS NULL
                       AND ($2::UUID IS NULL OR product_id = $2)
                       AND ($3::UUID IS NULL OR category_id = $3)"#,
                )
                .bind(tenant_id)
                .bind(query.product_id)
                .bind(query.category_id)
                .fetch_one(self.pool.as_ref())
                .await
            }
            TurnoverGroupBy::Category => {
                sqlx::query_as(
                    r#"SELECT COUNT(*) FROM product_categories WHERE tenant_id = $1 AND deleted_at IS NULL
                       AND ($2::UUID IS NULL OR category_id = $2)"#,
                )
                .bind(tenant_id)
                .bind(query.category_id)
                .fetch_one(self.pool.as_ref())
                .await
            }
            TurnoverGroupBy::Warehouse => {
                sqlx::query_as(
                    r#"SELECT COUNT(*) FROM warehouses WHERE tenant_id = $1 AND deleted_at IS NULL
                       AND ($2::UUID IS NULL OR warehouse_id = $2)"#,
                )
                .bind(tenant_id)
                .bind(query.warehouse_id)
                .fetch_one(self.pool.as_ref())
                .await
            }
        }
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to count turnover rows: {}", e))
        })?;

        // Convert SQL rows to response DTOs with calculated metrics
        let report_rows: Vec<TurnoverReportRow> = rows
            .into_iter()
            .map(|row| {
                let avg_inventory_value_cents =
                    calculate_avg_inventory(row.opening_value_cents, row.closing_value_cents);
                let turnover_ratio =
                    calculate_turnover_ratio(row.cogs_value_cents, avg_inventory_value_cents);
                let days_inventory_outstanding = calculate_dio(turnover_ratio, period_days);

                TurnoverReportRow {
                    group_id: row.group_id,
                    group_name: row.group_name,
                    opening_inventory_value_cents: row.opening_value_cents,
                    closing_inventory_value_cents: row.closing_value_cents,
                    avg_inventory_value_cents,
                    cogs_value_cents: row.cogs_value_cents,
                    turnover_ratio,
                    days_inventory_outstanding,
                    opening_qty: row.opening_qty,
                    closing_qty: row.closing_qty,
                    qty_consumed: row.qty_consumed,
                }
            })
            .collect();

        Ok(TurnoverReportResponse {
            rows: report_rows,
            from: query.from,
            to: query.to,
            period_days,
            group_by: query.group_by,
            total_count: total_count.0 as u64,
            page,
            page_size: limit as u32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for helper functions that don't require DB
    #[test]
    fn test_turnover_calculation() {
        let row = TurnoverRow {
            group_id: Uuid::new_v4(),
            group_name: "Test Product".to_string(),
            opening_qty: 100,
            closing_qty: 50,
            qty_consumed: 150,
            opening_value_cents: 100_000, // $1,000
            closing_value_cents: 50_000,  // $500
            cogs_value_cents: 150_000,    // $1,500
        };

        let avg = calculate_avg_inventory(row.opening_value_cents, row.closing_value_cents);
        assert_eq!(avg, 75_000); // $750

        let turnover = calculate_turnover_ratio(row.cogs_value_cents, avg);
        assert!((turnover - 2.0).abs() < 0.001); // 1500/750 = 2.0

        let dio = calculate_dio(turnover, 90);
        assert!(dio.is_some());
        assert!((dio.unwrap() - 45.0).abs() < 0.001); // 90/2 = 45 days
    }

    #[test]
    fn test_turnover_zero_inventory() {
        let turnover = calculate_turnover_ratio(100_000, 0);
        assert_eq!(turnover, 0.0);

        let dio = calculate_dio(0.0, 90);
        assert!(dio.is_none());
    }
}
