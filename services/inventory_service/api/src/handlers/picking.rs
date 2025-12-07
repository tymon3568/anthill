//! Picking HTTP handlers
//!
//! This module contains the Axum handlers for picking operations.

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use uuid::Uuid;
use validator::Validate;

use inventory_service_core::domains::inventory::dto::picking_method_dto::{
    ConfirmPickingPlanRequest, CreatePickingMethodRequest, PickingMethodResponse,
    PickingOptimizationRequest, PickingPlanResponse, UpdatePickingMethodRequest,
};
use shared_auth::extractors::AuthUser;
use shared_error::AppError;

use crate::state::AppState;

/// Create the picking routes
pub fn create_picking_routes() -> Router {
    Router::new()
        .route("/methods", post(create_picking_method))
        .route("/methods", get(list_picking_methods))
        .route("/methods/{method_id}", get(get_picking_method))
        .route("/methods/{method_id}", put(update_picking_method))
        .route("/methods/{method_id}", delete(delete_picking_method))
        .route("/methods/{method_id}/default", put(set_default_method))
        .route("/optimize", post(optimize_picking))
        .route("/confirm", post(confirm_picking_plan))
}

/// POST /api/v1/warehouse/picking/methods - Create a new picking method
#[utoipa::path(
    post,
    path = "/api/v1/warehouse/picking/methods",
    tag = "warehouse",
    operation_id = "create_picking_method",
    request_body = CreatePickingMethodRequest,
    responses(
        (status = 201, description = "Picking method created", body = PickingMethodResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_picking_method(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<CreatePickingMethodRequest>,
) -> Result<Json<PickingMethodResponse>, AppError> {
    // Validate request
    request.validate()?;

    let method = state
        .picking_method_service
        .create_method(auth_user.tenant_id, request, auth_user.user_id)
        .await?;

    Ok(Json(method.into()))
}

/// GET /api/v1/warehouse/picking/methods - List picking methods for warehouse
#[utoipa::path(
    get,
    path = "/api/v1/warehouse/picking/methods",
    tag = "warehouse",
    operation_id = "list_picking_methods",
    params(
        ("warehouse_id" = Uuid, Query, description = "Warehouse ID to filter methods")
    ),
    responses(
        (status = 200, description = "List of picking methods", body = Vec<PickingMethodResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_picking_methods(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<PickingMethodResponse>>, AppError> {
    let warehouse_id = params
        .get("warehouse_id")
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::ValidationError("warehouse_id is required".to_string()))?;

    let methods = state
        .picking_method_service
        .get_methods_by_warehouse(auth_user.tenant_id, warehouse_id)
        .await?;

    let response = methods.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

/// GET /api/v1/warehouse/picking/methods/{method_id} - Get picking method by ID
#[utoipa::path(
    get,
    path = "/api/v1/warehouse/picking/methods/{method_id}",
    tag = "warehouse",
    operation_id = "get_picking_method",
    params(
        ("method_id" = Uuid, Path, description = "Picking method ID")
    ),
    responses(
        (status = 200, description = "Picking method found", body = PickingMethodResponse),
        (status = 404, description = "Picking method not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_picking_method(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(method_id): Path<Uuid>,
) -> Result<Json<PickingMethodResponse>, AppError> {
    let method = state
        .picking_method_service
        .get_method(auth_user.tenant_id, method_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Picking method not found".to_string()))?;

    Ok(Json(method.into()))
}

/// PUT /api/v1/warehouse/picking/methods/{method_id} - Update picking method
#[utoipa::path(
    put,
    path = "/api/v1/warehouse/picking/methods/{method_id}",
    tag = "warehouse",
    operation_id = "update_picking_method",
    params(
        ("method_id" = Uuid, Path, description = "Picking method ID")
    ),
    request_body = UpdatePickingMethodRequest,
    responses(
        (status = 200, description = "Picking method updated", body = PickingMethodResponse),
        (status = 404, description = "Picking method not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_picking_method(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(method_id): Path<Uuid>,
    Json(request): Json<UpdatePickingMethodRequest>,
) -> Result<Json<PickingMethodResponse>, AppError> {
    // Validate request
    request.validate()?;

    let method = state
        .picking_method_service
        .update_method(auth_user.tenant_id, method_id, request, auth_user.user_id)
        .await?;

    Ok(Json(method.into()))
}

/// DELETE /api/v1/warehouse/picking/methods/{method_id} - Delete picking method
#[utoipa::path(
    delete,
    path = "/api/v1/warehouse/picking/methods/{method_id}",
    tag = "warehouse",
    operation_id = "delete_picking_method",
    params(
        ("method_id" = Uuid, Path, description = "Picking method ID")
    ),
    responses(
        (status = 204, description = "Picking method deleted"),
        (status = 404, description = "Picking method not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_picking_method(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(method_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = state
        .picking_method_service
        .delete_method(auth_user.tenant_id, method_id)
        .await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Picking method not found".to_string()))
    }
}

/// PUT /api/v1/warehouse/picking/methods/{method_id}/default - Set as default method
#[utoipa::path(
    put,
    path = "/api/v1/warehouse/picking/methods/{method_id}/default",
    tag = "warehouse",
    operation_id = "set_default_method",
    params(
        ("method_id" = Uuid, Path, description = "Picking method ID")
    ),
    responses(
        (status = 200, description = "Default method set"),
        (status = 404, description = "Picking method not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn set_default_method(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(method_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // First get the method to find its warehouse_id
    let method = state
        .picking_method_service
        .get_method(auth_user.tenant_id, method_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Picking method not found".to_string()))?;

    let success = state
        .picking_method_service
        .set_default_method(auth_user.tenant_id, method.warehouse_id, method_id, auth_user.user_id)
        .await?;

    if success {
        Ok(StatusCode::OK)
    } else {
        Err(AppError::InternalError("Failed to set default method".to_string()))
    }
}

/// POST /api/v1/warehouse/picking/optimize - Generate optimized picking plan
#[utoipa::path(
    post,
    path = "/api/v1/warehouse/picking/optimize",
    tag = "warehouse",
    operation_id = "optimize_picking",
    request_body = PickingOptimizationRequest,
    responses(
        (status = 200, description = "Picking plan generated", body = PickingPlanResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn optimize_picking(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<PickingOptimizationRequest>,
) -> Result<Json<PickingPlanResponse>, AppError> {
    // Validate request
    request.validate()?;

    let plan = state
        .picking_method_service
        .optimize_picking(auth_user.tenant_id, request)
        .await?;

    Ok(Json(plan))
}

/// POST /api/v1/warehouse/picking/confirm - Confirm picking plan execution
#[utoipa::path(
    post,
    path = "/api/v1/warehouse/picking/confirm",
    tag = "warehouse",
    operation_id = "confirm_picking_plan",
    request_body = ConfirmPickingPlanRequest,
    responses(
        (status = 200, description = "Picking plan confirmed"),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn confirm_picking_plan(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<ConfirmPickingPlanRequest>,
) -> Result<StatusCode, AppError> {
    // Validate request
    request.validate()?;

    let success = state
        .picking_method_service
        .confirm_picking_plan(auth_user.tenant_id, request)
        .await?;

    if success {
        Ok(StatusCode::OK)
    } else {
        Err(AppError::InternalError("Failed to confirm picking plan".to_string()))
    }
}
