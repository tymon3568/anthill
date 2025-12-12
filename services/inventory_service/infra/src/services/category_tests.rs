//! Unit tests for CategoryServiceImpl using mocks

use chrono::Utc;
use mockall::mock;
use mockall::predicate::*;
use uuid::Uuid;

use inventory_service_core::domains::category::{Category, CategoryNode};
use inventory_service_core::dto::category::{
    CategoryCreateRequest, CategoryListQuery, CategoryUpdateRequest, MoveToCategoryRequest,
};
use inventory_service_core::repositories::category::CategoryRepository;
use inventory_service_core::services::category::CategoryService;
use inventory_service_core::Result;
use shared_error::AppError;

use super::CategoryServiceImpl;

// Mock the CategoryRepository trait
mock! {
    pub CategoryRepositoryImpl {}

    #[async_trait::async_trait]
    impl CategoryRepository for CategoryRepositoryImpl {
        async fn create(&self, category: Category) -> Result<Category>;
        async fn find_by_id(&self, tenant_id: Uuid, category_id: Uuid) -> Result<Option<Category>>;
        async fn find_by_slug(&self, tenant_id: Uuid, slug: &str) -> Result<Option<Category>>;
        async fn find_by_code(&self, tenant_id: Uuid, code: &str) -> Result<Option<Category>>;
        async fn update(&self, category: Category) -> Result<Category>;
        async fn delete(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;
        async fn hard_delete(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;
        async fn exists(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;
        async fn list(&self, tenant_id: Uuid, query: &CategoryListQuery) -> Result<(Vec<Category>, i64)>;
        async fn get_root_categories(&self, tenant_id: Uuid) -> Result<Vec<Category>>;
        async fn get_tree(&self, tenant_id: Uuid, parent_id: Option<Uuid>) -> Result<Vec<CategoryNode>>;
        async fn get_children(&self, tenant_id: Uuid, parent_id: Uuid) -> Result<Vec<Category>>;
        async fn get_ancestors(&self, tenant_id: Uuid, category_id: Uuid) -> Result<Vec<Category>>;
        async fn get_descendants(&self, tenant_id: Uuid, category_id: Uuid) -> Result<Vec<Category>>;
        async fn get_stats(&self, tenant_id: Uuid, category_id: Uuid) -> Result<inventory_service_core::dto::category::CategoryStatsResponse>;
        async fn get_top_categories(&self, tenant_id: Uuid, limit: i32) -> Result<Vec<Category>>;
        async fn search(&self, tenant_id: Uuid, search_term: &str, limit: i32) -> Result<Vec<Category>>;
        async fn can_delete(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;
        async fn has_children(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;
        async fn has_products(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;
        async fn move_products_to_category(&self, tenant_id: Uuid, product_ids: Vec<Uuid>, category_id: Uuid) -> Result<i32>;
        async fn get_products_in_tree(&self, tenant_id: Uuid, category_id: Uuid) -> Result<Vec<Uuid>>;
        async fn bulk_activate(&self, tenant_id: Uuid, category_ids: Vec<Uuid>) -> Result<i32>;
        async fn bulk_deactivate(&self, tenant_id: Uuid, category_ids: Vec<Uuid>) -> Result<i32>;
        async fn bulk_delete(&self, tenant_id: Uuid, category_ids: Vec<Uuid>) -> Result<i32>;
        async fn update_product_counts(&self, tenant_id: Uuid, category_id: Uuid) -> Result<i32>;
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_category() -> Category {
        Category {
            category_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            parent_category_id: None,
            name: "Test Category".to_string(),
            description: Some("Test description".to_string()),
            code: Some("TEST".to_string()),
            path: "test-category".to_string(),
            level: 0,
            display_order: 0,
            icon: None,
            color: Some("#FF5733".to_string()),
            image_url: None,
            is_active: true,
            is_visible: true,
            slug: Some("test-category".to_string()),
            meta_title: None,
            meta_description: None,
            meta_keywords: None,
            product_count: 0,
            total_product_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    #[tokio::test]
    async fn test_create_category_success() {
        let mut mock_repo = MockCategoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let created_category = create_test_category();

        mock_repo.expect_exists().returning(|_, _| Ok(true));
        mock_repo
            .expect_create()
            .returning(move |_| Ok(created_category.clone()));

        let service = CategoryServiceImpl::new(mock_repo);

        let request = CategoryCreateRequest {
            parent_category_id: None,
            name: "Test Category".to_string(),
            description: Some("Test description".to_string()),
            code: Some("TEST".to_string()),
            display_order: 0,
            icon: None,
            color: Some("#FF5733".to_string()),
            image_url: None,
            is_active: true,
            is_visible: true,
            slug: Some("test-category".to_string()),
            meta_title: None,
            meta_description: None,
            meta_keywords: None,
        };

        let result = service.create_category(tenant_id, request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_category_validation_error() {
        let mock_repo = MockCategoryRepositoryImpl::new();
        let service = CategoryServiceImpl::new(mock_repo);
        let tenant_id = Uuid::new_v4();

        let request = CategoryCreateRequest {
            parent_category_id: None,
            name: "".to_string(), // Invalid: empty name
            description: Some("Test description".to_string()),
            code: Some("TEST".to_string()),
            display_order: 0,
            icon: None,
            color: Some("#FF5733".to_string()),
            image_url: None,
            is_active: true,
            is_visible: true,
            slug: Some("test-category".to_string()),
            meta_title: None,
            meta_description: None,
            meta_keywords: None,
        };

        let result = service.create_category(tenant_id, request).await;
        assert!(matches!(result, Err(AppError::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_get_category_success() {
        let mut mock_repo = MockCategoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let category_id = Uuid::new_v4();
        let category = create_test_category();

        mock_repo
            .expect_find_by_id()
            .with(eq(tenant_id), eq(category_id))
            .returning(move |_, _| Ok(Some(category.clone())));

        let service = CategoryServiceImpl::new(mock_repo);

        let result = service.get_category(tenant_id, category_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Test Category");
    }

    #[tokio::test]
    async fn test_update_category_success() {
        let mut mock_repo = MockCategoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let category_id = Uuid::new_v4();
        let existing_category = create_test_category();
        let updated_category = Category {
            name: "Updated Category".to_string(),
            ..existing_category.clone()
        };

        mock_repo
            .expect_find_by_id()
            .with(eq(tenant_id), eq(category_id))
            .returning(move |_, _| Ok(Some(existing_category.clone())));

        mock_repo
            .expect_update()
            .returning(move |_| Ok(updated_category.clone()));

        let service = CategoryServiceImpl::new(mock_repo);

        let request = CategoryUpdateRequest {
            parent_category_id: None,
            name: Some("Updated Category".to_string()),
            description: None,
            code: None,
            display_order: None,
            icon: None,
            color: None,
            image_url: None,
            is_active: None,
            is_visible: None,
            slug: None,
            meta_title: None,
            meta_description: None,
            meta_keywords: None,
        };

        let result = service
            .update_category(tenant_id, category_id, request)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Updated Category");
    }

    #[tokio::test]
    async fn test_delete_category_success() {
        let mut mock_repo = MockCategoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let category_id = Uuid::new_v4();

        mock_repo
            .expect_can_delete()
            .with(eq(tenant_id), eq(category_id))
            .returning(|_, _| Ok(true));

        mock_repo
            .expect_delete()
            .with(eq(tenant_id), eq(category_id))
            .returning(|_, _| Ok(true));

        mock_repo
            .expect_update_product_counts()
            .with(eq(tenant_id), eq(category_id))
            .returning(|_, _| Ok(1));

        let service = CategoryServiceImpl::new(mock_repo);

        let result = service.delete_category(tenant_id, category_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_bulk_activate_categories() {
        let mut mock_repo = MockCategoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let category_ids = vec![Uuid::new_v4()];

        mock_repo
            .expect_bulk_activate()
            .with(eq(tenant_id), eq(category_ids.clone()))
            .returning(|_, _| Ok(1));

        let service = CategoryServiceImpl::new(mock_repo);

        let result = service
            .bulk_activate_categories(tenant_id, category_ids)
            .await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.affected_count, 1);
    }

    #[tokio::test]
    async fn test_move_products_to_category() {
        let mut mock_repo = MockCategoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let category_id = Uuid::new_v4();
        let product_ids = vec![Uuid::new_v4(), Uuid::new_v4()];

        mock_repo
            .expect_move_products_to_category()
            .with(eq(tenant_id), eq(product_ids.clone()), eq(category_id))
            .returning(|_, _, _| Ok(2));

        let service = CategoryServiceImpl::new(mock_repo);

        let request = MoveToCategoryRequest {
            product_ids: product_ids.clone(),
            category_id,
        };

        let result = service.move_products_to_category(tenant_id, request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.affected_count, 2);
    }

    // =========================================================================
    // Additional Tests for Better Coverage
    // =========================================================================

    #[tokio::test]
    async fn test_get_category_not_found() {
        let mut mock_repo = MockCategoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let category_id = Uuid::new_v4();

        mock_repo
            .expect_find_by_id()
            .with(eq(tenant_id), eq(category_id))
            .returning(|_, _| Ok(None));

        let service = CategoryServiceImpl::new(mock_repo);

        let result = service.get_category(tenant_id, category_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_delete_category_cannot_delete() {
        let mut mock_repo = MockCategoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let category_id = Uuid::new_v4();

        // Category has children or products, cannot delete
        mock_repo
            .expect_can_delete()
            .with(eq(tenant_id), eq(category_id))
            .returning(|_, _| Ok(false));

        let service = CategoryServiceImpl::new(mock_repo);

        let result = service.delete_category(tenant_id, category_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::ValidationError(_)));
    }

    #[tokio::test]
    async fn test_create_category_parent_not_exists() {
        let mut mock_repo = MockCategoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let parent_id = Uuid::new_v4();

        // Parent category does not exist
        mock_repo
            .expect_exists()
            .with(eq(tenant_id), eq(parent_id))
            .returning(|_, _| Ok(false));

        let service = CategoryServiceImpl::new(mock_repo);

        let request = CategoryCreateRequest {
            parent_category_id: Some(parent_id),
            name: "Child Category".to_string(),
            description: None,
            code: None,
            display_order: 0,
            icon: None,
            color: None,
            image_url: None,
            is_active: true,
            is_visible: true,
            slug: None,
            meta_title: None,
            meta_description: None,
            meta_keywords: None,
        };

        let result = service.create_category(tenant_id, request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_get_category_tree_success() {
        let mut mock_repo = MockCategoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();

        let root_category = create_test_category();
        let root_node = CategoryNode::new(root_category);

        mock_repo
            .expect_get_tree()
            .with(eq(tenant_id), eq(None))
            .returning(move |_, _| Ok(vec![root_node.clone()]));

        let service = CategoryServiceImpl::new(mock_repo);

        let result = service.get_category_tree(tenant_id, None, None).await;
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.len(), 1);
    }

    #[tokio::test]
    async fn test_bulk_deactivate_categories() {
        let mut mock_repo = MockCategoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let category_ids = vec![Uuid::new_v4(), Uuid::new_v4()];

        mock_repo
            .expect_bulk_deactivate()
            .with(eq(tenant_id), eq(category_ids.clone()))
            .returning(|_, _| Ok(2));

        let service = CategoryServiceImpl::new(mock_repo);

        let result = service
            .bulk_deactivate_categories(tenant_id, category_ids)
            .await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.affected_count, 2);
    }

    #[tokio::test]
    async fn test_get_children_success() {
        let mut mock_repo = MockCategoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let parent_id = Uuid::new_v4();
        let child = create_test_category();

        mock_repo
            .expect_get_children()
            .with(eq(tenant_id), eq(parent_id))
            .returning(move |_, _| Ok(vec![child.clone()]));

        let service = CategoryServiceImpl::new(mock_repo);

        let result = service.get_children(tenant_id, parent_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_search_categories_success() {
        let mut mock_repo = MockCategoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let category = create_test_category();

        mock_repo
            .expect_search()
            .with(eq(tenant_id), eq("test"), eq(10))
            .returning(move |_, _, _| Ok(vec![category.clone()]));

        let service = CategoryServiceImpl::new(mock_repo);

        let result = service.search_categories(tenant_id, "test", 10).await;
        assert!(result.is_ok());
        let categories = result.unwrap();
        assert_eq!(categories.len(), 1);
        assert_eq!(categories[0].name, "Test Category");
    }
}
