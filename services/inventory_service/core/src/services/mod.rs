//! Service trait definitions for inventory service
//!
//! This module contains trait definitions for business logic operations.
//! No implementations here - pure interfaces.

pub mod category;
pub mod delivery;
pub mod product;
pub mod receipt;
pub mod transfer;
pub mod valuation;

// Re-export main types for convenience
pub use category::CategoryService;
pub use delivery::DeliveryService;
pub use product::ProductService;
pub use receipt::ReceiptService;
pub use transfer::TransferService;
pub use valuation::ValuationService;
