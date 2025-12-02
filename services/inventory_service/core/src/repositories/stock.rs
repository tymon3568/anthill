//! Repository traits for stock operations
//!
//! This module contains trait definitions for StockMove and InventoryLevel operations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{CreateStockMoveRequest, InventoryLevel, StockMove};
use shared_error::AppError;

#[async_trait]
pub trait StockMoveRepository: Send + Sync {
    /// Create a new stock move
    async fn create(
        &self,
        stock_move: &CreateStockMoveRequest,
        tenant_id: Uuid,
    ) -> Result<(), AppError>;

    /// Find stock moves by reference
    async fn find_by_reference(
        &self,
        tenant_id: Uuid,
        reference_type: &str,
        reference_id: Uuid,
    ) -> Result<Vec<StockMove>, AppError>;

    /// Check if idempotency key exists
    async fn exists_by_idempotency_key(
        &self,
        tenant_id: Uuid,
        idempotency_key: &str,
    ) -> Result<bool, AppError>;

    /// Find stock moves by lot serial ID
    async fn find_by_lot_serial(
        &self,
        tenant_id: Uuid,
        lot_serial_id: Uuid,
    ) -> Result<Vec<StockMove>, AppError>;
}

#[async_trait]
pub trait InventoryLevelRepository: Send + Sync {
    /// Find inventory level by product
    async fn find_by_product(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<Option<InventoryLevel>, AppError>;

    /// Update available quantity (increment/decrement)
    async fn update_available_quantity(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        quantity_change: i64,
    ) -> Result<(), AppError>;

    /// Create or update inventory level
    async fn upsert(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        available_quantity: i64,
        reserved_quantity: i64,
    ) -> Result<(), AppError>;
}
