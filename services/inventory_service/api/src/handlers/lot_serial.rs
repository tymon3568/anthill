use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use inventory_service_core::models::{LotSerial, LotSerialStatus, LotSerialTrackingType};

use shared_auth::extractors::{AuthUser, RequireAdmin};
use shared_error::AppError;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListLotSerialsQuery {
    pub tracking_type: Option<LotSerialTrackingType>,
    pub status: Option<LotSerialStatus>,
}

#[derive(Debug, Serialize)]
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
    request_body = LotSerial,
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
    Json(lot_serial): Json<LotSerial>,
) -> Result<StatusCode, AppError> {
    // Validate tenant_id matches auth_user
    if lot_serial.tenant_id != auth_user.tenant_id {
        return Err(AppError::ValidationError("Tenant ID mismatch".to_string()));
    }

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
    Json(lot_serial): Json<LotSerial>,
) -> Result<StatusCode, AppError> {
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
}
