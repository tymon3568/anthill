use axum::{
    extract::{Path, Query, State},
    Json,
};
use inventory_service_core::domains::quality::{
    CreateQualityControlPoint, QualityControlPoint, UpdateQualityControlPoint,
};
use inventory_service_core::AppError;
use serde::Deserialize;
use shared_auth::extractors::AuthUser;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListQcPointsQuery {
    pub product_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub active_only: Option<bool>,
}

/// Create a new quality control point
#[utoipa::path(
    post,
    path = "/api/v1/inventory/quality/points",
    tag = "quality",
    operation_id = "create_qc_point",
    request_body = CreateQualityControlPoint,
    responses(
        (status = 201, body = QualityControlPoint),
        (status = 400, body = shared_error::ErrorResponse),
        (status = 401, body = shared_error::ErrorResponse),
        (status = 500, body = shared_error::ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_qc_point(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(qc_point): Json<CreateQualityControlPoint>,
) -> Result<Json<QualityControlPoint>, AppError> {
    let created = state
        .quality_service
        .create_qc_point(user.tenant_id, qc_point)
        .await?;
    Ok(Json(created))
}

/// Get a quality control point by ID
#[utoipa::path(
    get,
    path = "/api/v1/inventory/quality/points/{qc_point_id}",
    tag = "quality",
    operation_id = "get_qc_point",
    params(
        ("qc_point_id" = Uuid, Path, description = "Quality control point ID")
    ),
    responses(
        (status = 200, body = QualityControlPoint),
        (status = 404, body = shared_error::ErrorResponse),
        (status = 401, body = shared_error::ErrorResponse),
        (status = 500, body = shared_error::ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_qc_point(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(qc_point_id): Path<Uuid>,
) -> Result<Json<QualityControlPoint>, AppError> {
    let qc_point = state
        .quality_service
        .get_qc_point(user.tenant_id, qc_point_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Quality control point {} not found", qc_point_id))
        })?;
    Ok(Json(qc_point))
}

/// List quality control points
#[utoipa::path(
    get,
    path = "/api/v1/inventory/quality/points",
    tag = "quality",
    operation_id = "list_qc_points",
    params(
        ListQcPointsQuery
    ),
    responses(
        (status = 200, body = Vec<QualityControlPoint>),
        (status = 401, body = shared_error::ErrorResponse),
        (status = 500, body = shared_error::ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_qc_points(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(query): Query<ListQcPointsQuery>,
) -> Result<Json<Vec<QualityControlPoint>>, AppError> {
    let qc_points = if query.active_only.unwrap_or(true) {
        state
            .quality_service
            .list_active_qc_points(user.tenant_id)
            .await?
    } else {
        state.quality_service.list_qc_points(user.tenant_id).await?
    };
    Ok(Json(qc_points))
}

/// Update a quality control point
#[utoipa::path(
    put,
    path = "/api/v1/inventory/quality/points/{qc_point_id}",
    tag = "quality",
    operation_id = "update_qc_point",
    params(
        ("qc_point_id" = Uuid, Path, description = "Quality control point ID")
    ),
    request_body = UpdateQualityControlPoint,
    responses(
        (status = 200, body = QualityControlPoint),
        (status = 404, body = shared_error::ErrorResponse),
        (status = 400, body = shared_error::ErrorResponse),
        (status = 401, body = shared_error::ErrorResponse),
        (status = 500, body = shared_error::ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_qc_point(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(qc_point_id): Path<Uuid>,
    Json(updates): Json<UpdateQualityControlPoint>,
) -> Result<Json<QualityControlPoint>, AppError> {
    let updated = state
        .quality_service
        .update_qc_point(user.tenant_id, qc_point_id, updates)
        .await?;
    Ok(Json(updated))
}

/// Delete (deactivate) a quality control point
#[utoipa::path(
    delete,
    path = "/api/v1/inventory/quality/points/{qc_point_id}",
    tag = "quality",
    operation_id = "delete_qc_point",
    params(
        ("qc_point_id" = Uuid, Path, description = "Quality control point ID")
    ),
    responses(
        (status = 204, description = "Quality control point deactivated"),
        (status = 404, body = shared_error::ErrorResponse),
        (status = 401, body = shared_error::ErrorResponse),
        (status = 500, body = shared_error::ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_qc_point(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(qc_point_id): Path<Uuid>,
) -> Result<(), AppError> {
    state
        .quality_service
        .delete_qc_point(user.tenant_id, qc_point_id)
        .await?;
    Ok(())
}
