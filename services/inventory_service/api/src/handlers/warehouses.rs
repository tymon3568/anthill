use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use inventory_service_core::domains::inventory::dto::warehouse_dto::{
    CreateWarehouseRequest, WarehouseResponse, WarehouseTreeResponse,
};
use inventory_service_core::repositories::warehouse::WarehouseRepository;
use inventory_service_infra::AppState;
use shared::auth::extractors::{AuthUser, RequirePermission};
use shared::error::AppError;

/// Create a new warehouse
#[utoipa::path(
    post,
    path = "/api/v1/inventory/warehouses",
    tag = "warehouses",
    operation_id = "create_warehouse",
    request_body = CreateWarehouseRequest,
    responses(
        (status = 201, body = WarehouseResponse),
        (status = 400, body = shared::error::ErrorResponse),
        (status = 401, body = shared::error::ErrorResponse),
        (status = 403, body = shared::error::ErrorResponse),
        (status = 500, body = shared::error::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_warehouse<R: WarehouseRepository>(
    State(state): State<AppState<R>>,
    AuthUser(user): AuthUser,
    RequirePermission(_perm): RequirePermission,
    Json(request): Json<CreateWarehouseRequest>,
) -> Result<Json<WarehouseResponse>, AppError> {
    // Validate request
    request.validate()?;

    // Validate hierarchy if parent is specified
    if let Some(parent_id) = request.parent_warehouse_id {
        let is_valid = state
            .warehouse_repository
            .validate_hierarchy(user.tenant_id, Uuid::new_v4(), Some(parent_id))
            .await?;
        if !is_valid {
            return Err(AppError::ValidationError("Invalid warehouse hierarchy".to_string()));
        }
    }

    // Create warehouse
    let warehouse = state
        .warehouse_repository
        .create(user.tenant_id, request)
        .await?;

    Ok(Json(warehouse.into()))
}

/// Get warehouse hierarchy/tree
#[utoipa::path(
    get,
    path = "/api/v1/inventory/warehouses/tree",
    tag = "warehouses",
    operation_id = "get_warehouse_tree",
    responses(
        (status = 200, body = WarehouseTreeResponse),
        (status = 401, body = shared::error::ErrorResponse),
        (status = 403, body = shared::error::ErrorResponse),
        (status = 500, body = shared::error::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_warehouse_tree<R: WarehouseRepository>(
    State(state): State<AppState<R>>,
    AuthUser(user): AuthUser,
    RequirePermission(_perm): RequirePermission,
) -> Result<Json<WarehouseTreeResponse>, AppError> {
    let tree = state
        .warehouse_repository
        .get_warehouse_tree(user.tenant_id)
        .await?;

    Ok(Json(tree))
}

/// Get warehouse by ID
#[utoipa::path(
    get,
    path = "/api/v1/inventory/warehouses/{id}",
    tag = "warehouses",
    operation_id = "get_warehouse",
    params(
        ("id" = Uuid, Path, description = "Warehouse ID")
    ),
    responses(
        (status = 200, body = WarehouseResponse),
        (status = 404, body = shared::error::ErrorResponse),
        (status = 401, body = shared::error::ErrorResponse),
        (status = 403, body = shared::error::ErrorResponse),
        (status = 500, body = shared::error::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_warehouse<R: WarehouseRepository>(
    State(state): State<AppState<R>>,
    AuthUser(user): AuthUser,
    RequirePermission(_perm): RequirePermission,
    Path(warehouse_id): Path<Uuid>,
) -> Result<Json<WarehouseResponse>, AppError> {
    let warehouse = state
        .warehouse_repository
        .find_by_id(user.tenant_id, warehouse_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Warehouse not found".to_string()))?;

    Ok(Json(warehouse.into()))
}

/// Get all warehouses
#[utoipa::path(
    get,
    path = "/api/v1/inventory/warehouses",
    tag = "warehouses",
    operation_id = "get_warehouses",
    responses(
        (status = 200, body = Vec<WarehouseResponse>),
        (status = 401, body = shared::error::ErrorResponse),
        (status = 403, body = shared::error::ErrorResponse),
        (status = 500, body = shared::error::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_warehouses<R: WarehouseRepository>(
    State(state): State<AppState<R>>,
    AuthUser(user): AuthUser,
    RequirePermission(_perm): RequirePermission,
) -> Result<Json<Vec<WarehouseResponse>>, AppError> {
    let warehouses = state.warehouse_repository.find_all(user.tenant_id).await?;

    let responses = warehouses
        .into_iter()
        .map(WarehouseResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Update warehouse
#[utoipa::path(
    put,
    path = "/api/v1/inventory/warehouses/{id}",
    tag = "warehouses",
    operation_id = "update_warehouse",
    params(
        ("id" = Uuid, Path, description = "Warehouse ID")
    ),
    request_body = CreateWarehouseRequest,
    responses(
        (status = 200, body = WarehouseResponse),
        (status = 404, body = shared::error::ErrorResponse),
        (status = 400, body = shared::error::ErrorResponse),
        (status = 401, body = shared::error::ErrorResponse),
        (status = 403, body = shared::error::ErrorResponse),
        (status = 500, body = shared::error::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_warehouse<R: WarehouseRepository>(
    State(state): State<AppState<R>>,
    AuthUser(user): AuthUser,
    RequirePermission(_perm): RequirePermission,
    Path(warehouse_id): Path<Uuid>,
    Json(request): Json<CreateWarehouseRequest>,
) -> Result<Json<WarehouseResponse>, AppError> {
    // Validate request
    request.validate()?;

    // Check if warehouse exists
    let existing = state
        .warehouse_repository
        .find_by_id(user.tenant_id, warehouse_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Warehouse not found".to_string()))?;

    // Validate hierarchy if parent is specified
    if let Some(parent_id) = request.parent_warehouse_id {
        let is_valid = state
            .warehouse_repository
            .validate_hierarchy(user.tenant_id, warehouse_id, Some(parent_id))
            .await?;
        if !is_valid {
            return Err(AppError::ValidationError("Invalid warehouse hierarchy".to_string()));
        }
    }

    // Create updated warehouse entity
    let mut updated_warehouse = existing;
    updated_warehouse.warehouse_code = request.warehouse_code;
    updated_warehouse.warehouse_name = request.warehouse_name;
    updated_warehouse.description = request.description;
    updated_warehouse.warehouse_type = request.warehouse_type;
    updated_warehouse.parent_warehouse_id = request.parent_warehouse_id;
    updated_warehouse.address = request.address;
    updated_warehouse.contact_info = request.contact_info;
    updated_warehouse.capacity_info = request.capacity_info;
    updated_warehouse.touch();

    // Update in database
    let warehouse = state
        .warehouse_repository
        .update(user.tenant_id, warehouse_id, &updated_warehouse)
        .await?;

    Ok(Json(warehouse.into()))
}

/// Delete warehouse
#[utoipa::path(
    delete,
    path = "/api/v1/inventory/warehouses/{id}",
    tag = "warehouses",
    operation_id = "delete_warehouse",
    params(
        ("id" = Uuid, Path, description = "Warehouse ID")
    ),
    responses(
        (status = 204, description = "Warehouse deleted successfully"),
        (status = 404, body = shared::error::ErrorResponse),
        (status = 401, body = shared::error::ErrorResponse),
        (status = 403, body = shared::error::ErrorResponse),
        (status = 500, body = shared::error::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_warehouse<R: WarehouseRepository>(
    State(state): State<AppState<R>>,
    AuthUser(user): AuthUser,
    RequirePermission(_perm): RequirePermission,
    Path(warehouse_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = state
        .warehouse_repository
        .delete(user.tenant_id, warehouse_id)
        .await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Warehouse not found".to_string()))
    }
}
