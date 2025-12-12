use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use inventory_service_core::models::{
    LotSerial, LotSerialLifecycle, LotSerialStatus, LotSerialTrackingType,
};

use shared_auth::extractors::{AuthUser, RequireAdmin};
use shared_error::AppError;

use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLotSerialRequest {
    pub product_id: Uuid,
    pub tracking_type: LotSerialTrackingType,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub initial_quantity: Option<i64>,
    pub remaining_quantity: Option<i64>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub status: LotSerialStatus,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListLotSerialsQuery {
    pub tracking_type: Option<LotSerialTrackingType>,
    pub status: Option<LotSerialStatus>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QuarantineResponse {
    pub quarantined_count: i64,
}

/// Error response for OpenAPI documentation
#[derive(utoipa::ToSchema)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/lot-serials",
    tag = "lot-serial",
    operation_id = "create_lot_serial",
    request_body = CreateLotSerialRequest,
    responses(
        (status = 201, description = "Lot serial created successfully"),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_lot_serial(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<CreateLotSerialRequest>,
) -> Result<StatusCode, AppError> {
    let now = Utc::now();
    let lot_serial = LotSerial {
        lot_serial_id: Uuid::now_v7(),
        tenant_id: auth_user.tenant_id,
        product_id: request.product_id,
        tracking_type: request.tracking_type,
        lot_number: request.lot_number,
        serial_number: request.serial_number,
        initial_quantity: request.initial_quantity,
        remaining_quantity: request.remaining_quantity,
        expiry_date: request.expiry_date,
        status: request.status,
        warehouse_id: None,
        location_id: None,
        created_by: auth_user.kanidm_user_id.unwrap_or(auth_user.user_id),
        updated_by: None,
        created_at: now,
        updated_at: now,
        deleted_at: None,
    };

    state
        .lot_serial_service
        .create_lot_serial(&lot_serial)
        .await?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/lot-serials/{lot_serial_id}",
    tag = "lot-serial",
    operation_id = "get_lot_serial",
    params(
        ("lot_serial_id" = Uuid, Path, description = "Lot serial ID")
    ),
    responses(
        (status = 200, body = Option<LotSerial>),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_lot_serial(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(lot_serial_id): Path<Uuid>,
) -> Result<Json<Option<LotSerial>>, AppError> {
    let lot_serial = state
        .lot_serial_service
        .get_lot_serial(auth_user.tenant_id, lot_serial_id)
        .await?;
    Ok(Json(lot_serial))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/lot-serials/products/{product_id}",
    tag = "lot-serial",
    operation_id = "list_lot_serials_by_product",
    params(
        ("product_id" = Uuid, Path, description = "Product ID"),
        ListLotSerialsQuery
    ),
    responses(
        (status = 200, body = Vec<LotSerial>),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_lot_serials_by_product(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
    Query(query): Query<ListLotSerialsQuery>,
) -> Result<Json<Vec<LotSerial>>, AppError> {
    let lot_serials = state
        .lot_serial_service
        .list_lot_serials_by_product(
            auth_user.tenant_id,
            product_id,
            query.tracking_type,
            query.status,
        )
        .await?;
    Ok(Json(lot_serials))
}

#[utoipa::path(
    put,
    path = "/api/v1/inventory/lot-serials/{lot_serial_id}",
    tag = "lot-serial",
    operation_id = "update_lot_serial",
    params(
        ("lot_serial_id" = Uuid, Path, description = "Lot serial ID")
    ),
    request_body = LotSerial,
    responses(
        (status = 200, description = "Lot serial updated successfully"),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_lot_serial(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(lot_serial_id): Path<Uuid>,
    Json(lot_serial): Json<LotSerial>,
) -> Result<StatusCode, AppError> {
    // Validate path ID matches body ID
    if lot_serial_id != lot_serial.lot_serial_id {
        return Err(AppError::ValidationError("Path ID does not match body ID".to_string()));
    }

    // Validate tenant_id matches auth_user
    if lot_serial.tenant_id != auth_user.tenant_id {
        return Err(AppError::ValidationError("Tenant ID mismatch".to_string()));
    }

    state
        .lot_serial_service
        .update_lot_serial(&lot_serial)
        .await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/api/v1/inventory/lot-serials/{lot_serial_id}",
    tag = "lot-serial",
    operation_id = "delete_lot_serial",
    params(
        ("lot_serial_id" = Uuid, Path, description = "Lot serial ID")
    ),
    responses(
        (status = 204, description = "Lot serial deleted successfully"),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_lot_serial(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(lot_serial_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .lot_serial_service
        .delete_lot_serial(auth_user.tenant_id, lot_serial_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/lot-serials/quarantine-expired",
    tag = "lot-serial",
    operation_id = "quarantine_expired_lots",
    responses(
        (status = 200, body = QuarantineResponse),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn quarantine_expired_lots(
    RequireAdmin(auth_user): RequireAdmin,
    Extension(state): Extension<AppState>,
) -> Result<Json<QuarantineResponse>, AppError> {
    let quarantined_count = state
        .lot_serial_service
        .quarantine_expired_lots(auth_user.tenant_id)
        .await?;
    Ok(Json(QuarantineResponse { quarantined_count }))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/lot-serials/tracking/{lot_serial_id}",
    tag = "lot-serial",
    operation_id = "get_lot_serial_lifecycle",
    params(
        ("lot_serial_id" = Uuid, Path, description = "Lot serial ID")
    ),
    responses(
        (status = 200, body = LotSerialLifecycle),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_lot_serial_lifecycle(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(lot_serial_id): Path<Uuid>,
) -> Result<Json<LotSerialLifecycle>, AppError> {
    let lifecycle = state
        .lot_serial_service
        .get_lifecycle(auth_user.tenant_id, lot_serial_id)
        .await?;
    Ok(Json(lifecycle))
}

pub fn create_lot_serial_routes() -> Router {
    Router::new()
        .route("/", axum::routing::post(create_lot_serial))
        .route(
            "/{lot_serial_id}",
            axum::routing::get(get_lot_serial)
                .put(update_lot_serial)
                .delete(delete_lot_serial),
        )
        .route("/products/{product_id}", axum::routing::get(list_lot_serials_by_product))
        .route("/quarantine-expired", axum::routing::post(quarantine_expired_lots))
        .route("/tracking/{lot_serial_id}", axum::routing::get(get_lot_serial_lifecycle))
}
