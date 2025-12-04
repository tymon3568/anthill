use crate::domains::quality::{
    CreateQualityControlPoint, QualityControlPoint, UpdateQualityControlPoint,
};
use crate::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait QualityControlPointService: Send + Sync {
    async fn create_qc_point(
        &self,
        tenant_id: Uuid,
        qc_point: CreateQualityControlPoint,
    ) -> Result<QualityControlPoint, AppError>;

    async fn get_qc_point(
        &self,
        tenant_id: Uuid,
        qc_point_id: Uuid,
    ) -> Result<Option<QualityControlPoint>, AppError>;

    async fn update_qc_point(
        &self,
        tenant_id: Uuid,
        qc_point_id: Uuid,
        updates: UpdateQualityControlPoint,
    ) -> Result<QualityControlPoint, AppError>;

    async fn delete_qc_point(&self, tenant_id: Uuid, qc_point_id: Uuid) -> Result<(), AppError>;

    async fn list_qc_points(&self, tenant_id: Uuid) -> Result<Vec<QualityControlPoint>, AppError>;

    async fn list_active_qc_points(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<QualityControlPoint>, AppError>;

    async fn list_qc_points_for_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<Vec<QualityControlPoint>, AppError>;

    async fn list_qc_points_for_warehouse(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<QualityControlPoint>, AppError>;
}
