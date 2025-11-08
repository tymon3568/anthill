//! Inventory Service API
//!
//! This crate contains the HTTP API handlers and routing for the inventory service.
//! It provides REST endpoints for category management.
//!
//! ## Architecture
//!
//! - `handlers/`: Axum HTTP handlers
//! `routes/`: Route definitions and middleware
//! `middleware/`: Custom middleware
//! `models/`: API-specific models and conversions

pub mod handlers;
pub mod routes;
pub mod middleware;
pub mod models;

// Re-export main components for convenience
pub use routes::create_router;
pub use handlers::category::CategoryHandler;
