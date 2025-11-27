//! Repository implementations
//!
//! This module contains PostgreSQL implementations of the repository traits.

pub mod category;

/// Transaction abstraction to avoid sqlx dependency in core
pub mod transaction {
    use async_trait::async_trait;
    use shared_error::AppError;

    /// Abstract transaction trait for database operations
    #[async_trait]
    pub trait Transaction: Send + Sync {
        /// Commit the transaction
        async fn commit(self) -> Result<(), AppError>;
    }
}
pub mod delivery_order;
pub mod product;
pub mod receipt;
pub mod reconciliation;
pub mod stock;
pub mod stock_take;
pub mod transfer;
pub mod valuation;
pub mod warehouse;

// Re-export repository traits for convenience
pub use category::CategoryRepository;
pub use delivery_order::{
    DeliveryOrderItemRepository, DeliveryOrderRepository, InventoryRepository,
};
pub use product::ProductRepository;
pub use receipt::ReceiptRepository;
pub use reconciliation::{StockReconciliationItemRepository, StockReconciliationRepository};
pub use stock::{InventoryLevelRepository, StockMoveRepository};
pub use stock_take::{StockTakeLineRepository, StockTakeRepository};
pub use transfer::{TransferItemRepository, TransferRepository};
pub use valuation::ValuationRepository;
pub use warehouse::WarehouseRepository;
