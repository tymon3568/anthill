use async_trait::async_trait;
use inventory_service_core::domains::quality::{
    CreateQualityControlPoint, QcPointType, QualityControlPoint, UpdateQualityControlPoint,
};
use inventory_service_core::repositories::quality::QualityControlPointRepository;
use inventory_service_core::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PgQualityControlPointRepository {
    pool: PgPool,
}

impl PgQualityControlPointRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QualityControlPointRepository for PgQualityControlPointRepository {
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        qc_point_id: Uuid,
    ) -> Result<Option<QualityControlPoint>, AppError> {
        let qc_point = sqlx::query_as!(
            QualityControlPoint,
            r#"
            SELECT
                qc_point_id, tenant_id, name,
                type as "type: QcPointType",
                product_id, warehouse_id, active,
                created_at, updated_at
            FROM quality_control_points
            WHERE tenant_id = $1 AND qc_point_id = $2
            "#,
            tenant_id,
            qc_point_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(qc_point)
    }

    async fn find_all(&self, tenant_id: Uuid) -> Result<Vec<QualityControlPoint>, AppError> {
        let qc_points = sqlx::query_as!(
            QualityControlPoint,
            r#"
            SELECT
                qc_point_id, tenant_id, name,
                type as "type: QcPointType",
                product_id, warehouse_id, active,
                created_at, updated_at
            FROM quality_control_points
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(qc_points)
    }

    async fn find_by_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<Vec<QualityControlPoint>, AppError> {
        let qc_points = sqlx::query_as!(
            QualityControlPoint,
            r#"
            SELECT
                qc_point_id, tenant_id, name,
                type as "type: QcPointType",
                product_id, warehouse_id, active,
                created_at, updated_at
            FROM quality_control_points
            WHERE tenant_id = $1 AND product_id = $2
            ORDER BY created_at DESC
            "#,
            tenant_id,
            product_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(qc_points)
    }

    async fn find_by_warehouse(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<QualityControlPoint>, AppError> {
        let qc_points = sqlx::query_as!(
            QualityControlPoint,
            r#"
            SELECT
                qc_point_id, tenant_id, name,
                type as "type: QcPointType",
                product_id, warehouse_id, active,
                created_at, updated_at
            FROM quality_control_points
            WHERE tenant_id = $1 AND warehouse_id = $2
            ORDER BY created_at DESC
            "#,
            tenant_id,
            warehouse_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(qc_points)
    }

    async fn find_active(&self, tenant_id: Uuid) -> Result<Vec<QualityControlPoint>, AppError> {
        let qc_points = sqlx::query_as!(
            QualityControlPoint,
            r#"
            SELECT
                qc_point_id, tenant_id, name,
                type as "type: QcPointType",
                product_id, warehouse_id, active,
                created_at, updated_at
            FROM quality_control_points
            WHERE tenant_id = $1 AND active = true
            ORDER BY created_at DESC
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(qc_points)
    }

    async fn create(
        &self,
        tenant_id: Uuid,
        qc_point: CreateQualityControlPoint,
    ) -> Result<QualityControlPoint, AppError> {
        let qc_type_str = match qc_point.r#type {
            QcPointType::Incoming => "incoming",
            QcPointType::Outgoing => "outgoing",
            QcPointType::Internal => "internal",
        };

        let new_qc_point = sqlx::query_as!(
            QualityControlPoint,
            r#"
            INSERT INTO quality_control_points (
                tenant_id, name, type, product_id, warehouse_id
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                qc_point_id, tenant_id, name,
                type as "type: QcPointType",
                product_id, warehouse_id, active,
                created_at, updated_at
            "#,
            tenant_id,
            qc_point.name,
            qc_type_str,
            qc_point.product_id,
            qc_point.warehouse_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(new_qc_point)
    }

    async fn update(
        &self,
        tenant_id: Uuid,
        qc_point_id: Uuid,
        updates: UpdateQualityControlPoint,
    ) -> Result<QualityControlPoint, AppError> {
        let qc_type_str = updates.r#type.as_ref().map(|t| match t {
            QcPointType::Incoming => "incoming",
            QcPointType::Outgoing => "outgoing",
            QcPointType::Internal => "internal",
        });

        let updated_qc_point = sqlx::query_as!(
            QualityControlPoint,
            r#"
            UPDATE quality_control_points
            SET
                name = COALESCE($3, name),
                type = COALESCE($4, type),
                product_id = COALESCE($5, product_id),
                warehouse_id = COALESCE($6, warehouse_id),
                active = COALESCE($7, active),
                updated_at = NOW()
            WHERE tenant_id = $1 AND qc_point_id = $2
            RETURNING
                qc_point_id, tenant_id, name,
                type as "type: QcPointType",
                product_id, warehouse_id, active,
                created_at, updated_at
            "#,
            tenant_id,
            qc_point_id,
            updates.name,
            qc_type_str,
            updates.product_id,
            updates.warehouse_id,
            updates.active
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| {
            AppError::NotFound(format!("Quality control point {} not found", qc_point_id))
        })?;

        Ok(updated_qc_point)
    }

    async fn delete(&self, tenant_id: Uuid, qc_point_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE quality_control_points
            SET active = false, updated_at = NOW()
            WHERE tenant_id = $1 AND qc_point_id = $2
            "#,
            tenant_id,
            qc_point_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
