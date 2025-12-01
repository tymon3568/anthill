//! Service trait definitions for inventory service
//!
//! This module contains trait definitions for business logic operations.
//! No implementations here - pure interfaces.

pub mod category;
pub mod delivery;
pub mod lot_serial;
pub mod product;
pub mod receipt;
pub mod reconciliation;
pub mod rma;
pub mod stock_take;
pub mod transfer;
pub mod valuation;

// Re-export main types for convenience
pub use category::CategoryService;
// pub use delivery::DeliveryService;
pub use lot_serial::LotSerialService;
pub use product::ProductService;
pub use receipt::ReceiptService;
pub use reconciliation::StockReconciliationService;
pub use rma::RmaService;
pub use stock_take::StockTakeService;
pub use transfer::TransferService;
pub use valuation::ValuationService;
