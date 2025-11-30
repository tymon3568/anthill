use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use uuid::Uuid;
use validator::Validate;

use crate::state::AppState;
use inventory_service_core::domains::inventory::dto::warehouse_dto::{
    CreateWarehouseLocationRequest, CreateWarehouseRequest, CreateWarehouseZoneRequest,
    WarehouseLocationResponse, WarehouseResponse, WarehouseTreeResponse, WarehouseZoneResponse,
};
use inventory_service_core::domains::inventory::BaseEntity;

use shared_auth::extractors::{AuthUser, RequirePermission};
use shared_error::AppError;

/// Error response for OpenAPI documentation
#[derive(utoipa::ToSchema)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
}

/// Create a new warehouse
#[utoipa::path(
    post,
    path = "/api/v1/inventory/warehouses",
    tag = "warehouses",
    operation_id = "create_warehouse",
    request_body = CreateWarehouseRequest,
    responses(
        (status = 201, body = WarehouseResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_warehouse(
    State(state): State<AppState>,
    user: AuthUser,
    RequirePermission { .. }: RequirePermission,
    Json(request): Json<CreateWarehouseRequest>,
) -> Result<Json<WarehouseResponse>, AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Validate parent warehouse exists and is active if specified
    if let Some(parent_id) = request.parent_warehouse_id {
        let parent = state
            .warehouse_repository
            .find_by_id(user.tenant_id, parent_id)
            .await?;
        match parent {
            None => {
                return Err(AppError::ValidationError(
                    "Parent warehouse does not exist".to_string(),
                ))
            },
            Some(p) if !p.is_active => {
                return Err(AppError::ValidationError("Parent warehouse is not active".to_string()))
            },
            _ => {},
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
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_warehouse_tree(
    State(state): State<AppState>,
    user: AuthUser,
    RequirePermission { .. }: RequirePermission,
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
        (status = 404, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_warehouse(
    State(state): State<AppState>,
    user: AuthUser,
    RequirePermission { .. }: RequirePermission,
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
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_warehouses(
    State(state): State<AppState>,
    user: AuthUser,
    RequirePermission { .. }: RequirePermission,
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
        (status = 404, body = ErrorResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_warehouse(
    State(state): State<AppState>,
    user: AuthUser,
    RequirePermission { .. }: RequirePermission,
    Path(warehouse_id): Path<Uuid>,
    Json(request): Json<CreateWarehouseRequest>,
) -> Result<Json<WarehouseResponse>, AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

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
        (status = 404, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_warehouse(
    State(state): State<AppState>,
    user: AuthUser,
    RequirePermission { .. }: RequirePermission,
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

/// Create a new zone in a warehouse
#[utoipa::path(
    post,
    path = "/api/v1/inventory/warehouses/{warehouse_id}/zones",
    tag = "warehouses",
    operation_id = "create_warehouse_zone",
    params(
        ("warehouse_id" = Uuid, Path, description = "Warehouse ID")
    ),
    request_body = CreateWarehouseZoneRequest,
    responses(
        (status = 201, body = WarehouseZoneResponse),
        (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_zone(
    State(state): State<AppState>,
    user: AuthUser,
    RequirePermission { .. }: RequirePermission,
    Path(warehouse_id): Path<Uuid>,
    Json(request): Json<CreateWarehouseZoneRequest>,
) -> Result<Json<WarehouseZoneResponse>, AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Check if warehouse exists
    let warehouse_exists = state
        .warehouse_repository
        .find_by_id(user.tenant_id, warehouse_id)
        .await?
        .is_some();
    if !warehouse_exists {
        return Err(AppError::NotFound("Warehouse not found".to_string()));
    }

    // Create zone
    let zone = state
        .warehouse_repository
        .create_zone(user.tenant_id, warehouse_id, request)
        .await?;

    Ok(Json(zone.into()))
}

/// Create a new location in a warehouse
#[utoipa::path(
    post,
    path = "/api/v1/inventory/warehouses/{warehouse_id}/locations",
    tag = "warehouses",
    operation_id = "create_warehouse_location",
    params(
        ("warehouse_id" = Uuid, Path, description = "Warehouse ID")
    ),
    request_body = CreateWarehouseLocationRequest,
    responses(
        (status = 201, body = WarehouseLocationResponse),
        (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_location(
    State(state): State<AppState>,
    user: AuthUser,
    RequirePermission { .. }: RequirePermission,
    Path(warehouse_id): Path<Uuid>,
    Json(request): Json<CreateWarehouseLocationRequest>,
) -> Result<Json<WarehouseLocationResponse>, AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Check if warehouse exists
    let warehouse_exists = state
        .warehouse_repository
        .find_by_id(user.tenant_id, warehouse_id)
        .await?
        .is_some();
    if !warehouse_exists {
        return Err(AppError::NotFound("Warehouse not found".to_string()));
    }

    // Check if zone exists and belongs to the warehouse (if zone_id is specified)
    if let Some(_zone_id) = request.zone_id {
        // For now, assume zones are validated by FK in DB
        // TODO: Add zone existence check if needed
    }

    // Create location
    let location = state
        .warehouse_repository
        .create_location(user.tenant_id, warehouse_id, request)
        .await?;

    Ok(Json(location.into()))
}

/// Create warehouse routes
pub fn create_warehouse_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_warehouses).post(create_warehouse))
        .route("/tree", get(get_warehouse_tree))
        .route(
            "/{warehouse_id}",
            get(get_warehouse)
                .put(update_warehouse)
                .delete(delete_warehouse),
        )
        .route("/{warehouse_id}/zones", post(create_zone))
        .route("/{warehouse_id}/locations", post(create_location))
        .with_state(state)
}
