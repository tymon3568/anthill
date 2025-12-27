use axum::{
    extract::{Extension, Query},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_auth::extractors::AuthUser;
use shared_error::AppError;
use sqlx::{FromRow, PgPool};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct StockLedgerQuery {
    /// Product ID to filter the ledger (required)
    pub product_id: Uuid,
    /// Warehouse ID to filter by location (optional)
    pub warehouse_id: Option<Uuid>,
    /// Start date for filtering (optional)
    pub date_from: Option<DateTime<Utc>>,
    /// End date for filtering (optional)
    pub date_to: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct StockLedgerEntry {
    /// Movement ID
    pub move_id: Uuid,
    /// Movement date
    pub move_date: DateTime<Utc>,
    /// Reference document type
    pub reference_type: String,
    /// Reference document ID
    pub reference_id: Uuid,
    /// Movement description/reason
    pub description: Option<String>,
    /// Quantity in (positive movements)
    pub quantity_in: Option<i64>,
    /// Quantity out (negative movements)
    pub quantity_out: Option<i64>,
    /// Running balance after this movement
    pub balance: i64,
    /// Unit cost
    pub unit_cost: Option<i64>,
    /// Total cost
    pub total_cost: Option<i64>,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct StockAgingEntry {
    /// Product ID
    pub product_id: Uuid,
    /// Product name
    pub product_name: String,
    /// Warehouse ID
    pub warehouse_id: Option<Uuid>,
    /// Warehouse name
    pub warehouse_name: Option<String>,
    /// Current stock quantity
    pub current_stock: i64,
    /// Aging bucket (e.g., "0-30 days", "31-60 days", etc.)
    pub aging_bucket: String,
    /// Days since last inbound movement
    pub days_since_last_inbound: Option<f64>,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct InventoryTurnoverEntry {
    /// Product ID
    pub product_id: Uuid,
    /// Product name
    pub product_name: String,
    /// Turnover ratio (COGS / Average Inventory Value)
    pub turnover_ratio: f64,
    /// Cost of Goods Sold
    pub cogs: i64,
    /// Average inventory value
    pub avg_inventory_value: i64,
    /// Reporting period
    pub period: String,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct LowStockEntry {
    /// Product ID
    pub product_id: Uuid,
    /// Product name
    pub product_name: String,
    /// Current stock quantity
    pub current_stock: i64,
    /// Effective reorder point (reorder_point + safety_stock)
    pub effective_reorder_point: i64,
    /// Warehouse ID
    pub warehouse_id: Option<Uuid>,
    /// Warehouse name
    pub warehouse_name: Option<String>,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct DeadStockEntry {
    /// Product ID
    pub product_id: Uuid,
    /// Product name
    pub product_name: String,
    /// Warehouse ID
    pub warehouse_id: Uuid,
    /// Warehouse name
    pub warehouse_name: String,
    /// Last outbound movement date
    pub last_outbound_date: Option<DateTime<Utc>>,
    /// Days since last outbound
    pub days_since_last_outbound: Option<i64>,
    /// Current stock quantity
    pub current_stock: i64,
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/reports/stock-ledger",
    tag = "reports",
    operation_id = "get_stock_ledger",
    params(
        ("product_id" = Uuid, Query, description = "Product ID to filter the ledger"),
        ("warehouse_id" = Option<Uuid>, Query, description = "Warehouse ID to filter by location"),
        ("date_from" = Option<DateTime<Utc>>, Query, description = "Start date for filtering"),
        ("date_to" = Option<DateTime<Utc>>, Query, description = "End date for filtering")
    ),
    responses(
        (status = 200, description = "Stock ledger report", body = Vec<StockLedgerEntry>),
        (status = 400, description = "Invalid query parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_stock_ledger(
    auth_user: AuthUser,
    Extension(pool): Extension<PgPool>,
    Query(query): Query<StockLedgerQuery>,
) -> Result<Json<Vec<StockLedgerEntry>>, AppError> {
    let tenant_id = auth_user.tenant_id;

    // Build the query with conditional WHERE clauses
    let sql = r#"
        SELECT
            sm.move_id,
            sm.move_date,
            sm.reference_type,
            sm.reference_id,
            sm.move_reason as description,
            CASE WHEN $3::uuid IS NOT NULL AND sm.destination_location_id = $3 THEN sm.quantity ELSE NULL END as quantity_in,
            CASE WHEN $3::uuid IS NOT NULL AND sm.source_location_id = $3 THEN sm.quantity ELSE NULL END as quantity_out,
            SUM(
                CASE
                    WHEN $3::uuid IS NOT NULL AND sm.destination_location_id = $3 THEN sm.quantity
                    WHEN $3::uuid IS NOT NULL AND sm.source_location_id = $3 THEN -sm.quantity
                    ELSE sm.quantity
                END
            ) OVER (
                PARTITION BY sm.product_id, $3::uuid
                ORDER BY sm.move_date, sm.created_at
                ROWS UNBOUNDED PRECEDING
            )::BIGINT as balance,
            sm.unit_cost,
            sm.total_cost
        FROM stock_moves sm
        WHERE sm.tenant_id = $1
        AND sm.product_id = $2
        AND ($3::UUID IS NULL OR sm.source_location_id = $3 OR sm.destination_location_id = $3)
        AND ($4::TIMESTAMPTZ IS NULL OR sm.move_date >= $4)
        AND ($5::TIMESTAMPTZ IS NULL OR sm.move_date <= $5)
        ORDER BY sm.move_date, sm.created_at
    "#;

    let entries = sqlx::query_as::<_, StockLedgerEntry>(sql)
        .bind(tenant_id)
        .bind(query.product_id)
        .bind(query.warehouse_id)
        .bind(query.date_from)
        .bind(query.date_to)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch stock ledger: {}", e)))?;

    Ok(Json(entries))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/reports/aging",
    tag = "reports",
    operation_id = "get_stock_aging",
    params(
        ("warehouse_id" = Option<Uuid>, Query, description = "Warehouse ID to filter by location")
    ),
    responses(
        (status = 200, description = "Stock aging report", body = Vec<StockAgingEntry>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_stock_aging(
    auth_user: AuthUser,
    Extension(pool): Extension<PgPool>,
    Query(query): Query<StockAgingQuery>,
) -> Result<Json<Vec<StockAgingEntry>>, AppError> {
    let tenant_id = auth_user.tenant_id;

    let sql = r#"
        WITH current_stock AS (
            SELECT
                product_id,
                source_location_id as warehouse_id,
                SUM(quantity) as on_hand
            FROM stock_moves
            WHERE tenant_id = $1 AND ($2::UUID IS NULL OR source_location_id = $2)
            GROUP BY product_id, source_location_id
            HAVING SUM(quantity) > 0
        ), last_inbound AS (
            SELECT
                product_id,
                source_location_id as warehouse_id,
                MAX(move_date) as last_inbound_date,
                (EXTRACT(EPOCH FROM (NOW() - MAX(move_date))) / 86400) as days
            FROM stock_moves
            WHERE tenant_id = $1 AND quantity > 0 AND ($2::UUID IS NULL OR source_location_id = $2)
            GROUP BY product_id, source_location_id
        )
        SELECT
            p.product_id,
            p.name as product_name,
            w.warehouse_id,
            w.name as warehouse_name,
            cs.on_hand as current_stock,
            li.days as days_since_last_inbound,
            CASE
                WHEN li.days IS NULL THEN 'Unknown'
                WHEN li.days <= 30 THEN '0-30 days'
                WHEN li.days <= 60 THEN '31-60 days'
                WHEN li.days <= 90 THEN '61-90 days'
                ELSE '>90 days'
            END as aging_bucket
        FROM current_stock cs
        JOIN products p ON cs.product_id = p.product_id
        JOIN warehouses w ON cs.warehouse_id = w.warehouse_id
        LEFT JOIN last_inbound li ON cs.product_id = li.product_id AND cs.warehouse_id = li.warehouse_id
        WHERE p.tenant_id = $1 AND w.tenant_id = $1
        ORDER BY p.name, w.name
    "#;

    let entries = sqlx::query_as::<_, StockAgingEntry>(sql)
        .bind(tenant_id)
        .bind(query.warehouse_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch stock aging: {}", e)))?;

    Ok(Json(entries))
}

#[derive(Deserialize, ToSchema)]
pub struct StockAgingQuery {
    /// Warehouse ID to filter by location (optional)
    pub warehouse_id: Option<Uuid>,
}

pub fn create_reports_routes() -> axum::Router {
    axum::Router::new()
        .route("/stock-ledger", axum::routing::get(get_stock_ledger))
        .route("/aging", axum::routing::get(get_stock_aging))
        .route("/turnover", axum::routing::get(get_inventory_turnover))
        .route("/low-stock", axum::routing::get(get_low_stock))
        .route("/dead-stock", axum::routing::get(get_dead_stock))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/reports/turnover",
    tag = "reports",
    operation_id = "get_inventory_turnover",
    params(
        ("period" = String, Query, description = "Reporting period in days (default: 90, e.g., '30', '90')")
    ),
    responses(
        (status = 200, description = "Inventory turnover report", body = Vec<InventoryTurnoverEntry>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_inventory_turnover(
    auth_user: AuthUser,
    Extension(pool): Extension<PgPool>,
    Query(query): Query<InventoryTurnoverQuery>,
) -> Result<Json<Vec<InventoryTurnoverEntry>>, AppError> {
    let tenant_id = auth_user.tenant_id;
    let period_days = query.period.parse::<i32>().map_err(|_| {
        AppError::ValidationError("period must be an integer number of days".to_string())
    })?;

    let sql = r#"
        WITH period_range AS (
            SELECT (NOW() - ($2::text || ' days')::interval) as start_date
        ),
        cogs AS (
            SELECT
                product_id,
                SUM(ABS(total_cost)) as total_cogs
            FROM stock_moves, period_range
            WHERE tenant_id = $1
              AND quantity < 0
              AND move_date >= period_range.start_date
            GROUP BY product_id
        ),
        start_inventory AS (
            SELECT product_id, SUM(total_cost) as value
            FROM stock_moves, period_range
            WHERE tenant_id = $1 AND move_date < period_range.start_date
            GROUP BY product_id
        ),
        end_inventory AS (
            SELECT product_id, SUM(total_cost) as value
            FROM stock_moves
            WHERE tenant_id = $1 AND move_date <= NOW()
            GROUP BY product_id
        ),
        avg_inventory AS (
            SELECT
                p.product_id,
                (COALESCE(si.value, 0) + COALESCE(ei.value, 0)) / 2 as avg_value
            FROM products p
            LEFT JOIN start_inventory si ON p.product_id = si.product_id
            LEFT JOIN end_inventory ei ON p.product_id = ei.product_id
            WHERE p.tenant_id = $1
        )
        SELECT
            p.product_id,
            p.name as product_name,
            COALESCE(c.total_cogs, 0) as cogs,
            COALESCE(a.avg_value, 0) as avg_inventory_value,
            CASE
                WHEN COALESCE(a.avg_value, 0) <= 0 THEN 0.0
                ELSE COALESCE(c.total_cogs, 0)::FLOAT / a.avg_value::FLOAT
            END as turnover_ratio,
            CONCAT($2, ' days') as period
        FROM products p
        LEFT JOIN cogs c ON p.product_id = c.product_id
        LEFT JOIN avg_inventory a ON p.product_id = a.product_id
        WHERE p.tenant_id = $1
        ORDER BY turnover_ratio DESC
    "#;

    let entries = sqlx::query_as::<_, InventoryTurnoverEntry>(sql)
        .bind(tenant_id)
        .bind(period_days)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch inventory turnover: {}", e))
        })?;

    Ok(Json(entries))
}

#[derive(Deserialize, ToSchema)]
pub struct InventoryTurnoverQuery {
    /// Reporting period in days (default: 90)
    #[serde(default = "default_period")]
    pub period: String,
}

fn default_period() -> String {
    "90".to_string()
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/reports/low-stock",
    tag = "reports",
    operation_id = "get_low_stock",
    params(
        ("warehouse_id" = Option<Uuid>, Query, description = "Warehouse ID to filter by location")
    ),
    responses(
        (status = 200, description = "Low stock report", body = Vec<LowStockEntry>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_low_stock(
    auth_user: AuthUser,
    Extension(pool): Extension<PgPool>,
    Query(query): Query<LowStockQuery>,
) -> Result<Json<Vec<LowStockEntry>>, AppError> {
    let tenant_id = auth_user.tenant_id;

    let sql = r#"
        SELECT
            p.product_id,
            p.name as product_name,
            COALESCE(SUM(sm.quantity), 0)::BIGINT as current_stock,
            (rr.reorder_point + rr.safety_stock) as effective_reorder_point,
            w.warehouse_id,
            w.name as warehouse_name
        FROM products p
        INNER JOIN reorder_rules rr ON p.product_id = rr.product_id AND p.tenant_id = rr.tenant_id
        CROSS JOIN warehouses w
        LEFT JOIN stock_moves sm ON p.product_id = sm.product_id
            AND w.warehouse_id = sm.source_location_id
            AND sm.tenant_id = $1
        WHERE p.tenant_id = $1
        AND w.tenant_id = $1
        AND rr.deleted_at IS NULL
        AND (rr.warehouse_id IS NULL OR rr.warehouse_id = w.warehouse_id)
        AND ($2::UUID IS NULL OR w.warehouse_id = $2)
        GROUP BY p.product_id, p.name, rr.reorder_point, rr.safety_stock, w.warehouse_id, w.name
        HAVING COALESCE(SUM(sm.quantity), 0) < (rr.reorder_point + rr.safety_stock)
        ORDER BY ((rr.reorder_point + rr.safety_stock) - COALESCE(SUM(sm.quantity), 0)) DESC
    "#;

    let entries = sqlx::query_as::<_, LowStockEntry>(sql)
        .bind(tenant_id)
        .bind(query.warehouse_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch low stock: {}", e)))?;

    Ok(Json(entries))
}

#[derive(Deserialize, ToSchema)]
pub struct LowStockQuery {
    /// Warehouse ID to filter by location (optional)
    pub warehouse_id: Option<Uuid>,
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/reports/dead-stock",
    tag = "reports",
    operation_id = "get_dead_stock",
    params(
        ("days_threshold" = Option<i32>, Query, description = "Days threshold for dead stock (default: 90)"),
        ("warehouse_id" = Option<Uuid>, Query, description = "Warehouse ID to filter by location")
    ),
    responses(
        (status = 200, description = "Dead stock report", body = Vec<DeadStockEntry>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dead_stock(
    auth_user: AuthUser,
    Extension(pool): Extension<PgPool>,
    Query(query): Query<DeadStockQuery>,
) -> Result<Json<Vec<DeadStockEntry>>, AppError> {
    let tenant_id = auth_user.tenant_id;
    let days_threshold = query.days_threshold.unwrap_or(90);

    let sql = r#"
        WITH stock_by_warehouse AS (
            SELECT
                product_id,
                source_location_id as warehouse_id,
                SUM(quantity) as stock_qty
            FROM stock_moves
            WHERE tenant_id = $1
              AND ($3::UUID IS NULL OR source_location_id = $3)
            GROUP BY product_id, source_location_id
            HAVING SUM(quantity) > 0
        ),
        last_outbound AS (
            SELECT
                product_id,
                source_location_id as warehouse_id,
                MAX(move_date) as last_date
            FROM stock_moves
            WHERE tenant_id = $1 AND quantity < 0 AND ($3::UUID IS NULL OR source_location_id = $3)
            GROUP BY product_id, source_location_id
        )
        SELECT
            p.product_id,
            p.name as product_name,
            w.warehouse_id,
            w.name as warehouse_name,
            lo.last_date as last_outbound_date,
            (EXTRACT(EPOCH FROM (NOW() - lo.last_date)) / 86400)::BIGINT as days_since_last_outbound,
            s.stock_qty as current_stock
        FROM stock_by_warehouse s
        JOIN products p ON s.product_id = p.product_id
        JOIN warehouses w ON s.warehouse_id = w.warehouse_id
        LEFT JOIN last_outbound lo ON s.product_id = lo.product_id AND s.warehouse_id = lo.warehouse_id
        WHERE p.tenant_id = $1
          AND w.tenant_id = $1
          AND (lo.last_date IS NULL OR lo.last_date < NOW() - INTERVAL '1 day' * $2)
        ORDER BY days_since_last_outbound DESC NULLS LAST
    "#;

    let entries = sqlx::query_as::<_, DeadStockEntry>(sql)
        .bind(tenant_id)
        .bind(days_threshold)
        .bind(query.warehouse_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch dead stock: {}", e)))?;

    Ok(Json(entries))
}

#[derive(Deserialize, ToSchema)]
pub struct DeadStockQuery {
    /// Days threshold for considering stock as dead (default: 90)
    pub days_threshold: Option<i32>,
    /// Warehouse ID to filter by location (optional)
    pub warehouse_id: Option<Uuid>,
}
