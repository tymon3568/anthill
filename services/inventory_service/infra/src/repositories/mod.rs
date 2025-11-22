//! Repository implementations
//!
//! This module contains PostgreSQL implementations of the repository traits.

pub mod category;
pub mod delivery_order;
pub mod product;
pub mod receipt;
pub mod stock;
pub mod valuation;
pub mod warehouse;

// Re-export repositories for convenience
pub use category::CategoryRepositoryImpl;
pub use delivery_order::{
    PgDeliveryOrderItemRepository, PgDeliveryOrderRepository, PgInventoryRepository,
};
pub use product::ProductRepositoryImpl;
pub use receipt::ReceiptRepositoryImpl;
pub use stock::{PgInventoryLevelRepository, PgStockMoveRepository};
pub use valuation::ValuationRepositoryImpl;
pub use warehouse::WarehouseRepositoryImpl;
