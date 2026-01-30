//! Product Variant repository implementation
//!
//! PostgreSQL implementation of the ProductVariantRepository trait.

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use inventory_service_core::domains::inventory::product_variant::ProductVariant;
use inventory_service_core::dto::product_variant::{
    VariantListQuery, VariantResponse, VariantSortDirection,
};
use inventory_service_core::repositories::product_variant::ProductVariantRepository;
use inventory_service_core::Result;
use shared_error::AppError;

/// PostgreSQL implementation of ProductVariantRepository
pub struct ProductVariantRepositoryImpl {
    pool: PgPool,
}

impl ProductVariantRepositoryImpl {
    /// Create new repository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Helper to map row to ProductVariant
    fn map_row_to_variant(row: &sqlx::postgres::PgRow) -> ProductVariant {
        use sqlx::Row;
        ProductVariant {
            variant_id: row.get("variant_id"),
            tenant_id: row.get("tenant_id"),
            parent_product_id: row.get("parent_product_id"),
            variant_attributes: row.get("variant_attributes"),
            sku: row.get("sku"),
            barcode: row.get("barcode"),
            price_difference: row.get("price_difference"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
        }
    }

    /// Helper to map row to VariantResponse with parent info
    fn map_row_to_response(row: &sqlx::postgres::PgRow) -> VariantResponse {
        use sqlx::Row;
        VariantResponse {
            variant_id: row.get("variant_id"),
            tenant_id: row.get("tenant_id"),
            parent_product_id: row.get("parent_product_id"),
            variant_attributes: row.get("variant_attributes"),
            sku: row.get("sku"),
            barcode: row.get("barcode"),
            price_difference: row.get("price_difference"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            parent_product_name: row.try_get("parent_product_name").ok(),
            parent_product_sku: row.try_get("parent_product_sku").ok(),
        }
    }
}

#[async_trait]
impl ProductVariantRepository for ProductVariantRepositoryImpl {
    // ========================================================================
    // CRUD Operations
    // ========================================================================

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        variant_id: Uuid,
    ) -> Result<Option<ProductVariant>> {
        let row = sqlx::query(
            r#"
            SELECT
                variant_id, tenant_id, parent_product_id, variant_attributes,
                sku, barcode, price_difference, is_active,
                created_at, updated_at, deleted_at
            FROM product_variants
            WHERE tenant_id = $1 AND variant_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(variant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_row_to_variant))
    }

    async fn find_by_id_with_parent(
        &self,
        tenant_id: Uuid,
        variant_id: Uuid,
    ) -> Result<Option<VariantResponse>> {
        let row = sqlx::query(
            r#"
            SELECT
                v.variant_id, v.tenant_id, v.parent_product_id, v.variant_attributes,
                v.sku, v.barcode, v.price_difference, v.is_active,
                v.created_at, v.updated_at,
                p.name as parent_product_name, p.sku as parent_product_sku
            FROM product_variants v
            LEFT JOIN products p ON v.parent_product_id = p.product_id AND p.deleted_at IS NULL
            WHERE v.tenant_id = $1 AND v.variant_id = $2 AND v.deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(variant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_row_to_response))
    }

    async fn find_by_sku(&self, tenant_id: Uuid, sku: &str) -> Result<Option<VariantResponse>> {
        let row = sqlx::query(
            r#"
            SELECT
                v.variant_id, v.tenant_id, v.parent_product_id, v.variant_attributes,
                v.sku, v.barcode, v.price_difference, v.is_active,
                v.created_at, v.updated_at,
                p.name as parent_product_name, p.sku as parent_product_sku
            FROM product_variants v
            LEFT JOIN products p ON v.parent_product_id = p.product_id AND p.deleted_at IS NULL
            WHERE v.tenant_id = $1 AND v.sku = $2 AND v.deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(sku)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_row_to_response))
    }

    async fn find_by_barcode(
        &self,
        tenant_id: Uuid,
        barcode: &str,
    ) -> Result<Option<VariantResponse>> {
        let row = sqlx::query(
            r#"
            SELECT
                v.variant_id, v.tenant_id, v.parent_product_id, v.variant_attributes,
                v.sku, v.barcode, v.price_difference, v.is_active,
                v.created_at, v.updated_at,
                p.name as parent_product_name, p.sku as parent_product_sku
            FROM product_variants v
            LEFT JOIN products p ON v.parent_product_id = p.product_id AND p.deleted_at IS NULL
            WHERE v.tenant_id = $1 AND v.barcode = $2 AND v.deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(barcode)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_row_to_response))
    }

    async fn list(
        &self,
        tenant_id: Uuid,
        query: &VariantListQuery,
    ) -> Result<(Vec<VariantResponse>, i64)> {
        // Build main query
        let mut query_builder = QueryBuilder::new(
            r#"
            SELECT
                v.variant_id, v.tenant_id, v.parent_product_id, v.variant_attributes,
                v.sku, v.barcode, v.price_difference, v.is_active,
                v.created_at, v.updated_at,
                p.name as parent_product_name, p.sku as parent_product_sku
            FROM product_variants v
            LEFT JOIN products p ON v.parent_product_id = p.product_id AND p.deleted_at IS NULL
            WHERE v.tenant_id =
            "#,
        );
        query_builder.push_bind(tenant_id);
        query_builder.push(" AND v.deleted_at IS NULL");

        // Add filters
        if let Some(parent_product_id) = query.parent_product_id {
            query_builder.push(" AND v.parent_product_id = ");
            query_builder.push_bind(parent_product_id);
        }

        if let Some(is_active) = query.is_active {
            query_builder.push(" AND v.is_active = ");
            query_builder.push_bind(is_active);
        }

        if let Some(search) = &query.search {
            let search_pattern = format!("%{}%", search);
            query_builder.push(" AND (v.sku ILIKE ");
            query_builder.push_bind(search_pattern.clone());
            query_builder.push(" OR v.barcode ILIKE ");
            query_builder.push_bind(search_pattern.clone());
            query_builder.push(" OR p.name ILIKE ");
            query_builder.push_bind(search_pattern);
            query_builder.push(")");
        }

        // Add sorting
        let sort_column = match query.sort_by.as_str() {
            "sku" => "v.sku",
            "barcode" => "v.barcode",
            "price_difference" => "v.price_difference",
            "is_active" => "v.is_active",
            "created_at" => "v.created_at",
            "updated_at" => "v.updated_at",
            "parent_product_name" => "p.name",
            _ => "v.sku",
        };

        query_builder.push(" ORDER BY ");
        query_builder.push(sort_column);

        match query.sort_dir {
            VariantSortDirection::Asc => query_builder.push(" ASC"),
            VariantSortDirection::Desc => query_builder.push(" DESC"),
        };

        // Add pagination
        let offset = (query.page - 1) * query.page_size;
        query_builder.push(" LIMIT ");
        query_builder.push_bind(query.page_size);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        // Execute main query
        let rows = query_builder.build().fetch_all(&self.pool).await?;
        let variants: Vec<VariantResponse> = rows.iter().map(Self::map_row_to_response).collect();

        // Build count query
        let mut count_builder = QueryBuilder::new(
            "SELECT COUNT(*) as count FROM product_variants v WHERE v.tenant_id = ",
        );
        count_builder.push_bind(tenant_id);
        count_builder.push(" AND v.deleted_at IS NULL");

        if let Some(parent_product_id) = query.parent_product_id {
            count_builder.push(" AND v.parent_product_id = ");
            count_builder.push_bind(parent_product_id);
        }

        if let Some(is_active) = query.is_active {
            count_builder.push(" AND v.is_active = ");
            count_builder.push_bind(is_active);
        }

        if let Some(search) = &query.search {
            let search_pattern = format!("%{}%", search);
            count_builder.push(
                " AND (v.sku ILIKE $x OR v.barcode ILIKE $x OR EXISTS (
                    SELECT 1 FROM products p WHERE p.product_id = v.parent_product_id AND p.name ILIKE ",
            );
            count_builder.push_bind(search_pattern);
            count_builder.push("))");
        }

        let total_count: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await?;

        Ok((variants, total_count))
    }

    async fn create(&self, variant: &ProductVariant) -> Result<ProductVariant> {
        let row = sqlx::query(
            r#"
            INSERT INTO product_variants (
                variant_id, tenant_id, parent_product_id, variant_attributes,
                sku, barcode, price_difference, is_active,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING
                variant_id, tenant_id, parent_product_id, variant_attributes,
                sku, barcode, price_difference, is_active,
                created_at, updated_at, deleted_at
            "#,
        )
        .bind(variant.variant_id)
        .bind(variant.tenant_id)
        .bind(variant.parent_product_id)
        .bind(&variant.variant_attributes)
        .bind(&variant.sku)
        .bind(&variant.barcode)
        .bind(variant.price_difference)
        .bind(variant.is_active)
        .bind(variant.created_at)
        .bind(variant.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::map_row_to_variant(&row))
    }

    async fn update(
        &self,
        tenant_id: Uuid,
        variant_id: Uuid,
        variant: &ProductVariant,
    ) -> Result<ProductVariant> {
        let row = sqlx::query(
            r#"
            UPDATE product_variants SET
                variant_attributes = $3,
                sku = $4,
                barcode = $5,
                price_difference = $6,
                is_active = $7,
                updated_at = $8
            WHERE tenant_id = $1 AND variant_id = $2 AND deleted_at IS NULL
            RETURNING
                variant_id, tenant_id, parent_product_id, variant_attributes,
                sku, barcode, price_difference, is_active,
                created_at, updated_at, deleted_at
            "#,
        )
        .bind(tenant_id)
        .bind(variant_id)
        .bind(&variant.variant_attributes)
        .bind(&variant.sku)
        .bind(&variant.barcode)
        .bind(variant.price_difference)
        .bind(variant.is_active)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Variant not found".to_string()))?;

        Ok(Self::map_row_to_variant(&row))
    }

    async fn delete(&self, tenant_id: Uuid, variant_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE product_variants
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND variant_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(variant_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========================================================================
    // Bulk Operations
    // ========================================================================

    async fn bulk_activate(&self, tenant_id: Uuid, variant_ids: &[Uuid]) -> Result<i64> {
        if variant_ids.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query(
            r#"
            UPDATE product_variants
            SET is_active = true, updated_at = NOW()
            WHERE tenant_id = $1 AND variant_id = ANY($2) AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(variant_ids)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    async fn bulk_deactivate(&self, tenant_id: Uuid, variant_ids: &[Uuid]) -> Result<i64> {
        if variant_ids.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query(
            r#"
            UPDATE product_variants
            SET is_active = false, updated_at = NOW()
            WHERE tenant_id = $1 AND variant_id = ANY($2) AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(variant_ids)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    async fn bulk_delete(&self, tenant_id: Uuid, variant_ids: &[Uuid]) -> Result<i64> {
        if variant_ids.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query(
            r#"
            UPDATE product_variants
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND variant_id = ANY($2) AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(variant_ids)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    // ========================================================================
    // Validation
    // ========================================================================

    async fn sku_exists(
        &self,
        tenant_id: Uuid,
        sku: &str,
        exclude_variant_id: Option<Uuid>,
    ) -> Result<bool> {
        let count: i64 = if let Some(exclude_id) = exclude_variant_id {
            sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM product_variants
                WHERE tenant_id = $1 AND sku = $2 AND variant_id != $3 AND deleted_at IS NULL
                "#,
            )
            .bind(tenant_id)
            .bind(sku)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM product_variants
                WHERE tenant_id = $1 AND sku = $2 AND deleted_at IS NULL
                "#,
            )
            .bind(tenant_id)
            .bind(sku)
            .fetch_one(&self.pool)
            .await?
        };

        Ok(count > 0)
    }

    async fn attributes_exist(
        &self,
        tenant_id: Uuid,
        parent_product_id: Uuid,
        variant_attributes: &serde_json::Value,
        exclude_variant_id: Option<Uuid>,
    ) -> Result<bool> {
        let count: i64 = if let Some(exclude_id) = exclude_variant_id {
            sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM product_variants
                WHERE tenant_id = $1 AND parent_product_id = $2
                  AND variant_attributes = $3
                  AND variant_id != $4 AND deleted_at IS NULL
                "#,
            )
            .bind(tenant_id)
            .bind(parent_product_id)
            .bind(variant_attributes)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM product_variants
                WHERE tenant_id = $1 AND parent_product_id = $2
                  AND variant_attributes = $3 AND deleted_at IS NULL
                "#,
            )
            .bind(tenant_id)
            .bind(parent_product_id)
            .bind(variant_attributes)
            .fetch_one(&self.pool)
            .await?
        };

        Ok(count > 0)
    }
}
