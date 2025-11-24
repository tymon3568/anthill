use async_trait::async_trait;
use sqlx::Transaction;
use uuid::Uuid;

use crate::models::{DeliveryOrder, DeliveryOrderItem};
use shared_error::AppError;

#[async_trait]
pub trait DeliveryOrderRepository: Send + Sync {
    async fn create(&self, delivery_order: &DeliveryOrder) -> Result<(), AppError>;
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<Option<DeliveryOrder>, AppError>;
    async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<DeliveryOrder>, AppError>;
    async fn update(&self, delivery_order: &DeliveryOrder) -> Result<(), AppError>;
    async fn delete(&self, tenant_id: Uuid, delivery_id: Uuid) -> Result<(), AppError>;
    async fn find_by_order_id(
        &self,
        tenant_id: Uuid,
        order_id: Uuid,
    ) -> Result<Option<DeliveryOrder>, AppError>;

    async fn begin_transaction(&self) -> Result<Transaction<'_, sqlx::Postgres>, AppError>;
    async fn find_by_id_with_tx(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<Option<DeliveryOrder>, AppError>;
    async fn update_with_tx(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        delivery_order: &DeliveryOrder,
    ) -> Result<(), AppError>;
}

#[async_trait]
pub trait DeliveryOrderItemRepository: Send + Sync {
    async fn create(&self, delivery_item: &DeliveryOrderItem) -> Result<(), AppError>;
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        delivery_item_id: Uuid,
    ) -> Result<Option<DeliveryOrderItem>, AppError>;
    async fn find_by_delivery_id(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<Vec<DeliveryOrderItem>, AppError>;
    async fn update(&self, delivery_item: &DeliveryOrderItem) -> Result<(), AppError>;
    async fn delete(&self, tenant_id: Uuid, delivery_item_id: Uuid) -> Result<(), AppError>;

    async fn find_by_delivery_id_with_tx(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<Vec<DeliveryOrderItem>, AppError>;
    async fn find_by_id_with_tx(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        tenant_id: Uuid,
        delivery_item_id: Uuid,
    ) -> Result<Option<DeliveryOrderItem>, AppError>;
    async fn update_with_tx(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        delivery_item: &DeliveryOrderItem,
    ) -> Result<(), AppError>;
}

#[async_trait]
pub trait InventoryRepository: Send + Sync {
    async fn reserve_stock(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        quantity: i64,
    ) -> Result<(), AppError>;
    async fn release_stock(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        quantity: i64,
    ) -> Result<(), AppError>;
    async fn get_available_stock(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<i64, AppError>;
}
