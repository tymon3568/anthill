//! Picking method repository implementation
//!
//! PostgreSQL implementation of the PickingMethodRepository trait.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::picking_method_dto::{
    CreatePickingMethodRequest, PickingMetrics, PickingOptimizationRequest, PickingPlanResponse,
    PickingTask, UpdatePickingMethodRequest,
};
use inventory_service_core::domains::inventory::picking_method::PickingMethod;
use inventory_service_core::repositories::picking_method::PickingMethodRepository;
use inventory_service_core::Result;

/// PostgreSQL implementation of PickingMethodRepository
pub struct PickingMethodRepositoryImpl {
    pool: PgPool,
}

impl PickingMethodRepositoryImpl {
    /// Create new repository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PickingMethodRepository for PickingMethodRepositoryImpl {
    // ========================================================================
    // CRUD Operations for Picking Methods
    // ========================================================================

    async fn create(
        &self,
        tenant_id: Uuid,
        request: CreatePickingMethodRequest,
        created_by: Uuid,
    ) -> Result<PickingMethod> {
        let method = sqlx::query_as!(
            PickingMethod,
            r#"
            INSERT INTO picking_methods (
                tenant_id, name, description, method_type, warehouse_id, config, is_default
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                method_id, tenant_id, name, description, method_type, warehouse_id, config,
                is_active, is_default, created_at, updated_at, deleted_at
            "#,
            tenant_id,
            request.name,
            request.description,
            request.method_type,
            request.warehouse_id,
            request.config,
            request.is_default.unwrap_or(false)
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(method)
    }

    async fn find_by_id(&self, tenant_id: Uuid, method_id: Uuid) -> Result<Option<PickingMethod>> {
        let method = sqlx::query_as!(
            PickingMethod,
            r#"
            SELECT
                method_id, tenant_id, name, description, method_type, warehouse_id, config,
                is_active, is_default, created_at, updated_at, deleted_at
            FROM picking_methods
            WHERE tenant_id = $1 AND method_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            method_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(method)
    }

    async fn find_by_name(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        name: &str,
    ) -> Result<Option<PickingMethod>> {
        let method = sqlx::query_as!(
            PickingMethod,
            r#"
            SELECT
                method_id, tenant_id, name, description, method_type, warehouse_id, config,
                is_active, is_default, created_at, updated_at, deleted_at
            FROM picking_methods
            WHERE tenant_id = $1 AND warehouse_id = $2 AND name = $3 AND deleted_at IS NULL
            "#,
            tenant_id,
            warehouse_id,
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(method)
    }

    async fn find_by_warehouse(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<PickingMethod>> {
        let methods = sqlx::query_as!(
            PickingMethod,
            r#"
            SELECT
                method_id, tenant_id, name, description, method_type, warehouse_id, config,
                is_active, is_default, created_at, updated_at, deleted_at
            FROM picking_methods
            WHERE tenant_id = $1 AND warehouse_id = $2 AND deleted_at IS NULL
            ORDER BY name
            "#,
            tenant_id,
            warehouse_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(methods)
    }

    async fn find_active_by_warehouse(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<PickingMethod>> {
        let methods = sqlx::query_as!(
            PickingMethod,
            r#"
            SELECT
                method_id, tenant_id, name, description, method_type, warehouse_id, config,
                is_active, is_default, created_at, updated_at, deleted_at
            FROM picking_methods
            WHERE tenant_id = $1 AND warehouse_id = $2 AND is_active = true AND deleted_at IS NULL
            ORDER BY name
            "#,
            tenant_id,
            warehouse_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(methods)
    }

    async fn find_default_by_warehouse(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Option<PickingMethod>> {
        let method = sqlx::query_as!(
            PickingMethod,
            r#"
            SELECT
                method_id, tenant_id, name, description, method_type, warehouse_id, config,
                is_active, is_default, created_at, updated_at, deleted_at
            FROM picking_methods
            WHERE tenant_id = $1 AND warehouse_id = $2 AND is_default = true AND deleted_at IS NULL
            "#,
            tenant_id,
            warehouse_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(method)
    }

    async fn update(
        &self,
        tenant_id: Uuid,
        method_id: Uuid,
        request: UpdatePickingMethodRequest,
        updated_by: Uuid,
    ) -> Result<PickingMethod> {
        let updated = sqlx::query_as!(
            PickingMethod,
            r#"
            UPDATE picking_methods SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                config = COALESCE($5, config),
                is_default = COALESCE($6, is_default),
                is_active = COALESCE($7, is_active),
                updated_at = NOW()
            WHERE tenant_id = $1 AND method_id = $2 AND deleted_at IS NULL
            RETURNING
                method_id, tenant_id, name, description, method_type, warehouse_id, config,
                is_active, is_default, created_at, updated_at, deleted_at
            "#,
            tenant_id,
            method_id,
            request.name,
            request.description,
            request.config,
            request.is_default,
            request.is_active
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    async fn delete(&self, tenant_id: Uuid, method_id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE picking_methods SET
                deleted_at = NOW(),
                updated_at = NOW()
            WHERE tenant_id = $1 AND method_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            method_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn set_default(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        method_id: Uuid,
        updated_by: Uuid,
    ) -> Result<bool> {
        // First, unset all defaults for this warehouse
        sqlx::query!(
            r#"
            UPDATE picking_methods SET
                is_default = false,
                updated_at = NOW()
            WHERE tenant_id = $1 AND warehouse_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            warehouse_id
        )
        .execute(&self.pool)
        .await?;

        // Then set the new default
        let result = sqlx::query!(
            r#"
            UPDATE picking_methods SET
                is_default = true,
                updated_at = NOW()
            WHERE tenant_id = $1 AND method_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            method_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========================================================================
    // Picking Optimization Operations
    // ========================================================================

    async fn generate_picking_plan(
        &self,
        tenant_id: Uuid,
        request: PickingOptimizationRequest,
    ) -> Result<PickingPlanResponse> {
        // This is a complex operation that would require:
        // 1. Getting order details and items
        // 2. Finding product locations
        // 3. Applying optimization algorithm based on method
        // 4. Calculating metrics
        // For now, return a placeholder implementation

        let plan_id = Uuid::now_v7();
        let method = if let Some(method_id) = request.method_id {
            self.find_by_id(tenant_id, method_id)
                .await?
                .ok_or_else(|| {
                    inventory_service_core::AppError::NotFound(
                        "Picking method not found".to_string(),
                    )
                })?
        } else {
            self.find_default_by_warehouse(tenant_id, request.warehouse_id)
                .await?
                .ok_or_else(|| {
                    inventory_service_core::AppError::NotFound(
                        "No default picking method found".to_string(),
                    )
                })?
        };

        // Placeholder: create empty tasks and metrics
        let tasks = vec![];
        let metrics = PickingMetrics {
            total_distance_meters: None,
            total_estimated_time_seconds: None,
            task_count: 0,
            efficiency_score: None,
            travel_time_reduction_percent: None,
        };

        Ok(PickingPlanResponse {
            plan_id,
            method_id: method.method_id,
            method_name: method.name,
            method_type: method.method_type,
            warehouse_id: request.warehouse_id,
            order_ids: request.order_ids,
            tasks,
            metrics,
            generated_at: chrono::Utc::now(),
        })
    }

    async fn validate_method_config(&self, tenant_id: Uuid, method_id: Uuid) -> Result<bool> {
        // Basic validation: check if method exists and config is valid JSON
        if let Some(method) = self.find_by_id(tenant_id, method_id).await? {
            // Check if config is valid JSON (it should be since it's stored as JSONB)
            // Additional validation logic can be added here
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
