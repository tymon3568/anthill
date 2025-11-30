//! Inventory valuation HTTP handlers
//!
//! This module contains the Axum handlers for inventory valuation endpoints.

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post, put},
    Router,
};
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::valuation_dto::{
    CostAdjustmentRequest, GetValuationHistoryRequest, GetValuationLayersRequest,
    GetValuationRequest, RevaluationRequest, SetStandardCostRequest, SetValuationMethodRequest,
    ValuationDto, ValuationHistoryResponse, ValuationLayersResponse,
};
use inventory_service_core::domains::inventory::valuation::ValuationMethod;

use crate::state::AppState;
use shared_auth::extractors::AuthUser;
use shared_error::AppError;

/// Create the valuation routes
pub fn create_valuation_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_valuation))
        .route("/method", put(set_valuation_method))
        .route("/standard-cost", put(set_standard_cost))
        .route("/layers", get(get_valuation_layers))
        .route("/history", get(get_valuation_history))
        .route("/adjust", post(adjust_cost))
        .route("/revalue", post(revalue_inventory))
        .with_state(state)
}

/// GET /api/v1/inventory/valuation - Get current valuation for a product
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
pub async fn get_valuation(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<ValuationDto>, AppError> {
    let request = GetValuationRequest {
        tenant_id: auth_user.tenant_id,
        product_id,
    };

    let valuation = state.valuation_service.get_valuation(request).await?;

    Ok(Json(valuation))
}

/// PUT /api/v1/inventory/valuation/method - Set valuation method for a product
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
pub async fn set_valuation_method(
    auth_user: AuthUser,
    State(state): State<AppState>,
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

/// PUT /api/v1/inventory/valuation/standard-cost - Set standard cost for a product
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
pub async fn set_standard_cost(
    auth_user: AuthUser,
    State(state): State<AppState>,
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

/// GET /api/v1/inventory/valuation/layers - Get valuation layers for FIFO
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
pub async fn get_valuation_layers(
    auth_user: AuthUser,
    State(state): State<AppState>,
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

/// GET /api/v1/inventory/valuation/history - Get valuation history
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
pub async fn get_valuation_history(
    auth_user: AuthUser,
    State(state): State<AppState>,
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

/// POST /api/v1/inventory/valuation/adjust - Adjust inventory cost
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
pub async fn adjust_cost(
    auth_user: AuthUser,
    State(state): State<AppState>,
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

/// POST /api/v1/inventory/valuation/revalue - Revalue inventory
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
pub async fn revalue_inventory(
    auth_user: AuthUser,
    State(state): State<AppState>,
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

// Payload structures for request bodies

#[derive(serde::Deserialize)]
pub struct SetValuationMethodPayload {
    pub valuation_method: ValuationMethod,
}

#[derive(serde::Deserialize)]
pub struct SetStandardCostPayload {
    pub standard_cost: i64,
}

#[derive(serde::Deserialize)]
pub struct CostAdjustmentPayload {
    pub adjustment_amount: i64,
    pub reason: String,
}

#[derive(serde::Deserialize)]
pub struct RevaluationPayload {
    pub new_unit_cost: i64,
    pub reason: String,
}

// Query parameters

#[derive(serde::Deserialize)]
pub struct HistoryQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
