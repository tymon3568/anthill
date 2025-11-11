//! Warehouse repository implementation
//!
//! PostgreSQL implementation of the WarehouseRepository trait.

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::warehouse_dto::{
    CreateWarehouseRequest, WarehouseResponse, WarehouseTreeNode, WarehouseTreeResponse,
    WarehouseZoneWithLocations,
};
use inventory_service_core::domains::inventory::warehouse::Warehouse;
use inventory_service_core::repositories::warehouse::WarehouseRepository;
use inventory_service_core::Result;

/// PostgreSQL implementation of WarehouseRepository
pub struct WarehouseRepositoryImpl {
    pool: PgPool,
}

impl WarehouseRepositoryImpl {
    /// Create new repository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WarehouseRepository for WarehouseRepositoryImpl {
    // ========================================================================
    // CRUD Operations
    // ========================================================================

    async fn create(&self, tenant_id: Uuid, request: CreateWarehouseRequest) -> Result<Warehouse> {
        let warehouse = sqlx::query_as!(
            Warehouse,
            r#"
            INSERT INTO warehouses (
                tenant_id, warehouse_code, warehouse_name, description,
                warehouse_type, parent_warehouse_id, address, contact_info, capacity_info
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING
                warehouse_id, tenant_id, warehouse_code, warehouse_name, description,
                warehouse_type, parent_warehouse_id, address, contact_info, capacity_info,
                is_active, created_at, updated_at, deleted_at
            "#,
            tenant_id,
            request.warehouse_code,
            request.warehouse_name,
            request.description,
            request.warehouse_type,
            request.parent_warehouse_id,
            request.address,
            request.contact_info,
            request.capacity_info
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(warehouse)
    }

    async fn find_by_id(&self, tenant_id: Uuid, warehouse_id: Uuid) -> Result<Option<Warehouse>> {
        let warehouse = sqlx::query_as!(
            Warehouse,
            r#"
            SELECT
                warehouse_id, tenant_id, warehouse_code, warehouse_name, description,
                warehouse_type, parent_warehouse_id, address, contact_info, capacity_info,
                is_active, created_at, updated_at, deleted_at
            FROM warehouses
            WHERE tenant_id = $1 AND warehouse_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            warehouse_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(warehouse)
    }

    async fn find_by_code(
        &self,
        tenant_id: Uuid,
        warehouse_code: &str,
    ) -> Result<Option<Warehouse>> {
        let warehouse = sqlx::query_as!(
            Warehouse,
            r#"
            SELECT
                warehouse_id, tenant_id, warehouse_code, warehouse_name, description,
                warehouse_type, parent_warehouse_id, address, contact_info, capacity_info,
                is_active, created_at, updated_at, deleted_at
            FROM warehouses
            WHERE tenant_id = $1 AND warehouse_code = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            warehouse_code
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(warehouse)
    }

    async fn find_all(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>> {
        let warehouses = sqlx::query_as!(
            Warehouse,
            r#"
            SELECT
                warehouse_id, tenant_id, warehouse_code, warehouse_name, description,
                warehouse_type, parent_warehouse_id, address, contact_info, capacity_info,
                is_active, created_at, updated_at, deleted_at
            FROM warehouses
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY warehouse_name
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(warehouses)
    }

    async fn get_warehouse_tree(&self, tenant_id: Uuid) -> Result<WarehouseTreeResponse> {
        // This is a complex query that would require recursive CTEs
        // For now, return a basic structure - will be implemented fully later
        let warehouses = self.find_all(tenant_id).await?;

        // TODO: Implement proper tree building with zones and locations
        let roots = warehouses
            .into_iter()
            .filter(|w| w.parent_warehouse_id.is_none())
            .map(|w| WarehouseTreeNode {
                warehouse: w.into(),
                children: vec![],
                zones: vec![],
            })
            .collect();

        Ok(WarehouseTreeResponse {
            roots,
            total_warehouses: 0, // TODO: implement counts
            total_zones: 0,
            total_locations: 0,
        })
    }

    async fn update(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        warehouse: &Warehouse,
    ) -> Result<Warehouse> {
        let updated = sqlx::query_as!(
            Warehouse,
            r#"
            UPDATE warehouses SET
                warehouse_code = $3,
                warehouse_name = $4,
                description = $5,
                warehouse_type = $6,
                parent_warehouse_id = $7,
                address = $8,
                contact_info = $9,
                capacity_info = $10,
                is_active = $11,
                updated_at = NOW()
            WHERE tenant_id = $1 AND warehouse_id = $2 AND deleted_at IS NULL
            RETURNING
                warehouse_id, tenant_id, warehouse_code, warehouse_name, description,
                warehouse_type, parent_warehouse_id, address, contact_info, capacity_info,
                is_active, created_at, updated_at, deleted_at
            "#,
            tenant_id,
            warehouse_id,
            warehouse.warehouse_code,
            warehouse.warehouse_name,
            warehouse.description,
            warehouse.warehouse_type,
            warehouse.parent_warehouse_id,
            warehouse.address,
            warehouse.contact_info,
            warehouse.capacity_info,
            warehouse.is_active
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    async fn delete(&self, tenant_id: Uuid, warehouse_id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE warehouses SET
                deleted_at = NOW(),
                updated_at = NOW()
            WHERE tenant_id = $1 AND warehouse_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            warehouse_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========================================================================
    // Hierarchy Operations
    // ========================================================================

    async fn get_children(
        &self,
        tenant_id: Uuid,
        parent_warehouse_id: Uuid,
    ) -> Result<Vec<Warehouse>> {
        let children = sqlx::query_as!(
            Warehouse,
            r#"
            SELECT
                warehouse_id, tenant_id, warehouse_code, warehouse_name, description,
                warehouse_type, parent_warehouse_id, address, contact_info, capacity_info,
                is_active, created_at, updated_at, deleted_at
            FROM warehouses
            WHERE tenant_id = $1 AND parent_warehouse_id = $2 AND deleted_at IS NULL
            ORDER BY warehouse_name
            "#,
            tenant_id,
            parent_warehouse_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(children)
    }

    async fn get_ancestors(&self, tenant_id: Uuid, warehouse_id: Uuid) -> Result<Vec<Warehouse>> {
        // TODO: Implement recursive query to get ancestors
        // For now, return empty vec
        Ok(vec![])
    }

    async fn validate_hierarchy(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        parent_warehouse_id: Option<Uuid>,
    ) -> Result<bool> {
        // TODO: Implement cycle detection
        // For now, basic validation
        if let Some(parent_id) = parent_warehouse_id {
            if parent_id == warehouse_id {
                return Ok(false);
            }
            // Check if parent exists and is active
            let parent = self.find_by_id(tenant_id, parent_id).await?;
            Ok(parent.is_some())
        } else {
            Ok(true)
        }
    }

    // ========================================================================
    // Capacity and Analytics
    // ========================================================================

    async fn get_capacity_utilization(
        &self,
        _tenant_id: Uuid,
        _warehouse_id: Uuid,
    ) -> Result<Option<serde_json::Value>> {
        // TODO: Implement capacity utilization calculation
        Ok(None)
    }

    async fn get_warehouse_stats(
        &self,
        _tenant_id: Uuid,
        _warehouse_id: Uuid,
    ) -> Result<Option<serde_json::Value>> {
        // TODO: Implement warehouse statistics
        Ok(None)
    }
}
