use axum::{
    extract::{Extension, Query},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_auth::extractors::AuthUser;
use shared_error::AppError;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Deserialize)]
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

#[derive(Serialize, ToSchema)]
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
            CASE WHEN sm.quantity > 0 THEN sm.quantity::BIGINT ELSE NULL END as quantity_in,
            CASE WHEN sm.quantity < 0 THEN ABS(sm.quantity)::BIGINT ELSE NULL END as quantity_out,
            SUM(sm.quantity) OVER (
                PARTITION BY sm.product_id
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
        .bind(&tenant_id)
        .bind(&query.product_id)
        .bind(&query.warehouse_id)
        .bind(&query.date_from)
        .bind(&query.date_to)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch stock ledger: {}", e)))?;

    Ok(Json(entries))
}
