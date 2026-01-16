//! Valuation service trait
//!
//! Defines the business logic interface for inventory valuation operations.
//! Supports multiple costing methods (FIFO, AVCO, Standard) with cost layer management.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::dto::valuation_dto::{
    CostAdjustmentRequest, DeleteValuationSettingsRequest, EffectiveValuationMethodResponse,
    GetEffectiveValuationMethodRequest, GetTenantValuationSettingsRequest,
    GetValuationHistoryRequest, GetValuationLayersRequest, GetValuationRequest,
    ListValuationSettingsRequest, RevaluationRequest, SetCategoryValuationMethodRequest,
    SetProductValuationMethodRequest, SetStandardCostRequest, SetTenantValuationMethodRequest,
    SetValuationMethodRequest, ValuationDto, ValuationHistoryResponse, ValuationLayersResponse,
    ValuationSettingsDto, ValuationSettingsListResponse,
};
use crate::domains::inventory::valuation::ValuationMethod;
use crate::Result;

/// Service trait for inventory valuation business logic
///
/// This trait defines all business operations for inventory valuation.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait ValuationService: Send + Sync {
    /// Get current valuation for a product
    ///
    /// # Business Rules
    /// - Applies tenant isolation
    /// - Returns current valuation with appropriate cost based on method
    /// - For FIFO/AVCO: returns calculated cost
    /// - For Standard: returns standard cost
    ///
    /// # Arguments
    /// * `request` - Valuation request with tenant and product IDs
    ///
    /// # Returns
    /// Current valuation data
    ///
    /// # Errors
    /// - `NotFound` if product valuation doesn't exist
    async fn get_valuation(&self, request: GetValuationRequest) -> Result<ValuationDto>;

    /// Set valuation method for a product
    ///
    /// # Business Rules
    /// - Validates that product exists
    /// - Creates valuation record if it doesn't exist
    /// - Updates existing valuation method
    /// - Records change in history
    /// - For FIFO: initializes cost layers if needed
    ///
    /// # Arguments
    /// * `request` - Request with tenant, product, and new method
    ///
    /// # Returns
    /// Updated valuation data
    async fn set_valuation_method(
        &self,
        request: SetValuationMethodRequest,
    ) -> Result<ValuationDto>;

    /// Set standard cost for a product
    ///
    /// # Business Rules
    /// - Only applies to products using Standard costing
    /// - Validates cost is positive
    /// - Updates valuation and records in history
    ///
    /// # Arguments
    /// * `request` - Request with tenant, product, and standard cost
    ///
    /// # Returns
    /// Updated valuation data
    ///
    /// # Errors
    /// - `ValidationError` if cost is not positive
    /// - `BusinessError` if product doesn't use standard costing
    async fn set_standard_cost(&self, request: SetStandardCostRequest) -> Result<ValuationDto>;

    /// Get cost layers for FIFO valuation
    ///
    /// # Business Rules
    /// - Only applies to products using FIFO costing
    /// - Returns active cost layers ordered by creation time
    ///
    /// # Arguments
    /// * `request` - Request with tenant and product IDs
    ///
    /// # Returns
    /// List of active cost layers
    async fn get_valuation_layers(
        &self,
        request: GetValuationLayersRequest,
    ) -> Result<ValuationLayersResponse>;

    /// Get valuation history for a product
    ///
    /// # Business Rules
    /// - Returns historical valuation changes
    /// - Supports pagination
    /// - Ordered by change date descending
    ///
    /// # Arguments
    /// * `request` - Request with tenant, product, and pagination
    ///
    /// # Returns
    /// Historical valuation records with pagination
    async fn get_valuation_history(
        &self,
        request: GetValuationHistoryRequest,
    ) -> Result<ValuationHistoryResponse>;

    /// Perform cost adjustment
    ///
    /// # Business Rules
    /// - Adjusts total inventory value
    /// - Records adjustment in history
    /// - Updates current valuation
    /// - Can be positive or negative
    ///
    /// # Arguments
    /// * `request` - Request with adjustment details
    ///
    /// # Returns
    /// Updated valuation data
    async fn adjust_cost(&self, request: CostAdjustmentRequest) -> Result<ValuationDto>;

    /// Revalue inventory at new cost
    ///
    /// # Business Rules
    /// - Changes the cost basis for existing inventory
    /// - Records revaluation in history
    /// - Updates current valuation
    /// - Only applies to current inventory quantity
    ///
    /// # Arguments
    /// * `request` - Request with revaluation details
    ///
    /// # Returns
    /// Updated valuation data
    async fn revalue_inventory(&self, request: RevaluationRequest) -> Result<ValuationDto>;

    /// Process stock movement for valuation
    ///
    /// # Business Rules
    /// - Called when stock moves occur
    /// - Updates valuation based on movement type and method
    /// - For receipts: adds new cost layers (FIFO) or updates average (AVCO)
    /// - For deliveries: consumes cost layers (FIFO) or reduces quantity (AVCO/Standard)
    /// - Records changes in history
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `quantity_change` - Quantity change (positive for receipts, negative for deliveries)
    /// * `unit_cost` - Unit cost of the movement (for receipts)
    /// * `user_id` - User performing the operation
    ///
    /// # Returns
    /// Updated valuation data
    async fn process_stock_movement(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity_change: i64,
        unit_cost: Option<i64>,
        user_id: Option<Uuid>,
    ) -> Result<ValuationDto>;

    /// Calculate current inventory value
    ///
    /// # Business Rules
    /// - Returns total value of current inventory
    /// - Based on current valuation method
    /// - For FIFO: sum of remaining layer values
    /// - For AVCO/Standard: quantity * unit_cost
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Current inventory value in cents
    async fn calculate_inventory_value(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64>;

    /// Get valuation method for a product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Current valuation method
    async fn get_valuation_method(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<ValuationMethod>;

    // ============================================
    // Valuation Settings Methods
    // ============================================

    /// Get tenant default valuation settings
    ///
    /// # Business Rules
    /// - Returns the default valuation method for the tenant
    /// - Returns NotFound error if no default exists
    /// - Use set_tenant_valuation_method to create initial settings
    ///
    /// # Arguments
    /// * `request` - Request with tenant ID
    ///
    /// # Returns
    /// Tenant default valuation settings
    ///
    /// # Errors
    /// - `NotFound` if tenant has no default valuation settings
    async fn get_tenant_valuation_settings(
        &self,
        request: GetTenantValuationSettingsRequest,
    ) -> Result<ValuationSettingsDto>;

    /// Get effective valuation method for a product
    ///
    /// # Business Rules
    /// - Checks for product-level override first
    /// - Then checks for category-level override
    /// - Falls back to tenant default
    /// - Returns the method and its source
    ///
    /// # Arguments
    /// * `request` - Request with tenant, product, and optional category IDs
    ///
    /// # Returns
    /// Effective valuation method with source information
    async fn get_effective_valuation_method(
        &self,
        request: GetEffectiveValuationMethodRequest,
    ) -> Result<EffectiveValuationMethodResponse>;

    /// Set tenant default valuation method
    ///
    /// # Business Rules
    /// - Creates or updates the tenant default
    /// - Does not affect existing product-level settings
    ///
    /// # Arguments
    /// * `request` - Request with tenant ID and method
    ///
    /// # Returns
    /// Updated settings
    async fn set_tenant_valuation_method(
        &self,
        request: SetTenantValuationMethodRequest,
    ) -> Result<ValuationSettingsDto>;

    /// Set category valuation method override
    ///
    /// # Business Rules
    /// - Creates or updates a category-level override
    /// - Products in this category will use this method unless overridden
    ///
    /// # Arguments
    /// * `request` - Request with tenant, category, and method
    ///
    /// # Returns
    /// Updated settings
    async fn set_category_valuation_method(
        &self,
        request: SetCategoryValuationMethodRequest,
    ) -> Result<ValuationSettingsDto>;

    /// Set product valuation method override
    ///
    /// # Business Rules
    /// - Creates or updates a product-level override
    /// - Takes precedence over category and tenant settings
    ///
    /// # Arguments
    /// * `request` - Request with tenant, product, and method
    ///
    /// # Returns
    /// Updated settings
    async fn set_product_valuation_method(
        &self,
        request: SetProductValuationMethodRequest,
    ) -> Result<ValuationSettingsDto>;

    /// Delete valuation settings override
    ///
    /// # Business Rules
    /// - Removes the specified override
    /// - Cannot delete tenant default (returns error)
    /// - Only category and product overrides can be deleted
    ///
    /// # Arguments
    /// * `request` - Request with scope details
    ///
    /// # Returns
    /// Success or error
    ///
    /// # Errors
    /// - `BusinessError` if attempting to delete tenant default
    async fn delete_valuation_settings(
        &self,
        request: DeleteValuationSettingsRequest,
    ) -> Result<()>;

    /// List all valuation settings for a tenant
    ///
    /// # Business Rules
    /// - Returns all settings including tenant default and overrides
    /// - Can optionally filter by scope type
    ///
    /// # Arguments
    /// * `request` - Request with tenant ID and optional scope filter
    ///
    /// # Returns
    /// List of valuation settings
    async fn list_valuation_settings(
        &self,
        request: ListValuationSettingsRequest,
    ) -> Result<ValuationSettingsListResponse>;
}
