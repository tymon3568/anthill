//! PostgreSQL implementation of CategoryRepository
//!
//! This module provides the concrete implementation of the CategoryRepository trait
//! using PostgreSQL as the data store. It handles all database operations for categories.

use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use inventory_service_core::domains::category::{Category, CategoryNode};
use inventory_service_core::dto::category::CategoryListQuery;
use inventory_service_core::repositories::category::CategoryRepository;
use inventory_service_core::Result;

/// PostgreSQL implementation of CategoryRepository
///
/// Provides concrete implementations of all category repository operations
/// using SQLx for database interactions with PostgreSQL.
pub struct CategoryRepositoryImpl {
    pool: PgPool,
}

impl CategoryRepositoryImpl {
    /// Create a new CategoryRepositoryImpl with the given database connection pool
    ///
    /// # Arguments
    /// * `pool` - PostgreSQL connection pool
    ///
    /// # Returns
    /// New CategoryRepositoryImpl instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CategoryRepository for CategoryRepositoryImpl {
    /// Create a new category in the database
    ///
    /// Inserts the category and triggers automatic path/level calculation
    /// via database triggers. Returns the created category with computed fields.
    async fn create(&self, category: Category) -> Result<Category> {
        let row = sqlx::query_as!(
            Category,
            r#"
            INSERT INTO product_categories (
                category_id, tenant_id, parent_category_id, name, description, code,
                display_order, icon, color, image_url, is_active, is_visible,
                slug, meta_title, meta_description, meta_keywords,
                product_count, total_product_count
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING
                category_id, tenant_id, parent_category_id, name, description, code,
                path, level, display_order, icon, color, image_url, is_active, is_visible,
                slug, meta_title, meta_description, meta_keywords,
                product_count, total_product_count,
                created_at, updated_at, deleted_at
            "#,
            category.category_id,
            category.tenant_id,
            category.parent_category_id,
            category.name,
            category.description,
            category.code,
            category.display_order,
            category.icon,
            category.color,
            category.image_url,
            category.is_active,
            category.is_visible,
            category.slug,
            category.meta_title,
            category.meta_description,
            category.meta_keywords,
            category.product_count,
            category.total_product_count
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Find a category by its ID within a tenant
    ///
    /// Returns the category if it exists and belongs to the specified tenant.
    async fn find_by_id(
        &self,
        tenant_id: uuid::Uuid,
        category_id: uuid::Uuid,
    ) -> Result<Option<Category>> {
        let category = sqlx::query_as!(
            Category,
            r#"
            SELECT
                category_id, tenant_id, parent_category_id, name, description, code,
                path, level, display_order, icon, color, image_url, is_active, is_visible,
                slug, meta_title, meta_description, meta_keywords,
                product_count, total_product_count,
                created_at, updated_at, deleted_at
            FROM product_categories
            WHERE tenant_id = $1 AND category_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            category_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(category)
    }

    /// Find a category by its slug within a tenant
    ///
    /// Returns the category if it exists and belongs to the specified tenant.
    async fn find_by_slug(&self, tenant_id: uuid::Uuid, slug: &str) -> Result<Option<Category>> {
        let category = sqlx::query_as!(
            Category,
            r#"
            SELECT
                category_id, tenant_id, parent_category_id, name, description, code,
                path, level, display_order, icon, color, image_url, is_active, is_visible,
                slug, meta_title, meta_description, meta_keywords,
                product_count, total_product_count,
                created_at, updated_at, deleted_at
            FROM product_categories
            WHERE tenant_id = $1 AND slug = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            slug
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(category)
    }

    /// Find a category by its code within a tenant
    ///
    /// Returns the category if it exists and belongs to the specified tenant.
    async fn find_by_code(&self, tenant_id: uuid::Uuid, code: &str) -> Result<Option<Category>> {
        let category = sqlx::query_as!(
            Category,
            r#"
            SELECT
                category_id, tenant_id, parent_category_id, name, description, code,
                path, level, display_order, icon, color, image_url, is_active, is_visible,
                slug, meta_title, meta_description, meta_keywords,
                product_count, total_product_count,
                created_at, updated_at, deleted_at
            FROM product_categories
            WHERE tenant_id = $1 AND code = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            code
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(category)
    }

    /// Update an existing category
    ///
    /// Updates the category and triggers path recalculation if parent changed.
    /// Returns the updated category with computed fields.
    async fn update(&self, category: Category) -> Result<Category> {
        let row = sqlx::query_as!(
            Category,
            r#"
            UPDATE product_categories
            SET
                parent_category_id = $3, name = $4, description = $5, code = $6,
                display_order = $7, icon = $8, color = $9, image_url = $10,
                is_active = $11, is_visible = $12, slug = $13, meta_title = $14,
                meta_description = $15, meta_keywords = $16, updated_at = NOW()
            WHERE tenant_id = $1 AND category_id = $2 AND deleted_at IS NULL
            RETURNING
                category_id, tenant_id, parent_category_id, name, description, code,
                path, level, display_order, icon, color, image_url, is_active, is_visible,
                slug, meta_title, meta_description, meta_keywords,
                product_count, total_product_count,
                created_at, updated_at, deleted_at
            "#,
            category.tenant_id,
            category.category_id,
            category.parent_category_id,
            category.name,
            category.description,
            category.code,
            category.display_order,
            category.icon,
            category.color,
            category.image_url,
            category.is_active,
            category.is_visible,
            category.slug,
            category.meta_title,
            category.meta_description,
            category.meta_keywords
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Soft delete a category
    ///
    /// Marks the category as deleted (soft delete) if it belongs to the tenant.
    /// Returns true if the category was found and deleted.
    async fn delete(&self, tenant_id: uuid::Uuid, category_id: uuid::Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE product_categories
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND category_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            category_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Permanently delete a category
    ///
    /// Completely removes the category from the database.
    /// Use with caution - this cannot be undone.
    async fn hard_delete(&self, tenant_id: uuid::Uuid, category_id: uuid::Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM product_categories
            WHERE tenant_id = $1 AND category_id = $2
            "#,
            tenant_id,
            category_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// List categories with filtering and pagination
    ///
    /// Returns a tuple of (categories, total_count) based on the query parameters.
    /// Supports filtering by parent, level, active status, visibility, and search terms.
    async fn list(
        &self,
        tenant_id: uuid::Uuid,
        query: &CategoryListQuery,
    ) -> Result<(Vec<Category>, i64)> {
        let offset = (query.page - 1) * query.page_size;

        // Build ORDER BY
        let order_field = match query.sort_by {
            inventory_service_core::dto::category::CategorySortField::DisplayOrder => {
                "pc.display_order"
            },
            inventory_service_core::dto::category::CategorySortField::Name => "pc.name",
            inventory_service_core::dto::category::CategorySortField::CreatedAt => "pc.created_at",
            inventory_service_core::dto::category::CategorySortField::UpdatedAt => "pc.updated_at",
            inventory_service_core::dto::category::CategorySortField::ProductCount => {
                "pc.product_count"
            },
            inventory_service_core::dto::category::CategorySortField::Level => "pc.level",
        };

        let order_dir = match query.sort_dir {
            inventory_service_core::dto::category::SortDirection::Asc => "ASC",
            inventory_service_core::dto::category::SortDirection::Desc => "DESC",
        };

        let order_clause = format!("{} {}", order_field, order_dir);

        let search_pattern = query
            .search
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{}%", s));
        let has_search = search_pattern.is_some();

        // Count query with search support
        let count_sql = if has_search {
            "SELECT COUNT(*) as count FROM product_categories pc
             WHERE pc.tenant_id = $1
               AND pc.deleted_at IS NULL
               AND (pc.parent_category_id = $2 OR $2 IS NULL)
               AND (pc.level = $3 OR $3 IS NULL)
               AND (pc.is_active = $4 OR $4 IS NULL)
               AND (pc.is_visible = $5 OR $5 IS NULL)
               AND (pc.name ILIKE $6 OR pc.description ILIKE $6)"
        } else {
            "SELECT COUNT(*) as count FROM product_categories pc
             WHERE pc.tenant_id = $1
               AND pc.deleted_at IS NULL
               AND (pc.parent_category_id = $2 OR $2 IS NULL)
               AND (pc.level = $3 OR $3 IS NULL)
               AND (pc.is_active = $4 OR $4 IS NULL)
               AND (pc.is_visible = $5 OR $5 IS NULL)"
        };

        let mut count_query = sqlx::query(count_sql)
            .bind(tenant_id)
            .bind(query.parent_id)
            .bind(query.level)
            .bind(query.is_active)
            .bind(query.is_visible);

        if let Some(ref search) = search_pattern {
            count_query = count_query.bind(search);
        }

        let count_row = count_query.fetch_one(&self.pool).await?;
        let count: i64 = count_row.get("count");

        // Data query with search and dynamic sort support
        let sql = if has_search {
            format!(
                r#"
                SELECT
                    pc.category_id, pc.tenant_id, pc.parent_category_id, pc.name, pc.description, pc.code,
                    pc.path, pc.level, pc.display_order, pc.icon, pc.color, pc.image_url, pc.is_active, pc.is_visible,
                    pc.slug, pc.meta_title, pc.meta_description, pc.meta_keywords,
                    pc.product_count, pc.total_product_count,
                    pc.created_at, pc.updated_at, pc.deleted_at
                FROM product_categories pc
                WHERE pc.tenant_id = $1
                  AND pc.deleted_at IS NULL
                  AND (pc.parent_category_id = $2 OR $2 IS NULL)
                  AND (pc.level = $3 OR $3 IS NULL)
                  AND (pc.is_active = $4 OR $4 IS NULL)
                  AND (pc.is_visible = $5 OR $5 IS NULL)
                  AND (pc.name ILIKE $6 OR pc.description ILIKE $6)
                ORDER BY {}
                LIMIT $7 OFFSET $8
                "#,
                order_clause
            )
        } else {
            format!(
                r#"
                SELECT
                    pc.category_id, pc.tenant_id, pc.parent_category_id, pc.name, pc.description, pc.code,
                    pc.path, pc.level, pc.display_order, pc.icon, pc.color, pc.image_url, pc.is_active, pc.is_visible,
                    pc.slug, pc.meta_title, pc.meta_description, pc.meta_keywords,
                    pc.product_count, pc.total_product_count,
                    pc.created_at, pc.updated_at, pc.deleted_at
                FROM product_categories pc
                WHERE pc.tenant_id = $1
                  AND pc.deleted_at IS NULL
                  AND (pc.parent_category_id = $2 OR $2 IS NULL)
                  AND (pc.level = $3 OR $3 IS NULL)
                  AND (pc.is_active = $4 OR $4 IS NULL)
                  AND (pc.is_visible = $5 OR $5 IS NULL)
                ORDER BY {}
                LIMIT $6 OFFSET $7
                "#,
                order_clause
            )
        };

        let categories = if let Some(ref search) = search_pattern {
            sqlx::query(&sql)
                .bind(tenant_id)
                .bind(query.parent_id)
                .bind(query.level)
                .bind(query.is_active)
                .bind(query.is_visible)
                .bind(search)
                .bind(query.page_size as i64)
                .bind(offset as i64)
                .map(|row: PgRow| Category {
                    category_id: row.get("category_id"),
                    tenant_id: row.get("tenant_id"),
                    parent_category_id: row.get("parent_category_id"),
                    name: row.get("name"),
                    description: row.get("description"),
                    code: row.get("code"),
                    path: row.get("path"),
                    level: row.get("level"),
                    display_order: row.get("display_order"),
                    icon: row.get("icon"),
                    color: row.get("color"),
                    image_url: row.get("image_url"),
                    is_active: row.get("is_active"),
                    is_visible: row.get("is_visible"),
                    slug: row.get("slug"),
                    meta_title: row.get("meta_title"),
                    meta_description: row.get("meta_description"),
                    meta_keywords: row.get("meta_keywords"),
                    product_count: row.get("product_count"),
                    total_product_count: row.get("total_product_count"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    deleted_at: row.get("deleted_at"),
                })
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query(&sql)
                .bind(tenant_id)
                .bind(query.parent_id)
                .bind(query.level)
                .bind(query.is_active)
                .bind(query.is_visible)
                .bind(query.page_size as i64)
                .bind(offset as i64)
                .map(|row: PgRow| Category {
                    category_id: row.get("category_id"),
                    tenant_id: row.get("tenant_id"),
                    parent_category_id: row.get("parent_category_id"),
                    name: row.get("name"),
                    description: row.get("description"),
                    code: row.get("code"),
                    path: row.get("path"),
                    level: row.get("level"),
                    display_order: row.get("display_order"),
                    icon: row.get("icon"),
                    color: row.get("color"),
                    image_url: row.get("image_url"),
                    is_active: row.get("is_active"),
                    is_visible: row.get("is_visible"),
                    slug: row.get("slug"),
                    meta_title: row.get("meta_title"),
                    meta_description: row.get("meta_description"),
                    meta_keywords: row.get("meta_keywords"),
                    product_count: row.get("product_count"),
                    total_product_count: row.get("total_product_count"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    deleted_at: row.get("deleted_at"),
                })
                .fetch_all(&self.pool)
                .await?
        };

        Ok((categories, count))
    }

    /// Get all root categories (no parent) for a tenant
    ///
    /// Returns categories that have no parent, ordered by display_order then name.
    async fn get_root_categories(&self, tenant_id: uuid::Uuid) -> Result<Vec<Category>> {
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT
                category_id, tenant_id, parent_category_id, name, description, code,
                path, level, display_order, icon, color, image_url, is_active, is_visible,
                slug, meta_title, meta_description, meta_keywords,
                product_count, total_product_count,
                created_at, updated_at, deleted_at
            FROM product_categories
            WHERE tenant_id = $1 AND parent_category_id IS NULL AND deleted_at IS NULL
            ORDER BY display_order ASC, name ASC
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    /// Get direct children of a category
    ///
    /// Returns immediate child categories of the specified parent,
    /// ordered by display_order then name.
    async fn get_children(
        &self,
        tenant_id: uuid::Uuid,
        parent_id: uuid::Uuid,
    ) -> Result<Vec<Category>> {
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT
                category_id, tenant_id, parent_category_id, name, description, code,
                path, level, display_order, icon, color, image_url, is_active, is_visible,
                slug, meta_title, meta_description, meta_keywords,
                product_count, total_product_count,
                created_at, updated_at, deleted_at
            FROM product_categories
            WHERE tenant_id = $1 AND parent_category_id = $2 AND deleted_at IS NULL
            ORDER BY display_order ASC, name ASC
            "#,
            tenant_id,
            parent_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    /// Get all ancestors of a category (from root to parent)
    ///
    /// Returns the complete path from root category to the immediate parent
    /// of the specified category, ordered by level.
    async fn get_ancestors(
        &self,
        tenant_id: uuid::Uuid,
        category_id: uuid::Uuid,
    ) -> Result<Vec<Category>> {
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT
                c.category_id, c.tenant_id, c.parent_category_id, c.name, c.description, c.code,
                c.path, c.level, c.display_order, c.icon, c.color, c.image_url, c.is_active, c.is_visible,
                c.slug, c.meta_title, c.meta_description, c.meta_keywords,
                c.product_count, c.total_product_count,
                c.created_at, c.updated_at, c.deleted_at
            FROM get_category_ancestors($2, $1) a
            JOIN product_categories c ON c.category_id = a.category_id
            ORDER BY c.level ASC
            "#,
            tenant_id,
            category_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    /// Get all descendants of a category (all subcategories)
    ///
    /// Returns all categories that are descendants of the specified category,
    /// ordered by path for consistent hierarchy display.
    async fn get_descendants(
        &self,
        tenant_id: uuid::Uuid,
        category_id: uuid::Uuid,
    ) -> Result<Vec<Category>> {
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT
                c.category_id, c.tenant_id, c.parent_category_id, c.name, c.description, c.code,
                c.path, c.level, c.display_order, c.icon, c.color, c.image_url, c.is_active, c.is_visible,
                c.slug, c.meta_title, c.meta_description, c.meta_keywords,
                c.product_count, c.total_product_count,
                c.created_at, c.updated_at, c.deleted_at
            FROM get_category_descendants($2, $1) d
            JOIN product_categories c ON c.category_id = d.category_id
            ORDER BY c.path ASC
            "#,
            tenant_id,
            category_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    /// Get full category tree (hierarchical structure)
    ///
    /// Builds a complete tree structure starting from root categories
    /// or from a specified parent. Uses recursive building to create
    /// the full hierarchy with all children populated.
    async fn get_tree(
        &self,
        tenant_id: uuid::Uuid,
        parent_id: Option<uuid::Uuid>,
    ) -> Result<Vec<inventory_service_core::domains::category::CategoryNode>> {
        // Get root categories for the tree
        let root_categories = if let Some(parent_id) = parent_id {
            // If parent_id specified, get children of that parent
            self.get_children(tenant_id, parent_id).await?
        } else {
            // Otherwise get root categories
            self.get_root_categories(tenant_id).await?
        };

        // Build tree recursively
        fn build_node<'a>(
            repo: &'a CategoryRepositoryImpl,
            tenant_id: uuid::Uuid,
            category: Category,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CategoryNode>> + Send + 'a>>
        {
            Box::pin(async move {
                let children = repo.get_children(tenant_id, category.category_id).await?;
                let mut child_nodes = Vec::with_capacity(children.len());
                for child in children {
                    child_nodes.push(build_node(repo, tenant_id, child).await?);
                }
                Ok(CategoryNode {
                    category,
                    children: child_nodes,
                })
            })
        }

        let mut tree = Vec::with_capacity(root_categories.len());
        for category in root_categories {
            tree.push(build_node(self, tenant_id, category).await?);
        }

        Ok(tree)
    }

    /// Get top categories by product count
    ///
    /// Returns root categories ordered by total product count (descending),
    /// then by display order. Limited to the specified number of results.
    async fn get_top_categories(&self, tenant_id: uuid::Uuid, limit: i32) -> Result<Vec<Category>> {
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT
                category_id, tenant_id, parent_category_id, name, description, code,
                path, level, display_order, icon, color, image_url, is_active, is_visible,
                slug, meta_title, meta_description, meta_keywords,
                product_count, total_product_count,
                created_at, updated_at, deleted_at
            FROM product_categories
            WHERE tenant_id = $1 AND deleted_at IS NULL AND parent_category_id IS NULL
            ORDER BY total_product_count DESC, display_order ASC
            LIMIT $2
            "#,
            tenant_id,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    /// Check if a category exists
    ///
    /// Returns true if the category exists and belongs to the specified tenant.
    async fn exists(&self, tenant_id: uuid::Uuid, category_id: uuid::Uuid) -> Result<bool> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*)::BIGINT as count
            FROM product_categories
            WHERE tenant_id = $1 AND category_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            category_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count.unwrap_or(0) > 0)
    }

    /// Check if a category has child categories
    ///
    /// Returns true if the category has any direct children.
    async fn has_children(&self, tenant_id: uuid::Uuid, category_id: uuid::Uuid) -> Result<bool> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*)::BIGINT as count
            FROM product_categories
            WHERE tenant_id = $1 AND parent_category_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            category_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count.unwrap_or(0) > 0)
    }

    /// Check if a category has products
    ///
    /// Returns true if the category has any products assigned to it.
    async fn has_products(&self, tenant_id: uuid::Uuid, category_id: uuid::Uuid) -> Result<bool> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*)::BIGINT as count
            FROM products
            WHERE tenant_id = $1 AND category_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            category_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count.unwrap_or(0) > 0)
    }

    /// Check if a category can be safely deleted
    ///
    /// Uses the database function to determine if the category has no children
    /// and no products, making it safe for deletion.
    async fn can_delete(&self, tenant_id: uuid::Uuid, category_id: uuid::Uuid) -> Result<bool> {
        let row = sqlx::query!(
            r#"
            SELECT can_delete_category($2, $1) as can_delete
            "#,
            tenant_id,
            category_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.can_delete.unwrap_or(false))
    }

    /// Get detailed statistics for a category
    ///
    /// Returns comprehensive statistics including product counts,
    /// subcategory counts, and active/inactive product breakdowns.
    async fn get_stats(
        &self,
        tenant_id: uuid::Uuid,
        category_id: uuid::Uuid,
    ) -> Result<inventory_service_core::dto::category::CategoryStatsResponse> {
        // Get basic category info
        let category = sqlx::query!(
            r#"
            SELECT category_id, name, level, product_count, total_product_count
            FROM product_categories
            WHERE tenant_id = $1 AND category_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            category_id
        )
        .fetch_one(&self.pool)
        .await?;

        // Get subcategory count
        let subcategory_count = sqlx::query!(
            r#"
            SELECT COUNT(*)::INTEGER as count
            FROM product_categories
            WHERE tenant_id = $1 AND parent_category_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            category_id
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);

        // Get active product count
        let active_count = sqlx::query!(
            r#"
            SELECT COUNT(*)::INTEGER as count
            FROM products
            WHERE tenant_id = $1 AND category_id = $2 AND is_active = true AND deleted_at IS NULL
            "#,
            tenant_id,
            category_id
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);

        // Get inactive product count
        let inactive_count = sqlx::query!(
            r#"
            SELECT COUNT(*)::INTEGER as count
            FROM products
            WHERE tenant_id = $1 AND category_id = $2 AND is_active = false AND deleted_at IS NULL
            "#,
            tenant_id,
            category_id
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);

        let stats = inventory_service_core::dto::category::CategoryStatsResponse {
            category_id: category.category_id,
            name: category.name,
            level: category.level,
            product_count: category.product_count,
            total_product_count: category.total_product_count,
            subcategory_count,
            active_product_count: active_count,
            inactive_product_count: inactive_count,
        };

        Ok(stats)
    }

    /// Manually recalculate product counts for a category and its ancestors
    ///
    /// This method recalculates both direct product_count and total_product_count
    /// for the specified category and all its ancestors. Normally this is handled
    /// automatically by database triggers, but this provides a manual override.
    async fn update_product_counts(
        &self,
        tenant_id: uuid::Uuid,
        category_id: uuid::Uuid,
    ) -> Result<i32> {
        // Manually recalculate product counts for this category and ancestors
        sqlx::query!(
            r#"
            UPDATE product_categories pc
            SET
                product_count = (
                    SELECT COUNT(*) FROM products
                    WHERE category_id = pc.category_id
                      AND tenant_id = pc.tenant_id
                      AND deleted_at IS NULL
                ),
                total_product_count = (
                    SELECT COUNT(*) FROM products p
                    JOIN product_categories child ON child.category_id = p.category_id
                    WHERE (child.path = pc.path OR child.path LIKE pc.path || '/%')
                      AND p.tenant_id = pc.tenant_id
                      AND p.deleted_at IS NULL
                )
            WHERE pc.tenant_id = $1
              AND (pc.category_id = $2 OR pc.path LIKE (
                  SELECT path || '/%' FROM product_categories WHERE category_id = $2
              ))
            "#,
            tenant_id,
            category_id
        )
        .execute(&self.pool)
        .await?;

        Ok(1)
    }

    /// Move multiple products to a category
    ///
    /// Uses the database function to efficiently move products in bulk.
    /// Returns the number of products successfully moved.
    async fn move_products_to_category(
        &self,
        tenant_id: uuid::Uuid,
        product_ids: Vec<uuid::Uuid>,
        category_id: uuid::Uuid,
    ) -> Result<i32> {
        let row = sqlx::query!(
            r#"
            SELECT move_products_to_category($3::UUID[], $2, $1) as moved_count
            "#,
            tenant_id,
            category_id,
            &product_ids
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.moved_count.unwrap_or(0))
    }

    /// Get all product IDs in a category tree
    ///
    /// Returns all product IDs that belong to the specified category
    /// or any of its subcategories.
    async fn get_products_in_tree(
        &self,
        tenant_id: uuid::Uuid,
        category_id: uuid::Uuid,
    ) -> Result<Vec<uuid::Uuid>> {
        let rows = sqlx::query!(
            r#"
            SELECT product_id
            FROM get_products_in_category_tree($2, $1)
            "#,
            tenant_id,
            category_id
        )
        .fetch_all(&self.pool)
        .await?;

        let product_ids = rows.into_iter().filter_map(|row| row.product_id).collect();
        Ok(product_ids)
    }

    /// Bulk activate multiple categories
    ///
    /// Sets is_active = true for all specified categories.
    /// Returns the number of categories successfully activated.
    async fn bulk_activate(
        &self,
        tenant_id: uuid::Uuid,
        category_ids: Vec<uuid::Uuid>,
    ) -> Result<i32> {
        let result = sqlx::query!(
            r#"
            UPDATE product_categories
            SET is_active = true, updated_at = NOW()
            WHERE tenant_id = $1 AND category_id = ANY($2) AND deleted_at IS NULL
            "#,
            tenant_id,
            &category_ids
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i32)
    }

    /// Bulk deactivate multiple categories
    ///
    /// Sets is_active = false for all specified categories.
    /// Returns the number of categories successfully deactivated.
    async fn bulk_deactivate(
        &self,
        tenant_id: uuid::Uuid,
        category_ids: Vec<uuid::Uuid>,
    ) -> Result<i32> {
        let result = sqlx::query!(
            r#"
            UPDATE product_categories
            SET is_active = false, updated_at = NOW()
            WHERE tenant_id = $1 AND category_id = ANY($2) AND deleted_at IS NULL
            "#,
            tenant_id,
            &category_ids
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i32)
    }

    /// Bulk soft delete multiple categories
    ///
    /// Marks all specified categories as deleted (soft delete).
    /// Returns the number of categories successfully deleted.
    async fn bulk_delete(
        &self,
        tenant_id: uuid::Uuid,
        category_ids: Vec<uuid::Uuid>,
    ) -> Result<i32> {
        let result = sqlx::query!(
            r#"
            UPDATE product_categories
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND category_id = ANY($2) AND deleted_at IS NULL
            "#,
            tenant_id,
            &category_ids
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i32)
    }

    /// Full-text search categories
    ///
    /// Searches category names and descriptions using case-insensitive
    /// pattern matching. Returns results ordered by name, limited to
    /// the specified number of results.
    async fn search(
        &self,
        tenant_id: uuid::Uuid,
        search_term: &str,
        limit: i32,
    ) -> Result<Vec<Category>> {
        let search_pattern = format!("%{}%", search_term);
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT
                category_id, tenant_id, parent_category_id, name, description, code,
                path, level, display_order, icon, color, image_url, is_active, is_visible,
                slug, meta_title, meta_description, meta_keywords,
                product_count, total_product_count,
                created_at, updated_at, deleted_at
            FROM product_categories
            WHERE tenant_id = $1 AND deleted_at IS NULL
              AND (name ILIKE $2 OR description ILIKE $2)
            ORDER BY name ASC
            LIMIT $3
            "#,
            tenant_id,
            search_pattern,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_service_core::dto::category::CategoryListQuery;
    use inventory_service_core::dto::category::CategorySortField;
    use inventory_service_core::dto::category::SortDirection;
    use serde_json;
    use sqlx::PgPool;
    use uuid::Uuid;

    // Note: These tests require a PostgreSQL database connection
    // In a real scenario, you would use a test database or mock

    #[tokio::test]
    async fn test_category_repository_impl_creation() {
        // This test would require a test database setup
        // For now, just test that the struct can be created
        // In production, you'd use a test database pool
        // let pool = get_test_db_pool().await;
        // let repo = CategoryRepositoryImpl::new(pool);
        // assert! is fine for basic instantiation
    }

    #[test]
    fn test_category_repository_new() {
        // Test that we can create a repository instance
        // Note: This doesn't test database connectivity
        // In a real test, you'd inject a test pool
    }

    #[test]
    fn test_category_sort_field_serialization() {
        let field = CategorySortField::Name;
        let serialized = serde_json::to_string(&field).unwrap();
        assert_eq!(serialized, "\"name\"");

        let deserialized: CategorySortField = serde_json::from_str("\"created_at\"").unwrap();
        assert!(matches!(deserialized, CategorySortField::CreatedAt));
    }

    #[test]
    fn test_category_sort_direction_serialization() {
        let dir = SortDirection::Desc;
        let serialized = serde_json::to_string(&dir).unwrap();
        assert_eq!(serialized, "\"desc\"");

        let deserialized: SortDirection = serde_json::from_str("\"asc\"").unwrap();
        assert!(matches!(deserialized, SortDirection::Asc));
    }

    #[test]
    fn test_category_list_query_defaults() {
        let query: CategoryListQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(query.page, 1);
        assert_eq!(query.page_size, 20);
        assert_eq!(query.sort_by, CategorySortField::DisplayOrder);
        assert_eq!(query.sort_dir, SortDirection::Asc);
    }

    #[test]
    fn test_category_list_query_validation() {
        // Valid query
        let query = CategoryListQuery {
            parent_id: Some(Uuid::new_v4()),
            level: Some(1),
            is_active: Some(true),
            is_visible: Some(false),
            search: Some("electronics".to_string()),
            page: 2,
            page_size: 50,
            sort_by: CategorySortField::Name,
            sort_dir: SortDirection::Desc,
        };
        assert!(query.validate().is_ok());

        // Invalid page
        let mut invalid_query = query.clone();
        invalid_query.page = 0;
        assert!(invalid_query.validate().is_err());

        // Invalid page_size
        let mut invalid_query = query.clone();
        invalid_query.page_size = 0;
        assert!(invalid_query.validate().is_err());

        invalid_query.page_size = 101;
        assert!(invalid_query.validate().is_err());

        // Invalid level
        let mut invalid_query = query.clone();
        invalid_query.level = Some(-1);
        assert!(invalid_query.validate().is_err());

        // Invalid search length
        let mut invalid_query = query.clone();
        invalid_query.search = Some("a".repeat(256));
        assert!(invalid_query.validate().is_err());
    }

    // Integration tests would go here, but require database setup
    // Example:
    /*
    #[tokio::test]
    async fn test_create_and_find_category() {
        let pool = get_test_db_pool().await;
        let repo = CategoryRepositoryImpl::new(pool);

        let tenant_id = Uuid::new_v4();
        let category = Category {
            category_id: Uuid::now_v7(),
            tenant_id,
            parent_category_id: None,
            name: "Test Category".to_string(),
            description: Some("Test description".to_string()),
            code: Some("TEST".to_string()),
            path: String::new(),
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
        };

        // Create category
        let created = repo.create(category.clone()).await.unwrap();

        // Find by ID
        let found = repo.find_by_id(tenant_id, created.category_id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Category");

        // Cleanup
        repo.hard_delete(tenant_id, created.category_id).await.unwrap();
    }
    */
}
