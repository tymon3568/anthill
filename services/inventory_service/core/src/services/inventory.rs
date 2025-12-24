//! Inventory Service Trait
//!
//! Handles stock reservation and availability checks.

use async_trait::async_trait;
use uuid::Uuid;

use shared_error::AppError;

/// Service for managing inventory stock and reservations
#[async_trait]
pub trait InventoryService: Send + Sync {
    /// Reserve stock for a product in a warehouse
    ///
    /// Validates availability and creates reservation.
    /// Supports both standard and lot-tracked products.
    async fn reserve_stock(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        quantity: i64,
    ) -> Result<(), AppError>;

    /// Release reserved stock
    ///
    /// Frees up reserved stock, making it available again.
    async fn release_stock(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        quantity: i64,
    ) -> Result<(), AppError>;

    /// Get current available stock quantity
    async fn get_available_stock(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<i64, AppError>;
}
