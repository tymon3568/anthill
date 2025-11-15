//! Repository trait definitions for inventory service
//!
//! This module contains trait definitions for data access operations.
//! No implementations here - pure interfaces.

pub mod category;
pub mod product;
pub mod valuation;
pub mod warehouse;

// Re-export main types for convenience
pub use category::CategoryRepository;
pub use product::ProductRepository;
pub use valuation::{ValuationHistoryRepository, ValuationLayerRepository, ValuationRepository};
pub use warehouse::WarehouseRepository;
