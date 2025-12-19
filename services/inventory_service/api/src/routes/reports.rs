use axum::routing::get;
use axum::Router;

use crate::handlers::reports::*;

/// Create reports routes
#[allow(dead_code)]
pub fn create_reports_routes() -> Router {
    Router::new()
        .route("/stock-ledger", get(get_stock_ledger))
        .route("/aging", get(get_stock_aging))
        .route("/turnover", get(get_inventory_turnover))
        .route("/low-stock", get(get_low_stock))
        .route("/dead-stock", get(get_dead_stock))
}
