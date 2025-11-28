//! Inventory Service Infrastructure
//!
//! This crate contains the infrastructure implementations for the inventory service.
//! It provides concrete implementations of repositories and services.
//!
//! ## Architecture
//!
//! - `repositories/`: PostgreSQL repository implementations
//! - `services/`: Service implementations with business logic

pub mod repositories;
pub mod services;

/// Helper type for infra-internal transaction operations
pub type InfraTx<'a> = &'a mut sqlx::Transaction<'a, sqlx::Postgres>;

// Re-export main implementations for convenience
pub use repositories::category::CategoryRepositoryImpl;
pub use repositories::product::ProductRepositoryImpl;
pub use repositories::valuation::ValuationRepositoryImpl;
pub use services::category::CategoryServiceImpl;
pub use services::product::ProductServiceImpl;
pub use services::valuation::ValuationServiceImpl;
