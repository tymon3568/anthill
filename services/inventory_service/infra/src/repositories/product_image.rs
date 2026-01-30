//! Product Image repository implementation
//!
//! PostgreSQL implementation of the ProductImageRepository trait.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use inventory_service_core::domains::inventory::product_image::ProductImage;
use inventory_service_core::repositories::product_image::ProductImageRepository;
use inventory_service_core::Result;
use shared_error::AppError;

/// PostgreSQL implementation of ProductImageRepository
pub struct ProductImageRepositoryImpl {
    pool: PgPool,
}

impl ProductImageRepositoryImpl {
    /// Create new repository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductImageRepository for ProductImageRepositoryImpl {
    async fn find_by_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<Vec<ProductImage>> {
        let rows = sqlx::query_as!(
            ProductImage,
            r#"
            SELECT
                id,
                product_id,
                tenant_id,
                url,
                alt_text,
                position,
                is_primary,
                file_size,
                mime_type,
                width,
                height,
                object_key,
                created_at,
                updated_at
            FROM product_images
            WHERE tenant_id = $1 AND product_id = $2
            ORDER BY position ASC
            "#,
            tenant_id,
            product_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to fetch product images: {}", e)))?;

        Ok(rows)
    }

    async fn find_by_id(&self, tenant_id: Uuid, image_id: Uuid) -> Result<Option<ProductImage>> {
        let row = sqlx::query_as!(
            ProductImage,
            r#"
            SELECT
                id,
                product_id,
                tenant_id,
                url,
                alt_text,
                position,
                is_primary,
                file_size,
                mime_type,
                width,
                height,
                object_key,
                created_at,
                updated_at
            FROM product_images
            WHERE tenant_id = $1 AND id = $2
            "#,
            tenant_id,
            image_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to fetch product image: {}", e)))?;

        Ok(row)
    }

    async fn count_by_product(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM product_images
            WHERE tenant_id = $1 AND product_id = $2
            "#,
            tenant_id,
            product_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to count product images: {}", e)))?;

        Ok(count)
    }

    async fn get_next_position(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i32> {
        let max_position = sqlx::query_scalar!(
            r#"
            SELECT COALESCE(MAX(position), -1) + 1 as "next_position!"
            FROM product_images
            WHERE tenant_id = $1 AND product_id = $2
            "#,
            tenant_id,
            product_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to get next position: {}", e)))?;

        Ok(max_position)
    }

    async fn save(&self, image: &ProductImage) -> Result<ProductImage> {
        let row = sqlx::query_as!(
            ProductImage,
            r#"
            INSERT INTO product_images (
                id, product_id, tenant_id, url, alt_text, position,
                is_primary, file_size, mime_type, width, height, object_key,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING
                id,
                product_id,
                tenant_id,
                url,
                alt_text,
                position,
                is_primary,
                file_size,
                mime_type,
                width,
                height,
                object_key,
                created_at,
                updated_at
            "#,
            image.id,
            image.product_id,
            image.tenant_id,
            image.url,
            image.alt_text,
            image.position,
            image.is_primary,
            image.file_size,
            image.mime_type,
            image.width,
            image.height,
            image.object_key,
            image.created_at,
            image.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to save product image: {}", e)))?;

        Ok(row)
    }

    async fn update(&self, image: &ProductImage) -> Result<ProductImage> {
        let now: DateTime<Utc> = Utc::now();
        let row = sqlx::query_as!(
            ProductImage,
            r#"
            UPDATE product_images
            SET alt_text = $3, updated_at = $4
            WHERE tenant_id = $1 AND id = $2
            RETURNING
                id,
                product_id,
                tenant_id,
                url,
                alt_text,
                position,
                is_primary,
                file_size,
                mime_type,
                width,
                height,
                object_key,
                created_at,
                updated_at
            "#,
            image.tenant_id,
            image.id,
            image.alt_text,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to update product image: {}", e)))?;

        Ok(row)
    }

    async fn delete(&self, tenant_id: Uuid, image_id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM product_images
            WHERE tenant_id = $1 AND id = $2
            "#,
            tenant_id,
            image_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to delete product image: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn reorder(&self, tenant_id: Uuid, product_id: Uuid, image_ids: &[Uuid]) -> Result<()> {
        // Use a transaction to update all positions atomically
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                AppError::InternalError(format!("Failed to begin transaction: {}", e))
            })?;

        for (position, image_id) in image_ids.iter().enumerate() {
            sqlx::query!(
                r#"
                UPDATE product_images
                SET position = $4, updated_at = NOW()
                WHERE tenant_id = $1 AND product_id = $2 AND id = $3
                "#,
                tenant_id,
                product_id,
                image_id,
                position as i32
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to reorder images: {}", e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    async fn set_primary(&self, tenant_id: Uuid, product_id: Uuid, image_id: Uuid) -> Result<()> {
        // Use a transaction to unset all then set the new primary
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                AppError::InternalError(format!("Failed to begin transaction: {}", e))
            })?;

        // First unset all primary flags for this product
        sqlx::query!(
            r#"
            UPDATE product_images
            SET is_primary = false, updated_at = NOW()
            WHERE tenant_id = $1 AND product_id = $2 AND is_primary = true
            "#,
            tenant_id,
            product_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to unset primary images: {}", e)))?;

        // Set the new primary
        sqlx::query!(
            r#"
            UPDATE product_images
            SET is_primary = true, updated_at = NOW()
            WHERE tenant_id = $1 AND product_id = $2 AND id = $3
            "#,
            tenant_id,
            product_id,
            image_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to set primary image: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    async fn unset_all_primary(&self, tenant_id: Uuid, product_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE product_images
            SET is_primary = false, updated_at = NOW()
            WHERE tenant_id = $1 AND product_id = $2 AND is_primary = true
            "#,
            tenant_id,
            product_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to unset primary images: {}", e)))?;

        Ok(())
    }
}
