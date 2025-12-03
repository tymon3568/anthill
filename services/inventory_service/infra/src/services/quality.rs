use async_trait::async_trait;
use inventory_service_core::domains::quality::{
    CreateQualityControlPoint, QualityControlPoint, UpdateQualityControlPoint,
};
use inventory_service_core::repositories::quality::QualityControlPointRepository;
use inventory_service_core::services::quality::QualityControlPointService;
use inventory_service_core::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct PgQualityControlPointService {
    repo: Arc<dyn QualityControlPointRepository>,
}

impl PgQualityControlPointService {
    pub fn new(repo: Arc<dyn QualityControlPointRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl QualityControlPointService for PgQualityControlPointService {
    async fn create_qc_point(
        &self,
        tenant_id: Uuid,
        qc_point: CreateQualityControlPoint,
    ) -> Result<QualityControlPoint, AppError> {
        self.repo.create(tenant_id, qc_point).await
    }

    async fn get_qc_point(
        &self,
        tenant_id: Uuid,
        qc_point_id: Uuid,
    ) -> Result<Option<QualityControlPoint>, AppError> {
        self.repo.find_by_id(tenant_id, qc_point_id).await
    }

    async fn update_qc_point(
        &self,
        tenant_id: Uuid,
        qc_point_id: Uuid,
        updates: UpdateQualityControlPoint,
    ) -> Result<QualityControlPoint, AppError> {
        self.repo.update(tenant_id, qc_point_id, updates).await
    }

    async fn delete_qc_point(&self, tenant_id: Uuid, qc_point_id: Uuid) -> Result<(), AppError> {
        self.repo.delete(tenant_id, qc_point_id).await
    }

    async fn list_qc_points(&self, tenant_id: Uuid) -> Result<Vec<QualityControlPoint>, AppError> {
        self.repo.find_all(tenant_id).await
    }

    async fn list_active_qc_points(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<QualityControlPoint>, AppError> {
        self.repo.find_active(tenant_id).await
    }

    async fn list_qc_points_for_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<Vec<QualityControlPoint>, AppError> {
        self.repo.find_by_product(tenant_id, product_id).await
    }

    async fn list_qc_points_for_warehouse(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<QualityControlPoint>, AppError> {
        self.repo.find_by_warehouse(tenant_id, warehouse_id).await
    }
}
