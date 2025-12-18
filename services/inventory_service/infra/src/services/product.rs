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
        if let Ok(Some(_)) = self.repository.find_by_sku(tenant_id, &request.sku).await {
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
        // TODO: Implement with proper filtering and pagination
        // For now, return empty response
        Ok(inventory_service_core::dto::product::ProductListResponse {
            products: vec![],
            pagination: inventory_service_core::dto::common::PaginationInfo {
                page: query.page as u32,
                page_size: query.page_size as u32,
                total_items: 0,
                total_pages: 0,
                has_next: false,
                has_prev: false,
            },
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
}
