//! Valuation service trait
//!
//! Defines the business logic interface for inventory valuation operations.
//! Supports multiple costing methods (FIFO, AVCO, Standard) with cost layer management.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::dto::valuation_dto::{
    CostAdjustmentRequest, GetValuationHistoryRequest, GetValuationLayersRequest,
    GetValuationRequest, RevaluationRequest, SetStandardCostRequest, SetValuationMethodRequest,
    ValuationDto, ValuationHistoryResponse, ValuationLayersResponse,
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
}
