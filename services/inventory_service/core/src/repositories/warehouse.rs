//! Warehouse repository trait
//!
//! Defines the data access interface for warehouse operations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::dto::warehouse_dto::{
    CreateWarehouseLocationRequest, CreateWarehouseRequest, CreateWarehouseZoneRequest,
    WarehouseTreeResponse,
};
use crate::domains::inventory::warehouse::Warehouse;
use crate::domains::inventory::warehouse_location::WarehouseLocation;
use crate::domains::inventory::warehouse_zone::WarehouseZone;
use crate::Result;

/// Repository trait for warehouse data access
///
/// This trait defines all database operations for warehouses.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait WarehouseRepository: Send + Sync {
    // ========================================================================
    // CRUD Operations
    // ========================================================================

    /// Create a new warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `request` - Warehouse creation data
    ///
    /// # Returns
    /// Created warehouse
    async fn create(&self, tenant_id: Uuid, request: CreateWarehouseRequest) -> Result<Warehouse>;

    /// Get warehouse by ID
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    ///
    /// # Returns
    /// Warehouse if found
    async fn find_by_id(&self, tenant_id: Uuid, warehouse_id: Uuid) -> Result<Option<Warehouse>>;

    /// Get warehouse by code
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_code` - Warehouse code
    ///
    /// # Returns
    /// Warehouse if found
    async fn find_by_code(
        &self,
        tenant_id: Uuid,
        warehouse_code: &str,
    ) -> Result<Option<Warehouse>>;

    /// Get all warehouses for a tenant
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    ///
    /// # Returns
    /// List of warehouses
    async fn find_all(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>>;

    /// Get warehouse hierarchy/tree
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    ///
    /// # Returns
    /// Hierarchical warehouse structure
    async fn get_warehouse_tree(&self, tenant_id: Uuid) -> Result<WarehouseTreeResponse>;

    /// Update warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse to update
    /// * `warehouse` - Updated warehouse data
    ///
    /// # Returns
    /// Updated warehouse
    async fn update(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        warehouse: &Warehouse,
    ) -> Result<Warehouse>;

    /// Delete warehouse (soft delete)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse to delete
    ///
    /// # Returns
    /// Success status
    async fn delete(&self, tenant_id: Uuid, warehouse_id: Uuid) -> Result<bool>;

    // ========================================================================
    // Hierarchy Operations
    // ========================================================================

    /// Get child warehouses
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `parent_warehouse_id` - Parent warehouse ID
    ///
    /// # Returns
    /// List of child warehouses
    async fn get_children(
        &self,
        tenant_id: Uuid,
        parent_warehouse_id: Uuid,
    ) -> Result<Vec<Warehouse>>;

    /// Get warehouse path (ancestors)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse ID
    ///
    /// # Returns
    /// List of ancestor warehouses (ordered from root to parent)
    async fn get_ancestors(&self, tenant_id: Uuid, warehouse_id: Uuid) -> Result<Vec<Warehouse>>;

    /// Get all descendants of a warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse ID
    ///
    /// # Returns
    /// List of descendant warehouses
    async fn get_descendants(&self, tenant_id: Uuid, warehouse_id: Uuid) -> Result<Vec<Warehouse>>;

    /// Get all zones for a tenant
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    ///
    /// # Returns
    /// List of warehouse zones
    async fn get_all_zones(&self, tenant_id: Uuid) -> Result<Vec<WarehouseZone>>;

    /// Get all locations for a tenant
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    ///
    /// # Returns
    /// List of warehouse locations
    async fn get_all_locations(&self, tenant_id: Uuid) -> Result<Vec<WarehouseLocation>>;

    /// Create a new zone in a warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse ID
    /// * `request` - Zone creation data
    ///
    /// # Returns
    /// Created zone
    async fn create_zone(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        request: CreateWarehouseZoneRequest,
    ) -> Result<WarehouseZone>;

    /// Create a new location in a warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse ID
    /// * `request` - Location creation data
    ///
    /// # Returns
    /// Created location
    async fn create_location(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        request: CreateWarehouseLocationRequest,
    ) -> Result<WarehouseLocation>;

    /// Check if warehouse hierarchy is valid (no cycles)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse ID to check
    /// * `parent_warehouse_id` - Proposed parent ID
    ///
    /// # Returns
    /// True if hierarchy would be valid
    async fn validate_hierarchy(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        parent_warehouse_id: Option<Uuid>,
    ) -> Result<bool>;

    // ========================================================================
    // Capacity and Analytics
    // ========================================================================

    /// Get warehouse capacity utilization
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse ID
    ///
    /// # Returns
    /// Capacity utilization data (JSON)
    async fn get_capacity_utilization(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Option<serde_json::Value>>;

    /// Get warehouse statistics
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse ID
    ///
    /// # Returns
    /// Warehouse statistics (JSON)
    async fn get_warehouse_stats(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Option<serde_json::Value>>;
}
