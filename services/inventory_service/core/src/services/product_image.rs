//! Product Image Service Trait
//!
//! Defines the business logic interface for product image operations.
//! This trait handles image upload, processing, and management.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::product_image::ProductImage;
use crate::dto::product_image::{
    DeleteImageResponse, ProductImagesListResponse, ReorderImagesRequest,
    UpdateProductImageRequest, UploadImageResponse,
};
use crate::Result;

/// Maximum number of images per product
pub const MAX_IMAGES_PER_PRODUCT: i64 = 10;

/// Maximum file size for image uploads (5MB)
pub const MAX_IMAGE_SIZE_BYTES: usize = 5 * 1024 * 1024;

/// Allowed MIME types for product images
pub const ALLOWED_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp", "image/gif"];

/// Service trait for product image business logic
///
/// This trait defines all business operations for product images.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait ProductImageService: Send + Sync {
    /// Upload a new product image
    ///
    /// # Business Rules
    /// - Maximum 10 images per product
    /// - Maximum 5MB file size
    /// - Only JPEG, PNG, WebP, GIF allowed
    /// - Images are processed (resized if too large)
    /// - First image becomes primary automatically
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `file_data` - Raw image data
    /// * `file_name` - Original file name
    /// * `content_type` - Claimed MIME type
    ///
    /// # Returns
    /// Upload response with image details
    ///
    /// # Errors
    /// - `ValidationError` if file validation fails
    /// - `NotFoundError` if product doesn't exist
    /// - `BusinessRuleViolation` if max images exceeded
    async fn upload_image(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        file_data: Vec<u8>,
        file_name: &str,
        content_type: &str,
    ) -> Result<UploadImageResponse>;

    /// List all images for a product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// List of product images ordered by position
    async fn list_images(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<ProductImagesListResponse>;

    /// Get a single image by ID
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `image_id` - Image identifier
    ///
    /// # Returns
    /// Product image if found
    async fn get_image(&self, tenant_id: Uuid, image_id: Uuid) -> Result<Option<ProductImage>>;

    /// Update image metadata (alt text)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `image_id` - Image identifier
    /// * `request` - Update request with new metadata
    ///
    /// # Returns
    /// Updated product image
    ///
    /// # Errors
    /// - `NotFoundError` if image doesn't exist
    async fn update_image(
        &self,
        tenant_id: Uuid,
        image_id: Uuid,
        request: UpdateProductImageRequest,
    ) -> Result<ProductImage>;

    /// Delete a product image
    ///
    /// # Business Rules
    /// - Deletes from storage and database
    /// - If deleted image was primary, next image becomes primary
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `image_id` - Image identifier
    ///
    /// # Returns
    /// Delete response
    ///
    /// # Errors
    /// - `NotFoundError` if image doesn't exist
    async fn delete_image(&self, tenant_id: Uuid, image_id: Uuid) -> Result<DeleteImageResponse>;

    /// Reorder images for a product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `request` - Reorder request with ordered image IDs
    ///
    /// # Errors
    /// - `ValidationError` if image IDs don't match product
    async fn reorder_images(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        request: ReorderImagesRequest,
    ) -> Result<()>;

    /// Set an image as primary
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `image_id` - Image to set as primary
    ///
    /// # Errors
    /// - `NotFoundError` if image doesn't exist
    async fn set_primary_image(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        image_id: Uuid,
    ) -> Result<()>;
}
