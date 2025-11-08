// TODO: Implement category repository
use async_trait::async_trait;
use sqlx::PgPool;

use inventory_service_core::domains::category::Category;
use inventory_service_core::dto::category::CategoryListQuery;
use inventory_service_core::repositories::category::CategoryRepository;
use inventory_service_core::Result;

pub struct CategoryRepositoryImpl {
    pool: PgPool,
}

impl CategoryRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CategoryRepository for CategoryRepositoryImpl {
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

    async fn list(
        &self,
        tenant_id: uuid::Uuid,
        query: &CategoryListQuery,
    ) -> Result<(Vec<Category>, i64)> {
        let offset = (query.page - 1) * query.page_size;

        // Build WHERE conditions
        let mut conditions = vec![
            "pc.tenant_id = $1".to_string(),
            "pc.deleted_at IS NULL".to_string(),
        ];
        let mut param_count = 1;

        if let Some(parent_id) = query.parent_id {
            param_count += 1;
            conditions.push(format!("pc.parent_category_id = ${}", param_count));
        }

        if let Some(level) = query.level {
            param_count += 1;
            conditions.push(format!("pc.level = ${}", param_count));
        }

        if let Some(is_active) = query.is_active {
            param_count += 1;
            conditions.push(format!("pc.is_active = ${}", param_count));
        }

        if let Some(is_visible) = query.is_visible {
            param_count += 1;
            conditions.push(format!("pc.is_visible = ${}", param_count));
        }

        if let Some(ref search) = query.search {
            param_count += 1;
            conditions.push(format!(
                "(pc.name ILIKE ${} OR pc.description ILIKE ${})",
                param_count,
                param_count + 1
            ));
            param_count += 1;
        }

        let where_clause = conditions.join(" AND ");

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

        // For now, implement a simpler version without dynamic SQL
        // TODO: Optimize with proper dynamic query building
        let search_pattern = query.search.as_ref().map(|s| format!("%{}%", s));

        // Count query
        let count_row = if let Some(ref search) = query.search {
            sqlx::query!(
                r#"
                SELECT COUNT(*) as count FROM product_categories pc
                WHERE pc.tenant_id = $1
                  AND pc.deleted_at IS NULL
                  AND (pc.parent_category_id = $2 OR $2 IS NULL)
                  AND (pc.level = $3 OR $3 IS NULL)
                  AND (pc.is_active = $4 OR $4 IS NULL)
                  AND (pc.is_visible = $5 OR $5 IS NULL)
                  AND (pc.name ILIKE $6 OR pc.description ILIKE $6 OR $6 IS NULL)
                "#,
                tenant_id,
                query.parent_id,
                query.level,
                query.is_active,
                query.is_visible,
                search_pattern
            )
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query!(
                r#"
                SELECT COUNT(*) as count FROM product_categories pc
                WHERE pc.tenant_id = $1
                  AND pc.deleted_at IS NULL
                  AND (pc.parent_category_id = $2 OR $2 IS NULL)
                  AND (pc.level = $3 OR $3 IS NULL)
                  AND (pc.is_active = $4 OR $4 IS NULL)
                  AND (pc.is_visible = $5 OR $5 IS NULL)
                "#,
                tenant_id,
                query.parent_id,
                query.level,
                query.is_active,
                query.is_visible
            )
            .fetch_one(&self.pool)
            .await?
        };

        // Data query
        let categories: Vec<Category> = if let Some(ref search) = query.search {
            sqlx::query_as!(
                Category,
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
                ORDER BY pc.display_order ASC, pc.name ASC
                LIMIT $7 OFFSET $8
                "#,
                tenant_id,
                query.parent_id,
                query.level,
                query.is_active,
                query.is_visible,
                search_pattern,
                query.page_size,
                offset
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                Category,
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
                ORDER BY pc.display_order ASC, pc.name ASC
                LIMIT $6 OFFSET $7
                "#,
                tenant_id,
                query.parent_id,
                query.level,
                query.is_active,
                query.is_visible,
                query.page_size,
                offset
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok((categories, count_row.count))
    }

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
        let mut tree = Vec::new();
        for category in root_categories {
            let node = self.build_category_node(tenant_id, category).await?;
            tree.push(node);
        }

        Ok(tree)
    }

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

        Ok(row.count > 0)
    }

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

        Ok(row.count > 0)
    }

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

        Ok(row.count > 0)
    }

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

        Ok(row.can_delete)
    }

    async fn get_stats(
        &self,
        tenant_id: uuid::Uuid,
        category_id: uuid::Uuid,
    ) -> Result<inventory_service_core::dto::category::CategoryStatsResponse> {
        let stats = sqlx::query_as!(
            inventory_service_core::dto::category::CategoryStatsResponse,
            r#"
            SELECT
                pc.category_id,
                pc.name,
                pc.level,
                pc.product_count,
                pc.total_product_count,
                COALESCE(sub.subcategory_count, 0) as subcategory_count,
                COALESCE(prod.active_count, 0) as active_product_count,
                COALESCE(prod.inactive_count, 0) as inactive_product_count
            FROM product_categories pc
            LEFT JOIN (
                SELECT
                    parent_category_id,
                    COUNT(*) as subcategory_count
                FROM product_categories
                WHERE tenant_id = $1 AND deleted_at IS NULL
                GROUP BY parent_category_id
            ) sub ON sub.parent_category_id = pc.category_id
            LEFT JOIN (
                SELECT
                    category_id,
                    COUNT(*) FILTER (WHERE is_active = true) as active_count,
                    COUNT(*) FILTER (WHERE is_active = false) as inactive_count
                FROM products
                WHERE tenant_id = $1 AND deleted_at IS NULL
                GROUP BY category_id
            ) prod ON prod.category_id = pc.category_id
            WHERE pc.tenant_id = $1 AND pc.category_id = $2 AND pc.deleted_at IS NULL
            "#,
            tenant_id,
            category_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(stats)
    }

    async fn update_product_counts(
        &self,
        tenant_id: uuid::Uuid,
        category_id: uuid::Uuid,
    ) -> Result<i32> {
        let result = sqlx::query!(
            r#"
            SELECT update_category_product_count()
            WHERE EXISTS (
                SELECT 1 FROM product_categories
                WHERE tenant_id = $1 AND category_id = $2 AND deleted_at IS NULL
            )
            "#,
            tenant_id,
            category_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i32)
    }

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

        let product_ids = rows.into_iter().map(|row| row.product_id).collect();
        Ok(product_ids)
    }

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
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    /// Helper method to build a category node with its children
    async fn build_category_node(
        &self,
        tenant_id: uuid::Uuid,
        category: Category,
    ) -> Result<inventory_service_core::domains::category::CategoryNode> {
        let children = self.get_children(tenant_id, category.category_id).await?;
        let mut child_nodes = Vec::new();

        for child in children {
            let child_node = self.build_category_node(tenant_id, child).await?;
            child_nodes.push(child_node);
        }

        Ok(inventory_service_core::domains::category::CategoryNode::with_children(
            category,
            child_nodes,
        ))
    }
}
