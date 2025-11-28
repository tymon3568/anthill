//! Service implementations
//!
//! This module contains implementations of the service traits with business logic.

pub mod category;
pub mod delivery;
pub mod product;
pub mod receipt;
pub mod reconciliation;
pub mod rma;
pub mod stock_take;
pub mod transfer;
pub mod valuation;

#[cfg(test)]
mod category_tests;

// Re-export services for convenience
pub use category::CategoryServiceImpl;
// pub use delivery::DeliveryServiceImpl;
pub use product::ProductServiceImpl;
pub use receipt::ReceiptServiceImpl;
pub use reconciliation::PgStockReconciliationService;
pub use rma::PgRmaService;
pub use stock_take::PgStockTakeService;
pub use transfer::PgTransferService;
pub use valuation::ValuationServiceImpl;
