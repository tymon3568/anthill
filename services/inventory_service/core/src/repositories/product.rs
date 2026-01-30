//! Product repository trait
//!
//! Defines the data access interface for product operations.
//! This trait abstracts database operations for products.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::dto::search_dto::{
    ProductSearchRequest, ProductSearchResponse, SearchSuggestionsRequest,
    SearchSuggestionsResponse,
};
use crate::domains::inventory::product::Product;
use crate::Result;

/// Repository trait for product data access
///
/// This trait defines all database operations for products.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait ProductRepository: Send + Sync {
    // ========================================================================
    // Search Operations
    // ========================================================================

    /// Advanced product search with full-text search and filtering
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `request` - Search request with filters and pagination
    ///
    /// # Returns
    /// Search results with pagination and facets
    async fn search_products(
        &self,
        tenant_id: Uuid,
        request: ProductSearchRequest,
    ) -> Result<ProductSearchResponse>;

    /// Get search suggestions/autocomplete
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `request` - Suggestions request
    ///
    /// # Returns
    /// List of search suggestions
    async fn get_search_suggestions(
        &self,
        tenant_id: Uuid,
        request: SearchSuggestionsRequest,
    ) -> Result<SearchSuggestionsResponse>;

    // ========================================================================
    // CRUD Operations (Future)
    // ========================================================================

    /// Get product by ID
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Product if found
    async fn find_by_id(&self, tenant_id: Uuid, product_id: Uuid) -> Result<Option<Product>>;

    /// Get multiple products by IDs
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_ids` - List of product identifiers
    ///
    /// # Returns
    /// List of products found (may be less than requested if some not found)
    async fn find_by_ids(&self, tenant_id: Uuid, product_ids: &[Uuid]) -> Result<Vec<Product>>;

    /// Get product by SKU
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `sku` - Product SKU
    ///
    /// # Returns
    /// Product if found
    async fn find_by_sku(&self, tenant_id: Uuid, sku: &str) -> Result<Option<Product>>;

    /// Get product by barcode
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `barcode` - Product barcode
    ///
    /// # Returns
    /// Product if found (searches both products.attributes and product_variants.barcode)
    async fn find_by_barcode(&self, tenant_id: Uuid, barcode: &str) -> Result<Option<Product>>;

    /// Create new product
    ///
    /// # Arguments
    /// * `product` - Product to create
    ///
    /// # Returns
    /// Created product
    async fn create(&self, product: &Product) -> Result<Product>;

    /// Update existing product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product to update
    /// * `product` - Updated product data
    ///
    /// # Returns
    /// Updated product
    async fn update(&self, tenant_id: Uuid, product_id: Uuid, product: &Product)
        -> Result<Product>;

    /// Delete product (soft delete)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product to delete
    ///
    /// # Returns
    /// Success status
    async fn delete(&self, tenant_id: Uuid, product_id: Uuid) -> Result<bool>;

    // ========================================================================
    // Analytics and Statistics (Future)
    // ========================================================================

    /// Record search analytics
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `query` - Search query
    /// * `result_count` - Number of results returned
    /// * `user_id` - Optional user identifier
    ///
    /// # Returns
    /// Success status
    async fn record_search_analytics(
        &self,
        tenant_id: Uuid,
        query: &str,
        result_count: u32,
        user_id: Option<Uuid>,
    ) -> Result<()>;

    /// Get popular search terms
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `limit` - Maximum number of terms to return
    ///
    /// # Returns
    /// List of popular search terms with counts
    async fn get_popular_search_terms(
        &self,
        tenant_id: Uuid,
        limit: u32,
    ) -> Result<Vec<(String, u32)>>;

    // ========================================================================
    // Inventory Operations (Future)
    // ========================================================================

    /// Check if product is in stock
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// True if product has available inventory
    async fn is_in_stock(&self, tenant_id: Uuid, product_id: Uuid) -> Result<bool>;

    /// Get current inventory level for product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Current inventory quantity
    async fn get_inventory_level(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64>;

    // ========================================================================
    // Bulk Operations
    // ========================================================================

    /// Bulk activate products
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_ids` - List of product IDs to activate
    ///
    /// # Returns
    /// Number of products activated
    async fn bulk_activate(&self, tenant_id: Uuid, product_ids: &[Uuid]) -> Result<i64>;

    /// Bulk deactivate products
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_ids` - List of product IDs to deactivate
    ///
    /// # Returns
    /// Number of products deactivated
    async fn bulk_deactivate(&self, tenant_id: Uuid, product_ids: &[Uuid]) -> Result<i64>;

    /// Bulk delete products (soft delete)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `product_ids` - List of product IDs to delete
    ///
    /// # Returns
    /// Number of products deleted
    async fn bulk_delete(&self, tenant_id: Uuid, product_ids: &[Uuid]) -> Result<i64>;

    // ========================================================================
    // Import/Export Operations
    // ========================================================================

    /// Save a product (insert or update)
    ///
    /// # Arguments
    /// * `product` - Product to save
    ///
    /// # Returns
    /// Success status
    async fn save(&self, product: &Product) -> Result<()>;

    /// Find all products for export with optional filters
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `category_id` - Optional category filter
    /// * `product_type` - Optional product type filter
    /// * `is_active` - Optional active status filter
    /// * `search` - Optional search term for SKU/name
    ///
    /// # Returns
    /// List of products matching the filters
    async fn find_all_for_export(
        &self,
        tenant_id: Uuid,
        category_id: Option<Uuid>,
        product_type: Option<&str>,
        is_active: Option<bool>,
        search: Option<&str>,
    ) -> Result<Vec<Product>>;
}
