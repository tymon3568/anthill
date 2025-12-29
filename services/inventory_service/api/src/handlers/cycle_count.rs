//! Cycle Counting API Handlers
//!
//! This module provides HTTP handlers for cycle counting operations.
//! All endpoints are tenant-scoped via authentication.

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use uuid::Uuid;

use inventory_service_core::dto::cycle_count::{
    CreateCycleCountRequest, CycleCountListQuery, CycleCountListResponse, CycleCountResponse,
    CycleCountWithLinesResponse, GenerateLinesRequest, ReconcileRequest, ReconcileResponse,
    SkipLinesRequest, SubmitCountsRequest,
};

use shared_auth::extractors::AuthUser;
use shared_error::AppError;

use crate::state::AppState;

/// Create the cycle counting routes
pub fn create_cycle_count_routes() -> Router {
    Router::new()
        .route("/", post(create_cycle_count))
        .route("/", get(list_cycle_counts))
        .route("/{cycle_count_id}", get(get_cycle_count))
        .route("/{cycle_count_id}/generate-lines", post(generate_lines))
        .route("/{cycle_count_id}/counts", post(submit_counts))
        .route("/{cycle_count_id}/skip", post(skip_lines))
        .route("/{cycle_count_id}/close", post(close_session))
        .route("/{cycle_count_id}/reconcile", post(reconcile))
        .route("/{cycle_count_id}/cancel", post(cancel_session))
}

/// POST /api/v1/inventory/cycle-counts - Create a new cycle count session
///
/// Creates a cycle count session in draft status with an optional as-of snapshot timestamp.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Request Body
/// ```json
/// {
///   "warehouse_id": "550e8400-e29b-41d4-a716-446655440000",
///   "location_id": null,
///   "schedule_id": null,
///   "as_of": "2023-11-23T12:00:00Z",
///   "count_type": "cycle",
///   "notes": "Monthly cycle count for high-value items"
/// }
/// ```
///
/// # Returns
/// * `201` - Cycle count created successfully
/// * `400` - Invalid request data
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    post,
    path = "/api/v1/inventory/cycle-counts",
    tag = "cycle_counts",
    operation_id = "inventory_cycle_count_create",
    request_body = CreateCycleCountRequest,
    responses(
        (status = 201, description = "Cycle count session created successfully", body = CycleCountResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn create_cycle_count(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<CreateCycleCountRequest>,
) -> Result<(StatusCode, Json<CycleCountResponse>), AppError> {
    let response = state
        .cycle_counting_service
        .create_session(auth_user.tenant_id, auth_user.user_id, request)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/v1/inventory/cycle-counts - List cycle count sessions
///
/// Returns a paginated list of cycle count sessions with optional filtering.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `warehouse_id` - Filter by warehouse (optional)
/// * `status` - Filter by status: draft, in_progress, ready_to_reconcile, reconciled, cancelled (optional)
/// * `count_type` - Filter by count type: full, cycle, spot (optional)
/// * `from_date` - Filter by created date from (optional)
/// * `to_date` - Filter by created date to (optional)
/// * `page` - Page number (optional, default 1)
/// * `limit` - Items per page (optional, default 50, max 100)
///
/// # Returns
/// * `200` - List retrieved successfully
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    get,
    path = "/api/v1/inventory/cycle-counts",
    tag = "cycle_counts",
    operation_id = "inventory_cycle_count_list",
    params(CycleCountListQuery),
    responses(
        (status = 200, description = "List retrieved successfully", body = CycleCountListResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn list_cycle_counts(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(query): Query<CycleCountListQuery>,
) -> Result<Json<CycleCountListResponse>, AppError> {
    let response = state
        .cycle_counting_service
        .list_sessions(auth_user.tenant_id, query)
        .await?;

    Ok(Json(response))
}

/// GET /api/v1/inventory/cycle-counts/{cycle_count_id} - Get cycle count details
///
/// Returns a cycle count session with all its lines and summary.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `cycle_count_id` - UUID of the cycle count session
///
/// # Returns
/// * `200` - Details retrieved successfully
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Cycle count not found
#[utoipa::path(
    get,
    path = "/api/v1/inventory/cycle-counts/{cycle_count_id}",
    tag = "cycle_counts",
    operation_id = "inventory_cycle_count_get",
    params(
        ("cycle_count_id" = Uuid, Path, description = "Cycle count session ID")
    ),
    responses(
        (status = 200, description = "Details retrieved successfully", body = CycleCountWithLinesResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Cycle count not found")
    )
)]
pub async fn get_cycle_count(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(cycle_count_id): Path<Uuid>,
) -> Result<Json<CycleCountWithLinesResponse>, AppError> {
    let response = state
        .cycle_counting_service
        .get_session(auth_user.tenant_id, cycle_count_id)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/cycle-counts/{cycle_count_id}/generate-lines - Generate count lines
///
/// Generates or refreshes count lines based on current inventory filtered by session scope.
/// Expected quantities are computed as of the session's as_of timestamp.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `cycle_count_id` - UUID of the cycle count session
///
/// # Request Body
/// ```json
/// {
///   "product_id": null,
///   "category_id": null,
///   "include_lots": false,
///   "replace_existing": true
/// }
/// ```
///
/// # Returns
/// * `200` - Lines generated successfully
/// * `400` - Invalid request or session status does not allow line generation
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Cycle count not found
#[utoipa::path(
    post,
    path = "/api/v1/inventory/cycle-counts/{cycle_count_id}/generate-lines",
    tag = "cycle_counts",
    operation_id = "inventory_cycle_count_generate_lines",
    params(
        ("cycle_count_id" = Uuid, Path, description = "Cycle count session ID")
    ),
    request_body = GenerateLinesRequest,
    responses(
        (status = 200, description = "Lines generated successfully", body = CycleCountWithLinesResponse),
        (status = 400, description = "Invalid request or status does not allow line generation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Cycle count not found")
    )
)]
pub async fn generate_lines(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(cycle_count_id): Path<Uuid>,
    Json(request): Json<GenerateLinesRequest>,
) -> Result<Json<CycleCountWithLinesResponse>, AppError> {
    let response = state
        .cycle_counting_service
        .generate_lines(auth_user.tenant_id, cycle_count_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/cycle-counts/{cycle_count_id}/counts - Submit counts
///
/// Updates counted quantities for one or more lines.
/// Automatically transitions session from Draft to InProgress on first count.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `cycle_count_id` - UUID of the cycle count session
///
/// # Request Body
/// ```json
/// {
///   "counts": [
///     {
///       "line_id": "550e8400-e29b-41d4-a716-446655440001",
///       "counted_qty": 150,
///       "notes": "Counted twice for accuracy"
///     }
///   ]
/// }
/// ```
///
/// # Returns
/// * `200` - Counts submitted successfully
/// * `400` - Invalid request or session status does not allow counting
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Cycle count or line not found
#[utoipa::path(
    post,
    path = "/api/v1/inventory/cycle-counts/{cycle_count_id}/counts",
    tag = "cycle_counts",
    operation_id = "inventory_cycle_count_submit_counts",
    params(
        ("cycle_count_id" = Uuid, Path, description = "Cycle count session ID")
    ),
    request_body = SubmitCountsRequest,
    responses(
        (status = 200, description = "Counts submitted successfully", body = CycleCountWithLinesResponse),
        (status = 400, description = "Invalid request or status does not allow counting"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Cycle count or line not found")
    )
)]
pub async fn submit_counts(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(cycle_count_id): Path<Uuid>,
    Json(request): Json<SubmitCountsRequest>,
) -> Result<Json<CycleCountWithLinesResponse>, AppError> {
    let response = state
        .cycle_counting_service
        .submit_counts(auth_user.tenant_id, cycle_count_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/cycle-counts/{cycle_count_id}/skip - Skip lines
///
/// Marks one or more lines as skipped so they don't require counting for closure.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `cycle_count_id` - UUID of the cycle count session
///
/// # Request Body
/// ```json
/// {
///   "line_ids": ["550e8400-e29b-41d4-a716-446655440001"],
///   "reason": "Item not accessible"
/// }
/// ```
///
/// # Returns
/// * `200` - Lines skipped successfully
/// * `400` - Invalid request or session status does not allow skipping
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Cycle count or lines not found
#[utoipa::path(
    post,
    path = "/api/v1/inventory/cycle-counts/{cycle_count_id}/skip",
    tag = "cycle_counts",
    operation_id = "inventory_cycle_count_skip_lines",
    params(
        ("cycle_count_id" = Uuid, Path, description = "Cycle count session ID")
    ),
    request_body = SkipLinesRequest,
    responses(
        (status = 200, description = "Lines skipped successfully", body = CycleCountWithLinesResponse),
        (status = 400, description = "Invalid request or status does not allow skipping"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Cycle count or lines not found")
    )
)]
pub async fn skip_lines(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(cycle_count_id): Path<Uuid>,
    Json(request): Json<SkipLinesRequest>,
) -> Result<Json<CycleCountWithLinesResponse>, AppError> {
    let response = state
        .cycle_counting_service
        .skip_lines(auth_user.tenant_id, cycle_count_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/cycle-counts/{cycle_count_id}/close - Close session
///
/// Validates that all lines are either counted or skipped and transitions to ReadyToReconcile.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `cycle_count_id` - UUID of the cycle count session
///
/// # Returns
/// * `200` - Session closed successfully
/// * `400` - Invalid request, uncounted lines exist, or status does not allow closing
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Cycle count not found
#[utoipa::path(
    post,
    path = "/api/v1/inventory/cycle-counts/{cycle_count_id}/close",
    tag = "cycle_counts",
    operation_id = "inventory_cycle_count_close",
    params(
        ("cycle_count_id" = Uuid, Path, description = "Cycle count session ID")
    ),
    responses(
        (status = 200, description = "Session closed successfully", body = CycleCountResponse),
        (status = 400, description = "Uncounted lines exist or status does not allow closing"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Cycle count not found")
    )
)]
pub async fn close_session(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(cycle_count_id): Path<Uuid>,
) -> Result<Json<CycleCountResponse>, AppError> {
    let response = state
        .cycle_counting_service
        .close_session(auth_user.tenant_id, cycle_count_id, auth_user.user_id)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/cycle-counts/{cycle_count_id}/reconcile - Reconcile differences
///
/// Atomically creates stock adjustments for all variances and transitions to Reconciled.
/// This operation is idempotent - if already reconciled, returns existing result.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `cycle_count_id` - UUID of the cycle count session
///
/// # Request Body
/// ```json
/// {
///   "idempotency_key": "reconcile-123",
///   "force": false
/// }
/// ```
///
/// # Business Rules
/// - Session must be in ReadyToReconcile status (or already Reconciled for idempotency)
/// - If force=false and stock movements detected after as_of, returns error
/// - If force=true, proceeds with reconciliation regardless of later movements
///
/// # Returns
/// * `200` - Reconciliation completed successfully
/// * `400` - Invalid request, status does not allow reconciliation, or movements detected
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Cycle count not found
#[utoipa::path(
    post,
    path = "/api/v1/inventory/cycle-counts/{cycle_count_id}/reconcile",
    tag = "cycle_counts",
    operation_id = "inventory_cycle_count_reconcile",
    params(
        ("cycle_count_id" = Uuid, Path, description = "Cycle count session ID")
    ),
    request_body = ReconcileRequest,
    responses(
        (status = 200, description = "Reconciliation completed successfully", body = ReconcileResponse),
        (status = 400, description = "Status does not allow reconciliation or movements detected"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Cycle count not found")
    )
)]
pub async fn reconcile(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(cycle_count_id): Path<Uuid>,
    Json(request): Json<ReconcileRequest>,
) -> Result<Json<ReconcileResponse>, AppError> {
    let response = state
        .cycle_counting_service
        .reconcile(auth_user.tenant_id, cycle_count_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/cycle-counts/{cycle_count_id}/cancel - Cancel session
///
/// Cancels a cycle count session. Only Draft or InProgress sessions can be cancelled.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `cycle_count_id` - UUID of the cycle count session
///
/// # Returns
/// * `200` - Session cancelled successfully
/// * `400` - Status does not allow cancellation
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Cycle count not found
#[utoipa::path(
    post,
    path = "/api/v1/inventory/cycle-counts/{cycle_count_id}/cancel",
    tag = "cycle_counts",
    operation_id = "inventory_cycle_count_cancel",
    params(
        ("cycle_count_id" = Uuid, Path, description = "Cycle count session ID")
    ),
    responses(
        (status = 200, description = "Session cancelled successfully", body = CycleCountResponse),
        (status = 400, description = "Status does not allow cancellation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Cycle count not found")
    )
)]
pub async fn cancel_session(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(cycle_count_id): Path<Uuid>,
) -> Result<Json<CycleCountResponse>, AppError> {
    let response = state
        .cycle_counting_service
        .cancel_session(auth_user.tenant_id, cycle_count_id, auth_user.user_id)
        .await?;

    Ok(Json(response))
}
