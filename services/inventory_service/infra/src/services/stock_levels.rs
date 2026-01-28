//! Stock Levels Service Implementation
//!
//! PostgreSQL implementation of the StockLevelsService trait for listing
//! inventory stock levels with product and warehouse details.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::dto::common::PaginationInfo;
use inventory_service_core::dto::stock_levels::{
    StockLevelListQuery, StockLevelListResponse, StockLevelResponse, StockLevelSummary, StockStatus,
};
use inventory_service_core::services::stock_levels::StockLevelsService;
use shared_error::AppError;

/// PostgreSQL implementation of StockLevelsService
pub struct PgStockLevelsService {
    pool: Arc<PgPool>,
}

impl PgStockLevelsService {
    /// Create a new stock levels service instance
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// Helper struct for stock level SQL query results
#[derive(Debug, sqlx::FromRow)]
struct StockLevelRow {
    inventory_id: Uuid,
    tenant_id: Uuid,
    product_id: Uuid,
    product_sku: String,
    product_name: String,
    warehouse_id: Uuid,
    warehouse_code: String,
    warehouse_name: String,
    available_quantity: i64,
    reserved_quantity: i64,
    updated_at: chrono::DateTime<chrono::Utc>,
}

/// Helper struct for summary SQL query results
#[derive(Debug, sqlx::FromRow)]
struct SummaryRow {
    total_products: Option<i64>,
    total_available_quantity: Option<i64>,
    total_reserved_quantity: Option<i64>,
    low_stock_count: Option<i64>,
    out_of_stock_count: Option<i64>,
}

#[async_trait]
impl StockLevelsService for PgStockLevelsService {
    async fn list_stock_levels(
        &self,
        tenant_id: Uuid,
        query: StockLevelListQuery,
    ) -> Result<StockLevelListResponse, AppError> {
        // Calculate offset
        let page = query.page.max(1);
        let page_size = query.page_size.clamp(1, 100);
        let offset = (page - 1) * page_size;

        // Build the base WHERE clause
        // Note: We use raw SQL with dynamic conditions since sqlx doesn't support
        // dynamic query building well

        // Determine sort column and direction
        let sort_col = match query.sort_by.as_str() {
            "product_sku" | "sku" => "p.sku",
            "product_name" | "name" => "p.name",
            "warehouse_code" => "w.warehouse_code",
            "warehouse_name" => "w.warehouse_name",
            "available_quantity" | "available" => "il.available_quantity",
            "reserved_quantity" | "reserved" => "il.reserved_quantity",
            "updated_at" => "il.updated_at",
            _ => "p.name",
        };

        let sort_dir = if query.sort_dir.to_lowercase() == "desc" {
            "DESC"
        } else {
            "ASC"
        };

        // Build query with all filters embedded
        // We use COALESCE for optional filters to avoid complex dynamic SQL
        let items: Vec<StockLevelRow> = sqlx::query_as::<_, StockLevelRow>(&format!(
            r#"
            SELECT
                il.inventory_id,
                il.tenant_id,
                il.product_id,
                p.sku as product_sku,
                p.name as product_name,
                il.warehouse_id,
                w.warehouse_code,
                w.warehouse_name,
                il.available_quantity,
                il.reserved_quantity,
                il.updated_at
            FROM inventory_levels il
            INNER JOIN products p ON p.product_id = il.product_id AND p.tenant_id = il.tenant_id
            INNER JOIN warehouses w ON w.warehouse_id = il.warehouse_id AND w.tenant_id = il.tenant_id
            WHERE il.tenant_id = $1
                AND il.deleted_at IS NULL
                AND p.deleted_at IS NULL
                AND w.deleted_at IS NULL
                AND ($2::uuid IS NULL OR il.warehouse_id = $2)
                AND ($3::uuid IS NULL OR il.product_id = $3)
                AND ($4::text IS NULL OR p.name ILIKE '%' || $4 || '%' OR p.sku ILIKE '%' || $4 || '%')
                AND ($5::boolean IS NOT TRUE OR il.available_quantity = 0)
            ORDER BY {} {}
            LIMIT $6 OFFSET $7
            "#,
            sort_col, sort_dir
        ))
        .bind(tenant_id)
        .bind(query.warehouse_id)
        .bind(query.product_id)
        .bind(query.search.as_deref())
        .bind(query.out_of_stock_only)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Get total count for pagination
        let total_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM inventory_levels il
            INNER JOIN products p ON p.product_id = il.product_id AND p.tenant_id = il.tenant_id
            INNER JOIN warehouses w ON w.warehouse_id = il.warehouse_id AND w.tenant_id = il.tenant_id
            WHERE il.tenant_id = $1
                AND il.deleted_at IS NULL
                AND p.deleted_at IS NULL
                AND w.deleted_at IS NULL
                AND ($2::uuid IS NULL OR il.warehouse_id = $2)
                AND ($3::uuid IS NULL OR il.product_id = $3)
                AND ($4::text IS NULL OR p.name ILIKE '%' || $4 || '%' OR p.sku ILIKE '%' || $4 || '%')
                AND ($5::boolean IS NOT TRUE OR il.available_quantity = 0)
            "#,
        )
        .bind(tenant_id)
        .bind(query.warehouse_id)
        .bind(query.product_id)
        .bind(query.search.as_deref())
        .bind(query.out_of_stock_only)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Get summary statistics (without filters to show overall tenant stats)
        let summary_row: SummaryRow = sqlx::query_as(
            r#"
            SELECT
                COUNT(DISTINCT il.product_id)::bigint as total_products,
                COALESCE(SUM(il.available_quantity), 0)::bigint as total_available_quantity,
                COALESCE(SUM(il.reserved_quantity), 0)::bigint as total_reserved_quantity,
                COUNT(DISTINCT CASE WHEN il.available_quantity > 0 AND il.available_quantity <= 10 THEN il.product_id END)::bigint as low_stock_count,
                COUNT(DISTINCT CASE WHEN il.available_quantity = 0 THEN il.product_id END)::bigint as out_of_stock_count
            FROM inventory_levels il
            INNER JOIN products p ON p.product_id = il.product_id AND p.tenant_id = il.tenant_id
            WHERE il.tenant_id = $1
                AND il.deleted_at IS NULL
                AND p.deleted_at IS NULL
                AND ($2::uuid IS NULL OR il.warehouse_id = $2)
            "#,
        )
        .bind(tenant_id)
        .bind(query.warehouse_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Convert rows to response items
        let stock_levels: Vec<StockLevelResponse> = items
            .into_iter()
            .map(|row| {
                let total = row.available_quantity + row.reserved_quantity;
                // Default reorder point - in a real system this would come from product settings
                let reorder_point: Option<i64> = None;
                let status = StockStatus::from_quantities(row.available_quantity, reorder_point);

                StockLevelResponse {
                    inventory_id: row.inventory_id,
                    tenant_id: row.tenant_id,
                    product_id: row.product_id,
                    product_sku: row.product_sku,
                    product_name: row.product_name,
                    warehouse_id: row.warehouse_id,
                    warehouse_code: row.warehouse_code,
                    warehouse_name: row.warehouse_name,
                    available_quantity: row.available_quantity,
                    reserved_quantity: row.reserved_quantity,
                    total_quantity: total,
                    status,
                    reorder_point,
                    updated_at: row.updated_at,
                }
            })
            .collect();

        Ok(StockLevelListResponse {
            items: stock_levels,
            pagination: PaginationInfo::new(page as u32, page_size as u32, total_count as u64),
            summary: StockLevelSummary {
                total_products: summary_row.total_products.unwrap_or(0),
                total_available_quantity: summary_row.total_available_quantity.unwrap_or(0),
                total_reserved_quantity: summary_row.total_reserved_quantity.unwrap_or(0),
                low_stock_count: summary_row.low_stock_count.unwrap_or(0),
                out_of_stock_count: summary_row.out_of_stock_count.unwrap_or(0),
            },
        })
    }
}
