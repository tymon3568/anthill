//! Putaway HTTP handlers
//!
//! This module contains the Axum handlers for putaway operations.

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use uuid::Uuid;
use validator::Validate;

use inventory_service_core::models::{
    ConfirmPutawayRequest, ConfirmPutawayResponse, PutawayRequest, PutawayResponse,
};
use shared_auth::extractors::AuthUser;
use shared_error::AppError;

use crate::state::AppState;

/// Create the putaway routes
pub fn create_putaway_routes() -> Router {
    Router::new()
        .route("/suggest", post(suggest_putaway))
        .route("/confirm", post(confirm_putaway))
}

/// POST /api/v1/warehouse/putaway/suggest - Get optimal putaway locations for items
///
/// Analyzes putaway rules and suggests optimal storage locations for incoming goods
/// based on product characteristics, warehouse layout, and business rules.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `request` - Putaway request with product ID, quantity, and preferences
///
/// # Returns
/// * `200` - List of suggested locations with scores and capacity information
/// * `400` - Invalid request data
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Example
/// ```json
/// POST /api/v1/warehouse/putaway/suggest
/// {
///   "product_id": "550e8400-e29b-41d4-a716-446655440000",
///   "quantity": 50,
///   "warehouse_id": "550e8400-e29b-41d4-a716-446655440001",
///   "preferred_location_type": "pallet_rack",
///   "attributes": {
///     "fragile": true,
///     "size": "large"
///   }
/// }
/// ```
///
/// Response:
/// ```json
/// {
///   "suggestions": [
///     {
///       "location_id": "550e8400-e29b-41d4-a716-446655440002",
///       "location_code": "A-01-01-01",
///       "warehouse_id": "550e8400-e29b-41d4-a716-446655440001",
///       "zone": "A",
///       "aisle": "01",
///       "rack": "01",
///       "level": 1,
///       "position": 1,
///       "available_capacity": 150,
///       "current_stock": 0,
///       "score": 95,
///       "rule_applied": "High-priority product rule"
///     }
///   ],
///   "total_quantity": 50,
///   "allocated_quantity": 50
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/warehouse/putaway/suggest",
    tag = "warehouse",
    operation_id = "suggest_putaway",
    request_body = PutawayRequest,
    responses(
        (status = 200, description = "Putaway suggestions generated", body = PutawayResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn suggest_putaway(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<PutawayRequest>,
) -> Result<Json<PutawayResponse>, AppError> {
    // Validate request
    if request.quantity <= 0 {
        return Err(AppError::ValidationError("Quantity must be positive".to_string()));
    }

    let suggestions = state
        .putaway_service
        .suggest_putaway_locations(&auth_user.tenant_id, &request)
        .await?;

    let total_quantity = request.quantity;
    let allocated_quantity = suggestions
        .iter()
        .map(|s| s.available_capacity.unwrap_or(0))
        .sum::<i64>()
        .min(total_quantity);

    let response = PutawayResponse {
        suggestions,
        total_quantity,
        allocated_quantity,
    };

    Ok(Json(response))
}

/// POST /api/v1/warehouse/putaway/confirm - Confirm putaway and create stock moves
///
/// Confirms the putaway of goods to specified locations, creating the necessary
/// stock moves and updating location inventory levels.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `request` - Putaway confirmation with allocations and reference information
///
/// # Returns
/// * `200` - Putaway confirmed with created stock moves
/// * `400` - Invalid allocations or capacity exceeded
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Location not found
///
/// # Example
/// ```json
/// POST /api/v1/warehouse/putaway/confirm
/// {
///   "allocations": [
///     {
///       "location_id": "550e8400-e29b-41d4-a716-446655440002",
///       "quantity": 50
///     }
///   ],
///   "reference_type": "goods_receipt",
///   "reference_id": "550e8400-e29b-41d4-a716-446655440003"
/// }
/// ```
///
/// Response:
/// ```json
/// {
///   "stock_moves_created": [
///     "550e8400-e29b-41d4-a716-446655440004"
///   ],
///   "total_quantity_putaway": 50
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/warehouse/putaway/confirm",
    tag = "warehouse",
    operation_id = "confirm_putaway",
    request_body = ConfirmPutawayRequest,
    responses(
        (status = 200, description = "Putaway confirmed", body = ConfirmPutawayResponse),
        (status = 400, description = "Invalid allocations"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Location not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn confirm_putaway(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<ConfirmPutawayRequest>,
) -> Result<Json<ConfirmPutawayResponse>, AppError> {
    // Validate request
    if request.allocations.is_empty() {
        return Err(AppError::ValidationError("At least one allocation is required".to_string()));
    }

    for allocation in &request.allocations {
        if allocation.quantity <= 0 {
            return Err(AppError::ValidationError(
                "Allocation quantity must be positive".to_string(),
            ));
        }
    }

    let response = state
        .putaway_service
        .confirm_putaway(&auth_user.tenant_id, &request, &auth_user.user_id)
        .await?;

    Ok(Json(response))
}
