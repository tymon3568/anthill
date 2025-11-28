use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{RmaItem, RmaRequest, RmaStatus};
use shared_error::AppError;

#[async_trait]
pub trait RmaRepository: Send + Sync {
    async fn create(&self, rma: &RmaRequest) -> Result<(), AppError>;
    async fn find_by_id(&self, tenant_id: Uuid, rma_id: Uuid) -> Result<Option<RmaRequest>, AppError>;
    async fn find_by_number(&self, tenant_id: Uuid, rma_number: &str) -> Result<Option<RmaRequest>, AppError>;
    async fn update(&self, rma: &RmaRequest) -> Result<(), AppError>;
    async fn update_status(&self, tenant_id: Uuid, rma_id: Uuid, status: RmaStatus, updated_by: Option<Uuid>) -> Result<(), AppError>;
    async fn list_by_tenant(&self, tenant_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<RmaRequest>, AppError>;
}

#[async_trait]
pub trait RmaItemRepository: Send + Sync {
    async fn create(&self, item: &RmaItem) -> Result<(), AppError>;
    async fn find_by_id(&self, tenant_id: Uuid, rma_item_id: Uuid) -> Result<Option<RmaItem>, AppError>;
    async fn find_by_rma_id(&self, tenant_id: Uuid, rma_id: Uuid) -> Result<Vec<RmaItem>, AppError>;
    async fn update(&self, item: &RmaItem) -> Result<(), AppError>;
    async fn delete(&self, tenant_id: Uuid, rma_item_id: Uuid) -> Result<(), AppError>;
}
