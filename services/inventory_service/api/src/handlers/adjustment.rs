//! Stock Adjustment API Handlers
//!
//! Axum handlers for stock adjustment operations.
//! Follows the 3-crate pattern: api → infra → core → shared/*

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use shared_auth::extractors::AuthUser;
use shared_error::AppError;
use uuid::Uuid;

use inventory_service_core::dto::adjustment::{
    AddAdjustmentLinesRequest, AdjustmentDocumentResponse, AdjustmentDocumentWithLinesResponse,
    AdjustmentListQuery, AdjustmentListResponse, AdjustmentSummary, CreateAdjustmentRequest,
    PostAdjustmentRequest,
};

use crate::state::AppState;

/// Create the stock adjustment routes
pub fn create_adjustment_routes() -> Router {
    Router::new()
        .route("/", post(create_adjustment).get(list_adjustments))
        .route("/{adjustment_id}", get(get_adjustment))
        .route("/{adjustment_id}/lines", post(add_adjustment_lines))
        .route("/{adjustment_id}/post", post(post_adjustment))
        .route("/{adjustment_id}/cancel", post(cancel_adjustment))
        .route("/summary", get(get_adjustment_summary))
}

/// Create a new adjustment document (draft)
#[utoipa::path(
    post,
    path = "/api/v1/inventory/adjustments",
    tag = "adjustments",
    operation_id = "inventory_adjustment_create",
    request_body = CreateAdjustmentRequest,
    responses(
        (status = 201, description = "Adjustment document created", body = AdjustmentDocumentWithLinesResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_adjustment(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<CreateAdjustmentRequest>,
) -> Result<(StatusCode, Json<AdjustmentDocumentWithLinesResponse>), AppError> {
    let tenant_id = auth_user.tenant_id;
    let user_id = auth_user.user_id;

    let response = state
        .adjustment_service
        .create_adjustment(tenant_id, user_id, request)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get an adjustment document by ID with its lines
#[utoipa::path(
    get,
    path = "/api/v1/inventory/adjustments/{adjustment_id}",
    tag = "adjustments",
    operation_id = "inventory_adjustment_get_by_id",
    params(
        ("adjustment_id" = Uuid, Path, description = "Adjustment document ID")
    ),
    responses(
        (status = 200, description = "Adjustment document with lines", body = AdjustmentDocumentWithLinesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Adjustment document not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_adjustment(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(adjustment_id): Path<Uuid>,
) -> Result<Json<AdjustmentDocumentWithLinesResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;

    let response = state
        .adjustment_service
        .get_adjustment(tenant_id, adjustment_id)
        .await?;

    Ok(Json(response))
}

/// List adjustment documents with filtering
#[utoipa::path(
    get,
    path = "/api/v1/inventory/adjustments",
    tag = "adjustments",
    operation_id = "inventory_adjustment_list",
    params(
        ("warehouse_id" = Option<Uuid>, Query, description = "Filter by warehouse"),
        ("status" = Option<String>, Query, description = "Filter by status: draft, posted, cancelled"),
        ("search" = Option<String>, Query, description = "Search in reference or notes"),
        ("from_date" = Option<String>, Query, description = "Filter by date from (RFC3339)"),
        ("to_date" = Option<String>, Query, description = "Filter by date to (RFC3339)"),
        ("page" = Option<u32>, Query, description = "Page number (1-based)"),
        ("limit" = Option<u32>, Query, description = "Items per page (max 100)")
    ),
    responses(
        (status = 200, description = "List of adjustment documents", body = AdjustmentListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_adjustments(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(query): Query<AdjustmentListQuery>,
) -> Result<Json<AdjustmentListResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;

    let response = state
        .adjustment_service
        .list_adjustments(tenant_id, query)
        .await?;

    Ok(Json(response))
}

/// Get summary statistics for adjustments
#[utoipa::path(
    get,
    path = "/api/v1/inventory/adjustments/summary",
    tag = "adjustments",
    operation_id = "inventory_adjustment_summary",
    params(
        ("warehouse_id" = Option<Uuid>, Query, description = "Filter by warehouse")
    ),
    responses(
        (status = 200, description = "Adjustment summary statistics", body = AdjustmentSummary),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_adjustment_summary(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(query): Query<AdjustmentListQuery>,
) -> Result<Json<AdjustmentSummary>, AppError> {
    let tenant_id = auth_user.tenant_id;

    let response = state
        .adjustment_service
        .get_adjustment_summary(tenant_id, query.warehouse_id)
        .await?;

    Ok(Json(response))
}

/// Add or replace lines on an adjustment document
#[utoipa::path(
    post,
    path = "/api/v1/inventory/adjustments/{adjustment_id}/lines",
    tag = "adjustments",
    operation_id = "inventory_adjustment_add_lines",
    params(
        ("adjustment_id" = Uuid, Path, description = "Adjustment document ID")
    ),
    request_body = AddAdjustmentLinesRequest,
    responses(
        (status = 200, description = "Adjustment document with updated lines", body = AdjustmentDocumentWithLinesResponse),
        (status = 400, description = "Invalid request or document not in draft status"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Adjustment document not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_adjustment_lines(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(adjustment_id): Path<Uuid>,
    Json(request): Json<AddAdjustmentLinesRequest>,
) -> Result<Json<AdjustmentDocumentWithLinesResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;
    let user_id = auth_user.user_id;

    let response = state
        .adjustment_service
        .add_lines(tenant_id, adjustment_id, user_id, request)
        .await?;

    Ok(Json(response))
}

/// Post an adjustment document
///
/// Executes the adjustment by creating stock moves and updating inventory.
/// This operation is idempotent - if already posted, returns existing result.
#[utoipa::path(
    post,
    path = "/api/v1/inventory/adjustments/{adjustment_id}/post",
    tag = "adjustments",
    operation_id = "inventory_adjustment_post",
    params(
        ("adjustment_id" = Uuid, Path, description = "Adjustment document ID")
    ),
    request_body = PostAdjustmentRequest,
    responses(
        (status = 200, description = "Adjustment document posted", body = AdjustmentDocumentResponse),
        (status = 400, description = "Invalid request, document not in draft status, or no lines"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Adjustment document not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn post_adjustment(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(adjustment_id): Path<Uuid>,
    Json(request): Json<PostAdjustmentRequest>,
) -> Result<Json<AdjustmentDocumentResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;
    let user_id = auth_user.user_id;

    let response = state
        .adjustment_service
        .post_adjustment(tenant_id, adjustment_id, user_id, request)
        .await?;

    Ok(Json(response))
}

/// Cancel a draft adjustment document
///
/// Only draft documents can be cancelled. Posted documents cannot be reversed.
#[utoipa::path(
    post,
    path = "/api/v1/inventory/adjustments/{adjustment_id}/cancel",
    tag = "adjustments",
    operation_id = "inventory_adjustment_cancel_draft",
    params(
        ("adjustment_id" = Uuid, Path, description = "Adjustment document ID")
    ),
    responses(
        (status = 200, description = "Adjustment document cancelled", body = AdjustmentDocumentResponse),
        (status = 400, description = "Document not in draft status"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Adjustment document not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn cancel_adjustment(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(adjustment_id): Path<Uuid>,
) -> Result<Json<AdjustmentDocumentResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;
    let user_id = auth_user.user_id;

    let response = state
        .adjustment_service
        .cancel_adjustment(tenant_id, adjustment_id, user_id)
        .await?;

    Ok(Json(response))
}
