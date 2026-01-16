//! Valuation repository traits
//!
//! Defines data access interfaces for inventory valuation operations.
//! Supports multiple valuation methods (FIFO, AVCO, Standard) with cost layer management.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::valuation::{
    Valuation, ValuationHistory, ValuationLayer, ValuationMethod, ValuationScopeType,
    ValuationSettings,
};
use crate::Result;

/// Repository trait for inventory valuation data access
#[async_trait]
pub trait ValuationRepository: Send + Sync {
    /// Get current valuation for a product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Current valuation if exists
    async fn find_by_product_id(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<Option<Valuation>>;

    /// Create new valuation record
    ///
    /// # Arguments
    /// * `valuation` - Valuation to create
    ///
    /// # Returns
    /// Created valuation
    async fn create(&self, valuation: &Valuation) -> Result<Valuation>;

    /// Update existing valuation
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    /// * `valuation` - Updated valuation data
    ///
    /// # Returns
    /// Updated valuation
    async fn update(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        valuation: &Valuation,
    ) -> Result<Valuation>;

    /// Set valuation method for a product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    /// * `method` - New valuation method
    /// * `updated_by` - User making the change
    ///
    /// # Returns
    /// Updated valuation
    async fn set_valuation_method(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        method: ValuationMethod,
        updated_by: Option<Uuid>,
    ) -> Result<Valuation>;

    /// Set standard cost for a product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    /// * `standard_cost` - Standard cost in cents
    /// * `updated_by` - User making the change
    ///
    /// # Returns
    /// Updated valuation
    async fn set_standard_cost(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        standard_cost: i64,
        updated_by: Option<Uuid>,
    ) -> Result<Valuation>;

    /// Update valuation from stock movement
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    /// * `quantity_change` - Quantity change (positive for receipts, negative for deliveries)
    /// * `unit_cost` - Unit cost of the movement (for receipts)
    /// * `updated_by` - User making the change
    ///
    /// # Returns
    /// Updated valuation
    async fn update_from_stock_move(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity_change: i64,
        unit_cost: Option<i64>,
        updated_by: Option<Uuid>,
    ) -> Result<Valuation>;

    /// Perform cost adjustment
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    /// * `adjustment_amount` - Adjustment amount in cents
    /// * `reason` - Reason for adjustment
    /// * `updated_by` - User making the adjustment
    ///
    /// # Returns
    /// Updated valuation
    async fn adjust_cost(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        adjustment_amount: i64,
        reason: &str,
        updated_by: Option<Uuid>,
    ) -> Result<Valuation>;

    /// Revalue inventory at new cost
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    /// * `new_unit_cost` - New unit cost in cents
    /// * `reason` - Reason for revaluation
    /// * `updated_by` - User making the revaluation
    ///
    /// # Returns
    /// Updated valuation
    async fn revalue_inventory(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        new_unit_cost: i64,
        reason: &str,
        updated_by: Option<Uuid>,
    ) -> Result<Valuation>;
}

/// Repository trait for valuation layer data access (FIFO)
#[async_trait]
pub trait ValuationLayerRepository: Send + Sync {
    /// Get all active cost layers for a product (ordered by creation time)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// List of active cost layers
    async fn find_active_by_product_id(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<Vec<ValuationLayer>>;

    /// Create new cost layer
    ///
    /// # Arguments
    /// * `layer` - Cost layer to create
    ///
    /// # Returns
    /// Created cost layer
    async fn create(&self, layer: &ValuationLayer) -> Result<ValuationLayer>;

    /// Consume quantity from cost layers (FIFO)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    /// * `quantity_to_consume` - Quantity to consume
    ///
    /// # Returns
    /// Total cost of consumed quantity
    async fn consume_layers(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity_to_consume: i64,
    ) -> Result<i64>;

    /// Get total remaining quantity in layers
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Total remaining quantity
    async fn get_total_quantity(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64>;

    /// Clean up empty layers
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Number of layers cleaned up
    async fn cleanup_empty_layers(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64>;
}

/// Repository trait for valuation history data access
#[async_trait]
pub trait ValuationHistoryRepository: Send + Sync {
    /// Get valuation history for a product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    /// * `limit` - Maximum number of records to return
    /// * `offset` - Number of records to skip
    ///
    /// # Returns
    /// List of historical valuation records
    async fn find_by_product_id(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ValuationHistory>>;

    /// Create new history record
    ///
    /// # Arguments
    /// * `history` - History record to create
    ///
    /// # Returns
    /// Created history record
    async fn create(&self, history: &ValuationHistory) -> Result<ValuationHistory>;

    /// Get history count for a product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Total number of history records
    async fn count_by_product_id(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64>;
}

/// Repository trait for valuation settings data access
#[async_trait]
pub trait ValuationSettingsRepository: Send + Sync {
    /// Get tenant default valuation settings
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    ///
    /// # Returns
    /// Tenant default settings if exists
    async fn find_tenant_default(&self, tenant_id: Uuid) -> Result<Option<ValuationSettings>>;

    /// Get settings by scope
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `scope_type` - Scope type (tenant, category, product)
    /// * `scope_id` - Scope ID (None for tenant, Some for category/product)
    ///
    /// # Returns
    /// Settings if exists
    async fn find_by_scope(
        &self,
        tenant_id: Uuid,
        scope_type: ValuationScopeType,
        scope_id: Option<Uuid>,
    ) -> Result<Option<ValuationSettings>>;

    /// Create new valuation settings
    ///
    /// # Arguments
    /// * `settings` - Settings to create
    ///
    /// # Returns
    /// Created settings
    async fn create(&self, settings: &ValuationSettings) -> Result<ValuationSettings>;

    /// Update existing valuation settings
    ///
    /// # Arguments
    /// * `settings` - Updated settings
    ///
    /// # Returns
    /// Updated settings
    async fn update(&self, settings: &ValuationSettings) -> Result<ValuationSettings>;

    /// Upsert valuation settings (create or update)
    ///
    /// # Arguments
    /// * `settings` - Settings to upsert
    ///
    /// # Returns
    /// Upserted settings
    async fn upsert(&self, settings: &ValuationSettings) -> Result<ValuationSettings>;

    /// Delete valuation settings
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `scope_type` - Scope type
    /// * `scope_id` - Scope ID
    ///
    /// # Returns
    /// Success or error
    async fn delete(
        &self,
        tenant_id: Uuid,
        scope_type: ValuationScopeType,
        scope_id: Option<Uuid>,
    ) -> Result<()>;

    /// List all settings for a tenant
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `scope_type` - Optional filter by scope type
    ///
    /// # Returns
    /// List of settings
    async fn list_by_tenant(
        &self,
        tenant_id: Uuid,
        scope_type: Option<ValuationScopeType>,
    ) -> Result<Vec<ValuationSettings>>;
}
