use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use uuid::Uuid;

use inventory_service_core::dto::stock_take::{
    CountStockTakeRequest, CountStockTakeResponse, CreateStockTakeRequest, CreateStockTakeResponse,
    FinalizeStockTakeRequest, FinalizeStockTakeResponse, StockTakeDetailResponse,
    StockTakeListQuery, StockTakeListResponse,
};

use shared_auth::extractors::AuthUser;
use shared_error::AppError;

use crate::handlers::category::AppState;

/// Create the stock take routes with state
pub fn create_stock_take_routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_stock_take))
        .route("/:stock_take_id/count", post(count_stock_take))
        .route("/:stock_take_id/finalize", post(finalize_stock_take))
        .route("/", get(list_stock_takes))
        .route("/:stock_take_id", get(get_stock_take))
        .with_state(state)
}

/// POST /api/v1/inventory/stock-takes - Create a new stock take session
///
/// Creates a stock take session in draft status and snapshots current inventory levels.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Request Body
/// ```json
/// {
///   "warehouse_id": "550e8400-e29b-41d4-a716-446655440000",
///   "notes": "Monthly inventory count"
/// }
/// ```
///
/// # Returns
/// * `201` - Stock take created successfully
/// * `400` - Invalid request data
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Business Rules
/// - Creates stock take lines from current inventory levels
/// - Stock take is created in 'draft' status
///
/// # Example Response
/// ```json
/// {
///   "stock_take": {
///     "stock_take_id": "550e8400-e29b-41d4-a716-446655440001",
///     "stock_take_number": "ST-20231123-123456",
///     "warehouse_id": "550e8400-e29b-41d4-a716-446655440000",
///     "status": "draft",
///     "started_at": "2023-11-23T12:34:56Z",
///     "created_by": "550e8400-e29b-41d4-a716-446655440002",
///     "notes": "Monthly inventory count"
///   }
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/stock-takes",
    tag = "stock_takes",
    operation_id = "create_stock_take",
    request_body = CreateStockTakeRequest,
    responses(
        (status = 201, description = "Stock take created successfully", body = CreateStockTakeResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn create_stock_take(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateStockTakeRequest>,
) -> Result<(StatusCode, Json<CreateStockTakeResponse>), AppError> {
    let response = state
        .stock_take_service
        .create_stock_take(auth_user.tenant_id, auth_user.user_id, request)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// POST /api/v1/inventory/stock-takes/{stock_take_id}/count - Submit counted quantities
///
/// Updates actual quantities for stock take lines and changes status to in_progress if needed.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `stock_take_id` - UUID of the stock take
///
/// # Request Body
/// ```json
/// {
///   "items": [
///     {
///       "product_id": "550e8400-e29b-41d4-a716-446655440003",
///       "actual_quantity": 150,
///       "notes": "Counted twice for accuracy"
///     }
///   ]
/// }
/// ```
///
/// # Returns
/// * `200` - Counts submitted successfully
/// * `400` - Invalid request or business rule violations
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Stock take not found
///
/// # Business Rules
/// - Stock take must be in draft or in_progress status
/// - Updates status to in_progress if it was draft
///
/// # Example Response
/// ```json
/// {
///   "lines": [
///     {
///       "line_id": "550e8400-e29b-41d4-a716-446655440004",
///       "stock_take_id": "550e8400-e29b-41d4-a716-446655440001",
///       "product_id": "550e8400-e29b-41d4-a716-446655440003",
///       "expected_quantity": 100,
///       "actual_quantity": 150,
///       "difference_quantity": 50,
///       "counted_by": "550e8400-e29b-41d4-a716-446655440002",
///       "counted_at": "2023-11-23T12:35:00Z",
///       "notes": "Counted twice for accuracy"
///     }
///   ]
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/stock-takes/{stock_take_id}/count",
    tag = "stock_takes",
    operation_id = "count_stock_take",
    params(
        ("stock_take_id" = Uuid, Path, description = "Stock take ID")
    ),
    request_body = CountStockTakeRequest,
    responses(
        (status = 200, description = "Counts submitted successfully", body = CountStockTakeResponse),
        (status = 400, description = "Invalid request or business rule violation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Stock take not found")
    )
)]
pub async fn count_stock_take(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(stock_take_id): Path<Uuid>,
    Json(request): Json<CountStockTakeRequest>,
) -> Result<Json<CountStockTakeResponse>, AppError> {
    let response = state
        .stock_take_service
        .count_stock_take(auth_user.tenant_id, stock_take_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/stock-takes/{stock_take_id}/finalize - Finalize stock take
///
/// Completes the stock take, generates inventory adjustments for discrepancies, and updates inventory levels.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `stock_take_id` - UUID of the stock take
///
/// # Request Body
/// Empty (no body required)
///
/// # Returns
/// * `200` - Stock take finalized successfully
/// * `400` - Invalid request or business rule violations
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Stock take not found
///
/// # Business Rules
/// - Stock take must be in in_progress status
/// - All lines must have actual quantities
/// - Creates stock adjustments for discrepancies
/// - Updates inventory levels
/// - Sets stock take status to completed
///
/// # Example Response
/// ```json
/// {
///   "stock_take": {
///     "stock_take_id": "550e8400-e29b-41d4-a716-446655440001",
///     "status": "completed",
///     "completed_at": "2023-11-23T12:40:00Z"
///   },
///   "adjustments": [
///     {
///       "adjustment_id": "550e8400-e29b-41d4-a716-446655440005",
///       "product_id": "550e8400-e29b-41d4-a716-446655440003",
///       "warehouse_id": "550e8400-e29b-41d4-a716-446655440000",
///       "quantity": 50,
///       "reason": "Stock take discrepancy",
///       "adjusted_at": "2023-11-23T12:40:00Z"
///     }
///   ]
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/stock-takes/{stock_take_id}/finalize",
    tag = "stock_takes",
    operation_id = "finalize_stock_take",
    params(
        ("stock_take_id" = Uuid, Path, description = "Stock take ID")
    ),
    request_body = FinalizeStockTakeRequest,
    responses(
        (status = 200, description = "Stock take finalized successfully", body = FinalizeStockTakeResponse),
        (status = 400, description = "Invalid request or business rule violation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Stock take not found")
    )
)]
pub async fn finalize_stock_take(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(stock_take_id): Path<Uuid>,
) -> Result<Json<FinalizeStockTakeResponse>, AppError> {
    let response = state
        .stock_take_service
        .finalize_stock_take(
            auth_user.tenant_id,
            stock_take_id,
            auth_user.user_id,
            FinalizeStockTakeRequest {},
        )
        .await?;

    Ok(Json(response))
}

/// GET /api/v1/inventory/stock-takes - List stock takes
///
/// Returns a paginated list of stock takes with optional filtering.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `warehouse_id` - Filter by warehouse (optional)
/// * `status` - Filter by status (optional)
/// * `page` - Page number (optional, default 1)
/// * `limit` - Items per page (optional, default 50, max 100)
///
/// # Returns
/// * `200` - List retrieved successfully
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Example Response
/// ```json
/// {
///   "stock_takes": [...],
///   "pagination": {
///     "page": 1,
///     "limit": 50,
///     "total": 25,
///     "total_pages": 1
///   }
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/inventory/stock-takes",
    tag = "stock_takes",
    operation_id = "list_stock_takes",
    params(
        StockTakeListQuery
    ),
    responses(
        (status = 200, description = "List retrieved successfully", body = StockTakeListResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn list_stock_takes(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<StockTakeListQuery>,
) -> Result<Json<StockTakeListResponse>, AppError> {
    let response = state
        .stock_take_service
        .list_stock_takes(auth_user.tenant_id, query)
        .await?;

    Ok(Json(response))
}

/// GET /api/v1/inventory/stock-takes/{stock_take_id} - Get stock take details
///
/// Returns a stock take with all its lines.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `stock_take_id` - UUID of the stock take
///
/// # Returns
/// * `200` - Details retrieved successfully
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Stock take not found
///
/// # Example Response
/// ```json
/// {
///   "stock_take": {...},
///   "lines": [...]
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/inventory/stock-takes/{stock_take_id}",
    tag = "stock_takes",
    operation_id = "get_stock_take",
    params(
        ("stock_take_id" = Uuid, Path, description = "Stock take ID")
    ),
    responses(
        (status = 200, description = "Details retrieved successfully", body = StockTakeDetailResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Stock take not found")
    )
)]
pub async fn get_stock_take(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(stock_take_id): Path<Uuid>,
) -> Result<Json<StockTakeDetailResponse>, AppError> {
    let response = state
        .stock_take_service
        .get_stock_take(auth_user.tenant_id, stock_take_id)
        .await?;

    Ok(Json(response))
}
