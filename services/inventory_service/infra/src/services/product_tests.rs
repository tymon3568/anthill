//! Unit tests for ProductServiceImpl using mocks

use mockall::mock;
use mockall::predicate::*;
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::search_dto::{
    AppliedFilters, ProductSearchRequest, ProductSearchResponse, SearchFacets, SearchMeta,
    SearchSuggestionsRequest, SearchSuggestionsResponse,
};
use inventory_service_core::domains::inventory::product::Product;
use inventory_service_core::dto::PaginationInfo;
use inventory_service_core::repositories::product::ProductRepository;
use inventory_service_core::services::product::ProductService;
use inventory_service_core::Result;
use shared_error::AppError;

use super::ProductServiceImpl;
use std::sync::Arc;

// Mock the ProductRepository trait
mock! {
    pub ProductRepositoryImpl {}

    #[async_trait::async_trait]
    impl ProductRepository for ProductRepositoryImpl {
        async fn search_products(
            &self,
            tenant_id: Uuid,
            request: ProductSearchRequest,
        ) -> Result<ProductSearchResponse>;

        async fn get_search_suggestions(
            &self,
            tenant_id: Uuid,
            request: SearchSuggestionsRequest,
        ) -> Result<SearchSuggestionsResponse>;

        async fn find_by_id(&self, tenant_id: Uuid, product_id: Uuid) -> Result<Option<Product>>;
        async fn find_by_ids(&self, tenant_id: Uuid, product_ids: &[Uuid]) -> Result<Vec<Product>>;
        async fn find_by_sku(&self, tenant_id: Uuid, sku: &str) -> Result<Option<Product>>;
        async fn find_by_barcode(&self, tenant_id: Uuid, barcode: &str) -> Result<Option<Product>>;
        async fn create(&self, product: &Product) -> Result<Product>;
        async fn update(&self, tenant_id: Uuid, product_id: Uuid, product: &Product) -> Result<Product>;
        async fn delete(&self, tenant_id: Uuid, product_id: Uuid) -> Result<bool>;
        async fn record_search_analytics(
            &self,
            tenant_id: Uuid,
            query: &str,
            result_count: u32,
            user_id: Option<Uuid>,
        ) -> Result<()>;
        async fn get_popular_search_terms(
            &self,
            tenant_id: Uuid,
            limit: u32,
        ) -> Result<Vec<(String, u32)>>;
        async fn is_in_stock(&self, tenant_id: Uuid, product_id: Uuid) -> Result<bool>;
        async fn get_inventory_level(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test product
    fn create_test_product() -> Product {
        Product::new(
            Uuid::new_v4(),
            "TEST-SKU-001".to_string(),
            "Test Product".to_string(),
            "goods".to_string(),
            "USD".to_string(),
        )
    }

    /// Helper function to create empty search response
    fn create_empty_search_response() -> ProductSearchResponse {
        ProductSearchResponse {
            products: vec![],
            pagination: PaginationInfo {
                page: 1,
                page_size: 20,
                total_items: 0,
                total_pages: 0,
                has_next: false,
                has_prev: false,
            },
            facets: SearchFacets {
                categories: vec![],
                price_ranges: vec![],
                product_types: vec![],
            },
            meta: SearchMeta {
                query: Some("test".to_string()),
                execution_time_ms: 10,
                total_found: 0,
                applied_filters: AppliedFilters {
                    category_ids: None,
                    price_min: None,
                    price_max: None,
                    in_stock_only: None,
                    product_types: None,
                    active_only: Some(true),
                    sellable_only: Some(true),
                },
            },
        }
    }

    // =========================================================================
    // get_product Tests
    // =========================================================================

    #[tokio::test]
    async fn test_get_product_success() {
        let mut mock_repo = MockProductRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let product = create_test_product();

        mock_repo
            .expect_find_by_id()
            .with(eq(tenant_id), eq(product_id))
            .returning(move |_, _| Ok(Some(product.clone())));

        let service = ProductServiceImpl::new(Arc::new(mock_repo));

        let result = service.get_product(tenant_id, product_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().sku, "TEST-SKU-001");
    }

    #[tokio::test]
    async fn test_get_product_not_found() {
        let mut mock_repo = MockProductRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_repo
            .expect_find_by_id()
            .with(eq(tenant_id), eq(product_id))
            .returning(|_, _| Ok(None));

        let service = ProductServiceImpl::new(Arc::new(mock_repo));

        let result = service.get_product(tenant_id, product_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // =========================================================================
    // get_product_by_sku Tests
    // =========================================================================

    #[tokio::test]
    async fn test_get_product_by_sku_success() {
        let mut mock_repo = MockProductRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let sku = "TEST-SKU-001";
        let product = create_test_product();

        mock_repo
            .expect_find_by_sku()
            .with(eq(tenant_id), eq(sku))
            .returning(move |_, _| Ok(Some(product.clone())));

        let service = ProductServiceImpl::new(Arc::new(mock_repo));

        let result = service.get_product_by_sku(tenant_id, sku).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().sku, "TEST-SKU-001");
    }

    #[tokio::test]
    async fn test_get_product_by_sku_not_found() {
        let mut mock_repo = MockProductRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let sku = "NONEXISTENT-SKU";

        mock_repo
            .expect_find_by_sku()
            .with(eq(tenant_id), eq(sku))
            .returning(|_, _| Ok(None));

        let service = ProductServiceImpl::new(Arc::new(mock_repo));

        let result = service.get_product_by_sku(tenant_id, sku).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // =========================================================================
    // search_products Tests
    // =========================================================================

    #[tokio::test]
    async fn test_search_products_success() {
        let mut mock_repo = MockProductRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();

        let search_response = create_empty_search_response();

        mock_repo
            .expect_search_products()
            .returning(move |_, _| Ok(search_response.clone()));

        let service = ProductServiceImpl::new(Arc::new(mock_repo));

        let request = ProductSearchRequest::default();

        let result = service.search_products(tenant_id, request).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().pagination.total_items, 0);
    }

    #[tokio::test]
    async fn test_search_products_with_query() {
        let mut mock_repo = MockProductRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();

        let search_response = create_empty_search_response();

        mock_repo
            .expect_search_products()
            .returning(move |_, req| {
                assert_eq!(req.query, Some("laptop".to_string()));
                Ok(search_response.clone())
            });

        let service = ProductServiceImpl::new(Arc::new(mock_repo));

        let request = ProductSearchRequest {
            query: Some("laptop".to_string()),
            ..Default::default()
        };

        let result = service.search_products(tenant_id, request).await;
        assert!(result.is_ok());
    }

    // =========================================================================
    // get_popular_search_terms Tests
    // =========================================================================

    #[tokio::test]
    async fn test_get_popular_search_terms_success() {
        let mut mock_repo = MockProductRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();

        let popular_terms = vec![
            ("laptop".to_string(), 150u32),
            ("phone".to_string(), 100u32),
            ("tablet".to_string(), 50u32),
        ];

        mock_repo
            .expect_get_popular_search_terms()
            .with(eq(tenant_id), eq(10u32))
            .returning(move |_, _| Ok(popular_terms.clone()));

        let service = ProductServiceImpl::new(Arc::new(mock_repo));

        let result = service.get_popular_search_terms(tenant_id, 10).await;
        assert!(result.is_ok());
        let terms = result.unwrap();
        assert_eq!(terms.len(), 3);
        assert_eq!(terms[0].0, "laptop");
        assert_eq!(terms[0].1, 150);
    }

    #[tokio::test]
    async fn test_get_popular_search_terms_empty() {
        let mut mock_repo = MockProductRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();

        mock_repo
            .expect_get_popular_search_terms()
            .returning(|_, _| Ok(vec![]));

        let service = ProductServiceImpl::new(Arc::new(mock_repo));

        let result = service.get_popular_search_terms(tenant_id, 10).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // =========================================================================
    // record_search_analytics Tests
    // =========================================================================

    #[tokio::test]
    async fn test_record_search_analytics_success() {
        let mut mock_repo = MockProductRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        mock_repo
            .expect_record_search_analytics()
            .with(eq(tenant_id), eq("test query"), eq(25u32), eq(Some(user_id)))
            .returning(|_, _, _, _| Ok(()));

        let service = ProductServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .record_search_analytics(tenant_id, "test query", 25, Some(user_id))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_search_analytics_without_user() {
        let mut mock_repo = MockProductRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();

        mock_repo
            .expect_record_search_analytics()
            .with(eq(tenant_id), eq("anonymous query"), eq(10u32), eq(None))
            .returning(|_, _, _, _| Ok(()));

        let service = ProductServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .record_search_analytics(tenant_id, "anonymous query", 10, None)
            .await;
        assert!(result.is_ok());
    }
}
