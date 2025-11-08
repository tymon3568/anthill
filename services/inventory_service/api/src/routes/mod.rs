//! Route definitions and router creation
//!
//! This module defines the API routes and creates the main router.

use axum::{middleware, Router};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;

use crate::middleware::auth::auth_middleware;
use crate::handlers::CategoryHandler;

/// Create the main application router
pub fn create_router(pool: PgPool) -> Router {
    // Create category handler
    let category_handler = CategoryHandler::new(pool.clone());

    // Define routes
    let api_routes = Router::new()
        .nest("/categories", category_handler.routes())
        .layer(middleware::from_fn(auth_middleware));

    // Add CORS
    let cors = CorsLayer::permissive();

    Router::new()
        .nest("/api/v1/inventory", api_routes)
        .layer(cors)
}
