//! Inventory valuation HTTP handlers
//!
//! This module contains the Axum handlers for inventory valuation endpoints.

use axum::{
    extract::{Extension, Path, Query},
    response::Json,
    routing::{get, post, put},
    Router,
};
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::valuation_dto::{
    CostAdjustmentRequest, DeleteValuationSettingsRequest, EffectiveValuationMethodResponse,
    GetEffectiveValuationMethodRequest, GetTenantValuationSettingsRequest,
    GetValuationHistoryRequest, GetValuationLayersRequest, GetValuationRequest,
    ListValuationSettingsRequest, RevaluationRequest, SetCategoryValuationMethodRequest,
    SetProductValuationMethodRequest, SetStandardCostRequest, SetTenantValuationMethodRequest,
    SetValuationMethodRequest, ValuationDto, ValuationHistoryResponse, ValuationLayersResponse,
    ValuationSettingsDto, ValuationSettingsListResponse,
};
use inventory_service_core::domains::inventory::valuation::{ValuationMethod, ValuationScopeType};

use crate::state::AppState;
use shared_auth::extractors::AuthUser;
use shared_error::AppError;

/// Error response for OpenAPI documentation
#[derive(utoipa::ToSchema)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
}

/// Create the valuation routes
pub fn create_valuation_routes() -> Router {
    Router::new()
        .route("/{product_id}", get(get_valuation))
        .route("/{product_id}/method", put(set_valuation_method))
        .route("/{product_id}/standard-cost", put(set_standard_cost))
        .route("/{product_id}/layers", get(get_valuation_layers))
        .route("/{product_id}/history", get(get_valuation_history))
        .route("/{product_id}/adjust", post(adjust_cost))
        .route("/{product_id}/revalue", post(revalue_inventory))
        // Valuation settings endpoints
        .route("/settings", get(list_valuation_settings))
        .route("/settings/tenant", get(get_tenant_settings).put(set_tenant_method))
        .route("/settings/effective/{product_id}", get(get_effective_method))
        .route("/settings/category/{category_id}", put(set_category_method).delete(delete_category_settings))
        .route("/settings/product/{product_id}", put(set_product_method).delete(delete_product_settings))
}

/// GET /api/v1/inventory/valuation/{product_id} - Get current valuation for a product
///
/// Returns the current inventory valuation for a specific product.
/// The valuation includes current quantity, value, and cost based on the
/// product's valuation method (FIFO, AVCO, or Standard).
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `product_id` - Product UUID
///
/// # Returns
/// * `200` - Current valuation data
/// * `404` - Product or valuation not found
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    get,
    path = "/api/v1/inventory/valuation/{product_id}",
    tag = "valuation",
    operation_id = "get_valuation",
    params(
        ("product_id" = Uuid, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Current valuation data", body = ValuationDto),
        (status = 404, description = "Product or valuation not found", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_valuation(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<ValuationDto>, AppError> {
    let request = GetValuationRequest {
        tenant_id: auth_user.tenant_id,
        product_id,
    };

    let valuation = state.valuation_service.get_valuation(request).await?;

    Ok(Json(valuation))
}

/// PUT /api/v1/inventory/valuation/{product_id}/method - Set valuation method for a product
///
/// Changes the valuation method for a product. This affects how inventory
/// costs are calculated and tracked.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `product_id` - Product UUID
///
/// # Request Body
/// ```json
/// {
///   "valuation_method": "fifo" | "avco" | "standard"
/// }
/// ```
///
/// # Returns
/// * `200` - Updated valuation data
/// * `400` - Invalid valuation method
/// * `404` - Product not found
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    put,
    path = "/api/v1/inventory/valuation/{product_id}/method",
    tag = "valuation",
    operation_id = "set_valuation_method",
    params(
        ("product_id" = Uuid, Path, description = "Product ID")
    ),
    request_body = SetValuationMethodPayload,
    responses(
        (status = 200, description = "Updated valuation data", body = ValuationDto),
        (status = 400, description = "Invalid valuation method", body = ErrorResponse),
        (status = 404, description = "Product not found", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn set_valuation_method(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<SetValuationMethodPayload>,
) -> Result<Json<ValuationDto>, AppError> {
    let request = SetValuationMethodRequest {
        tenant_id: auth_user.tenant_id,
        product_id,
        valuation_method: payload.valuation_method,
    };

    let valuation = state
        .valuation_service
        .set_valuation_method(request)
        .await?;

    Ok(Json(valuation))
}

/// PUT /api/v1/inventory/valuation/{product_id}/standard-cost - Set standard cost for a product
///
/// Sets the standard cost for products using Standard costing method.
/// Only applicable when the product uses standard valuation.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `product_id` - Product UUID
///
/// # Request Body
/// ```json
/// {
///   "standard_cost": 15000  // Cost in cents
/// }
/// ```
///
/// # Returns
/// * `200` - Updated valuation data
/// * `400` - Invalid cost or product doesn't use standard costing
/// * `404` - Product not found
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    put,
    path = "/api/v1/inventory/valuation/{product_id}/standard-cost",
    tag = "valuation",
    operation_id = "set_standard_cost",
    params(
        ("product_id" = Uuid, Path, description = "Product ID")
    ),
    request_body = SetStandardCostPayload,
    responses(
        (status = 200, description = "Updated valuation data", body = ValuationDto),
        (status = 400, description = "Invalid cost or product doesn't use standard costing", body = ErrorResponse),
        (status = 404, description = "Product not found", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn set_standard_cost(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<SetStandardCostPayload>,
) -> Result<Json<ValuationDto>, AppError> {
    let request = SetStandardCostRequest {
        tenant_id: auth_user.tenant_id,
        product_id,
        standard_cost: payload.standard_cost,
    };

    let valuation = state.valuation_service.set_standard_cost(request).await?;

    Ok(Json(valuation))
}

/// GET /api/v1/inventory/valuation/{product_id}/layers - Get valuation layers for FIFO
///
/// Returns the active cost layers for products using FIFO valuation.
/// Layers represent different cost levels for remaining inventory.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `product_id` - Product UUID
///
/// # Returns
/// * `200` - List of active cost layers
/// * `404` - Product not found or doesn't use FIFO
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    get,
    path = "/api/v1/inventory/valuation/{product_id}/layers",
    tag = "valuation",
    operation_id = "get_valuation_layers",
    params(
        ("product_id" = Uuid, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "List of active cost layers", body = ValuationLayersResponse),
        (status = 404, description = "Product not found or doesn't use FIFO", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_valuation_layers(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<ValuationLayersResponse>, AppError> {
    let request = GetValuationLayersRequest {
        tenant_id: auth_user.tenant_id,
        product_id,
    };

    let layers = state
        .valuation_service
        .get_valuation_layers(request)
        .await?;

    Ok(Json(layers))
}

/// GET /api/v1/inventory/valuation/{product_id}/history - Get valuation history
///
/// Returns the historical changes to a product's valuation.
/// Useful for auditing and financial reporting.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `product_id` - Product UUID
///
/// # Query Parameters
/// * `limit` - Maximum number of records (default: 50, max: 100)
/// * `offset` - Number of records to skip (default: 0)
///
/// # Returns
/// * `200` - Historical valuation records with pagination
/// * `404` - Product not found
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    get,
    path = "/api/v1/inventory/valuation/{product_id}/history",
    tag = "valuation",
    operation_id = "get_valuation_history",
    params(
        ("product_id" = Uuid, Path, description = "Product ID"),
        HistoryQueryParams
    ),
    responses(
        (status = 200, description = "Historical valuation records with pagination", body = ValuationHistoryResponse),
        (status = 404, description = "Product not found", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_valuation_history(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
    Query(params): Query<HistoryQueryParams>,
) -> Result<Json<ValuationHistoryResponse>, AppError> {
    let limit = params.limit.map(|l| l.min(100)).or(Some(50));
    let offset = params.offset.or(Some(0));

    let request = GetValuationHistoryRequest {
        tenant_id: auth_user.tenant_id,
        product_id,
        limit,
        offset,
    };

    let history = state
        .valuation_service
        .get_valuation_history(request)
        .await?;

    Ok(Json(history))
}

/// POST /api/v1/inventory/valuation/{product_id}/adjust - Adjust inventory cost
///
/// Performs a cost adjustment to the inventory valuation.
/// This can be used for write-offs, revaluations, or corrections.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `product_id` - Product UUID
///
/// # Request Body
/// ```json
/// {
///   "adjustment_amount": -5000,  // Adjustment in cents (can be negative)
///   "reason": "Damaged goods write-off"
/// }
/// ```
///
/// # Returns
/// * `200` - Updated valuation data
/// * `400` - Invalid adjustment amount
/// * `404` - Product not found
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    post,
    path = "/api/v1/inventory/valuation/{product_id}/adjust",
    tag = "valuation",
    operation_id = "adjust_cost",
    params(
        ("product_id" = Uuid, Path, description = "Product ID")
    ),
    request_body = CostAdjustmentPayload,
    responses(
        (status = 200, description = "Updated valuation data", body = ValuationDto),
        (status = 400, description = "Invalid adjustment amount", body = ErrorResponse),
        (status = 404, description = "Product not found", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn adjust_cost(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<CostAdjustmentPayload>,
) -> Result<Json<ValuationDto>, AppError> {
    let request = CostAdjustmentRequest {
        tenant_id: auth_user.tenant_id,
        product_id,
        adjustment_amount: payload.adjustment_amount,
        reason: payload.reason,
    };

    let valuation = state.valuation_service.adjust_cost(request).await?;

    Ok(Json(valuation))
}

/// POST /api/v1/inventory/valuation/{product_id}/revalue - Revalue inventory
///
/// Revalues the entire inventory at a new cost basis.
/// This changes the cost of existing inventory without affecting quantity.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `product_id` - Product UUID
///
/// # Request Body
/// ```json
/// {
///   "new_unit_cost": 12000,  // New cost per unit in cents
///   "reason": "Market price adjustment"
/// }
/// ```
///
/// # Returns
/// * `200` - Updated valuation data
/// * `400` - Invalid cost
/// * `404` - Product not found
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    post,
    path = "/api/v1/inventory/valuation/{product_id}/revalue",
    tag = "valuation",
    operation_id = "revalue_inventory",
    params(
        ("product_id" = Uuid, Path, description = "Product ID")
    ),
    request_body = RevaluationPayload,
    responses(
        (status = 200, description = "Updated valuation data", body = ValuationDto),
        (status = 400, description = "Invalid cost", body = ErrorResponse),
        (status = 404, description = "Product not found", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn revalue_inventory(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<RevaluationPayload>,
) -> Result<Json<ValuationDto>, AppError> {
    let request = RevaluationRequest {
        tenant_id: auth_user.tenant_id,
        product_id,
        new_unit_cost: payload.new_unit_cost,
        reason: payload.reason,
    };

    let valuation = state.valuation_service.revalue_inventory(request).await?;

    Ok(Json(valuation))
}

// ============================================
// Valuation Settings Handlers
// ============================================

/// GET /api/v1/inventory/valuation/settings - List all valuation settings for tenant
#[utoipa::path(
    get,
    path = "/api/v1/inventory/valuation/settings",
    tag = "valuation",
    operation_id = "list_valuation_settings",
    params(ListSettingsQueryParams),
    responses(
        (status = 200, description = "List of valuation settings", body = ValuationSettingsListResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_valuation_settings(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(params): Query<ListSettingsQueryParams>,
) -> Result<Json<ValuationSettingsListResponse>, AppError> {
    let request = ListValuationSettingsRequest {
        tenant_id: auth_user.tenant_id,
        scope_type: params.scope_type,
    };

    let response = state
        .valuation_service
        .list_valuation_settings(request)
        .await?;
    Ok(Json(response))
}

/// GET /api/v1/inventory/valuation/settings/tenant - Get tenant default settings
#[utoipa::path(
    get,
    path = "/api/v1/inventory/valuation/settings/tenant",
    tag = "valuation",
    operation_id = "get_tenant_valuation_settings",
    responses(
        (status = 200, description = "Tenant default valuation settings", body = ValuationSettingsDto),
        (status = 404, description = "No tenant default configured", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_tenant_settings(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
) -> Result<Json<ValuationSettingsDto>, AppError> {
    let request = GetTenantValuationSettingsRequest {
        tenant_id: auth_user.tenant_id,
    };

    let settings = state
        .valuation_service
        .get_tenant_valuation_settings(request)
        .await?;
    Ok(Json(settings))
}

/// PUT /api/v1/inventory/valuation/settings/tenant - Set tenant default method
#[utoipa::path(
    put,
    path = "/api/v1/inventory/valuation/settings/tenant",
    tag = "valuation",
    operation_id = "set_tenant_valuation_method",
    request_body = SetMethodPayload,
    responses(
        (status = 200, description = "Updated tenant valuation settings", body = ValuationSettingsDto),
        (status = 400, description = "Invalid valuation method", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn set_tenant_method(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(payload): Json<SetMethodPayload>,
) -> Result<Json<ValuationSettingsDto>, AppError> {
    let request = SetTenantValuationMethodRequest {
        tenant_id: auth_user.tenant_id,
        method: payload.method,
    };

    let settings = state
        .valuation_service
        .set_tenant_valuation_method(request)
        .await?;
    Ok(Json(settings))
}

/// GET /api/v1/inventory/valuation/settings/effective/{product_id} - Get effective method for product
#[utoipa::path(
    get,
    path = "/api/v1/inventory/valuation/settings/effective/{product_id}",
    tag = "valuation",
    operation_id = "get_effective_valuation_method",
    params(
        ("product_id" = Uuid, Path, description = "Product ID"),
        EffectiveMethodQueryParams
    ),
    responses(
        (status = 200, description = "Effective valuation method with source", body = EffectiveValuationMethodResponse),
        (status = 404, description = "No valuation settings configured", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_effective_method(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
    Query(params): Query<EffectiveMethodQueryParams>,
) -> Result<Json<EffectiveValuationMethodResponse>, AppError> {
    let request = GetEffectiveValuationMethodRequest {
        tenant_id: auth_user.tenant_id,
        product_id,
        category_id: params.category_id,
    };

    let response = state
        .valuation_service
        .get_effective_valuation_method(request)
        .await?;
    Ok(Json(response))
}

/// PUT /api/v1/inventory/valuation/settings/category/{category_id} - Set category override
#[utoipa::path(
    put,
    path = "/api/v1/inventory/valuation/settings/category/{category_id}",
    tag = "valuation",
    operation_id = "set_category_valuation_method",
    params(("category_id" = Uuid, Path, description = "Category ID")),
    request_body = SetMethodPayload,
    responses(
        (status = 200, description = "Category valuation settings", body = ValuationSettingsDto),
        (status = 400, description = "Invalid valuation method", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn set_category_method(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(category_id): Path<Uuid>,
    Json(payload): Json<SetMethodPayload>,
) -> Result<Json<ValuationSettingsDto>, AppError> {
    let request = SetCategoryValuationMethodRequest {
        tenant_id: auth_user.tenant_id,
        category_id,
        method: payload.method,
    };

    let settings = state
        .valuation_service
        .set_category_valuation_method(request)
        .await?;
    Ok(Json(settings))
}

/// DELETE /api/v1/inventory/valuation/settings/category/{category_id} - Delete category override
#[utoipa::path(
    delete,
    path = "/api/v1/inventory/valuation/settings/category/{category_id}",
    tag = "valuation",
    operation_id = "delete_category_valuation_settings",
    params(("category_id" = Uuid, Path, description = "Category ID")),
    responses(
        (status = 204, description = "Category settings deleted"),
        (status = 404, description = "Settings not found", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_category_settings(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let request = DeleteValuationSettingsRequest {
        tenant_id: auth_user.tenant_id,
        scope_type: ValuationScopeType::Category,
        scope_id: Some(category_id),
    };

    state
        .valuation_service
        .delete_valuation_settings(request)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// PUT /api/v1/inventory/valuation/settings/product/{product_id} - Set product override
#[utoipa::path(
    put,
    path = "/api/v1/inventory/valuation/settings/product/{product_id}",
    tag = "valuation",
    operation_id = "set_product_valuation_method",
    params(("product_id" = Uuid, Path, description = "Product ID")),
    request_body = SetMethodPayload,
    responses(
        (status = 200, description = "Product valuation settings", body = ValuationSettingsDto),
        (status = 400, description = "Invalid valuation method", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn set_product_method(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<SetMethodPayload>,
) -> Result<Json<ValuationSettingsDto>, AppError> {
    let request = SetProductValuationMethodRequest {
        tenant_id: auth_user.tenant_id,
        product_id,
        method: payload.method,
    };

    let settings = state
        .valuation_service
        .set_product_valuation_method(request)
        .await?;
    Ok(Json(settings))
}

/// DELETE /api/v1/inventory/valuation/settings/product/{product_id} - Delete product override
#[utoipa::path(
    delete,
    path = "/api/v1/inventory/valuation/settings/product/{product_id}",
    tag = "valuation",
    operation_id = "delete_product_valuation_settings",
    params(("product_id" = Uuid, Path, description = "Product ID")),
    responses(
        (status = 204, description = "Product settings deleted"),
        (status = 404, description = "Settings not found", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_product_settings(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let request = DeleteValuationSettingsRequest {
        tenant_id: auth_user.tenant_id,
        scope_type: ValuationScopeType::Product,
        scope_id: Some(product_id),
    };

    state
        .valuation_service
        .delete_valuation_settings(request)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// Payload structures for request bodies

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct SetValuationMethodPayload {
    pub valuation_method: ValuationMethod,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct SetStandardCostPayload {
    pub standard_cost: i64,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CostAdjustmentPayload {
    pub adjustment_amount: i64,
    pub reason: String,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct RevaluationPayload {
    pub new_unit_cost: i64,
    pub reason: String,
}

// Query parameters

#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct HistoryQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct ListSettingsQueryParams {
    /// Filter by scope type (tenant, category, product)
    pub scope_type: Option<ValuationScopeType>,
}

#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct EffectiveMethodQueryParams {
    /// Category ID for hierarchical lookup
    pub category_id: Option<Uuid>,
}

/// Payload for setting valuation method
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct SetMethodPayload {
    pub method: ValuationMethod,
}
