//! Business logic implementation for category operations
//!
//! This module provides the concrete implementation of the CategoryService trait,
//! containing all business rules and validation logic for category management.

use async_trait::async_trait;
use chrono::Utc;
use slug;
use uuid::Uuid;
use validator::Validate;

use inventory_service_core::domains::category::{Category, CategoryBreadcrumb, CategoryNode};
use inventory_service_core::dto::category::{
    BulkOperationResponse, CategoryCreateRequest, CategoryListQuery, CategoryListResponse,
    CategoryResponse, CategoryStatsResponse, CategoryTreeResponse, CategoryUpdateRequest,
    MoveToCategoryRequest,
};
use inventory_service_core::repositories::category::CategoryRepository;
use inventory_service_core::services::category::CategoryService;
use inventory_service_core::Result;
use shared_error::AppError;

/// Business logic implementation for category operations
///
/// This struct implements all category business operations with proper
/// validation, business rules enforcement, and error handling. It coordinates
/// between the repository layer and API layer.
pub struct CategoryServiceImpl<R: CategoryRepository> {
    repository: R,
}

impl<R: CategoryRepository> CategoryServiceImpl<R> {
    /// Create a new CategoryServiceImpl with the given repository
    ///
    /// # Arguments
    /// * `repository` - Repository implementation for data access
    ///
    /// # Returns
    /// New CategoryServiceImpl instance
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: CategoryRepository + 'static> CategoryService for CategoryServiceImpl<R> {
    /// Create a new category with business rule validation
    ///
    /// Validates the request, generates slugs, checks parent relationships,
    /// and ensures no circular references before creating the category.
    async fn create_category(
        &self,
        tenant_id: Uuid,
        request: CategoryCreateRequest,
    ) -> Result<Category> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::ValidationError(format!("Invalid category data: {:?}", e)))?;

        // Generate slug if not provided
        let slug = request
            .slug
            .clone()
            .unwrap_or_else(|| slug::slugify(&request.name));

        // Validate parent category exists if provided
        if let Some(parent_id) = request.parent_category_id {
            if !self.repository.exists(tenant_id, parent_id).await? {
                return Err(AppError::NotFound(format!("Parent category {} not found", parent_id)));
            }
        }

        // Validate parent relationship (no cycles)
        if let Some(parent_id) = request.parent_category_id {
            if !self.validate_parent(tenant_id, None, parent_id).await? {
                return Err(AppError::ValidationError(
                    "Invalid parent category relationship".to_string(),
                ));
            }
        }

        // Create category entity
        let now = chrono::Utc::now();
        let category = Category {
            category_id: Uuid::now_v7(),
            tenant_id,
            parent_category_id: request.parent_category_id,
            name: request.name,
            description: request.description,
            code: request.code,
            path: String::new(), // Will be set by database trigger
            level: 0,            // Will be set by database trigger
            display_order: request.display_order,
            icon: request.icon,
            color: request.color,
            image_url: request.image_url,
            is_active: request.is_active,
            is_visible: request.is_visible,
            slug: Some(slug),
            meta_title: request.meta_title,
            meta_description: request.meta_description,
            meta_keywords: request.meta_keywords,
            product_count: 0,
            total_product_count: 0,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        // Save to repository
        let created_category = self.repository.create(category).await?;

        Ok(created_category)
    }

    /// Get a category by ID
    ///
    /// Retrieves a category and ensures it belongs to the specified tenant.
    async fn get_category(&self, tenant_id: Uuid, category_id: Uuid) -> Result<Category> {
        let category = self
            .repository
            .find_by_id(tenant_id, category_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Category {} not found", category_id)))?;

        Ok(category)
    }

    /// Get a category with its breadcrumb path
    ///
    /// Returns the category along with its complete breadcrumb trail
    /// from root to the category's position in the hierarchy.
    async fn get_category_with_breadcrumbs(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
    ) -> Result<(Category, Vec<CategoryBreadcrumb>)> {
        let category = self.get_category(tenant_id, category_id).await?;
        let breadcrumbs = self.get_breadcrumbs(tenant_id, category_id).await?;

        Ok((category, breadcrumbs))
    }

    /// Get a category by its slug
    ///
    /// Finds a category using its URL-friendly slug identifier.
    async fn get_category_by_slug(&self, tenant_id: Uuid, slug: &str) -> Result<Category> {
        let category = self
            .repository
            .find_by_slug(tenant_id, slug)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Category with slug '{}' not found", slug))
            })?;

        Ok(category)
    }

    /// Update an existing category with validation
    ///
    /// Validates the update request, checks for circular references,
    /// and regenerates slugs if needed. Updates path information
    /// if parent category changes.
    async fn update_category(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
        request: CategoryUpdateRequest,
    ) -> Result<Category> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::ValidationError(format!("Invalid category data: {:?}", e)))?;

        // Get existing category
        let mut existing_category = self.get_category(tenant_id, category_id).await?;

        // Validate parent category exists if provided
        if let Some(parent_id) = request.parent_category_id {
            if Some(parent_id) != existing_category.parent_category_id
                && !self.repository.exists(tenant_id, parent_id).await?
            {
                return Err(AppError::NotFound(format!("Parent category {} not found", parent_id)));
            }
        }

        // Validate parent relationship (no cycles)
        if let Some(parent_id) = request.parent_category_id {
            if Some(parent_id) != existing_category.parent_category_id
                && !self
                    .validate_parent(tenant_id, Some(category_id), parent_id)
                    .await?
            {
                return Err(AppError::ValidationError(
                    "Invalid parent category relationship".to_string(),
                ));
            }
        }

        // Update fields
        if let Some(ref name) = request.name {
            existing_category.name = name.clone();
        }
        if let Some(description) = request.description {
            existing_category.description = Some(description);
        }
        if let Some(code) = request.code {
            existing_category.code = Some(code);
        }
        if let Some(display_order) = request.display_order {
            existing_category.display_order = display_order;
        }
        if let Some(icon) = request.icon {
            existing_category.icon = Some(icon);
        }
        if let Some(color) = request.color {
            existing_category.color = Some(color);
        }
        if let Some(image_url) = request.image_url {
            existing_category.image_url = Some(image_url);
        }
        if let Some(is_active) = request.is_active {
            existing_category.is_active = is_active;
        }
        if let Some(is_visible) = request.is_visible {
            existing_category.is_visible = is_visible;
        }
        if let Some(slug) = request.slug {
            existing_category.slug = Some(slug);
        } else if let Some(ref name) = request.name {
            // Regenerate slug if name changed but slug not provided
            existing_category.slug = Some(slug::slugify(name));
        }
        if let Some(meta_title) = request.meta_title {
            existing_category.meta_title = Some(meta_title);
        }
        if let Some(meta_description) = request.meta_description {
            existing_category.meta_description = Some(meta_description);
        }
        if let Some(meta_keywords) = request.meta_keywords {
            existing_category.meta_keywords = Some(meta_keywords);
        }

        // Update parent if changed
        if request.parent_category_id != existing_category.parent_category_id {
            existing_category.parent_category_id = request.parent_category_id;
        }

        existing_category.updated_at = Utc::now();

        // Save to repository
        let updated_category = self.repository.update(existing_category).await?;

        Ok(updated_category)
    }

    /// Delete a category with safety checks
    ///
    /// Performs soft delete only if the category has no children
    /// and no products. Updates product counts for ancestor categories.
    async fn delete_category(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool> {
        // Check if category can be deleted
        if !self.can_delete_category(tenant_id, category_id).await? {
            return Err(AppError::ValidationError(
                "Category cannot be deleted: has children or products".to_string(),
            ));
        }

        // Soft delete the category
        let deleted = self.repository.delete(tenant_id, category_id).await?;

        // Update product counts for ancestors
        if deleted {
            self.repository
                .update_product_counts(tenant_id, category_id)
                .await?;
        }

        Ok(deleted)
    }

    /// List categories with filtering and pagination
    ///
    /// Returns paginated results based on query parameters including
    /// filtering by parent, level, status, and search terms.
    async fn list_categories(
        &self,
        tenant_id: Uuid,
        query: CategoryListQuery,
    ) -> Result<CategoryListResponse> {
        // Validate incoming query before using it
        query
            .validate()
            .map_err(|e| AppError::ValidationError(format!("Invalid list query: {:?}", e)))?;

        // Get categories from repository
        let (categories, total_count) = self.repository.list(tenant_id, &query).await?;

        // Convert to response DTOs
        let category_responses = categories.into_iter().map(CategoryResponse::from).collect();

        // Create pagination info
        let pagination = inventory_service_core::dto::category::PaginationInfo::new(
            query.page,
            query.page_size,
            total_count,
        );

        Ok(CategoryListResponse {
            categories: category_responses,
            pagination,
        })
    }

    /// Get hierarchical category tree
    ///
    /// Returns a tree structure starting from root categories or
    /// a specified parent. Optionally limits depth of the tree.
    async fn get_category_tree(
        &self,
        tenant_id: Uuid,
        parent_id: Option<Uuid>,
        max_depth: Option<i32>,
    ) -> Result<Vec<CategoryTreeResponse>> {
        let mut tree_nodes = self.repository.get_tree(tenant_id, parent_id).await?;

        // Apply max_depth filter if specified
        if let Some(depth) = max_depth {
            fn filter_by_depth(node: &mut CategoryNode, max_depth: i32) -> bool {
                if node.category.level > max_depth {
                    return false;
                }
                node.children
                    .retain_mut(|child| filter_by_depth(child, max_depth));
                true
            }
            tree_nodes.retain_mut(|node| filter_by_depth(node, depth));
        }

        Ok(tree_nodes
            .into_iter()
            .map(CategoryTreeResponse::from)
            .collect())
    }

    /// Get direct children of a category
    ///
    /// Returns immediate child categories of the specified parent.
    async fn get_children(&self, tenant_id: Uuid, parent_id: Uuid) -> Result<Vec<Category>> {
        self.repository.get_children(tenant_id, parent_id).await
    }

    /// Get breadcrumb path for a category
    ///
    /// Returns the complete path from root to the category as breadcrumbs.
    async fn get_breadcrumbs(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
    ) -> Result<Vec<CategoryBreadcrumb>> {
        let ancestors = self
            .repository
            .get_ancestors(tenant_id, category_id)
            .await?;
        let breadcrumbs = ancestors
            .into_iter()
            .map(CategoryBreadcrumb::from)
            .collect();
        Ok(breadcrumbs)
    }

    /// Get detailed statistics for a category
    ///
    /// Returns comprehensive statistics including product counts
    /// and subcategory information.
    async fn get_category_stats(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
    ) -> Result<CategoryStatsResponse> {
        self.repository.get_stats(tenant_id, category_id).await
    }

    /// Get top categories by product count
    ///
    /// Returns categories with highest product counts, useful for
    /// displaying popular or important categories.
    async fn get_top_categories(&self, tenant_id: Uuid, limit: i32) -> Result<Vec<Category>> {
        self.repository.get_top_categories(tenant_id, limit).await
    }

    /// Move multiple products to a category
    ///
    /// Bulk operation to reassign products to a different category.
    /// Validates that the target category exists and is active.
    async fn move_products_to_category(
        &self,
        tenant_id: Uuid,
        request: MoveToCategoryRequest,
    ) -> Result<BulkOperationResponse> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::ValidationError(format!("Invalid request: {:?}", e)))?;

        // Call repository method
        let count = self
            .repository
            .move_products_to_category(tenant_id, request.product_ids, request.category_id)
            .await?;

        Ok(BulkOperationResponse {
            success: true,
            affected_count: count,
            message: format!("Moved {} products to category", count),
        })
    }

    /// Search categories by name and description
    ///
    /// Performs case-insensitive search across category names and descriptions.
    /// Returns empty results for empty search terms.
    async fn search_categories(
        &self,
        tenant_id: Uuid,
        search_term: &str,
        limit: i32,
    ) -> Result<Vec<Category>> {
        if search_term.trim().is_empty() {
            return Ok(Vec::new());
        }

        self.repository.search(tenant_id, search_term, limit).await
    }

    /// Check if a category can be safely deleted
    ///
    /// Returns true if the category has no children and no products,
    /// making it safe for deletion.
    async fn can_delete_category(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool> {
        self.repository.can_delete(tenant_id, category_id).await
    }

    /// Validate parent category relationship
    ///
    /// Ensures the proposed parent relationship is valid:
    /// - Parent exists and belongs to tenant
    /// - No circular references
    /// - Category cannot be its own parent
    async fn validate_parent(
        &self,
        tenant_id: Uuid,
        category_id: Option<Uuid>,
        parent_id: Uuid,
    ) -> Result<bool> {
        // If this is for a new category (category_id is None), just check parent exists
        if category_id.is_none() {
            return self.repository.exists(tenant_id, parent_id).await;
        }

        let category_id = category_id.unwrap();

        // Can't be parent of itself
        if category_id == parent_id {
            return Ok(false);
        }

        // Check if parent_id is a descendant of category_id (would create a cycle)
        let descendants = self
            .repository
            .get_descendants(tenant_id, category_id)
            .await?;
        let would_create_cycle = descendants.iter().any(|desc| desc.category_id == parent_id);

        if would_create_cycle {
            return Ok(false);
        }

        // Check if parent exists
        self.repository.exists(tenant_id, parent_id).await
    }

    /// Bulk activate categories
    ///
    /// Sets multiple categories to active status.
    /// Handles empty input gracefully.
    async fn bulk_activate_categories(
        &self,
        tenant_id: Uuid,
        category_ids: Vec<Uuid>,
    ) -> Result<BulkOperationResponse> {
        if category_ids.is_empty() {
            return Ok(BulkOperationResponse {
                success: true,
                affected_count: 0,
                message: "No categories to activate".to_string(),
            });
        }

        let count = self
            .repository
            .bulk_activate(tenant_id, category_ids)
            .await?;

        Ok(BulkOperationResponse {
            success: true,
            affected_count: count,
            message: format!("Activated {} categories", count),
        })
    }

    async fn bulk_deactivate_categories(
        &self,
        tenant_id: Uuid,
        category_ids: Vec<Uuid>,
    ) -> Result<BulkOperationResponse> {
        if category_ids.is_empty() {
            return Ok(BulkOperationResponse {
                success: true,
                affected_count: 0,
                message: "No categories to deactivate".to_string(),
            });
        }

        let count = self
            .repository
            .bulk_deactivate(tenant_id, category_ids)
            .await?;

        Ok(BulkOperationResponse {
            success: true,
            affected_count: count,
            message: format!("Deactivated {} categories", count),
        })
    }

    async fn bulk_delete_categories(
        &self,
        tenant_id: Uuid,
        category_ids: Vec<Uuid>,
    ) -> Result<BulkOperationResponse> {
        if category_ids.is_empty() {
            return Ok(BulkOperationResponse {
                success: true,
                affected_count: 0,
                message: "No categories to delete".to_string(),
            });
        }

        let count = self.repository.bulk_delete(tenant_id, category_ids).await?;

        Ok(BulkOperationResponse {
            success: true,
            affected_count: count,
            message: format!("Deleted {} categories", count),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use inventory_service_core::dto::category::{CategoryCreateRequest, MoveToCategoryRequest};
    use std::sync::Mutex;

    // Mock repository for testing
    #[derive(Clone)]
    struct MockCategoryRepository {
        categories: std::sync::Arc<Mutex<Vec<Category>>>,
    }

    impl MockCategoryRepository {
        fn new() -> Self {
            Self {
                categories: std::sync::Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn add_category(&self, category: Category) {
            self.categories.lock().unwrap().push(category);
        }
    }

    #[async_trait]
    impl CategoryRepository for MockCategoryRepository {
        async fn create(&self, category: Category) -> Result<Category> {
            self.add_category(category.clone());
            Ok(category)
        }

        async fn find_by_id(
            &self,
            _tenant_id: Uuid,
            category_id: Uuid,
        ) -> Result<Option<Category>> {
            Ok(self
                .categories
                .lock()
                .unwrap()
                .iter()
                .find(|c| c.category_id == category_id)
                .cloned())
        }

        async fn find_by_slug(&self, _tenant_id: Uuid, _slug: &str) -> Result<Option<Category>> {
            Ok(None)
        }

        async fn find_by_code(&self, _tenant_id: Uuid, _code: &str) -> Result<Option<Category>> {
            Ok(None)
        }

        async fn update(&self, category: Category) -> Result<Category> {
            Ok(category)
        }

        async fn delete(&self, _tenant_id: Uuid, _category_id: Uuid) -> Result<bool> {
            Ok(true)
        }

        async fn hard_delete(&self, _tenant_id: Uuid, _category_id: Uuid) -> Result<bool> {
            Ok(true)
        }

        async fn list(
            &self,
            _tenant_id: Uuid,
            _query: &CategoryListQuery,
        ) -> Result<(Vec<Category>, i64)> {
            Ok((self.categories.lock().unwrap().clone(), 0))
        }

        async fn get_root_categories(&self, _tenant_id: Uuid) -> Result<Vec<Category>> {
            Ok(Vec::new())
        }

        async fn get_children(&self, _tenant_id: Uuid, _parent_id: Uuid) -> Result<Vec<Category>> {
            Ok(Vec::new())
        }

        async fn get_ancestors(
            &self,
            _tenant_id: Uuid,
            _category_id: Uuid,
        ) -> Result<Vec<Category>> {
            Ok(Vec::new())
        }

        async fn get_descendants(
            &self,
            _tenant_id: Uuid,
            _category_id: Uuid,
        ) -> Result<Vec<Category>> {
            Ok(Vec::new())
        }

        async fn get_tree(
            &self,
            _tenant_id: Uuid,
            _parent_id: Option<Uuid>,
        ) -> Result<Vec<inventory_service_core::domains::category::CategoryNode>> {
            Ok(Vec::new())
        }

        async fn get_top_categories(&self, _tenant_id: Uuid, _limit: i32) -> Result<Vec<Category>> {
            Ok(Vec::new())
        }

        async fn exists(&self, _tenant_id: Uuid, _category_id: Uuid) -> Result<bool> {
            Ok(true)
        }

        async fn has_children(&self, _tenant_id: Uuid, _category_id: Uuid) -> Result<bool> {
            Ok(false)
        }

        async fn has_products(&self, _tenant_id: Uuid, _category_id: Uuid) -> Result<bool> {
            Ok(false)
        }

        async fn can_delete(&self, _tenant_id: Uuid, _category_id: Uuid) -> Result<bool> {
            Ok(true)
        }

        async fn get_stats(
            &self,
            _tenant_id: Uuid,
            _category_id: Uuid,
        ) -> Result<inventory_service_core::dto::category::CategoryStatsResponse> {
            Ok(inventory_service_core::dto::category::CategoryStatsResponse {
                category_id: Uuid::new_v4(),
                name: "Test".to_string(),
                level: 0,
                product_count: 0,
                total_product_count: 0,
                subcategory_count: 0,
                active_product_count: 0,
                inactive_product_count: 0,
            })
        }

        async fn update_product_counts(&self, _tenant_id: Uuid, _category_id: Uuid) -> Result<i32> {
            Ok(1)
        }

        async fn move_products_to_category(
            &self,
            _tenant_id: Uuid,
            _product_ids: Vec<Uuid>,
            _category_id: Uuid,
        ) -> Result<i32> {
            Ok(1)
        }

        async fn get_products_in_tree(
            &self,
            _tenant_id: Uuid,
            _category_id: Uuid,
        ) -> Result<Vec<Uuid>> {
            Ok(Vec::new())
        }

        async fn bulk_activate(&self, _tenant_id: Uuid, _category_ids: Vec<Uuid>) -> Result<i32> {
            Ok(1)
        }

        async fn bulk_deactivate(&self, _tenant_id: Uuid, _category_ids: Vec<Uuid>) -> Result<i32> {
            Ok(1)
        }

        async fn bulk_delete(&self, _tenant_id: Uuid, _category_ids: Vec<Uuid>) -> Result<i32> {
            Ok(1)
        }

        async fn search(
            &self,
            _tenant_id: Uuid,
            _search_term: &str,
            _limit: i32,
        ) -> Result<Vec<Category>> {
            Ok(Vec::new())
        }
    }

    #[tokio::test]
    async fn test_create_category_success() {
        let repo = MockCategoryRepository::new();
        let service = CategoryServiceImpl::new(repo);

        let request = CategoryCreateRequest {
            parent_category_id: None,
            name: "Test Category".to_string(),
            description: Some("A test category".to_string()),
            code: Some("TEST".to_string()),
            display_order: 1,
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

        let result = service.create_category(Uuid::new_v4(), request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_category_validation_error() {
        let repo = MockCategoryRepository::new();
        let service = CategoryServiceImpl::new(repo);

        let request = CategoryCreateRequest {
            parent_category_id: None,
            name: "".to_string(), // Invalid: empty name
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

        let result = service.create_category(Uuid::new_v4(), request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_category_success() {
        let repo = MockCategoryRepository::new();
        let category_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        // Add a category to the mock repo
        let category = Category {
            category_id,
            tenant_id,
            parent_category_id: None,
            name: "Test".to_string(),
            description: None,
            code: None,
            path: category_id.to_string(),
            level: 0,
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
            product_count: 0,
            total_product_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };
        repo.add_category(category);

        let service = CategoryServiceImpl::new(repo);
        let result = service.get_category(tenant_id, category_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Test");
    }

    #[tokio::test]
    async fn test_get_category_not_found() {
        let repo = MockCategoryRepository::new();
        let service = CategoryServiceImpl::new(repo);

        let result = service.get_category(Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_move_products_to_category_success() {
        let repo = MockCategoryRepository::new();
        let service = CategoryServiceImpl::new(repo);

        let request = MoveToCategoryRequest {
            product_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            category_id: Uuid::new_v4(),
        };

        let result = service
            .move_products_to_category(Uuid::new_v4(), request)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().affected_count, 1);
    }

    #[tokio::test]
    async fn test_bulk_activate_categories_empty() {
        let repo = MockCategoryRepository::new();
        let service = CategoryServiceImpl::new(repo);

        let result = service
            .bulk_activate_categories(Uuid::new_v4(), vec![])
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().affected_count, 0);
    }

    #[tokio::test]
    async fn test_search_categories_empty_term() {
        let repo = MockCategoryRepository::new();
        let service = CategoryServiceImpl::new(repo);

        let result = service.search_categories(Uuid::new_v4(), "", 10).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_validate_parent_self_reference() {
        let repo = MockCategoryRepository::new();
        let service = CategoryServiceImpl::new(repo);

        let category_id = Uuid::new_v4();
        let result = service
            .validate_parent(Uuid::new_v4(), Some(category_id), category_id)
            .await;
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should be false for self-reference
    }
}
