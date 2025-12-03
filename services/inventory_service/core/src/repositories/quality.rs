use crate::domains::quality::{
    CreateQualityControlPoint, QualityControlPoint, UpdateQualityControlPoint,
};
use crate::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait QualityControlPointRepository: Send + Sync {
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        qc_point_id: Uuid,
    ) -> Result<Option<QualityControlPoint>, AppError>;
    async fn find_all(&self, tenant_id: Uuid) -> Result<Vec<QualityControlPoint>, AppError>;
    async fn find_by_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<Vec<QualityControlPoint>, AppError>;
    async fn find_by_warehouse(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<QualityControlPoint>, AppError>;
    async fn find_active(&self, tenant_id: Uuid) -> Result<Vec<QualityControlPoint>, AppError>;
    async fn create(
        &self,
        tenant_id: Uuid,
        qc_point: CreateQualityControlPoint,
    ) -> Result<QualityControlPoint, AppError>;
    async fn update(
        &self,
        tenant_id: Uuid,
        qc_point_id: Uuid,
        updates: UpdateQualityControlPoint,
    ) -> Result<QualityControlPoint, AppError>;
    async fn delete(&self, tenant_id: Uuid, qc_point_id: Uuid) -> Result<(), AppError>;
}
