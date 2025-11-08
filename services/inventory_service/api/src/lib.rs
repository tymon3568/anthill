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
pub mod middleware;
pub mod models;
pub mod routes;

// Re-export main components for convenience
pub use handlers::category::CategoryHandler;
pub use routes::create_router;
