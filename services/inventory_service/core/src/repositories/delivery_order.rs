use async_trait::async_trait;
use uuid::Uuid;

use shared_error::AppError;

#[async_trait]
pub trait DeliveryOrderRepository: Send + Sync {
    async fn create(&self, delivery_order: &crate::models::DeliveryOrder) -> Result<(), AppError>;
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<Option<crate::models::DeliveryOrder>, AppError>;
    async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<crate::models::DeliveryOrder>, AppError>;
    async fn update(&self, delivery_order: &crate::models::DeliveryOrder) -> Result<(), AppError>;
    async fn delete(&self, tenant_id: Uuid, delivery_id: Uuid) -> Result<(), AppError>;
    async fn find_by_order_id(
        &self,
        tenant_id: Uuid,
        order_id: Uuid,
    ) -> Result<Option<crate::models::DeliveryOrder>, AppError>;
}

#[async_trait]
pub trait DeliveryOrderItemRepository: Send + Sync {
    async fn create(
        &self,
        delivery_item: &crate::models::DeliveryOrderItem,
    ) -> Result<(), AppError>;
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        delivery_item_id: Uuid,
    ) -> Result<Option<crate::models::DeliveryOrderItem>, AppError>;
    async fn find_by_delivery_id(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<Vec<crate::models::DeliveryOrderItem>, AppError>;
    async fn update(
        &self,
        delivery_item: &crate::models::DeliveryOrderItem,
    ) -> Result<(), AppError>;
    async fn delete(&self, tenant_id: Uuid, delivery_item_id: Uuid) -> Result<(), AppError>;
}

#[async_trait]
pub trait InventoryRepository: Send + Sync {
    /// Reserve stock for a product
    async fn reserve_stock(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity: i64,
    ) -> Result<(), AppError>;

    /// Release reserved stock
    async fn release_stock(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity: i64,
    ) -> Result<(), AppError>;

    /// Check available stock for a product
    async fn get_available_stock(&self, tenant_id: Uuid, product_id: Uuid)
        -> Result<i64, AppError>;
}
