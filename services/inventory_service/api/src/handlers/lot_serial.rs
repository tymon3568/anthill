use axum::{
    extract::{Path, Query, State},
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

pub async fn create_lot_serial(
    auth_user: AuthUser,
    State(state): State<AppState>,
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

pub async fn get_lot_serial(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(lot_serial_id): Path<Uuid>,
) -> Result<Json<Option<LotSerial>>, AppError> {
    let lot_serial = state
        .lot_serial_service
        .get_lot_serial(auth_user.tenant_id, lot_serial_id)
        .await?;
    Ok(Json(lot_serial))
}

pub async fn list_lot_serials_by_product(
    auth_user: AuthUser,
    State(state): State<AppState>,
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

pub async fn update_lot_serial(
    auth_user: AuthUser,
    State(state): State<AppState>,
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

pub async fn delete_lot_serial(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(lot_serial_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .lot_serial_service
        .delete_lot_serial(auth_user.tenant_id, lot_serial_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn quarantine_expired_lots(
    RequireAdmin(auth_user): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<QuarantineResponse>, AppError> {
    let quarantined_count = state
        .lot_serial_service
        .quarantine_expired_lots(auth_user.tenant_id)
        .await?;
    Ok(Json(QuarantineResponse { quarantined_count }))
}

pub fn create_lot_serial_routes(state: AppState) -> Router<AppState> {
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
        .with_state(state)
}
