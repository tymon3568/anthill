//! Repository implementations
//!
//! This module contains PostgreSQL implementations of the repository traits.

pub mod category;
pub mod delivery_order;
pub mod product;
pub mod receipt;
pub mod valuation;
pub mod warehouse;

// Re-export repository traits for convenience
pub use category::CategoryRepository;
pub use delivery_order::{
    DeliveryOrderItemRepository, DeliveryOrderRepository, InventoryRepository,
};
pub use product::ProductRepository;
pub use receipt::ReceiptRepository;
pub use valuation::ValuationRepository;
pub use warehouse::WarehouseRepository;
