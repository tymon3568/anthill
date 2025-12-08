use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::dto::picking_method_dto::{
    ConfirmPickingPlanRequest, CreatePickingMethodRequest, PickingOptimizationRequest,
    PickingPlanResponse, UpdatePickingMethodRequest,
};
use crate::domains::inventory::picking_method::PickingMethod;
use crate::Result;

/// Service trait for picking method business logic
///
/// This trait defines all business operations for picking methods.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait PickingMethodService: Send + Sync {
    // ========================================================================
    // Picking Method Management
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
    async fn create_method(
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
    async fn get_method(&self, tenant_id: Uuid, method_id: Uuid) -> Result<Option<PickingMethod>>;

    /// Get all picking methods for a warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    ///
    /// # Returns
    /// List of picking methods
    async fn get_methods_by_warehouse(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<PickingMethod>>;

    /// Get active picking methods for a warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    ///
    /// # Returns
    /// List of active picking methods
    async fn get_active_methods_by_warehouse(
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
    async fn get_default_method_by_warehouse(
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
    async fn update_method(
        &self,
        tenant_id: Uuid,
        method_id: Uuid,
        request: UpdatePickingMethodRequest,
        updated_by: Uuid,
    ) -> Result<PickingMethod>;

    /// Delete picking method
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `method_id` - Picking method to delete
    /// * `deleted_by` - User performing the deletion
    ///
    /// # Returns
    /// Success status
    async fn delete_method(
        &self,
        tenant_id: Uuid,
        method_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<bool>;

    /// Set default picking method for warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `method_id` - Method to set as default (warehouse_id derived internally)
    /// * `updated_by` - User making the change
    ///
    /// # Returns
    /// Success status
    async fn set_default_method(
        &self,
        tenant_id: Uuid,
        method_id: Uuid,
        updated_by: Uuid,
    ) -> Result<bool>;

    // ========================================================================
    // Picking Optimization
    // ========================================================================

    /// Generate optimized picking plan
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `request` - Optimization request parameters
    ///
    /// # Returns
    /// Optimized picking plan
    async fn optimize_picking(
        &self,
        tenant_id: Uuid,
        request: PickingOptimizationRequest,
    ) -> Result<PickingPlanResponse>;

    /// Confirm and execute picking plan
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `request` - Confirmation request
    /// * `confirmed_by` - User confirming the plan
    ///
    /// # Returns
    /// Success status
    async fn confirm_picking_plan(
        &self,
        tenant_id: Uuid,
        request: ConfirmPickingPlanRequest,
        confirmed_by: Uuid,
    ) -> Result<bool>;

    // ========================================================================
    // Batch Picking Operations
    // ========================================================================

    /// Generate batch picking plan
    ///
    /// Groups multiple orders into single picking runs based on product locations
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    /// * `order_ids` - Orders to include in batch
    /// * `batch_config` - Batch-specific configuration
    /// * `method_id` - Picking method identifier
    ///
    /// # Returns
    /// Optimized batch picking plan
    async fn generate_batch_picking_plan(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        order_ids: Vec<Uuid>,
        batch_config: serde_json::Value,
        method_id: Uuid,
    ) -> Result<PickingPlanResponse>;

    // ========================================================================
    // Cluster Picking Operations
    // ========================================================================

    /// Generate cluster picking plan
    ///
    /// Allows pickers to handle multiple orders simultaneously with sorting later
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    /// * `order_ids` - Orders to include in cluster
    /// * `cluster_config` - Cluster-specific configuration
    /// * `method_id` - Picking method identifier
    ///
    /// # Returns
    /// Optimized cluster picking plan
    async fn generate_cluster_picking_plan(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        order_ids: Vec<Uuid>,
        cluster_config: serde_json::Value,
        method_id: Uuid,
    ) -> Result<PickingPlanResponse>;

    // ========================================================================
    // Wave Picking Operations
    // ========================================================================

    /// Generate wave picking plan
    ///
    /// Creates picking waves based on time slots or order priorities
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    /// * `order_ids` - Orders to include in wave
    /// * `wave_config` - Wave-specific configuration
    /// * `method_id` - Picking method identifier
    ///
    /// # Returns
    /// Optimized wave picking plan
    async fn generate_wave_picking_plan(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        order_ids: Vec<Uuid>,
        wave_config: serde_json::Value,
        method_id: Uuid,
    ) -> Result<PickingPlanResponse>;

    // ========================================================================
    // Validation and Analytics
    // ========================================================================

    /// Validate picking method configuration
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `method_id` - Method to validate
    ///
    /// # Returns
    /// Validation result (true if valid)
    async fn validate_method(&self, tenant_id: Uuid, method_id: Uuid) -> Result<bool>;

    /// Get picking method performance metrics
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `method_id` - Method to analyze
    /// * `date_range` - Date range for analysis
    ///
    /// # Returns
    /// Performance metrics as JSON
    async fn get_method_performance(
        &self,
        tenant_id: Uuid,
        method_id: Uuid,
        date_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    ) -> Result<Option<serde_json::Value>>;
}
