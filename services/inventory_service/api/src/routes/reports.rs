use axum::routing::get;
use axum::Router;

use crate::handlers::reports;

/// Create reports routes
pub fn create_reports_routes() -> Router {
    Router::new().route("/stock-ledger", get(reports::get_stock_ledger))
}
