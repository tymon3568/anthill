//! Category service trait
//!
//! Defines the business logic interface for category operations.
//! This trait coordinates between repositories and implements business rules.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::category::{Category, CategoryBreadcrumb};
use crate::dto::category::{
    BulkOperationResponse, CategoryCreateRequest, CategoryListQuery, CategoryListResponse,
    CategoryStatsResponse, CategoryTreeResponse, CategoryUpdateRequest, MoveToCategoryRequest,
};
use crate::Result;

/// Service trait for category business logic
///
/// This trait defines all business operations for categories.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait CategoryService: Send + Sync {
    // ========================================================================
    // CRUD Operations
    // ========================================================================

    /// Create a new category
    ///
    /// # Business Rules
    /// - Auto-generates slug from name if not provided
    /// - Validates parent category exists (if provided)
    /// - Validates parent belongs to same tenant
    /// - Validates unique slug per tenant
    /// - Validates unique code per tenant (if provided)
    /// - Validates color format if provided
    /// - Auto-calculates path and level via database trigger
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `request` - Category creation request
    ///
    /// # Returns
    /// Created category with computed fields
    ///
    /// # Errors
    /// - `ValidationError` if validation fails
    /// - `NotFound` if parent category doesn't exist
    /// - `Conflict` if slug or code already exists
    async fn create_category(
        &self,
        tenant_id: Uuid,
        request: CategoryCreateRequest,
    ) -> Result<Category>;

    /// Get category by ID
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category identifier
    ///
    /// # Returns
    /// Category if found
    ///
    /// # Errors
    /// - `NotFound` if category doesn't exist
    async fn get_category(&self, tenant_id: Uuid, category_id: Uuid) -> Result<Category>;

    /// Get category by ID with breadcrumbs
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category identifier
    ///
    /// # Returns
    /// Category response with breadcrumb path
    ///
    /// # Errors
    /// - `NotFound` if category doesn't exist
    async fn get_category_with_breadcrumbs(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
    ) -> Result<(Category, Vec<CategoryBreadcrumb>)>;

    /// Get category by slug
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `slug` - URL-friendly identifier
    ///
    /// # Returns
    /// Category if found
    ///
    /// # Errors
    /// - `NotFound` if category doesn't exist
    async fn get_category_by_slug(&self, tenant_id: Uuid, slug: &str) -> Result<Category>;

    /// Update category
    ///
    /// # Business Rules
    /// - Validates category exists and belongs to tenant
    /// - Prevents circular parent references
    /// - Updates path if parent changes (including all descendants)
    /// - Validates unique slug/code if changed
    /// - Validates color format if changed
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category to update
    /// * `request` - Update request with new values
    ///
    /// # Returns
    /// Updated category
    ///
    /// # Errors
    /// - `NotFound` if category doesn't exist
    /// - `ValidationError` if validation fails
    /// - `Conflict` if slug or code conflicts
    async fn update_category(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
        request: CategoryUpdateRequest,
    ) -> Result<Category>;

    /// Delete category (soft delete)
    ///
    /// # Business Rules
    /// - Cannot delete if has child categories
    /// - Cannot delete if has products
    /// - Marks as deleted (soft delete)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category to delete
    ///
    /// # Returns
    /// True if deleted successfully
    ///
    /// # Errors
    /// - `NotFound` if category doesn't exist
    /// - `Conflict` if category has children or products
    async fn delete_category(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;

    // ========================================================================
    // List and Query Operations
    // ========================================================================

    /// List categories with filtering and pagination
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `query` - Query parameters
    ///
    /// # Returns
    /// Paginated list of categories
    async fn list_categories(
        &self,
        tenant_id: Uuid,
        query: CategoryListQuery,
    ) -> Result<CategoryListResponse>;

    /// Get category tree (hierarchical structure)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `parent_id` - Optional parent to get subtree (None for full tree)
    /// * `max_depth` - Optional maximum depth to return (None for unlimited)
    ///
    /// # Returns
    /// Hierarchical tree structure
    async fn get_category_tree(
        &self,
        tenant_id: Uuid,
        parent_id: Option<Uuid>,
        max_depth: Option<i32>,
    ) -> Result<Vec<CategoryTreeResponse>>;

    /// Get direct children of a category
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `parent_id` - Parent category ID
    ///
    /// # Returns
    /// List of child categories
    async fn get_children(&self, tenant_id: Uuid, parent_id: Uuid) -> Result<Vec<Category>>;

    /// Get breadcrumb path for a category
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category ID
    ///
    /// # Returns
    /// List of breadcrumb items from root to category
    async fn get_breadcrumbs(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
    ) -> Result<Vec<CategoryBreadcrumb>>;

    // ========================================================================
    // Statistics and Analytics
    // ========================================================================

    /// Get category statistics
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category ID
    ///
    /// # Returns
    /// Category statistics including product counts
    async fn get_category_stats(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
    ) -> Result<CategoryStatsResponse>;

    /// Get top categories by product count
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `limit` - Maximum number of categories to return
    ///
    /// # Returns
    /// List of top categories sorted by product count
    async fn get_top_categories(&self, tenant_id: Uuid, limit: i32) -> Result<Vec<Category>>;

    // ========================================================================
    // Bulk Operations
    // ========================================================================

    /// Move products to a category
    ///
    /// # Business Rules
    /// - Validates target category exists and is active
    /// - Validates all products belong to tenant
    /// - Updates category product counts
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `request` - Move request with product IDs and target category
    ///
    /// # Returns
    /// Bulk operation result
    async fn move_products_to_category(
        &self,
        tenant_id: Uuid,
        request: MoveToCategoryRequest,
    ) -> Result<BulkOperationResponse>;

    /// Bulk activate categories
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_ids` - List of category IDs to activate
    ///
    /// # Returns
    /// Bulk operation result
    async fn bulk_activate_categories(
        &self,
        tenant_id: Uuid,
        category_ids: Vec<Uuid>,
    ) -> Result<BulkOperationResponse>;

    /// Bulk deactivate categories
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_ids` - List of category IDs to deactivate
    ///
    /// # Returns
    /// Bulk operation result
    async fn bulk_deactivate_categories(
        &self,
        tenant_id: Uuid,
        category_ids: Vec<Uuid>,
    ) -> Result<BulkOperationResponse>;

    /// Bulk delete categories
    ///
    /// # Business Rules
    /// - Only deletes categories without children or products
    /// - Skips categories that cannot be deleted
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_ids` - List of category IDs to delete
    ///
    /// # Returns
    /// Bulk operation result with count of deleted categories
    async fn bulk_delete_categories(
        &self,
        tenant_id: Uuid,
        category_ids: Vec<Uuid>,
    ) -> Result<BulkOperationResponse>;

    // ========================================================================
    // Search Operations
    // ========================================================================

    /// Search categories by name and description
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `search_term` - Search term
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    /// List of matching categories
    async fn search_categories(
        &self,
        tenant_id: Uuid,
        search_term: &str,
        limit: i32,
    ) -> Result<Vec<Category>>;

    // ========================================================================
    // Validation Helpers
    // ========================================================================

    /// Check if category can be deleted
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category ID
    ///
    /// # Returns
    /// True if can be deleted (no children, no products)
    async fn can_delete_category(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;

    /// Validate parent category reference
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Current category ID (for circular check)
    /// * `parent_id` - Proposed parent ID
    ///
    /// # Returns
    /// True if valid parent reference
    ///
    /// # Errors
    /// - `ValidationError` if creates circular reference
    /// - `NotFound` if parent doesn't exist
    async fn validate_parent(
        &self,
        tenant_id: Uuid,
        category_id: Option<Uuid>,
        parent_id: Uuid,
    ) -> Result<bool>;
}
