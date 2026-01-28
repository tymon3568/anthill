//! Product service implementation
//!
//! Business logic implementation for product operations.

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::search_dto::{
    ProductSearchRequest, ProductSearchResponse, SearchSuggestionsRequest,
    SearchSuggestionsResponse,
};
use inventory_service_core::domains::inventory::product::Product;
use inventory_service_core::repositories::product::ProductRepository;
use inventory_service_core::services::product::ProductService;
use inventory_service_core::Result;

/// Implementation of ProductService
pub struct ProductServiceImpl {
    repository: Arc<dyn ProductRepository>,
}

impl ProductServiceImpl {
    /// Create new service instance
    pub fn new(repository: Arc<dyn ProductRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl ProductService for ProductServiceImpl {
    // ========================================================================
    // Search Operations
    // ========================================================================

    async fn search_products(
        &self,
        tenant_id: Uuid,
        request: ProductSearchRequest,
    ) -> Result<ProductSearchResponse> {
        // TODO: Implement advanced product search
        // - Validate request parameters
        // - Call repository search method
        // - Apply business rules (e.g., tenant isolation)
        // - Record search analytics
        self.repository.search_products(tenant_id, request).await
    }

    async fn get_search_suggestions(
        &self,
        tenant_id: Uuid,
        request: SearchSuggestionsRequest,
    ) -> Result<SearchSuggestionsResponse> {
        // TODO: Implement search suggestions
        // - Validate request
        // - Call repository method
        // - Apply business rules
        self.repository
            .get_search_suggestions(tenant_id, request)
            .await
    }

    // ========================================================================
    // CRUD Operations (Future)
    // ========================================================================

    async fn create_product(
        &self,
        tenant_id: Uuid,
        request: inventory_service_core::dto::product::ProductCreateRequest,
    ) -> Result<Product> {
        // Check if SKU already exists
        if self
            .repository
            .find_by_sku(tenant_id, &request.sku)
            .await?
            .is_some()
        {
            return Err(shared_error::AppError::Conflict(format!(
                "Product with SKU '{}' already exists",
                request.sku
            )));
        }

        // Create product entity
        let mut product = Product::new(
            tenant_id,
            request.sku,
            request.name,
            request.product_type,
            request.currency_code,
        );

        // Apply optional fields
        if let Some(description) = request.description {
            product.description = Some(description);
        }
        if let Some(category_id) = request.category_id {
            product.category_id = Some(category_id);
        }
        if let Some(item_group_id) = request.item_group_id {
            product.item_group_id = Some(item_group_id);
        }
        product.track_inventory = request.track_inventory.unwrap_or(true);
        if let Some(tracking_method) = request.tracking_method {
            product.tracking_method = tracking_method;
        }
        if let Some(default_uom_id) = request.default_uom_id {
            product.default_uom_id = Some(default_uom_id);
        }
        if let Some(sale_price) = request.sale_price {
            product.sale_price = Some(sale_price);
        }
        if let Some(cost_price) = request.cost_price {
            product.cost_price = Some(cost_price);
        }
        product.weight_grams = request.weight_grams;
        product.dimensions = request.dimensions;
        product.attributes = request.attributes;
        product.is_active = request.is_active.unwrap_or(true);
        product.is_sellable = request.is_sellable.unwrap_or(true);
        product.is_purchaseable = request.is_purchaseable.unwrap_or(true);

        // Save to repository
        self.repository.create(&product).await
    }

    async fn get_product(&self, tenant_id: Uuid, product_id: Uuid) -> Result<Product> {
        self.repository
            .find_by_id(tenant_id, product_id)
            .await?
            .ok_or_else(|| shared_error::AppError::NotFound("Product not found".to_string()))
    }

    async fn list_products(
        &self,
        tenant_id: Uuid,
        query: inventory_service_core::dto::product::ProductListQuery,
    ) -> Result<inventory_service_core::dto::product::ProductListResponse> {
        use inventory_service_core::domains::inventory::dto::search_dto::ProductSearchRequest;
        use inventory_service_core::dto::product::ProductResponse;

        // Convert ProductListQuery to ProductSearchRequest
        let search_request = ProductSearchRequest {
            query: query.search.clone(),
            category_ids: query.category_id.map(|id| vec![id]),
            price_min: None,
            price_max: None,
            in_stock_only: None,
            product_types: query.product_type.clone().map(|t| vec![t]),
            active_only: query.is_active,
            sellable_only: query.is_sellable,
            sort_by: None,
            sort_order: None,
            page: Some(query.page as u32),
            limit: Some(query.page_size as u32),
        };

        // Use the existing search_products method
        let search_response = self
            .repository
            .search_products(tenant_id, search_request)
            .await?;

        // Convert ProductSearchResult to ProductResponse
        let products: Vec<ProductResponse> = search_response
            .products
            .into_iter()
            .map(|p| {
                // Fetch full product details for each result
                ProductResponse {
                    product_id: p.product_id,
                    tenant_id,
                    sku: p.sku,
                    name: p.name,
                    description: p.description,
                    product_type: p.product_type,
                    category_id: p.category_id,
                    item_group_id: None,
                    track_inventory: p.track_inventory,
                    tracking_method: inventory_service_core::domains::inventory::product::ProductTrackingMethod::None,
                    default_uom_id: None,
                    sale_price: p.sale_price,
                    cost_price: p.cost_price,
                    currency_code: p.currency_code,
                    weight_grams: None,
                    dimensions: None,
                    attributes: None,
                    is_active: p.is_active,
                    is_sellable: p.is_sellable,
                    is_purchaseable: true,
                    created_at: p.created_at,
                    updated_at: p.updated_at,
                }
            })
            .collect();

        Ok(inventory_service_core::dto::product::ProductListResponse {
            products,
            pagination: search_response.pagination,
        })
    }

    async fn update_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        request: inventory_service_core::dto::product::ProductUpdateRequest,
    ) -> Result<Product> {
        // Get existing product
        let mut product = self.get_product(tenant_id, product_id).await?;

        // SKU cannot be updated for now - it's the primary identifier
        // TODO: Implement SKU update with proper conflict checking

        // Apply updates
        if let Some(name) = request.name {
            product.name = name;
        }
        if let Some(description) = request.description {
            product.description = Some(description);
        }
        if let Some(product_type) = request.product_type {
            product.product_type = product_type;
        }
        if let Some(category_id) = request.category_id {
            product.category_id = Some(category_id);
        }
        if let Some(item_group_id) = request.item_group_id {
            product.item_group_id = Some(item_group_id);
        }
        if let Some(track_inventory) = request.track_inventory {
            product.track_inventory = track_inventory;
        }
        if let Some(tracking_method) = request.tracking_method {
            product.tracking_method = tracking_method;
        }
        if let Some(default_uom_id) = request.default_uom_id {
            product.default_uom_id = Some(default_uom_id);
        }
        if let Some(sale_price) = request.sale_price {
            product.sale_price = Some(sale_price);
        }
        if let Some(cost_price) = request.cost_price {
            product.cost_price = Some(cost_price);
        }
        if let Some(currency_code) = request.currency_code {
            product.currency_code = currency_code;
        }
        if let Some(weight_grams) = request.weight_grams {
            product.weight_grams = Some(weight_grams);
        }
        if let Some(dimensions) = request.dimensions {
            product.dimensions = Some(dimensions);
        }
        if let Some(attributes) = request.attributes {
            product.attributes = Some(attributes);
        }
        if let Some(is_active) = request.is_active {
            product.is_active = is_active;
        }
        if let Some(is_sellable) = request.is_sellable {
            product.is_sellable = is_sellable;
        }
        if let Some(is_purchaseable) = request.is_purchaseable {
            product.is_purchaseable = is_purchaseable;
        }

        product.touch();

        // Save to repository
        self.repository
            .update(tenant_id, product_id, &product)
            .await
    }

    async fn delete_product(&self, tenant_id: Uuid, product_id: Uuid) -> Result<()> {
        // Get product to check if it exists
        let _product = self.get_product(tenant_id, product_id).await?;

        // TODO: Check for active transactions before deleting

        // Soft delete
        let deleted = self.repository.delete(tenant_id, product_id).await?;
        if !deleted {
            return Err(shared_error::AppError::NotFound("Product not found".to_string()));
        }
        Ok(())
    }

    async fn get_product_by_sku(&self, tenant_id: Uuid, sku: &str) -> Result<Product> {
        self.repository
            .find_by_sku(tenant_id, sku)
            .await?
            .ok_or_else(|| shared_error::AppError::NotFound("Product not found".to_string()))
    }

    // ========================================================================
    // Analytics and Statistics (Future)
    // ========================================================================

    async fn get_popular_search_terms(
        &self,
        tenant_id: Uuid,
        limit: u32,
    ) -> Result<Vec<(String, u32)>> {
        // TODO: Implement popular search terms
        self.repository
            .get_popular_search_terms(tenant_id, limit)
            .await
    }

    async fn record_search_analytics(
        &self,
        tenant_id: Uuid,
        query: &str,
        result_count: u32,
        user_id: Option<Uuid>,
    ) -> Result<()> {
        // TODO: Implement search analytics recording
        self.repository
            .record_search_analytics(tenant_id, query, result_count, user_id)
            .await
    }

    // ========================================================================
    // Bulk Operations
    // ========================================================================

    async fn bulk_activate_products(&self, tenant_id: Uuid, product_ids: &[Uuid]) -> Result<i64> {
        if product_ids.is_empty() {
            return Err(shared_error::AppError::ValidationError(
                "No product IDs provided".to_string(),
            ));
        }

        self.repository.bulk_activate(tenant_id, product_ids).await
    }

    async fn bulk_deactivate_products(&self, tenant_id: Uuid, product_ids: &[Uuid]) -> Result<i64> {
        if product_ids.is_empty() {
            return Err(shared_error::AppError::ValidationError(
                "No product IDs provided".to_string(),
            ));
        }

        self.repository
            .bulk_deactivate(tenant_id, product_ids)
            .await
    }

    async fn bulk_delete_products(&self, tenant_id: Uuid, product_ids: &[Uuid]) -> Result<i64> {
        if product_ids.is_empty() {
            return Err(shared_error::AppError::ValidationError(
                "No product IDs provided".to_string(),
            ));
        }

        self.repository.bulk_delete(tenant_id, product_ids).await
    }
}
