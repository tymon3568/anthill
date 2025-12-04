use axum::routing::get;
use axum::Router;

use crate::handlers::reports;

/// Create reports routes
pub fn create_reports_routes() -> Router {
    Router::new()
        .route("/stock-ledger", get(reports::get_stock_ledger))
        .route("/aging", get(reports::get_stock_aging))
        .route("/turnover", get(reports::get_inventory_turnover))
        .route("/low-stock", get(reports::get_low_stock))
        .route("/dead-stock", get(reports::get_dead_stock))
}
