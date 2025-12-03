use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use inventory_service_core::{
    domains::replenishment::{CreateReorderRule, ReplenishmentCheckResult, UpdateReorderRule},
    services::replenishment::ReplenishmentService,
};
use serde::Deserialize;
use shared_auth::AuthUser;
use shared_error::AppError;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::state::AppState;

/// Create a new reorder rule
#[utoipa::path(
    post,
    path = "/api/v1/inventory/replenishment/rules",
    tag = "replenishment",
    operation_id = "create_reorder_rule",
    request_body = CreateReorderRule,
    responses(
        (status = 201, body = inventory_service_core::domains::replenishment::ReorderRule),
        (status = 400, body = shared_error::ErrorResponse),
        (status = 401, body = shared_error::ErrorResponse),
        (status = 500, body = shared_error::ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_reorder_rule(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(rule): Json<CreateReorderRule>,
) -> Result<Json<inventory_service_core::domains::replenishment::ReorderRule>, AppError> {
    let created_rule = state
        .replenishment_service
        .create_reorder_rule(user.tenant_id, rule)
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
        (status = 404, body = shared_error::ErrorResponse),
        (status = 401, body = shared_error::ErrorResponse),
        (status = 500, body = shared_error::ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_reorder_rule(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(rule_id): Path<Uuid>,
) -> Result<Json<inventory_service_core::domains::replenishment::ReorderRule>, AppError> {
    let rule = state
        .replenishment_service
        .get_reorder_rule(user.tenant_id, rule_id)
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
        (status = 404, body = shared_error::ErrorResponse),
        (status = 401, body = shared_error::ErrorResponse),
        (status = 500, body = shared_error::ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_reorder_rule(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(rule_id): Path<Uuid>,
    Json(updates): Json<UpdateReorderRule>,
) -> Result<Json<inventory_service_core::domains::replenishment::ReorderRule>, AppError> {
    let updated_rule = state
        .replenishment_service
        .update_reorder_rule(user.tenant_id, rule_id, updates)
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
        (status = 404, body = shared_error::ErrorResponse),
        (status = 401, body = shared_error::ErrorResponse),
        (status = 500, body = shared_error::ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_reorder_rule(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(rule_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .replenishment_service
        .delete_reorder_rule(user.tenant_id, rule_id)
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
        (status = 401, body = shared_error::ErrorResponse),
        (status = 500, body = shared_error::ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_reorder_rules_for_product(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(product_id): Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<ListRulesQuery>,
) -> Result<Json<Vec<inventory_service_core::domains::replenishment::ReorderRule>>, AppError> {
    let rules = state
        .replenishment_service
        .list_reorder_rules_for_product(user.tenant_id, product_id, params.warehouse_id)
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
        (status = 401, body = shared_error::ErrorResponse),
        (status = 500, body = shared_error::ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn run_replenishment_check(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<ReplenishmentCheckResult>>, AppError> {
    let results = state
        .replenishment_service
        .run_replenishment_check(user.tenant_id)
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
        (status = 404, body = shared_error::ErrorResponse),
        (status = 401, body = shared_error::ErrorResponse),
        (status = 500, body = shared_error::ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn check_product_replenishment(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(product_id): Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<CheckProductQuery>,
) -> Result<Json<ReplenishmentCheckResult>, AppError> {
    let result = state
        .replenishment_service
        .check_product_replenishment(user.tenant_id, product_id, params.warehouse_id)
        .await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct ListRulesQuery {
    pub warehouse_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CheckProductQuery {
    pub warehouse_id: Option<Uuid>,
}
