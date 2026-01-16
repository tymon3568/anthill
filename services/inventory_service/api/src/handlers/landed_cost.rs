//! Landed Cost Management API Handlers
//!
//! Axum handlers for landed cost operations.
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

use inventory_service_core::domains::inventory::dto::landed_cost_dto::{
    AddLandedCostLineRequest, ComputeAllocationsRequest, ComputeAllocationsResponse,
    CreateLandedCostRequest, LandedCostDetailDto, LandedCostDto, LandedCostLineDto,
    ListLandedCostsRequest, ListLandedCostsResponse, PostLandedCostRequest, PostLandedCostResponse,
};

use crate::state::AppState;

/// Create the landed cost routes
pub fn create_landed_cost_routes() -> Router {
    Router::new()
        .route("/", post(create_landed_cost).get(list_landed_costs))
        .route("/{landed_cost_id}", get(get_landed_cost))
        .route("/{landed_cost_id}/lines", post(add_landed_cost_line))
        .route("/{landed_cost_id}/compute", post(compute_allocations))
        .route("/{landed_cost_id}/post", post(post_landed_cost))
        .route("/{landed_cost_id}/cancel", post(cancel_landed_cost))
}

/// Create a new landed cost document (draft)
#[utoipa::path(
    post,
    path = "/api/v1/inventory/landed-costs",
    tag = "landed_costs",
    operation_id = "inventory_landed_cost_create",
    request_body = CreateLandedCostRequest,
    responses(
        (status = 201, description = "Landed cost document created", body = LandedCostDto),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_landed_cost(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<CreateLandedCostRequest>,
) -> Result<(StatusCode, Json<LandedCostDto>), AppError> {
    let tenant_id = auth_user.tenant_id;
    let user_id = auth_user.user_id;

    let response = state
        .landed_cost_service
        .create_draft(tenant_id, user_id, request)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a landed cost document by ID with lines and allocations
#[utoipa::path(
    get,
    path = "/api/v1/inventory/landed-costs/{landed_cost_id}",
    tag = "landed_costs",
    operation_id = "inventory_landed_cost_get_by_id",
    params(
        ("landed_cost_id" = Uuid, Path, description = "Landed cost document ID")
    ),
    responses(
        (status = 200, description = "Landed cost document with lines and allocations", body = LandedCostDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Landed cost document not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_landed_cost(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(landed_cost_id): Path<Uuid>,
) -> Result<Json<LandedCostDetailDto>, AppError> {
    let tenant_id = auth_user.tenant_id;

    let response = state
        .landed_cost_service
        .get_by_id(tenant_id, landed_cost_id)
        .await?;

    Ok(Json(response))
}

/// List landed cost documents with filtering
#[utoipa::path(
    get,
    path = "/api/v1/inventory/landed-costs",
    tag = "landed_costs",
    operation_id = "inventory_landed_cost_list",
    params(
        ("status" = Option<String>, Query, description = "Filter by status: draft, posted, cancelled"),
        ("grn_id" = Option<Uuid>, Query, description = "Filter by goods receipt ID"),
        ("limit" = Option<i64>, Query, description = "Items per page (max 100, default 50)"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination")
    ),
    responses(
        (status = 200, description = "List of landed cost documents", body = ListLandedCostsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_landed_costs(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(query): Query<ListLandedCostsRequest>,
) -> Result<Json<ListLandedCostsResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;

    let response = state.landed_cost_service.list(tenant_id, query).await?;

    Ok(Json(response))
}

/// Add a cost line to a landed cost document
#[utoipa::path(
    post,
    path = "/api/v1/inventory/landed-costs/{landed_cost_id}/lines",
    tag = "landed_costs",
    operation_id = "inventory_landed_cost_add_line",
    params(
        ("landed_cost_id" = Uuid, Path, description = "Landed cost document ID")
    ),
    request_body = AddLandedCostLineRequest,
    responses(
        (status = 201, description = "Cost line added", body = LandedCostLineDto),
        (status = 400, description = "Invalid request or document not in draft status"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Landed cost document not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_landed_cost_line(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(landed_cost_id): Path<Uuid>,
    Json(request): Json<AddLandedCostLineRequest>,
) -> Result<(StatusCode, Json<LandedCostLineDto>), AppError> {
    let tenant_id = auth_user.tenant_id;

    let response = state
        .landed_cost_service
        .add_line(tenant_id, landed_cost_id, request)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Compute allocations for cost lines
///
/// Computes proportional allocations based on target values.
/// This operation is idempotent - recalculates and replaces existing allocations.
#[utoipa::path(
    post,
    path = "/api/v1/inventory/landed-costs/{landed_cost_id}/compute",
    tag = "landed_costs",
    operation_id = "inventory_landed_cost_compute_allocations",
    params(
        ("landed_cost_id" = Uuid, Path, description = "Landed cost document ID")
    ),
    request_body = ComputeAllocationsRequest,
    responses(
        (status = 200, description = "Allocations computed", body = ComputeAllocationsResponse),
        (status = 400, description = "Invalid request, no GRN linked, or no targets"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Landed cost document not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn compute_allocations(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(landed_cost_id): Path<Uuid>,
    Json(request): Json<ComputeAllocationsRequest>,
) -> Result<Json<ComputeAllocationsResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;

    let response = state
        .landed_cost_service
        .compute_allocations(tenant_id, landed_cost_id, request)
        .await?;

    Ok(Json(response))
}

/// Post a landed cost document
///
/// Finalizes the landed cost by applying allocations to inventory valuation.
/// This operation is idempotent - posting twice returns success without double-applying.
#[utoipa::path(
    post,
    path = "/api/v1/inventory/landed-costs/{landed_cost_id}/post",
    tag = "landed_costs",
    operation_id = "inventory_landed_cost_post",
    params(
        ("landed_cost_id" = Uuid, Path, description = "Landed cost document ID")
    ),
    request_body = PostLandedCostRequest,
    responses(
        (status = 200, description = "Landed cost document posted", body = PostLandedCostResponse),
        (status = 400, description = "Invalid request, no allocations, or document not in draft status"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Landed cost document not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn post_landed_cost(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(landed_cost_id): Path<Uuid>,
    Json(request): Json<PostLandedCostRequest>,
) -> Result<Json<PostLandedCostResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;
    let user_id = auth_user.user_id;

    let response = state
        .landed_cost_service
        .post(tenant_id, user_id, landed_cost_id, request)
        .await?;

    Ok(Json(response))
}

/// Cancel a draft landed cost document
///
/// Only draft documents can be cancelled. Posted documents cannot be reversed.
#[utoipa::path(
    post,
    path = "/api/v1/inventory/landed-costs/{landed_cost_id}/cancel",
    tag = "landed_costs",
    operation_id = "inventory_landed_cost_cancel_draft",
    params(
        ("landed_cost_id" = Uuid, Path, description = "Landed cost document ID")
    ),
    responses(
        (status = 200, description = "Landed cost document cancelled", body = LandedCostDto),
        (status = 400, description = "Document not in draft status"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Landed cost document not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn cancel_landed_cost(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(landed_cost_id): Path<Uuid>,
) -> Result<Json<LandedCostDto>, AppError> {
    let tenant_id = auth_user.tenant_id;

    let response = state
        .landed_cost_service
        .cancel(tenant_id, landed_cost_id)
        .await?;

    Ok(Json(response))
}
