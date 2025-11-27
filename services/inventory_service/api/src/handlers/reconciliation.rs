use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use uuid::Uuid;

use inventory_service_core::dto::reconciliation::{
    ApproveReconciliationRequest, ApproveReconciliationResponse, CountReconciliationRequest,
    CountReconciliationResponse, CreateReconciliationRequest, CreateReconciliationResponse,
    FinalizeReconciliationRequest, FinalizeReconciliationResponse, ReconciliationAnalyticsQuery,
    ReconciliationAnalyticsResponse, ReconciliationDetailResponse, ReconciliationListQuery,
    ReconciliationListResponse, VarianceAnalysisResponse,
};

use shared_auth::extractors::AuthUser;
use shared_error::AppError;

use crate::state::AppState;

/// Create the reconciliation routes with state
pub fn create_reconciliation_routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_reconciliation))
        .route("/analytics", get(get_reconciliation_analytics))
        .route("/:reconciliation_id/count", post(count_reconciliation))
        .route("/:reconciliation_id/finalize", post(finalize_reconciliation))
        .route("/:reconciliation_id/approve", post(approve_reconciliation))
        .route("/:reconciliation_id/variance", get(get_variance_analysis))
        .route("/", get(list_reconciliations))
        .route("/:reconciliation_id", get(get_reconciliation))
        .with_state(state)
}

/// POST /api/v1/inventory/reconciliations - Start reconciliation
///
/// Creates a new reconciliation session and populates it with items based on the cycle counting strategy.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Request Body
/// ```json
/// {
///   "name": "Monthly Warehouse Count",
///   "description": "Full inventory reconciliation for main warehouse",
///   "cycle_type": "Full",
///   "warehouse_id": "550e8400-e29b-41d4-a716-446655440000",
///   "notes": "Scheduled monthly count"
/// }
/// ```
///
/// # Returns
/// * `201` - Reconciliation created successfully
/// * `400` - Invalid request data
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Business Rules
/// - Creates reconciliation in 'Draft' status
/// - Populates items based on cycle counting strategy
/// - Supports ABC analysis, location-based, and random sampling
///
/// # Example Response
/// ```json
/// {
///   "reconciliation": {
///     "reconciliation_id": "550e8400-e29b-41d4-a716-446655440001",
///     "reconciliation_number": "REC-20241127-0001",
///     "name": "Monthly Warehouse Count",
///     "status": "Draft",
///     "cycle_type": "Full",
///     "total_items": 150,
///     "created_by": "550e8400-e29b-41d4-a716-446655440002",
///     "created_at": "2024-11-27T10:00:00Z"
///   }
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/reconciliations",
    tag = "reconciliations",
    operation_id = "create_reconciliation",
    request_body = CreateReconciliationRequest,
    responses(
        (status = 201, description = "Reconciliation created successfully", body = CreateReconciliationResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn create_reconciliation(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateReconciliationRequest>,
) -> Result<(StatusCode, Json<CreateReconciliationResponse>), AppError> {
    let response = state
        .reconciliation_service
        .create_reconciliation(auth_user.tenant_id, auth_user.user_id, request)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// POST /api/v1/inventory/reconciliations/{reconciliation_id}/count - Record counts
///
/// Records counted quantities for reconciliation items and updates variance calculations.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `reconciliation_id` - UUID of the reconciliation
///
/// # Request Body
/// ```json
/// {
///   "items": [
///     {
///       "product_id": "550e8400-e29b-41d4-a716-446655440003",
///       "warehouse_id": "550e8400-e29b-41d4-a716-446655440000",
///       "location_id": "550e8400-e29b-41d4-a716-446655440004",
///       "counted_quantity": 95,
///       "unit_cost": 25.50,
///       "notes": "Counted twice for accuracy"
///     }
///   ]
/// }
/// ```
///
/// # Returns
/// * `200` - Counts recorded successfully
/// * `400` - Invalid request or business rule violations
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Reconciliation not found
///
/// # Business Rules
/// - Reconciliation must be in Draft or InProgress status
/// - Updates status to InProgress if it was Draft
/// - Calculates variances and variance percentages automatically
/// - Tracks who counted each item and when
///
/// # Example Response
/// ```json
/// {
///   "items": [
///     {
///       "tenant_id": "550e8400-e29b-41d4-a716-446655440005",
///       "reconciliation_id": "550e8400-e29b-41d4-a716-446655440001",
///       "product_id": "550e8400-e29b-41d4-a716-446655440003",
///       "warehouse_id": "550e8400-e29b-41d4-a716-446655440000",
///       "expected_quantity": 100,
///       "counted_quantity": 95,
///       "variance": -5,
///       "variance_percentage": -5.0,
///       "unit_cost": 25.50,
///       "variance_value": -127.50,
///       "counted_by": "550e8400-e29b-41d4-a716-446655440002",
///       "counted_at": "2024-11-27T10:15:00Z",
///       "notes": "Counted twice for accuracy"
///     }
///   ]
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/reconciliations/{reconciliation_id}/count",
    tag = "reconciliations",
    operation_id = "count_reconciliation",
    params(
        ("reconciliation_id" = Uuid, Path, description = "Reconciliation ID")
    ),
    request_body = CountReconciliationRequest,
    responses(
        (status = 200, description = "Counts recorded successfully", body = CountReconciliationResponse),
        (status = 400, description = "Invalid request or business rule violation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Reconciliation not found")
    )
)]
pub async fn count_reconciliation(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(reconciliation_id): Path<Uuid>,
    Json(request): Json<CountReconciliationRequest>,
) -> Result<Json<CountReconciliationResponse>, AppError> {
    let response = state
        .reconciliation_service
        .count_reconciliation(auth_user.tenant_id, reconciliation_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/reconciliations/{reconciliation_id}/finalize - Finalize reconciliation
///
/// Completes the reconciliation, calculates final variances, and creates automatic adjustments.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `reconciliation_id` - UUID of the reconciliation
///
/// # Returns
/// * `200` - Reconciliation finalized successfully
/// * `400` - Invalid request or business rule violations
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Reconciliation not found
///
/// # Business Rules
/// - Reconciliation must be in InProgress status
/// - All items must have counted quantities
/// - Creates stock adjustments for discrepancies
/// - Updates inventory levels
/// - Sets status to Completed
///
/// # Example Response
/// ```json
/// {
///   "reconciliation": {
///     "reconciliation_id": "550e8400-e29b-41d4-a716-446655440001",
///     "status": "Completed",
///     "completed_at": "2024-11-27T11:00:00Z",
///     "total_variance": -25
///   },
///   "adjustments": [
///     {
///       "adjustment_id": "550e8400-e29b-41d4-a716-446655440006",
///       "product_id": "550e8400-e29b-41d4-a716-446655440003",
///       "warehouse_id": "550e8400-e29b-41d4-a716-446655440000",
///       "quantity": -5,
///       "reason": "Reconciliation discrepancy",
///       "adjusted_at": "2024-11-27T11:00:00Z"
///     }
///   ]
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/reconciliations/{reconciliation_id}/finalize",
    tag = "reconciliations",
    operation_id = "finalize_reconciliation",
    params(
        ("reconciliation_id" = Uuid, Path, description = "Reconciliation ID")
    ),
    responses(
        (status = 200, description = "Reconciliation finalized successfully", body = FinalizeReconciliationResponse),
        (status = 400, description = "Invalid request or business rule violation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Reconciliation not found")
    )
)]
pub async fn finalize_reconciliation(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(reconciliation_id): Path<Uuid>,
) -> Result<Json<FinalizeReconciliationResponse>, AppError> {
    let response = state
        .reconciliation_service
        .finalize_reconciliation(
            auth_user.tenant_id,
            reconciliation_id,
            auth_user.user_id,
            FinalizeReconciliationRequest {},
        )
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/reconciliations/{reconciliation_id}/approve - Approve reconciliation
///
/// Approves a completed reconciliation, making adjustments permanent.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `reconciliation_id` - UUID of the reconciliation
///
/// # Request Body
/// ```json
/// {
///   "notes": "Approved after review of variances"
/// }
/// ```
///
/// # Returns
/// * `200` - Reconciliation approved successfully
/// * `400` - Invalid request or business rule violations
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Reconciliation not found
///
/// # Business Rules
/// - Reconciliation must be in Completed status
/// - Requires approval permissions for large variances
/// - Records approval timestamp and approver
///
/// # Example Response
/// ```json
/// {
///   "reconciliation": {
///     "reconciliation_id": "550e8400-e29b-41d4-a716-446655440001",
///     "status": "Completed",
///     "approved_by": "550e8400-e29b-41d4-a716-446655440007",
///     "approved_at": "2024-11-27T11:30:00Z",
///     "notes": "Approved after review of variances"
///   }
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/reconciliations/{reconciliation_id}/approve",
    tag = "reconciliations",
    operation_id = "approve_reconciliation",
    params(
        ("reconciliation_id" = Uuid, Path, description = "Reconciliation ID")
    ),
    request_body = ApproveReconciliationRequest,
    responses(
        (status = 200, description = "Reconciliation approved successfully", body = ApproveReconciliationResponse),
        (status = 400, description = "Invalid request or business rule violation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Reconciliation not found")
    )
)]
pub async fn approve_reconciliation(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(reconciliation_id): Path<Uuid>,
    Json(request): Json<ApproveReconciliationRequest>,
) -> Result<Json<ApproveReconciliationResponse>, AppError> {
    let response = state
        .reconciliation_service
        .approve_reconciliation(auth_user.tenant_id, reconciliation_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}

/// GET /api/v1/inventory/reconciliations - List reconciliations
///
/// Returns a paginated list of reconciliations with optional filtering.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `warehouse_id` - Filter by warehouse (optional)
/// * `status` - Filter by status (optional)
/// * `cycle_type` - Filter by cycle type (optional)
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
///   "reconciliations": [...],
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
    path = "/api/v1/inventory/reconciliations",
    tag = "reconciliations",
    operation_id = "list_reconciliations",
    params(
        ReconciliationListQuery
    ),
    responses(
        (status = 200, description = "List retrieved successfully", body = ReconciliationListResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn list_reconciliations(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ReconciliationListQuery>,
) -> Result<Json<ReconciliationListResponse>, AppError> {
    let response = state
        .reconciliation_service
        .list_reconciliations(auth_user.tenant_id, query)
        .await?;

    Ok(Json(response))
}

/// GET /api/v1/inventory/reconciliations/{reconciliation_id} - Get reconciliation details
///
/// Returns a reconciliation with all its items.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `reconciliation_id` - UUID of the reconciliation
///
/// # Returns
/// * `200` - Details retrieved successfully
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Reconciliation not found
///
/// # Example Response
/// ```json
/// {
///   "reconciliation": {...},
///   "items": [...]
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/inventory/reconciliations/{reconciliation_id}",
    tag = "reconciliations",
    operation_id = "get_reconciliation",
    params(
        ("reconciliation_id" = Uuid, Path, description = "Reconciliation ID")
    ),
    responses(
        (status = 200, description = "Details retrieved successfully", body = ReconciliationDetailResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Reconciliation not found")
    )
)]
pub async fn get_reconciliation(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(reconciliation_id): Path<Uuid>,
) -> Result<Json<ReconciliationDetailResponse>, AppError> {
    let response = state
        .reconciliation_service
        .get_reconciliation(auth_user.tenant_id, reconciliation_id)
        .await?;

    Ok(Json(response))
}

/// GET /api/v1/inventory/reconciliations/analytics - Get reconciliation analytics
///
/// Returns analytics and metrics for reconciliations.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `warehouse_id` - Filter by warehouse (optional)
///
/// # Returns
/// * `200` - Analytics retrieved successfully
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Example Response
/// ```json
/// {
///   "total_reconciliations": 25,
///   "completed_reconciliations": 23,
///   "average_variance_percentage": -2.5,
///   "total_variance_value": -1250.00,
///   "high_variance_items": 5,
///   "accuracy_rate": 92.0
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/inventory/reconciliations/analytics",
    tag = "reconciliations",
    operation_id = "get_reconciliation_analytics",
    params(ReconciliationAnalyticsQuery),
    responses(
        (status = 200, description = "Analytics retrieved successfully", body = ReconciliationAnalyticsResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn get_reconciliation_analytics(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ReconciliationAnalyticsQuery>,
) -> Result<Json<ReconciliationAnalyticsResponse>, AppError> {
    let response = state
        .reconciliation_service
        .get_analytics(auth_user.tenant_id, query.warehouse_id)
        .await?;

    Ok(Json(response))
}

/// GET /api/v1/inventory/reconciliations/{reconciliation_id}/variance - Get variance analysis
///
/// Returns detailed variance analysis for a specific reconciliation.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `reconciliation_id` - UUID of the reconciliation
///
/// # Returns
/// * `200` - Variance analysis retrieved successfully
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Reconciliation not found
///
/// # Example Response
/// ```json
/// {
///   "reconciliation": {...},
///   "variance_ranges": [
///     {"range": "0-5%", "count": 120, "total_variance_value": -250.00},
///     {"range": "5-10%", "count": 15, "total_variance_value": -180.00}
///   ],
///   "top_variance_items": [...]
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/inventory/reconciliations/{reconciliation_id}/variance",
    tag = "reconciliations",
    operation_id = "get_variance_analysis",
    params(
        ("reconciliation_id" = Uuid, Path, description = "Reconciliation ID")
    ),
    responses(
        (status = 200, description = "Variance analysis retrieved successfully", body = VarianceAnalysisResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Reconciliation not found")
    )
)]
pub async fn get_variance_analysis(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(reconciliation_id): Path<Uuid>,
) -> Result<Json<VarianceAnalysisResponse>, AppError> {
    let response = state
        .reconciliation_service
        .get_variance_analysis(auth_user.tenant_id, reconciliation_id)
        .await?;

    Ok(Json(response))
}
