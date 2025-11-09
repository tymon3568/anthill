//! Inventory Service API
//!
//! This crate contains the HTTP API handlers and routing for the inventory service.
//! It provides REST endpoints for category management.
//!
//! ## Architecture
//!
//! - `handlers/`: Axum HTTP handlers
//! - `routes/`: Route definitions and middleware
//! - `middleware/`: Custom middleware
//! - `models/`: API-specific models and conversions

pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;

// Re-export main components for convenience
pub use routes::create_router;

use axum::Router;
use shared_config::Config;
use shared_db::init_pool;

/// Create the complete application with database initialization
/// Used for integration tests
pub async fn create_app(config: Config) -> Router {
    let pool = init_pool(&config.database_url, config.max_connections.unwrap_or(10))
        .await
        .expect("Failed to initialize database pool");
    create_router(pool, &config).await
}
