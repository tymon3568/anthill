//! Stock Levels HTTP handlers
//!
//! This module contains the Axum handlers for stock levels API endpoints.

use axum::{
    extract::{Extension, Query},
    response::Json,
    routing::get,
    Router,
};

use validator::Validate;

// Import DTOs for requests/responses
use inventory_service_core::dto::stock_levels::{StockLevelListQuery, StockLevelListResponse};

use shared_auth::extractors::AuthUser;
use shared_error::AppError;

use crate::state::AppState;

/// Create the stock levels routes
pub fn create_stock_levels_routes() -> Router {
    Router::new().route("/", get(list_stock_levels))
}

/// GET /api/v1/inventory/stock-levels - List stock levels with pagination and filtering
///
/// Retrieves a paginated list of inventory stock levels with product and warehouse details.
/// Supports filtering by warehouse, product, and search terms.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `warehouse_id` - Filter by warehouse (optional)
/// * `product_id` - Filter by product (optional)
/// * `search` - Search by product name or SKU (optional)
/// * `low_stock_only` - Filter for low stock items only (optional)
/// * `out_of_stock_only` - Filter for out of stock items only (optional)
/// * `page` - Page number (default: 1)
/// * `page_size` - Items per page (default: 20, max: 100)
/// * `sort_by` - Sort field (default: product_name)
/// * `sort_dir` - Sort direction: asc/desc (default: asc)
///
/// # Returns
/// * `200` - Paginated list of stock levels with summary statistics
/// * `400` - Invalid query parameters
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    get,
    path = "/api/v1/inventory/stock-levels/",
    tag = "stock-levels",
    operation_id = "list_stock_levels",
    params(StockLevelListQuery),
    responses(
        (status = 200, description = "Paginated list of stock levels", body = StockLevelListResponse),
        (status = 400, description = "Invalid query parameters"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_stock_levels(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(query): Query<StockLevelListQuery>,
) -> Result<Json<StockLevelListResponse>, AppError> {
    // Validate query parameters if StockLevelListQuery implements Validate
    if let Err(e) = query.validate() {
        return Err(AppError::ValidationError(e.to_string()));
    }

    let response = state
        .stock_levels_service
        .list_stock_levels(auth_user.tenant_id, query)
        .await?;

    Ok(Json(response))
}
