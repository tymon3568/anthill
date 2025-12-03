use axum::{
    routing::{delete, get, post, put, Router},
    extract::State,
};
use inventory_service_infra::services::quality::PgQualityControlPointService;
use inventory_service_infra::repositories::quality::PgQualityControlPointRepository;
use std::sync::Arc;

use crate::handlers::quality;
use crate::state::AppState;

/// Create quality management routes
pub fn create_quality_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/points", post(quality::create_qc_point))
        .route("/points", get(quality::list_qc_points))
        .route("/points/:qc_point_id", get(quality::get_qc_point))
        .route("/points/:qc_point_id", put(quality::update_qc_point))
        .route("/points/:qc_point_id", delete(quality::delete_qc_point))
        .with_state(Arc::new(state.clone()))
}
