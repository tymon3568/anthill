//! Warehouse repository implementation
//!
//! PostgreSQL implementation of the WarehouseRepository trait.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::warehouse_dto::{
    CreateWarehouseLocationRequest, CreateWarehouseRequest, CreateWarehouseZoneRequest,
    UpdateWarehouseLocationRequest, UpdateWarehouseZoneRequest, WarehouseTreeNode,
    WarehouseTreeResponse, WarehouseZoneWithLocations,
};
use inventory_service_core::domains::inventory::warehouse::Warehouse;
use inventory_service_core::domains::inventory::warehouse_location::WarehouseLocation;
use inventory_service_core::domains::inventory::warehouse_zone::WarehouseZone;
use inventory_service_core::repositories::warehouse::WarehouseRepository;
use inventory_service_core::Result;
use serde_json;
use shared_error::AppError;

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
        let warehouses = self.find_all(tenant_id).await?;
        let zones = self.get_all_zones(tenant_id).await?;
        let locations = self.get_all_locations(tenant_id).await?;

        let total_warehouses = warehouses.len() as u32;
        let total_zones = zones.len() as u32;
        let total_locations = locations.len() as u32;

        // Group zones by warehouse_id
        let mut zones_by_warehouse: std::collections::HashMap<Uuid, Vec<_>> =
            std::collections::HashMap::new();
        for zone in zones {
            zones_by_warehouse
                .entry(zone.warehouse_id)
                .or_insert_with(Vec::new)
                .push(zone);
        }

        // Group locations by zone_id
        let mut locations_by_zone: std::collections::HashMap<Uuid, Vec<_>> =
            std::collections::HashMap::new();
        for location in locations {
            if let Some(zone_id) = location.zone_id {
                locations_by_zone
                    .entry(zone_id)
                    .or_insert_with(Vec::new)
                    .push(location);
            }
        }

        // Group warehouses by parent_id
        let mut warehouses_by_parent: std::collections::HashMap<Option<Uuid>, Vec<_>> =
            std::collections::HashMap::new();
        for warehouse in warehouses.clone() {
            warehouses_by_parent
                .entry(warehouse.parent_warehouse_id)
                .or_insert_with(Vec::new)
                .push(warehouse);
        }

        // Build tree recursively starting from roots
        let roots = Self::build_tree_nodes(
            warehouses_by_parent.get(&None).unwrap_or(&vec![]).clone(),
            &warehouses_by_parent,
            &zones_by_warehouse,
            &locations_by_zone,
        );

        Ok(WarehouseTreeResponse {
            roots,
            total_warehouses,
            total_zones,
            total_locations,
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
        // Get ancestor IDs first using CTE
        let ancestor_ids: Vec<Uuid> = sqlx::query_scalar!(
            r#"
            WITH RECURSIVE ancestor_chain AS (
                -- Base case: start with the given warehouse
                SELECT warehouse_id, parent_warehouse_id, 0 AS depth
                FROM warehouses
                WHERE tenant_id = $1 AND warehouse_id = $2 AND deleted_at IS NULL

                UNION ALL

                -- Recursive case: step up to the parent
                SELECT w.warehouse_id, w.parent_warehouse_id, ac.depth + 1
                FROM warehouses w
                INNER JOIN ancestor_chain ac ON w.warehouse_id = ac.parent_warehouse_id
                WHERE w.tenant_id = $1 AND w.deleted_at IS NULL
            )
            SELECT warehouse_id
            FROM ancestor_chain
            WHERE depth > 0  -- Exclude the original warehouse, only return ancestors
            ORDER BY depth DESC  -- Root first, then immediate parent
            "#,
            tenant_id,
            warehouse_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .flatten() // Filter out None values
        .collect();

        // Now fetch full warehouse details for these IDs
        let mut ancestors = Vec::new();
        for id in ancestor_ids {
            if let Some(warehouse) = self.find_by_id(tenant_id, id).await? {
                ancestors.push(warehouse);
            }
        }

        Ok(ancestors)
    }

    async fn get_descendants(&self, tenant_id: Uuid, warehouse_id: Uuid) -> Result<Vec<Warehouse>> {
        // Get descendant IDs first using CTE
        let descendant_ids: Vec<Uuid> = sqlx::query_scalar!(
            r#"
            WITH RECURSIVE descendant_chain AS (
                -- Base case: start with the given warehouse
                SELECT warehouse_id, 0 as depth
                FROM warehouses
                WHERE tenant_id = $1 AND warehouse_id = $2 AND deleted_at IS NULL

                UNION ALL

                -- Recursive case: get children
                SELECT w.warehouse_id, dc.depth + 1
                FROM warehouses w
                INNER JOIN descendant_chain dc ON w.parent_warehouse_id = dc.warehouse_id
                WHERE w.tenant_id = $1 AND w.deleted_at IS NULL
            )
            SELECT warehouse_id
            FROM descendant_chain
            WHERE depth > 0  -- Exclude the original warehouse, only return descendants
            "#,
            tenant_id,
            warehouse_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .flatten() // Filter out None values
        .collect();

        // Now fetch full warehouse details for these IDs
        let mut descendants = Vec::new();
        for id in descendant_ids {
            if let Some(warehouse) = self.find_by_id(tenant_id, id).await? {
                descendants.push(warehouse);
            }
        }

        Ok(descendants)
    }

    async fn get_all_zones(&self, tenant_id: Uuid) -> Result<Vec<WarehouseZone>> {
        let zones = sqlx::query_as!(
            WarehouseZone,
            r#"
            SELECT
                zone_id, tenant_id, warehouse_id, zone_code, zone_name, description,
                zone_type, zone_attributes, capacity_info, is_active, created_at, updated_at, deleted_at
            FROM warehouse_zones
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY warehouse_id, zone_name
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(zones)
    }

    async fn get_all_locations(&self, tenant_id: Uuid) -> Result<Vec<WarehouseLocation>> {
        let locations = sqlx::query_as!(
            WarehouseLocation,
            r#"
            SELECT
                location_id, tenant_id, warehouse_id, zone_id, location_code, location_name, description,
                location_type, coordinates, dimensions, capacity_info, location_attributes,
                is_active, created_at, updated_at, deleted_at
            FROM warehouse_locations
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY warehouse_id, location_code
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(locations)
    }

    async fn find_zone_by_id(
        &self,
        tenant_id: Uuid,
        zone_id: Uuid,
    ) -> Result<Option<WarehouseZone>> {
        let zone = sqlx::query_as!(
            WarehouseZone,
            "SELECT zone_id, tenant_id, warehouse_id, zone_code, zone_name, description, zone_type, zone_attributes, capacity_info, is_active, created_at, updated_at, deleted_at FROM warehouse_zones WHERE tenant_id = $1 AND zone_id = $2 AND deleted_at IS NULL",
            tenant_id,
            zone_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(zone)
    }

    async fn find_location_by_id(
        &self,
        tenant_id: Uuid,
        location_id: Uuid,
    ) -> Result<Option<WarehouseLocation>> {
        let location = sqlx::query_as!(
            WarehouseLocation,
            "SELECT location_id, tenant_id, warehouse_id, zone_id, location_code, location_name, description, location_type, coordinates, dimensions, capacity_info, location_attributes, is_active, created_at, updated_at, deleted_at FROM warehouse_locations WHERE tenant_id = $1 AND location_id = $2 AND deleted_at IS NULL",
            tenant_id,
            location_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(location)
    }

    async fn validate_hierarchy(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        parent_warehouse_id: Option<Uuid>,
    ) -> Result<bool> {
        if let Some(parent_id) = parent_warehouse_id {
            // Check for self-reference
            if parent_id == warehouse_id {
                return Ok(false);
            }

            // Check if parent exists and is active
            let parent = self.find_by_id(tenant_id, parent_id).await?;
            if parent.is_none() {
                return Ok(false);
            }

            // Check for cycles: ensure the proposed parent is not a descendant of current warehouse
            let descendants = self.get_descendants(tenant_id, warehouse_id).await?;
            let would_create_cycle = descendants
                .iter()
                .any(|desc| desc.warehouse_id == parent_id);

            Ok(!would_create_cycle)
        } else {
            // No parent specified (making it a root warehouse) is always valid
            Ok(true)
        }
    }

    async fn create_zone(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        request: CreateWarehouseZoneRequest,
    ) -> Result<WarehouseZone> {
        let zone = sqlx::query_as!(
            WarehouseZone,
            r#"
        INSERT INTO warehouse_zones (
            tenant_id, warehouse_id, zone_code, zone_name, description,
            zone_type, zone_attributes, capacity_info
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            zone_id, tenant_id, warehouse_id, zone_code, zone_name, description,
            zone_type, zone_attributes, capacity_info, is_active, created_at, updated_at, deleted_at
        "#,
            tenant_id,
            warehouse_id,
            request.zone_code,
            request.zone_name,
            request.description,
            request.zone_type,
            request.zone_attributes,
            request.capacity_info
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(zone)
    }

    async fn create_location(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        request: CreateWarehouseLocationRequest,
    ) -> Result<WarehouseLocation> {
        let location = sqlx::query_as!(
        WarehouseLocation,
        r#"
        INSERT INTO warehouse_locations (
            tenant_id, warehouse_id, zone_id, location_code, location_name, description,
            location_type, coordinates, dimensions, capacity_info, location_attributes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
            location_id, tenant_id, warehouse_id, zone_id, location_code, location_name, description,
            location_type, coordinates, dimensions, capacity_info, location_attributes,
            is_active, created_at, updated_at, deleted_at
        "#,
        tenant_id,
        warehouse_id,
        request.zone_id,
        request.location_code,
        request.location_name,
        request.description,
        request.location_type,
        request.coordinates,
        request.dimensions,
        request.capacity_info,
        request.location_attributes
    )
    .fetch_one(&self.pool)
    .await?;

        Ok(location)
    }

    // ========================================================================
    // Zone CRUD Operations
    // ========================================================================

    async fn get_zones_by_warehouse(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<WarehouseZone>> {
        let zones = sqlx::query_as!(
            WarehouseZone,
            r#"
            SELECT
                zone_id,
                tenant_id,
                warehouse_id,
                zone_code,
                zone_name,
                description,
                zone_type,
                zone_attributes,
                capacity_info,
                is_active,
                created_at,
                updated_at,
                deleted_at
            FROM warehouse_zones
            WHERE tenant_id = $1
              AND warehouse_id = $2
              AND deleted_at IS NULL
            ORDER BY zone_code ASC
            "#,
            tenant_id,
            warehouse_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(zones)
    }

    async fn update_zone(
        &self,
        tenant_id: Uuid,
        zone_id: Uuid,
        request: UpdateWarehouseZoneRequest,
    ) -> Result<WarehouseZone> {
        let zone = sqlx::query_as!(
            WarehouseZone,
            r#"
            UPDATE warehouse_zones
            SET zone_code = $3,
                zone_name = $4,
                description = $5,
                zone_type = $6,
                zone_attributes = $7,
                capacity_info = $8,
                is_active = COALESCE($9, is_active),
                updated_at = NOW()
            WHERE tenant_id = $1
              AND zone_id = $2
              AND deleted_at IS NULL
            RETURNING
                zone_id,
                tenant_id,
                warehouse_id,
                zone_code,
                zone_name,
                description,
                zone_type,
                zone_attributes,
                capacity_info,
                is_active,
                created_at,
                updated_at,
                deleted_at
            "#,
            tenant_id,
            zone_id,
            request.zone_code,
            request.zone_name,
            request.description,
            request.zone_type,
            request.zone_attributes,
            request.capacity_info,
            request.is_active
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            AppError::NotFound("Zone not found or does not belong to this tenant".to_string())
        })?;

        Ok(zone)
    }

    async fn delete_zone(&self, tenant_id: Uuid, zone_id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE warehouse_zones
            SET deleted_at = NOW(),
                updated_at = NOW()
            WHERE tenant_id = $1
              AND zone_id = $2
              AND deleted_at IS NULL
            "#,
            tenant_id,
            zone_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========================================================================
    // Location CRUD Operations
    // ========================================================================

    async fn get_locations_by_warehouse(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<WarehouseLocation>> {
        let locations = sqlx::query_as!(
            WarehouseLocation,
            r#"
            SELECT
                l.location_id,
                l.tenant_id,
                l.warehouse_id,
                l.zone_id,
                l.location_code,
                l.location_name,
                l.description,
                l.location_type,
                l.coordinates,
                l.dimensions,
                l.capacity_info,
                l.location_attributes,
                l.is_active,
                l.created_at,
                l.updated_at,
                l.deleted_at
            FROM warehouse_locations l
            WHERE l.tenant_id = $1
              AND l.warehouse_id = $2
              AND l.deleted_at IS NULL
            ORDER BY l.location_code ASC
            "#,
            tenant_id,
            warehouse_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(locations)
    }

    async fn update_location(
        &self,
        tenant_id: Uuid,
        location_id: Uuid,
        request: UpdateWarehouseLocationRequest,
    ) -> Result<WarehouseLocation> {
        let location = sqlx::query_as!(
            WarehouseLocation,
            r#"
            UPDATE warehouse_locations
            SET zone_id = COALESCE($3, zone_id),
                location_code = $4,
                location_name = $5,
                description = $6,
                location_type = $7,
                coordinates = $8,
                dimensions = $9,
                capacity_info = $10,
                location_attributes = $11,
                is_active = COALESCE($12, is_active),
                updated_at = NOW()
            WHERE tenant_id = $1
              AND location_id = $2
              AND deleted_at IS NULL
            RETURNING
                location_id,
                tenant_id,
                warehouse_id,
                zone_id,
                location_code,
                location_name,
                description,
                location_type,
                coordinates,
                dimensions,
                capacity_info,
                location_attributes,
                is_active,
                created_at,
                updated_at,
                deleted_at
            "#,
            tenant_id,
            location_id,
            request.zone_id,
            request.location_code,
            request.location_name,
            request.description,
            request.location_type,
            request.coordinates,
            request.dimensions,
            request.capacity_info,
            request.location_attributes,
            request.is_active
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            AppError::NotFound("Location not found or does not belong to this tenant".to_string())
        })?;

        Ok(location)
    }

    async fn delete_location(&self, tenant_id: Uuid, location_id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE warehouse_locations
            SET deleted_at = NOW(),
                updated_at = NOW()
            WHERE tenant_id = $1
              AND location_id = $2
              AND deleted_at IS NULL
            "#,
            tenant_id,
            location_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
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

impl WarehouseRepositoryImpl {
    fn build_tree_nodes(
        warehouses: Vec<Warehouse>,
        warehouses_by_parent: &std::collections::HashMap<Option<Uuid>, Vec<Warehouse>>,
        zones_by_warehouse: &std::collections::HashMap<Uuid, Vec<WarehouseZone>>,
        locations_by_zone: &std::collections::HashMap<Uuid, Vec<WarehouseLocation>>,
    ) -> Vec<WarehouseTreeNode> {
        warehouses
            .into_iter()
            .map(|warehouse| {
                // Get child warehouses
                let children = Self::build_tree_nodes(
                    warehouses_by_parent
                        .get(&Some(warehouse.warehouse_id))
                        .unwrap_or(&vec![])
                        .clone(),
                    warehouses_by_parent,
                    zones_by_warehouse,
                    locations_by_zone,
                );

                // Get zones for this warehouse
                let zones = zones_by_warehouse
                    .get(&warehouse.warehouse_id)
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|zone| {
                        // Get locations for this zone
                        let locations = locations_by_zone
                            .get(&zone.zone_id)
                            .unwrap_or(&vec![])
                            .iter()
                            .map(|loc| loc.clone().into())
                            .collect();

                        WarehouseZoneWithLocations {
                            zone: zone.clone().into(),
                            locations,
                        }
                    })
                    .collect();

                WarehouseTreeNode {
                    warehouse: warehouse.into(),
                    children,
                    zones,
                }
            })
            .collect()
    }
}
