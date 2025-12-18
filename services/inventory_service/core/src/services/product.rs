//! Product service trait
//!
//! Defines the business logic interface for product operations.
//! This trait coordinates between repositories and implements business rules.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::dto::search_dto::{
    ProductSearchRequest, ProductSearchResponse, SearchSuggestionsRequest,
    SearchSuggestionsResponse,
};
use crate::domains::inventory::product::Product;
use crate::Result;

/// Service trait for product business logic
///
/// This trait defines all business operations for products.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait ProductService: Send + Sync {
    // ========================================================================
    // Search Operations
    // ========================================================================

    /// Advanced product search with filtering and pagination
    ///
    /// # Business Rules
    /// - Applies tenant isolation
    /// - Supports full-text search on name and description
    /// - Filters by category hierarchy
    /// - Applies price range filtering
    /// - Filters by product type and status
    /// - Supports multiple sorting options
    /// - Returns paginated results with facets
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `request` - Search request with filters and pagination
    ///
    /// # Returns
    /// Search results with pagination and facets
    ///
    /// # Errors
    /// - `ValidationError` if request validation fails
    async fn search_products(
        &self,
        tenant_id: Uuid,
        request: ProductSearchRequest,
    ) -> Result<ProductSearchResponse>;

    /// Get search suggestions/autocomplete
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
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

    /// Create a new product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `request` - Product creation data
    ///
    /// # Returns
    /// Created product
    ///
    /// # Errors
    /// - `ValidationError` if request validation fails
    /// - `Conflict` if SKU already exists
    async fn create_product(
        &self,
        tenant_id: Uuid,
        request: crate::dto::product::ProductCreateRequest,
    ) -> Result<Product>;

    /// Get product by ID
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Product if found
    ///
    /// # Errors
    /// - `NotFound` if product doesn't exist
    async fn get_product(&self, tenant_id: Uuid, product_id: Uuid) -> Result<Product>;

    /// List products with filtering and pagination
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `query` - List query parameters
    ///
    /// # Returns
    /// Paginated list of products
    async fn list_products(
        &self,
        tenant_id: Uuid,
        query: crate::dto::product::ProductListQuery,
    ) -> Result<crate::dto::product::ProductListResponse>;

    /// Update an existing product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `request` - Update data
    ///
    /// # Returns
    /// Updated product
    ///
    /// # Errors
    /// - `NotFound` if product doesn't exist
    /// - `ValidationError` if request validation fails
    /// - `Conflict` if SKU conflicts with existing product
    async fn update_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        request: crate::dto::product::ProductUpdateRequest,
    ) -> Result<Product>;

    /// Delete a product (soft delete)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Success status
    ///
    /// # Errors
    /// - `NotFound` if product doesn't exist
    /// - `Conflict` if product has active transactions
    async fn delete_product(&self, tenant_id: Uuid, product_id: Uuid) -> Result<()>;

    /// Get product by SKU
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `sku` - Product SKU
    ///
    /// # Returns
    /// Product if found
    ///
    /// # Errors
    /// - `NotFound` if product doesn't exist
    async fn get_product_by_sku(&self, tenant_id: Uuid, sku: &str) -> Result<Product>;

    // ========================================================================
    // Analytics and Statistics (Future)
    // ========================================================================

    /// Get popular search terms
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `limit` - Maximum number of terms to return
    ///
    /// # Returns
    /// List of popular search terms with counts
    async fn get_popular_search_terms(
        &self,
        tenant_id: Uuid,
        limit: u32,
    ) -> Result<Vec<(String, u32)>>;

    /// Record search analytics
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
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
}
