use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::transfer_dto::{
    ConfirmTransferRequest, ConfirmTransferResponse, CreateTransferRequest, CreateTransferResponse,
    ReceiveTransferRequest, ReceiveTransferResponse,
};

use shared_auth::extractors::AuthUser;
use shared_error::AppError;

use crate::handlers::category::AppState;

/// Create the transfer routes with state
pub fn create_transfer_routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_transfer))
        .route("/:transfer_id/confirm", post(confirm_transfer))
        .route("/:transfer_id/receive", post(receive_transfer))
        .with_state(state)
}

/// POST /api/v1/inventory/transfers - Create a new stock transfer
///
/// Creates a stock transfer in draft status with the specified items.
/// The transfer will be assigned an auto-generated transfer number.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Request Body
/// ```json
/// {
///   "reference_number": "REF123",
///   "source_warehouse_id": "550e8400-e29b-41d4-a716-446655440000",
///   "destination_warehouse_id": "550e8400-e29b-41d4-a716-446655440001",
///   "transfer_type": "manual",
///   "priority": "normal",
///   "expected_ship_date": "2023-10-15T10:00:00Z",
///   "expected_receive_date": "2023-10-16T10:00:00Z",
///   "shipping_method": "truck",
///   "notes": "Urgent transfer for high-demand product",
///   "reason": "Stock replenishment",
///   "items": [
///     {
///       "product_id": "550e8400-e29b-41d4-a716-446655440002",
///       "quantity": 100,
///       "uom_id": "550e8400-e29b-41d4-a716-446655440003",
///       "unit_cost": 5000,
///       "line_number": 1,
///       "notes": "Fragile item"
///     }
///   ]
/// }
/// ```
///
/// # Returns
/// * `201` - Transfer created successfully
/// * `400` - Invalid request data or business rule violations
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Business Rules
/// - Source and destination warehouses must be different
/// - Quantities must be positive
/// - Transfer is created in 'draft' status
/// - Product and UOM existence is validated by database constraints
///
/// # Example Response
/// ```json
/// {
///   "transfer_id": "550e8400-e29b-41d4-a716-446655440004",
///   "transfer_number": "ST-2023-00001",
///   "status": "draft",
///   "items_count": 1
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/transfers",
    tag = "transfers",
    operation_id = "create_transfer",
    request_body = CreateTransferRequest,
    responses(
        (status = 201, description = "Transfer created successfully", body = CreateTransferResponse),
        (status = 400, description = "Invalid request or business rule violation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn create_transfer(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateTransferRequest>,
) -> Result<(StatusCode, Json<CreateTransferResponse>), AppError> {
    let response = state
        .transfer_service
        .create_transfer(auth_user.tenant_id, auth_user.user_id, request)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// POST /api/v1/inventory/transfers/{transfer_id}/confirm - Confirm a stock transfer
///
/// Confirms a draft transfer, moves inventory to transit location, and updates
/// the transfer status to 'confirmed'. This initiates the physical transfer process.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `transfer_id` - UUID of the transfer to confirm
///
/// # Request Body
/// ```json
/// {
///   "notes": "Confirmed and ready for picking"
/// }
/// ```
///
/// # Returns
/// * `200` - Transfer confirmed successfully
/// * `400` - Invalid request or business rule violations
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Transfer not found
///
/// # Business Rules
/// - Transfer must be in 'draft' status
/// - Creates stock moves for inventory tracking
/// - Updates inventory levels (decrements source warehouse)
/// - Sets approved_by and approved_at timestamps
///
/// # Example Response
/// ```json
/// {
///   "transfer_id": "550e8400-e29b-41d4-a716-446655440004",
///   "status": "confirmed",
///   "confirmed_at": "2023-10-15T09:30:00Z"
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/transfers/{transfer_id}/confirm",
    tag = "transfers",
    operation_id = "confirm_transfer",
    params(
        ("transfer_id" = Uuid, Path, description = "Transfer ID")
    ),
    request_body = ConfirmTransferRequest,
    responses(
        (status = 200, description = "Transfer confirmed successfully", body = ConfirmTransferResponse),
        (status = 400, description = "Invalid request or business rule violation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Transfer not found")
    )
)]
pub async fn confirm_transfer(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(transfer_id): Path<Uuid>,
    Json(request): Json<ConfirmTransferRequest>,
) -> Result<Json<ConfirmTransferResponse>, AppError> {
    let response = state
        .transfer_service
        .confirm_transfer(auth_user.tenant_id, transfer_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/transfers/{transfer_id}/receive - Receive a stock transfer
///
/// Receives a shipped transfer at the destination warehouse, moves inventory
/// to the destination location, and completes the transfer process.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `transfer_id` - UUID of the transfer to receive
///
/// # Request Body
/// ```json
/// {
///   "notes": "Received in good condition"
/// }
/// ```
///
/// # Returns
/// * `200` - Transfer received successfully
/// * `400` - Invalid request or business rule violations
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Transfer not found
///
/// # Business Rules
/// - Transfer must be in 'shipped' status
/// - Creates stock moves for inventory tracking
/// - Updates inventory levels (increments destination warehouse)
/// - Will publish inventory.transfer.completed event (planned for future implementation)
/// - Sets actual_receive_date timestamp
///
/// # Example Response
/// ```json
/// {
///   "transfer_id": "550e8400-e29b-41d4-a716-446655440004",
///   "status": "received",
///   "received_at": "2023-10-16T14:00:00Z",
///   "stock_moves_created": 2
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/transfers/{transfer_id}/receive",
    tag = "transfers",
    operation_id = "receive_transfer",
    params(
        ("transfer_id" = Uuid, Path, description = "Transfer ID")
    ),
    request_body = ReceiveTransferRequest,
    responses(
        (status = 200, description = "Transfer received successfully", body = ReceiveTransferResponse),
        (status = 400, description = "Invalid request or business rule violation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Transfer not found")
    )
)]
pub async fn receive_transfer(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(transfer_id): Path<Uuid>,
    Json(request): Json<ReceiveTransferRequest>,
) -> Result<Json<ReceiveTransferResponse>, AppError> {
    let response = state
        .transfer_service
        .receive_transfer(auth_user.tenant_id, transfer_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}
