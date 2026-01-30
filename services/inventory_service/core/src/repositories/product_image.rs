//! Product Image Repository Trait
//!
//! Defines the data access interface for product image operations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::product_image::ProductImage;
use crate::Result;

/// Repository trait for product image data access
#[async_trait]
pub trait ProductImageRepository: Send + Sync {
    /// Find all images for a product, ordered by position
    async fn find_by_product(&self, tenant_id: Uuid, product_id: Uuid)
        -> Result<Vec<ProductImage>>;

    /// Find a single image by ID
    async fn find_by_id(&self, tenant_id: Uuid, image_id: Uuid) -> Result<Option<ProductImage>>;

    /// Count images for a product
    async fn count_by_product(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64>;

    /// Get the next available position for a product
    async fn get_next_position(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i32>;

    /// Save a new image
    async fn save(&self, image: &ProductImage) -> Result<ProductImage>;

    /// Update image metadata (alt_text only)
    async fn update(&self, image: &ProductImage) -> Result<ProductImage>;

    /// Delete an image by ID
    async fn delete(&self, tenant_id: Uuid, image_id: Uuid) -> Result<bool>;

    /// Reorder images for a product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `image_ids` - Ordered list of image IDs (first = position 0)
    async fn reorder(&self, tenant_id: Uuid, product_id: Uuid, image_ids: &[Uuid]) -> Result<()>;

    /// Set an image as primary (and unset others)
    async fn set_primary(&self, tenant_id: Uuid, product_id: Uuid, image_id: Uuid) -> Result<()>;

    /// Unset primary flag for all images of a product
    async fn unset_all_primary(&self, tenant_id: Uuid, product_id: Uuid) -> Result<()>;
}
