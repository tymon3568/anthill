//! Removal strategy repository trait
//!
//! Defines the data access interface for removal strategy operations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::removal_strategy::RemovalStrategy;
use crate::dto::removal_strategy::{
    RemovalStrategyCreateRequest, RemovalStrategyListQuery, RemovalStrategyUpdateRequest,
    StockLocationInfo, SuggestRemovalRequest, SuggestRemovalResponse,
};
use crate::Result;

/// Repository trait for removal strategy data access
///
/// This trait defines all database operations for removal strategies.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait RemovalStrategyRepository: Send + Sync {
    // ========================================================================
    // CRUD Operations for Removal Strategies
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
    async fn create(
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
    async fn find_by_id(
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
    async fn find_by_name(&self, tenant_id: Uuid, name: &str) -> Result<Option<RemovalStrategy>>;

    /// List removal strategies with filtering and pagination
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `query` - Query parameters for filtering and pagination
    ///
    /// # Returns
    /// List of removal strategies with pagination info
    async fn list(
        &self,
        tenant_id: Uuid,
        query: RemovalStrategyListQuery,
    ) -> Result<(Vec<RemovalStrategy>, u64)>;

    /// Get active removal strategies for a warehouse and product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `warehouse_id` - Warehouse identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// List of applicable active strategies
    async fn find_active_for_scope(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<Vec<RemovalStrategy>>;

    /// Update removal strategy
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `strategy_id` - Removal strategy to update
    /// * `request` - Update data
    /// * `updated_by` - User updating the strategy
    ///
    /// # Returns
    /// Updated removal strategy
    async fn update(
        &self,
        tenant_id: Uuid,
        strategy_id: Uuid,
        request: RemovalStrategyUpdateRequest,
        updated_by: Uuid,
    ) -> Result<RemovalStrategy>;

    /// Delete removal strategy (soft delete)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `strategy_id` - Removal strategy to delete
    ///
    /// # Returns
    /// Success status
    async fn delete(&self, tenant_id: Uuid, strategy_id: Uuid, deleted_by: Uuid) -> Result<bool>;

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
    async fn toggle_active(
        &self,
        tenant_id: Uuid,
        strategy_id: Uuid,
        active: bool,
        updated_by: Uuid,
    ) -> Result<RemovalStrategy>;

    // ========================================================================
    // Removal Suggestion Operations
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
}
