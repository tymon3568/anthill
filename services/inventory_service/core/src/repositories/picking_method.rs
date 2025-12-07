//! Picking method repository trait
//!
//! Defines the data access interface for picking method operations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::dto::picking_method_dto::{
    CreatePickingMethodRequest, PickingOptimizationRequest, PickingPlanResponse,
    UpdatePickingMethodRequest,
};
use crate::domains::inventory::picking_method::PickingMethod;
use crate::Result;

/// Repository trait for picking method data access
///
/// This trait defines all database operations for picking methods.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait PickingMethodRepository: Send + Sync {
    // ========================================================================
    // CRUD Operations for Picking Methods
    // ========================================================================

    /// Create a new picking method
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `request` - Picking method creation data
    /// * `created_by` - User creating the method
    ///
    /// # Returns
    /// Created picking method
    async fn create(
        &self,
        tenant_id: Uuid,
        request: CreatePickingMethodRequest,
        created_by: Uuid,
    ) -> Result<PickingMethod>;

    /// Get picking method by ID
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `method_id` - Picking method identifier
    ///
    /// # Returns
    /// Picking method if found
    async fn find_by_id(&self, tenant_id: Uuid, method_id: Uuid) -> Result<Option<PickingMethod>>;

    /// Get picking method by name for a warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    /// * `name` - Method name
    ///
    /// # Returns
    /// Picking method if found
    async fn find_by_name(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        name: &str,
    ) -> Result<Option<PickingMethod>>;

    /// Get all picking methods for a warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    ///
    /// # Returns
    /// List of picking methods
    async fn find_by_warehouse(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<PickingMethod>>;

    /// Get all active picking methods for a warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    ///
    /// # Returns
    /// List of active picking methods
    async fn find_active_by_warehouse(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<PickingMethod>>;

    /// Get default picking method for a warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    ///
    /// # Returns
    /// Default picking method if found
    async fn find_default_by_warehouse(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Option<PickingMethod>>;

    /// Update picking method
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `method_id` - Picking method to update
    /// * `request` - Update data
    /// * `updated_by` - User updating the method
    ///
    /// # Returns
    /// Updated picking method
    async fn update(
        &self,
        tenant_id: Uuid,
        method_id: Uuid,
        request: UpdatePickingMethodRequest,
        updated_by: Uuid,
    ) -> Result<PickingMethod>;

    /// Delete picking method (soft delete)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `method_id` - Picking method to delete
    ///
    /// # Returns
    /// Success status
    async fn delete(&self, tenant_id: Uuid, method_id: Uuid) -> Result<bool>;

    /// Set default picking method for warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    /// * `method_id` - Method to set as default
    /// * `updated_by` - User making the change
    ///
    /// # Returns
    /// Success status
    async fn set_default(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        method_id: Uuid,
        updated_by: Uuid,
    ) -> Result<bool>;

    // ========================================================================
    // Picking Optimization Operations
    // ========================================================================

    /// Generate optimized picking plan
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `request` - Optimization request parameters
    ///
    /// # Returns
    /// Optimized picking plan
    async fn generate_picking_plan(
        &self,
        tenant_id: Uuid,
        request: PickingOptimizationRequest,
    ) -> Result<PickingPlanResponse>;

    /// Validate picking method configuration
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `method_id` - Method to validate
    ///
    /// # Returns
    /// Validation result (true if valid)
    async fn validate_method_config(&self, tenant_id: Uuid, method_id: Uuid) -> Result<bool>;
}
