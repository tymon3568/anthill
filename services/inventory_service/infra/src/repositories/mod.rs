//! Repository implementations
//!
//! This module contains PostgreSQL implementations of the repository traits.

pub mod category;
pub mod delivery_order;
pub mod product;
pub mod receipt;
pub mod reconciliation;
pub mod stock;
pub mod stock_take;
pub mod transfer;
pub mod valuation;
pub mod warehouse;

// Re-export repositories for convenience
pub use category::CategoryRepositoryImpl;
pub use delivery_order::{
    PgDeliveryOrderItemRepository, PgDeliveryOrderRepository, PgInventoryRepository,
};
pub use product::ProductRepositoryImpl;
pub use receipt::ReceiptRepositoryImpl;
pub use reconciliation::{PgStockReconciliationItemRepository, PgStockReconciliationRepository};
pub use stock::{PgInventoryLevelRepository, PgStockMoveRepository};
pub use stock_take::{PgStockTakeLineRepository, PgStockTakeRepository};
pub use transfer::{PgTransferItemRepository, PgTransferRepository};
pub use valuation::ValuationRepositoryImpl;
pub use warehouse::WarehouseRepositoryImpl;
