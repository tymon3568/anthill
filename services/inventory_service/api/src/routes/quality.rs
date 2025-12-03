use axum::routing::{delete, get, post, put, Router};

use crate::handlers::quality;

/// Create quality management routes
pub fn create_quality_routes() -> Router {
    Router::new()
        .route("/points", post(quality::create_qc_point).get(quality::list_qc_points))
        .route(
            "/points/:qc_point_id",
            get(quality::get_qc_point)
                .put(quality::update_qc_point)
                .delete(quality::delete_qc_point),
        )
}
