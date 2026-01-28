use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::transfer_dto::{
    CancelTransferRequest, CancelTransferResponse, ConfirmTransferRequest, ConfirmTransferResponse,
    CreateTransferRequest, CreateTransferResponse, ListTransfersParams, ListTransfersResponse,
    ReceiveTransferRequest, ReceiveTransferResponse, TransferResponse,
};

use shared_auth::extractors::AuthUser;
use shared_error::AppError;

use crate::state::AppState;

/// Create the transfer routes
pub fn create_transfer_routes() -> Router {
    Router::new()
        .route("/", get(list_transfers).post(create_transfer))
        .route("/{transfer_id}", get(get_transfer))
        .route("/{transfer_id}/confirm", post(confirm_transfer))
        .route("/{transfer_id}/receive", post(receive_transfer))
        .route("/{transfer_id}/cancel", post(cancel_transfer))
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
    Extension(state): Extension<AppState>,
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
    Extension(state): Extension<AppState>,
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
    Extension(state): Extension<AppState>,
    Path(transfer_id): Path<Uuid>,
    Json(request): Json<ReceiveTransferRequest>,
) -> Result<Json<ReceiveTransferResponse>, AppError> {
    let response = state
        .transfer_service
        .receive_transfer(auth_user.tenant_id, transfer_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}

/// GET /api/v1/inventory/transfers - List stock transfers
///
/// Lists all stock transfers for the tenant with optional filtering.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `source_warehouse_id` - Filter by source warehouse (optional)
/// * `destination_warehouse_id` - Filter by destination warehouse (optional)
/// * `status` - Filter by transfer status (optional)
/// * `page` - Page number (default: 1)
/// * `page_size` - Items per page (default: 20, max: 100)
///
/// # Returns
/// * `200` - List of transfers with pagination info
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Example Response
/// ```json
/// {
///   "items": [...],
///   "total": 50,
///   "page": 1,
///   "page_size": 20,
///   "total_pages": 3
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/inventory/transfers",
    tag = "transfers",
    operation_id = "list_transfers",
    params(
        ("source_warehouse_id" = Option<Uuid>, Query, description = "Filter by source warehouse"),
        ("destination_warehouse_id" = Option<Uuid>, Query, description = "Filter by destination warehouse"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("page_size" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List of transfers", body = ListTransfersResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn list_transfers(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(params): Query<ListTransfersParams>,
) -> Result<Json<ListTransfersResponse>, AppError> {
    let response = state
        .transfer_service
        .list_transfers(auth_user.tenant_id, params)
        .await?;

    Ok(Json(response))
}

/// GET /api/v1/inventory/transfers/{transfer_id} - Get a stock transfer
///
/// Retrieves details of a specific stock transfer including all items.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `transfer_id` - UUID of the transfer
///
/// # Returns
/// * `200` - Transfer details with items
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Transfer not found
///
/// # Example Response
/// ```json
/// {
///   "transfer": {...},
///   "items": [...]
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/inventory/transfers/{transfer_id}",
    tag = "transfers",
    operation_id = "get_transfer",
    params(
        ("transfer_id" = Uuid, Path, description = "Transfer ID")
    ),
    responses(
        (status = 200, description = "Transfer details", body = TransferResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Transfer not found")
    )
)]
pub async fn get_transfer(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<TransferResponse>, AppError> {
    let response = state
        .transfer_service
        .get_transfer(auth_user.tenant_id, transfer_id)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/transfers/{transfer_id}/cancel - Cancel a stock transfer
///
/// Cancels a transfer that is in draft or confirmed status.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `transfer_id` - UUID of the transfer to cancel
///
/// # Request Body
/// ```json
/// {
///   "reason": "No longer needed"
/// }
/// ```
///
/// # Returns
/// * `200` - Transfer cancelled successfully
/// * `400` - Invalid request or business rule violations
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Transfer not found
///
/// # Business Rules
/// - Transfer must be in 'draft' or 'confirmed' status
/// - Cannot cancel transfers that are picked, shipped, or received
///
/// # Example Response
/// ```json
/// {
///   "transfer_id": "550e8400-e29b-41d4-a716-446655440004",
///   "status": "cancelled",
///   "cancelled_at": "2023-10-15T09:30:00Z"
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/transfers/{transfer_id}/cancel",
    tag = "transfers",
    operation_id = "cancel_transfer",
    params(
        ("transfer_id" = Uuid, Path, description = "Transfer ID")
    ),
    request_body = CancelTransferRequest,
    responses(
        (status = 200, description = "Transfer cancelled successfully", body = CancelTransferResponse),
        (status = 400, description = "Invalid request or business rule violation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Transfer not found")
    )
)]
pub async fn cancel_transfer(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(transfer_id): Path<Uuid>,
    Json(request): Json<CancelTransferRequest>,
) -> Result<Json<CancelTransferResponse>, AppError> {
    let response = state
        .transfer_service
        .cancel_transfer(auth_user.tenant_id, transfer_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}
