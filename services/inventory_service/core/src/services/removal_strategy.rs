use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::removal_strategy::RemovalStrategy;
use crate::dto::removal_strategy::{
    RemovalStrategyCreateRequest, RemovalStrategyListQuery, RemovalStrategyListResponse,
    RemovalStrategyUpdateRequest, StrategyAnalyticsResponse, SuggestRemovalRequest,
    SuggestRemovalResponse,
};
use crate::Result;

/// Service trait for removal strategy business logic
///
/// This trait defines all business operations for removal strategies.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait RemovalStrategyService: Send + Sync {
    // ========================================================================
    // Removal Strategy Management
    // ========================================================================

    /// Create a new removal strategy
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `request` - Removal strategy creation data
    /// * `created_by` - User creating the strategy
    ///
    /// # Returns
    /// Created removal strategy
    async fn create_strategy(
        &self,
        tenant_id: Uuid,
        request: RemovalStrategyCreateRequest,
        created_by: Uuid,
    ) -> Result<RemovalStrategy>;

    /// Get removal strategy by ID
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `strategy_id` - Removal strategy identifier
    ///
    /// # Returns
    /// Removal strategy if found
    async fn get_strategy(
        &self,
        tenant_id: Uuid,
        strategy_id: Uuid,
    ) -> Result<Option<RemovalStrategy>>;

    /// Get removal strategy by name
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `name` - Strategy name
    ///
    /// # Returns
    /// Removal strategy if found
    async fn get_strategy_by_name(
        &self,
        tenant_id: Uuid,
        name: &str,
    ) -> Result<Option<RemovalStrategy>>;

    /// List removal strategies with filtering and pagination
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `query` - Query parameters
    ///
    /// # Returns
    /// List response with strategies and pagination
    async fn list_strategies(
        &self,
        tenant_id: Uuid,
        query: RemovalStrategyListQuery,
    ) -> Result<RemovalStrategyListResponse>;

    /// Update removal strategy
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `strategy_id` - Strategy to update
    /// * `request` - Update data
    /// * `updated_by` - User updating the strategy
    ///
    /// # Returns
    /// Updated removal strategy
    async fn update_strategy(
        &self,
        tenant_id: Uuid,
        strategy_id: Uuid,
        request: RemovalStrategyUpdateRequest,
        updated_by: Uuid,
    ) -> Result<RemovalStrategy>;

    /// Delete removal strategy
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `strategy_id` - Strategy to delete
    /// * `deleted_by` - User performing the deletion
    ///
    /// # Returns
    /// Success status
    async fn delete_strategy(
        &self,
        tenant_id: Uuid,
        strategy_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<bool>;

    /// Toggle active status of removal strategy
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `strategy_id` - Strategy to toggle
    /// * `active` - New active status
    /// * `updated_by` - User making the change
    ///
    /// # Returns
    /// Updated strategy
    async fn toggle_strategy_active(
        &self,
        tenant_id: Uuid,
        strategy_id: Uuid,
        active: bool,
        updated_by: Uuid,
    ) -> Result<RemovalStrategy>;

    // ========================================================================
    // Removal Suggestion Engine
    // ========================================================================

    /// Suggest optimal stock locations for removal/picking
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `request` - Suggestion request parameters
    ///
    /// # Returns
    /// Suggested stock locations and quantities
    async fn suggest_removal(
        &self,
        tenant_id: Uuid,
        request: SuggestRemovalRequest,
    ) -> Result<SuggestRemovalResponse>;

    /// Get available stock locations for a product in warehouse
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// List of available stock locations with quantities
    async fn get_available_stock_locations(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<Vec<StockLocationInfo>>;

    // ========================================================================
    // Strategy Selection and Validation
    // ========================================================================

    /// Get applicable strategies for a warehouse and product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// List of applicable active strategies
    async fn get_applicable_strategies(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<Vec<RemovalStrategy>>;

    /// Select best strategy for given conditions
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    /// * `product_id` - Product identifier
    /// * `strategies` - Available strategies to choose from
    ///
    /// # Returns
    /// Selected strategy with reasoning
    async fn select_best_strategy(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        strategies: Vec<RemovalStrategy>,
    ) -> Result<(RemovalStrategy, String)>;

    /// Validate strategy configuration
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `strategy_id` - Strategy to validate
    ///
    /// # Returns
    /// Validation result (true if valid)
    async fn validate_strategy(&self, tenant_id: Uuid, strategy_id: Uuid) -> Result<bool>;

    // ========================================================================
    // Analytics and Performance
    // ========================================================================

    /// Record strategy usage for analytics
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `strategy_id` - Strategy that was used
    /// * `product_id` - Product being picked
    /// * `quantity` - Quantity picked
    /// * `pick_time_seconds` - Time taken for picking (optional)
    ///
    /// # Returns
    /// Success status
    async fn record_strategy_usage(
        &self,
        tenant_id: Uuid,
        strategy_id: Uuid,
        product_id: Uuid,
        quantity: i64,
        pick_time_seconds: Option<f64>,
    ) -> Result<bool>;

    /// Get strategy performance analytics
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `strategy_id` - Strategy to analyze (None for all)
    /// * `period_start` - Analysis period start
    /// * `period_end` - Analysis period end
    ///
    /// # Returns
    /// Performance analytics
    async fn get_strategy_analytics(
        &self,
        tenant_id: Uuid,
        strategy_id: Option<Uuid>,
        period_start: chrono::DateTime<chrono::Utc>,
        period_end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<StrategyAnalyticsResponse>>;
}

/// Information about available stock in a location
#[derive(Debug, Clone)]
pub struct StockLocationInfo {
    pub location_id: Uuid,
    pub location_code: String,
    pub available_quantity: i64,
    pub lot_serial_id: Option<Uuid>,
    pub expiry_date: Option<chrono::DateTime<chrono::Utc>>,
    pub last_receipt_date: Option<chrono::DateTime<chrono::Utc>>,
}
