//! Scrap Management API Handlers
//!
//! Axum handlers for scrap management operations.
//! Follows the 3-crate pattern: api → infra → core → shared/*

use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use shared_auth::extractors::AuthUser;
use shared_error::AppError;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::dto::scrap::{
    AddScrapLinesRequest, CreateScrapRequest, PostScrapRequest, ScrapDocumentResponse,
    ScrapDocumentWithLinesResponse, ScrapListQuery, ScrapListResponse,
};
use inventory_service_core::services::scrap::ScrapService;

/// Create a new scrap document (draft)
#[utoipa::path(
    post,
    path = "/api/v1/inventory/scrap",
    tag = "scrap",
    operation_id = "inventory_scrap_create",
    request_body = CreateScrapRequest,
    responses(
        (status = 201, description = "Scrap document created", body = ScrapDocumentResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_scrap<S: ScrapService>(
    auth_user: AuthUser,
    Extension(scrap_service): Extension<Arc<S>>,
    Json(request): Json<CreateScrapRequest>,
) -> Result<Json<ScrapDocumentResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;
    let user_id = auth_user.user_id;

    let response = scrap_service
        .create_scrap(tenant_id, user_id, request)
        .await?;

    Ok(Json(response))
}

/// Get a scrap document by ID with its lines
#[utoipa::path(
    get,
    path = "/api/v1/inventory/scrap/{scrap_id}",
    tag = "scrap",
    operation_id = "inventory_scrap_get_by_id",
    params(
        ("scrap_id" = Uuid, Path, description = "Scrap document ID")
    ),
    responses(
        (status = 200, description = "Scrap document with lines", body = ScrapDocumentWithLinesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Scrap document not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_scrap<S: ScrapService>(
    auth_user: AuthUser,
    Extension(scrap_service): Extension<Arc<S>>,
    Path(scrap_id): Path<Uuid>,
) -> Result<Json<ScrapDocumentWithLinesResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;

    let response = scrap_service.get_scrap(tenant_id, scrap_id).await?;

    Ok(Json(response))
}

/// List scrap documents with filtering
#[utoipa::path(
    get,
    path = "/api/v1/inventory/scrap",
    tag = "scrap",
    operation_id = "inventory_scrap_list",
    params(
        ("warehouse_id" = Option<Uuid>, Query, description = "Filter by warehouse"),
        ("status" = Option<String>, Query, description = "Filter by status: draft, posted, cancelled"),
        ("from_date" = Option<String>, Query, description = "Filter by date from (RFC3339)"),
        ("to_date" = Option<String>, Query, description = "Filter by date to (RFC3339)"),
        ("page" = Option<u32>, Query, description = "Page number (1-based)"),
        ("limit" = Option<u32>, Query, description = "Items per page (max 100)")
    ),
    responses(
        (status = 200, description = "List of scrap documents", body = ScrapListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_scraps<S: ScrapService>(
    auth_user: AuthUser,
    Extension(scrap_service): Extension<Arc<S>>,
    Query(query): Query<ScrapListQuery>,
) -> Result<Json<ScrapListResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;

    let response = scrap_service.list_scraps(tenant_id, query).await?;

    Ok(Json(response))
}

/// Add or replace lines on a scrap document
#[utoipa::path(
    post,
    path = "/api/v1/inventory/scrap/{scrap_id}/lines",
    tag = "scrap",
    operation_id = "inventory_scrap_add_lines",
    params(
        ("scrap_id" = Uuid, Path, description = "Scrap document ID")
    ),
    request_body = AddScrapLinesRequest,
    responses(
        (status = 200, description = "Scrap document with updated lines", body = ScrapDocumentWithLinesResponse),
        (status = 400, description = "Invalid request or document not in draft status"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Scrap document not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_scrap_lines<S: ScrapService>(
    auth_user: AuthUser,
    Extension(scrap_service): Extension<Arc<S>>,
    Path(scrap_id): Path<Uuid>,
    Json(request): Json<AddScrapLinesRequest>,
) -> Result<Json<ScrapDocumentWithLinesResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;
    let user_id = auth_user.user_id;

    let response = scrap_service
        .add_lines(tenant_id, scrap_id, user_id, request)
        .await?;

    Ok(Json(response))
}

/// Post a scrap document
///
/// Executes the scrap by creating stock moves and updating inventory.
/// This operation is idempotent - if already posted, returns existing result.
#[utoipa::path(
    post,
    path = "/api/v1/inventory/scrap/{scrap_id}/post",
    tag = "scrap",
    operation_id = "inventory_scrap_post",
    params(
        ("scrap_id" = Uuid, Path, description = "Scrap document ID")
    ),
    request_body = PostScrapRequest,
    responses(
        (status = 200, description = "Scrap document posted", body = ScrapDocumentResponse),
        (status = 400, description = "Invalid request, document not in draft status, or no lines"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Scrap document not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn post_scrap<S: ScrapService>(
    auth_user: AuthUser,
    Extension(scrap_service): Extension<Arc<S>>,
    Path(scrap_id): Path<Uuid>,
    Json(request): Json<PostScrapRequest>,
) -> Result<Json<ScrapDocumentResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;
    let user_id = auth_user.user_id;

    let response = scrap_service
        .post_scrap(tenant_id, scrap_id, user_id, request)
        .await?;

    Ok(Json(response))
}

/// Cancel a draft scrap document
///
/// Only draft documents can be cancelled. Posted documents cannot be reversed.
#[utoipa::path(
    post,
    path = "/api/v1/inventory/scrap/{scrap_id}/cancel",
    tag = "scrap",
    operation_id = "inventory_scrap_cancel_draft",
    params(
        ("scrap_id" = Uuid, Path, description = "Scrap document ID")
    ),
    responses(
        (status = 200, description = "Scrap document cancelled", body = ScrapDocumentResponse),
        (status = 400, description = "Document not in draft status"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Scrap document not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn cancel_scrap<S: ScrapService>(
    auth_user: AuthUser,
    Extension(scrap_service): Extension<Arc<S>>,
    Path(scrap_id): Path<Uuid>,
) -> Result<Json<ScrapDocumentResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;
    let user_id = auth_user.user_id;

    let response = scrap_service
        .cancel_scrap(tenant_id, scrap_id, user_id)
        .await?;

    Ok(Json(response))
}

/// Create router for scrap management endpoints
pub fn create_scrap_routes<S: ScrapService + 'static>() -> axum::Router {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/", post(create_scrap::<S>).get(list_scraps::<S>))
        .route("/{scrap_id}", get(get_scrap::<S>))
        .route("/{scrap_id}/lines", post(add_scrap_lines::<S>))
        .route("/{scrap_id}/post", post(post_scrap::<S>))
        .route("/{scrap_id}/cancel", post(cancel_scrap::<S>))
}
