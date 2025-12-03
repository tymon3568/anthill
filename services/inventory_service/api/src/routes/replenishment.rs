use axum::routing::{delete, get, post, put, Router};
use axum::extract::State;

use crate::handlers::replenishment::*;
use crate::state::AppState;

/// Create replenishment routes
pub fn create_replenishment_routes() -> Router<AppState> {
    Router::new()
        .route("/rules", post(create_reorder_rule))
        .route("/rules/:rule_id", get(get_reorder_rule).put(update_reorder_rule).delete(delete_reorder_rule))
        .route("/rules/product/:product_id", get(list_reorder_rules_for_product))
        .route("/check", post(run_replenishment_check))
        .route("/check/product/:product_id", post(check_product_replenishment))
}
