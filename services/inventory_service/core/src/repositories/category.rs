//! Category repository trait
//!
//! Defines the interface for category data access operations.
//! No implementation details - pure trait definition.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::category::{Category, CategoryNode};
use crate::dto::category::{CategoryListQuery, CategoryStatsResponse};
use crate::Result;

/// Repository trait for category operations
///
/// This trait defines all data access operations for categories.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    // ========================================================================
    // Basic CRUD Operations
    // ========================================================================

    /// Create a new category
    ///
    /// # Arguments
    /// * `category` - Category to create
    ///
    /// # Returns
    /// Created category with generated ID and computed path
    async fn create(&self, category: Category) -> Result<Category>;

    /// Find category by ID
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for multi-tenancy
    /// * `category_id` - Category identifier
    ///
    /// # Returns
    /// Category if found, None otherwise
    async fn find_by_id(&self, tenant_id: Uuid, category_id: Uuid) -> Result<Option<Category>>;

    /// Find category by slug
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `slug` - URL-friendly identifier
    ///
    /// # Returns
    /// Category if found, None otherwise
    async fn find_by_slug(&self, tenant_id: Uuid, slug: &str) -> Result<Option<Category>>;

    /// Find category by code
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `code` - Category code
    ///
    /// # Returns
    /// Category if found, None otherwise
    async fn find_by_code(&self, tenant_id: Uuid, code: &str) -> Result<Option<Category>>;

    /// Update an existing category
    ///
    /// # Arguments
    /// * `category` - Category with updated values
    ///
    /// # Returns
    /// Updated category
    async fn update(&self, category: Category) -> Result<Category>;

    /// Soft delete a category
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category to delete
    ///
    /// # Returns
    /// True if deleted, false if not found
    async fn delete(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;

    /// Hard delete a category (permanent)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category to permanently delete
    ///
    /// # Returns
    /// True if deleted, false if not found
    async fn hard_delete(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;

    // ========================================================================
    // Query Operations
    // ========================================================================

    /// List categories with filtering and pagination
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `query` - Query parameters for filtering and pagination
    ///
    /// # Returns
    /// Tuple of (categories, total_count)
    async fn list(
        &self,
        tenant_id: Uuid,
        query: &CategoryListQuery,
    ) -> Result<(Vec<Category>, i64)>;

    /// Get all root categories (no parent)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    ///
    /// # Returns
    /// List of root categories
    async fn get_root_categories(&self, tenant_id: Uuid) -> Result<Vec<Category>>;

    /// Get direct children of a category
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `parent_id` - Parent category ID
    ///
    /// # Returns
    /// List of child categories
    async fn get_children(&self, tenant_id: Uuid, parent_id: Uuid) -> Result<Vec<Category>>;

    /// Get all ancestors of a category (from root to parent)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category ID
    ///
    /// # Returns
    /// List of ancestor categories ordered from root to immediate parent
    async fn get_ancestors(&self, tenant_id: Uuid, category_id: Uuid) -> Result<Vec<Category>>;

    /// Get all descendants of a category (all subcategories)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category ID
    ///
    /// # Returns
    /// List of descendant categories
    async fn get_descendants(&self, tenant_id: Uuid, category_id: Uuid) -> Result<Vec<Category>>;

    /// Get full category tree
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `parent_id` - Optional parent ID to get subtree (None for full tree)
    ///
    /// # Returns
    /// Tree structure of categories
    async fn get_tree(&self, tenant_id: Uuid, parent_id: Option<Uuid>)
        -> Result<Vec<CategoryNode>>;

    /// Check if category exists
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category ID
    ///
    /// # Returns
    /// True if exists, false otherwise
    async fn exists(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;

    /// Check if category has children
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category ID
    ///
    /// # Returns
    /// True if has children, false otherwise
    async fn has_children(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;

    /// Check if category has products
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category ID
    ///
    /// # Returns
    /// True if has products, false otherwise
    async fn has_products(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;

    /// Check if category can be deleted
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category ID
    ///
    /// # Returns
    /// True if can be deleted (no children, no products), false otherwise
    async fn can_delete(&self, tenant_id: Uuid, category_id: Uuid) -> Result<bool>;

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
    /// Category statistics
    async fn get_stats(&self, tenant_id: Uuid, category_id: Uuid) -> Result<CategoryStatsResponse>;

    /// Update product counts for a category and its ancestors
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Category ID to update
    ///
    /// # Returns
    /// Number of categories updated
    async fn update_product_counts(&self, tenant_id: Uuid, category_id: Uuid) -> Result<i32>;

    // ========================================================================
    // Bulk Operations
    // ========================================================================

    /// Move products to a category
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_ids` - List of product IDs to move
    /// * `category_id` - Target category ID
    ///
    /// # Returns
    /// Number of products moved
    async fn move_products_to_category(
        &self,
        tenant_id: Uuid,
        product_ids: Vec<Uuid>,
        category_id: Uuid,
    ) -> Result<i32>;

    /// Get products in category tree (category + all subcategories)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_id` - Root category ID
    ///
    /// # Returns
    /// List of product IDs in the category tree
    async fn get_products_in_tree(&self, tenant_id: Uuid, category_id: Uuid) -> Result<Vec<Uuid>>;

    /// Bulk activate categories
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_ids` - List of category IDs to activate
    ///
    /// # Returns
    /// Number of categories activated
    async fn bulk_activate(&self, tenant_id: Uuid, category_ids: Vec<Uuid>) -> Result<i32>;

    /// Bulk deactivate categories
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_ids` - List of category IDs to deactivate
    ///
    /// # Returns
    /// Number of categories deactivated
    async fn bulk_deactivate(&self, tenant_id: Uuid, category_ids: Vec<Uuid>) -> Result<i32>;

    /// Bulk delete categories (soft delete)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `category_ids` - List of category IDs to delete
    ///
    /// # Returns
    /// Number of categories deleted
    async fn bulk_delete(&self, tenant_id: Uuid, category_ids: Vec<Uuid>) -> Result<i32>;

    // ========================================================================
    // Search Operations
    // ========================================================================

    /// Full-text search categories
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `search_term` - Search term
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    /// List of matching categories
    async fn search(&self, tenant_id: Uuid, search_term: &str, limit: i32)
        -> Result<Vec<Category>>;
}
