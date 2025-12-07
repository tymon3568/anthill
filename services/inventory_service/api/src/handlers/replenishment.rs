use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    Json,
};
use inventory_service_core::domains::replenishment::{
    CreateReorderRule, ReplenishmentCheckResult, UpdateReorderRule,
};
use serde::Deserialize;
use shared_auth::AuthUser;
use shared_error::AppError;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::state::AppState;

#[derive(utoipa::ToSchema)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
}

/// Create a new reorder rule
#[utoipa::path(
    post,
    path = "/api/v1/inventory/replenishment/rules",
    tag = "replenishment",
    operation_id = "create_reorder_rule",
    request_body = CreateReorderRule,
    responses(
        (status = 201, body = inventory_service_core::domains::replenishment::ReorderRule),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_reorder_rule(
    Extension(state): Extension<AppState>,
    auth_user: AuthUser,
    Json(rule): Json<CreateReorderRule>,
) -> Result<Json<inventory_service_core::domains::replenishment::ReorderRule>, AppError> {
    let created_rule = state
        .replenishment_service
        .create_reorder_rule(auth_user.tenant_id, rule)
        .await?;
    Ok(Json(created_rule))
}

/// Get a reorder rule by ID
#[utoipa::path(
    get,
    path = "/api/v1/inventory/replenishment/rules/{rule_id}",
    tag = "replenishment",
    operation_id = "get_reorder_rule",
    params(
        ("rule_id" = Uuid, Path, description = "Reorder rule ID")
    ),
    responses(
        (status = 200, body = inventory_service_core::domains::replenishment::ReorderRule),
        (status = 404, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_reorder_rule(
    Extension(state): Extension<AppState>,
    auth_user: AuthUser,
    Path(rule_id): Path<Uuid>,
) -> Result<Json<inventory_service_core::domains::replenishment::ReorderRule>, AppError> {
    let rule = state
        .replenishment_service
        .get_reorder_rule(auth_user.tenant_id, rule_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Reorder rule not found".to_string()))?;
    Ok(Json(rule))
}

/// Update a reorder rule
#[utoipa::path(
    put,
    path = "/api/v1/inventory/replenishment/rules/{rule_id}",
    tag = "replenishment",
    operation_id = "update_reorder_rule",
    params(
        ("rule_id" = Uuid, Path, description = "Reorder rule ID")
    ),
    request_body = UpdateReorderRule,
    responses(
        (status = 200, body = inventory_service_core::domains::replenishment::ReorderRule),
        (status = 404, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_reorder_rule(
    Extension(state): Extension<AppState>,
    auth_user: AuthUser,
    Path(rule_id): Path<Uuid>,
    Json(updates): Json<UpdateReorderRule>,
) -> Result<Json<inventory_service_core::domains::replenishment::ReorderRule>, AppError> {
    let updated_rule = state
        .replenishment_service
        .update_reorder_rule(auth_user.tenant_id, rule_id, updates)
        .await?;
    Ok(Json(updated_rule))
}

/// Delete a reorder rule
#[utoipa::path(
    delete,
    path = "/api/v1/inventory/replenishment/rules/{rule_id}",
    tag = "replenishment",
    operation_id = "delete_reorder_rule",
    params(
        ("rule_id" = Uuid, Path, description = "Reorder rule ID")
    ),
    responses(
        (status = 204, description = "Rule deleted"),
        (status = 404, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_reorder_rule(
    Extension(state): Extension<AppState>,
    auth_user: AuthUser,
    Path(rule_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .replenishment_service
        .delete_reorder_rule(auth_user.tenant_id, rule_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// List reorder rules for a product
#[utoipa::path(
    get,
    path = "/api/v1/inventory/replenishment/rules/product/{product_id}",
    tag = "replenishment",
    operation_id = "list_reorder_rules_for_product",
    params(
        ("product_id" = Uuid, Path, description = "Product ID"),
        ("warehouse_id" = Option<Uuid>, Query, description = "Warehouse ID filter")
    ),
    responses(
        (status = 200, body = Vec<inventory_service_core::domains::replenishment::ReorderRule>),
        (status = 401, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_reorder_rules_for_product(
    Extension(state): Extension<AppState>,
    auth_user: AuthUser,
    Path(product_id): Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<WarehouseFilterQuery>,
) -> Result<Json<Vec<inventory_service_core::domains::replenishment::ReorderRule>>, AppError> {
    let rules = state
        .replenishment_service
        .list_reorder_rules_for_product(auth_user.tenant_id, product_id, params.warehouse_id)
        .await?;
    Ok(Json(rules))
}

/// Run replenishment check for all active rules
#[utoipa::path(
    post,
    path = "/api/v1/inventory/replenishment/check",
    tag = "replenishment",
    operation_id = "run_replenishment_check",
    responses(
        (status = 200, body = Vec<ReplenishmentCheckResult>),
        (status = 401, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn run_replenishment_check(
    Extension(state): Extension<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<ReplenishmentCheckResult>>, AppError> {
    let results = state
        .replenishment_service
        .run_replenishment_check(auth_user.tenant_id)
        .await?;
    Ok(Json(results))
}

/// Run replenishment check for a specific product
#[utoipa::path(
    post,
    path = "/api/v1/inventory/replenishment/check/product/{product_id}",
    tag = "replenishment",
    operation_id = "check_product_replenishment",
    params(
        ("product_id" = Uuid, Path, description = "Product ID"),
        ("warehouse_id" = Option<Uuid>, Query, description = "Warehouse ID")
    ),
    responses(
        (status = 200, body = ReplenishmentCheckResult),
        (status = 404, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn check_product_replenishment(
    Extension(state): Extension<AppState>,
    auth_user: AuthUser,
    Path(product_id): Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<WarehouseFilterQuery>,
) -> Result<Json<ReplenishmentCheckResult>, AppError> {
    let result = state
        .replenishment_service
        .check_product_replenishment(auth_user.tenant_id, product_id, params.warehouse_id)
        .await?;
    Ok(Json(result))
}

#[derive(Deserialize, ToSchema)]
pub struct WarehouseFilterQuery {
    pub warehouse_id: Option<Uuid>,
}
