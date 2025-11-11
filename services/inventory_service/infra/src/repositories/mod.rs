//! Repository implementations
//!
//! This module contains PostgreSQL implementations of the repository traits.

pub mod category;
pub mod product;
pub mod warehouse;

// Re-export repositories for convenience
pub use category::CategoryRepositoryImpl;
pub use product::ProductRepositoryImpl;
pub use warehouse::WarehouseRepositoryImpl;
