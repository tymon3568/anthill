use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{DeliveryOrder, DeliveryOrderItem};
use crate::Result;

/// Repository trait for Delivery Order operations
#[async_trait]
pub trait DeliveryOrderRepository: Send + Sync {
    /// Create a new delivery order
    async fn create(&self, delivery_order: &DeliveryOrder) -> Result<DeliveryOrder>;

    /// Find delivery order by ID
    async fn find_by_id(&self, tenant_id: Uuid, delivery_id: Uuid)
        -> Result<Option<DeliveryOrder>>;

    /// Find delivery orders by tenant with pagination
    async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DeliveryOrder>>;

    /// Update delivery order
    async fn update(&self, delivery_order: &DeliveryOrder) -> Result<DeliveryOrder>;

    /// Delete delivery order (soft delete)
    async fn delete(&self, tenant_id: Uuid, delivery_id: Uuid) -> Result<()>;

    /// Find delivery orders by order ID
    async fn find_by_order_id(&self, tenant_id: Uuid, order_id: Uuid)
        -> Result<Vec<DeliveryOrder>>;
}

/// Repository trait for Delivery Order Item operations
#[async_trait]
pub trait DeliveryOrderItemRepository: Send + Sync {
    /// Create a new delivery order item
    async fn create(&self, item: &DeliveryOrderItem) -> Result<DeliveryOrderItem>;

    /// Find items by delivery order ID
    async fn find_by_delivery_id(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<Vec<DeliveryOrderItem>>;

    /// Update delivery order item
    async fn update(&self, item: &DeliveryOrderItem) -> Result<DeliveryOrderItem>;

    /// Delete delivery order item
    async fn delete(&self, tenant_id: Uuid, delivery_item_id: Uuid) -> Result<()>;
}

/// Repository trait for Inventory operations
#[async_trait]
pub trait InventoryRepository: Send + Sync {
    /// Reserve stock for a product in a specific warehouse
    async fn reserve_stock(&self, tenant_id: Uuid, product_id: Uuid, quantity: i32) -> Result<()>;

    /// Release reserved stock
    async fn release_stock(&self, tenant_id: Uuid, product_id: Uuid, quantity: i32) -> Result<()>;

    /// Check available stock for a product
    async fn get_available_stock(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i32>;
}
