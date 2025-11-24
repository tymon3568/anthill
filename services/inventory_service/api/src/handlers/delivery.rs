//! Delivery HTTP handlers
//!
//! This module contains the Axum handlers for Delivery Order operations.

use axum::{
    extract::{Path, State},
    response::Json,
    routing::post,
    Router,
};
use uuid::Uuid;

use inventory_service_core::dto::delivery::{
    PackItemsRequest, PackItemsResponse, PickItemsRequest, PickItemsResponse, ShipItemsRequest,
    ShipItemsResponse,
};

use crate::handlers::category::AppState;
use shared_auth::extractors::AuthUser;
use shared_error::AppError;

/// Create the delivery routes with state
pub fn create_delivery_routes(state: AppState) -> Router {
    Router::new()
        .route("/{delivery_id}/pick", post(pick_items))
        .route("/{delivery_id}/pack", post(pack_items))
        .route("/{delivery_id}/ship", post(ship_items))
        .with_state(state)
}

/// POST /api/v1/inventory/deliveries/{delivery_id}/pick - Pick items for a delivery order
///
/// Updates the picked quantities for specified delivery order items and changes
/// the delivery order status to 'picked'. This operation is performed by warehouse
/// staff when physically picking items from shelves for shipment.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `delivery_id` - UUID of the delivery order to pick items for
///
/// # Request Body
/// ```json
/// {
///   "items": [
///     {
///       "delivery_item_id": "550e8400-e29b-41d4-a716-446655440000",
///       "picked_quantity": 50
///     },
///     {
///       "delivery_item_id": "550e8400-e29b-41d4-a716-446655440001",
///       "picked_quantity": 25
///     }
///   ]
/// }
/// ```
///
/// # Returns
/// * `200` - Items picked successfully with summary
/// * `400` - Invalid request data or business rule violations
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Delivery order or items not found
///
/// # Business Rules
/// - Delivery order must be in 'Confirmed' status
/// - Picked quantities cannot exceed remaining ordered quantities
/// - All specified items must belong to the delivery order
/// - Picked quantities must be positive
///
/// # Example Response
/// ```json
/// {
///   "delivery_id": "550e8400-e29b-41d4-a716-446655440002",
///   "status": "Picked",
///   "picked_items_count": 2,
///   "total_picked_quantity": 75
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/deliveries/{delivery_id}/pick",
    tag = "deliveries",
    operation_id = "pick_delivery_items",
    params(
        ("delivery_id" = Uuid, Path, description = "Delivery order ID")
    ),
    request_body = PickItemsRequest,
    responses(
        (status = 200, description = "Items picked successfully", body = PickItemsResponse),
        (status = 400, description = "Invalid request or business rule violation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Delivery order or items not found")
    )
)]
pub async fn pick_items(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(delivery_id): Path<Uuid>,
    Json(request): Json<PickItemsRequest>,
) -> Result<Json<PickItemsResponse>, AppError> {
    let response = state
        .delivery_service
        .pick_items(auth_user.tenant_id, delivery_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/deliveries/{delivery_id}/pack - Pack items for a delivery order
///
/// Marks a delivery order as packed after items have been picked and prepared for shipment.
/// This operation is performed by warehouse staff when physically packing items into shipping containers.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `delivery_id` - UUID of the delivery order to pack
///
/// # Request Body
/// ```json
/// {
///   "notes": "Packed with extra padding for fragile items"
/// }
/// ```
///
/// # Returns
/// * `200` - Delivery order packed successfully
/// * `400` - Invalid request or business rule violations
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Delivery order not found
///
/// # Business Rules
/// - Delivery order must be in 'Picked' status
/// - Only picked orders can be packed
///
/// # Example Response
/// ```json
/// {
///   "delivery_id": "550e8400-e29b-41d4-a716-446655440002",
///   "status": "Packed",
///   "packed_at": "2023-10-15T14:30:00Z"
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/deliveries/{delivery_id}/pack",
    tag = "deliveries",
    operation_id = "pack_delivery_items",
    params(
        ("delivery_id" = Uuid, Path, description = "Delivery order ID")
    ),
    request_body = PackItemsRequest,
    responses(
        (status = 200, description = "Delivery order packed successfully", body = PackItemsResponse),
        (status = 400, description = "Invalid request or business rule violation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Delivery order not found")
    )
)]
pub async fn pack_items(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(delivery_id): Path<Uuid>,
    Json(request): Json<PackItemsRequest>,
) -> Result<Json<PackItemsResponse>, AppError> {
    let response = state
        .delivery_service
        .pack_items(auth_user.tenant_id, delivery_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/deliveries/{delivery_id}/ship - Ship items for a delivery order
///
/// Ships a delivery order by creating stock moves, updating inventory levels, calculating COGS,
/// and marking the order as shipped. This is the final step in the delivery process.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `delivery_id` - UUID of the delivery order to ship
///
/// # Request Body
/// ```json
/// {
///   "tracking_number": "TRACK123456",
///   "carrier": "FedEx",
///   "shipping_cost": 500,
///   "notes": "Shipped with priority handling"
/// }
/// ```
///
/// # Returns
/// * `200` - Delivery order shipped successfully
/// * `400` - Invalid request or business rule violations
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Delivery order not found
///
/// # Business Rules
/// - Delivery order must be in 'Packed' status
/// - Creates immutable stock moves for audit trail
/// - Updates inventory levels (decrements available stock)
/// - Calculates and records Cost of Goods Sold (COGS)
/// - Publishes inventory.delivery.completed event
///
/// # Example Response
/// ```json
/// {
///   "delivery_id": "550e8400-e29b-41d4-a716-446655440002",
///   "status": "Shipped",
///   "shipped_at": "2023-10-15T14:30:00Z",
///   "stock_moves_created": 3,
///   "total_cogs": 15000
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/deliveries/{delivery_id}/ship",
    tag = "deliveries",
    operation_id = "ship_delivery_items",
    params(
        ("delivery_id" = Uuid, Path, description = "Delivery order ID")
    ),
    request_body = ShipItemsRequest,
    responses(
        (status = 200, description = "Delivery order shipped successfully", body = ShipItemsResponse),
        (status = 400, description = "Invalid request or business rule violation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Delivery order not found")
    )
)]
pub async fn ship_items(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(delivery_id): Path<Uuid>,
    Json(request): Json<ShipItemsRequest>,
) -> Result<Json<ShipItemsResponse>, AppError> {
    let response = state
        .delivery_service
        .ship_items(auth_user.tenant_id, delivery_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}
