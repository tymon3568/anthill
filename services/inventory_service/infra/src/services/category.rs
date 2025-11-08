// TODO: Implement category service
use async_trait::async_trait;
use chrono::Utc;
use slug;
use uuid::Uuid;
use validator::Validate;

use inventory_service_core::domains::category::{Category, CategoryBreadcrumb};
use inventory_service_core::dto::category::{
    BulkOperationResponse, CategoryCreateRequest, CategoryListQuery, CategoryListResponse,
    CategoryResponse, CategoryStatsResponse, CategoryTreeResponse, CategoryUpdateRequest,
    MoveToCategoryRequest,
};
use inventory_service_core::repositories::category::CategoryRepository;
use inventory_service_core::services::category::CategoryService;
use inventory_service_core::Result;
use shared_error::AppError;

pub struct CategoryServiceImpl<R: CategoryRepository> {
    repository: R,
}

impl<R: CategoryRepository> CategoryServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: CategoryRepository> CategoryService for CategoryServiceImpl<R> {
    // TODO: Implement all service methods
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

    async fn get_category(&self, tenant_id: Uuid, category_id: Uuid) -> Result<Category> {
        let category = self
            .repository
            .find_by_id(tenant_id, category_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Category {} not found", category_id)))?;

        Ok(category)
    }

    async fn get_category_with_breadcrumbs(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
    ) -> Result<(Category, Vec<CategoryBreadcrumb>)> {
        let category = self.get_category(tenant_id, category_id).await?;
        let breadcrumbs = self.get_breadcrumbs(tenant_id, category_id).await?;

        Ok((category, breadcrumbs))
    }

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

    async fn list_categories(
        &self,
        tenant_id: Uuid,
        query: CategoryListQuery,
    ) -> Result<CategoryListResponse> {
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

    async fn get_category_tree(
        &self,
        _tenant_id: Uuid,
        _parent_id: Option<Uuid>,
        _max_depth: Option<i32>,
    ) -> Result<Vec<CategoryTreeResponse>> {
        todo!()
    }

    async fn get_children(&self, tenant_id: Uuid, parent_id: Uuid) -> Result<Vec<Category>> {
        self.repository.get_children(tenant_id, parent_id).await
    }

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

    async fn get_category_stats(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
    ) -> Result<CategoryStatsResponse> {
        self.repository.get_stats(tenant_id, category_id).await
    }

    async fn get_top_categories(&self, _tenant_id: Uuid, _limit: i32) -> Result<Vec<Category>> {
        todo!()
    }

    async fn move_products_to_category(
        &self,
        _tenant_id: Uuid,
        _request: MoveToCategoryRequest,
    ) -> Result<BulkOperationResponse> {
        todo!()
    }

    async fn bulk_activate_categories(
        &self,
        _tenant_id: Uuid,
        _category_ids: Vec<Uuid>,
    ) -> Result<BulkOperationResponse> {
        todo!()
    }

    async fn bulk_deactivate_categories(
        &self,
        _tenant_id: Uuid,
        _category_ids: Vec<Uuid>,
    ) -> Result<BulkOperationResponse> {
        todo!()
    }

    async fn bulk_delete_categories(
        &self,
        _tenant_id: Uuid,
        _category_ids: Vec<Uuid>,
    ) -> Result<BulkOperationResponse> {
        todo!()
    }

    async fn search_categories(
        &self,
        _tenant_id: Uuid,
        _search_term: &str,
        _limit: i32,
    ) -> Result<Vec<Category>> {
        todo!()
    }

    async fn can_delete_category(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool> {
        self.repository.can_delete(tenant_id, category_id).await
    }

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
}
