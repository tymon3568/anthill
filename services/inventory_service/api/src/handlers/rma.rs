//! RMA handlers
//!
//! This module contains HTTP handlers for RMA operations.

use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use uuid::Uuid;

use inventory_service_core::dto::rma::{
    ApproveRmaRequest, ApproveRmaResponse, CreateRmaRequest, CreateRmaResponse, ReceiveRmaRequest,
    ReceiveRmaResponse,
};
use shared_auth::extractors::AuthUser;
use shared_error::AppError;

use crate::state::AppState;

/// POST /api/v1/inventory/rma - Create a new RMA request
#[utoipa::path(
    post,
    path = "/api/v1/inventory/rma",
    tag = "rma",
    operation_id = "create_rma",
    request_body = CreateRmaRequest,
    responses(
        (status = 201, description = "RMA created successfully", body = CreateRmaResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn create_rma(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateRmaRequest>,
) -> Result<Json<CreateRmaResponse>, AppError> {
    let response = state
        .rma_service
        .create_rma(auth_user.tenant_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/rma/{rma_id}/approve - Approve or reject an RMA request
#[utoipa::path(
    post,
    path = "/api/v1/inventory/rma/{rma_id}/approve",
    tag = "rma",
    operation_id = "approve_rma",
    params(
        ("rma_id" = Uuid, Path, description = "RMA ID")
    ),
    request_body = ApproveRmaRequest,
    responses(
        (status = 200, description = "RMA approved/rejected successfully", body = ApproveRmaResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "RMA not found")
    )
)]
pub async fn approve_rma(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(rma_id): Path<Uuid>,
    Json(request): Json<ApproveRmaRequest>,
) -> Result<Json<ApproveRmaResponse>, AppError> {
    let response = state
        .rma_service
        .approve_rma(auth_user.tenant_id, rma_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}

/// Create RMA routes
pub fn create_rma_routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_rma))
        .route("/:rma_id/approve", post(approve_rma))
        .route("/:rma_id/receive", post(receive_rma))
        .with_state(state)
}

/// POST /api/v1/inventory/rma/{rma_id}/receive - Process the receipt of returned goods
#[utoipa::path(
    post,
    path = "/api/v1/inventory/rma/{rma_id}/receive",
    tag = "rma",
    operation_id = "receive_rma",
    params(
        ("rma_id" = Uuid, Path, description = "RMA ID")
    ),
    request_body = ReceiveRmaRequest,
    responses(
        (status = 200, description = "RMA received successfully", body = ReceiveRmaResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "RMA not found")
    )
)]
pub async fn receive_rma(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(rma_id): Path<Uuid>,
    Json(request): Json<ReceiveRmaRequest>,
) -> Result<Json<ReceiveRmaResponse>, AppError> {
    let response = state
        .rma_service
        .receive_rma(auth_user.tenant_id, rma_id, auth_user.user_id, request)
        .await?;

    Ok(Json(response))
}
