//! Repository implementations
//!
//! This module contains PostgreSQL implementations of the repository traits.

pub mod category;
pub mod product;
pub mod receipt;
pub mod valuation;
pub mod warehouse;

// Re-export repositories for convenience
pub use category::CategoryRepositoryImpl;
pub use product::ProductRepositoryImpl;
pub use receipt::{OutboxRepositoryImpl, ReceiptRepositoryImpl, StockMoveRepositoryImpl};
pub use valuation::ValuationRepositoryImpl;
pub use warehouse::WarehouseRepositoryImpl;
