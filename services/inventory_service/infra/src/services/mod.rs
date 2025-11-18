//! Service implementations
//!
//! This module contains implementations of the service traits with business logic.

pub mod category;
pub mod product;
pub mod receipt;
pub mod valuation;

#[cfg(test)]
mod category_tests;

// Re-export services for convenience
pub use category::CategoryServiceImpl;
pub use product::ProductServiceImpl;
pub use receipt::ReceiptServiceImpl;
pub use valuation::ValuationServiceImpl;
