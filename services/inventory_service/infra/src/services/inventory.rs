use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::repositories::InventoryRepository;
use inventory_service_core::services::InventoryService;
use shared_error::AppError;

/// Implementation of InventoryService
pub struct InventoryServiceImpl {
    inventory_repo: Arc<dyn InventoryRepository>,
}

impl InventoryServiceImpl {
    pub fn new(inventory_repo: Arc<dyn InventoryRepository>) -> Self {
        Self { inventory_repo }
    }
}

#[async_trait]
impl InventoryService for InventoryServiceImpl {
    async fn reserve_stock(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        quantity: i64,
    ) -> Result<(), AppError> {
        self.inventory_repo
            .reserve_stock(tenant_id, warehouse_id, product_id, quantity)
            .await
    }

    async fn release_stock(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        quantity: i64,
    ) -> Result<(), AppError> {
        self.inventory_repo
            .release_stock(tenant_id, warehouse_id, product_id, quantity)
            .await
    }

    async fn get_available_stock(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<i64, AppError> {
        self.inventory_repo
            .get_available_stock(tenant_id, warehouse_id, product_id)
            .await
    }
}
