//! Product Image Service Implementation
//!
//! Handles product image upload, processing, and management.

use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::domains::inventory::product_image::ProductImage;
use inventory_service_core::dto::product_image::{
    DeleteImageResponse, ProductImageResponse, ProductImagesListResponse, ReorderImagesRequest,
    UpdateProductImageRequest, UploadImageResponse,
};
use inventory_service_core::repositories::ProductImageRepository;
use inventory_service_core::services::product_image::{
    ProductImageService, ALLOWED_MIME_TYPES, MAX_IMAGES_PER_PRODUCT, MAX_IMAGE_SIZE_BYTES,
};
use inventory_service_core::Result;
use shared_error::AppError;

use crate::storage::{
    process_product_image, validate_image_magic_bytes, ImageProcessingConfig, SharedStorageClient,
};

/// PostgreSQL + RustFS implementation of ProductImageService
pub struct ProductImageServiceImpl {
    repository: Arc<dyn ProductImageRepository>,
    storage: SharedStorageClient,
    processing_config: ImageProcessingConfig,
}

impl ProductImageServiceImpl {
    /// Create a new ProductImageService
    pub fn new(repository: Arc<dyn ProductImageRepository>, storage: SharedStorageClient) -> Self {
        Self {
            repository,
            storage,
            processing_config: ImageProcessingConfig::default(),
        }
    }

    /// Generate object key for product image
    fn generate_object_key(tenant_id: Uuid, product_id: Uuid, image_id: Uuid, ext: &str) -> String {
        format!("products/{}/{}/{}.{}", tenant_id, product_id, image_id, ext)
    }

    /// Get file extension from MIME type
    fn get_extension_from_mime(mime_type: &str) -> &'static str {
        match mime_type {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            "image/gif" => "gif",
            _ => "jpg",
        }
    }
}

#[async_trait]
impl ProductImageService for ProductImageServiceImpl {
    async fn upload_image(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        file_data: Vec<u8>,
        _file_name: &str,
        content_type: &str,
    ) -> Result<UploadImageResponse> {
        // Validate file size
        if file_data.len() > MAX_IMAGE_SIZE_BYTES {
            return Err(AppError::ValidationError(format!(
                "Image file size exceeds maximum of {} MB",
                MAX_IMAGE_SIZE_BYTES / (1024 * 1024)
            )));
        }

        // Validate magic bytes and get actual MIME type
        let detected_mime = validate_image_magic_bytes(&file_data)?;

        // Validate MIME type is allowed
        if !ALLOWED_MIME_TYPES.contains(&detected_mime.as_str()) {
            return Err(AppError::ValidationError(format!(
                "Image type '{}' is not allowed. Allowed types: {:?}",
                detected_mime, ALLOWED_MIME_TYPES
            )));
        }

        // Log content type mismatch
        if content_type != detected_mime {
            tracing::warn!(
                claimed = %content_type,
                detected = %detected_mime,
                "Content-Type mismatch for product image upload"
            );
        }

        // Check max images limit
        let current_count = self
            .repository
            .count_by_product(tenant_id, product_id)
            .await?;
        if current_count >= MAX_IMAGES_PER_PRODUCT {
            return Err(AppError::ValidationError(format!(
                "Maximum of {} images per product exceeded",
                MAX_IMAGES_PER_PRODUCT
            )));
        }

        // Process image (resize if needed, compress)
        let processed = process_product_image(&file_data, &detected_mime, &self.processing_config)?;

        // Generate IDs and paths
        let image_id = Uuid::now_v7();
        let extension = Self::get_extension_from_mime(&processed.content_type);
        let object_key = Self::generate_object_key(tenant_id, product_id, image_id, extension);

        // Upload to storage
        let url = self
            .storage
            .upload(&object_key, processed.data, &processed.content_type)
            .await?;

        // Get next position
        let position = self
            .repository
            .get_next_position(tenant_id, product_id)
            .await?;

        // First image is automatically primary
        let is_primary = current_count == 0;

        // Create database record
        let now = Utc::now();
        let image = ProductImage {
            id: image_id,
            product_id,
            tenant_id,
            url: url.clone(),
            alt_text: None,
            position,
            is_primary,
            file_size: Some(processed.file_size as i32),
            mime_type: Some(processed.content_type.clone()),
            width: Some(processed.final_dimensions.0 as i32),
            height: Some(processed.final_dimensions.1 as i32),
            object_key: object_key.clone(),
            created_at: now,
            updated_at: now,
        };

        let saved_image = self.repository.save(&image).await?;

        tracing::info!(
            image_id = %image_id,
            product_id = %product_id,
            tenant_id = %tenant_id,
            file_size = %processed.file_size,
            dimensions = %format!("{}x{}", processed.final_dimensions.0, processed.final_dimensions.1),
            "Product image uploaded successfully"
        );

        Ok(UploadImageResponse {
            image: ProductImageResponse::from(saved_image),
            message: "Image uploaded successfully".to_string(),
        })
    }

    async fn list_images(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<ProductImagesListResponse> {
        let images = self
            .repository
            .find_by_product(tenant_id, product_id)
            .await?;
        let total = images.len();

        Ok(ProductImagesListResponse {
            images: images.into_iter().map(ProductImageResponse::from).collect(),
            total,
        })
    }

    async fn get_image(&self, tenant_id: Uuid, image_id: Uuid) -> Result<Option<ProductImage>> {
        self.repository.find_by_id(tenant_id, image_id).await
    }

    async fn update_image(
        &self,
        tenant_id: Uuid,
        image_id: Uuid,
        request: UpdateProductImageRequest,
    ) -> Result<ProductImage> {
        // Find existing image
        let image = self
            .repository
            .find_by_id(tenant_id, image_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Image not found".to_string()))?;

        // Update with new alt_text
        let mut updated_image = image;
        updated_image.alt_text = request.alt_text;

        self.repository.update(&updated_image).await
    }

    async fn delete_image(&self, tenant_id: Uuid, image_id: Uuid) -> Result<DeleteImageResponse> {
        // Find existing image
        let image = self
            .repository
            .find_by_id(tenant_id, image_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Image not found".to_string()))?;

        let product_id = image.product_id;
        let was_primary = image.is_primary;
        let object_key = image.object_key.clone();

        // Delete from database first
        let deleted = self.repository.delete(tenant_id, image_id).await?;

        if !deleted {
            return Err(AppError::NotFound("Image not found".to_string()));
        }

        // Delete from storage (silent - don't fail if storage delete fails)
        self.storage.delete_silent(&object_key).await;

        // If deleted image was primary, set next image as primary
        if was_primary {
            let remaining_images = self
                .repository
                .find_by_product(tenant_id, product_id)
                .await?;
            if let Some(first_image) = remaining_images.first() {
                self.repository
                    .set_primary(tenant_id, product_id, first_image.id)
                    .await?;
            }
        }

        tracing::info!(
            image_id = %image_id,
            product_id = %product_id,
            tenant_id = %tenant_id,
            "Product image deleted successfully"
        );

        Ok(DeleteImageResponse {
            success: true,
            message: "Image deleted successfully".to_string(),
        })
    }

    async fn reorder_images(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        request: ReorderImagesRequest,
    ) -> Result<()> {
        // Validate that all image IDs belong to this product
        let existing_images = self
            .repository
            .find_by_product(tenant_id, product_id)
            .await?;
        let existing_ids: std::collections::HashSet<_> =
            existing_images.iter().map(|img| img.id).collect();

        for id in &request.image_ids {
            if !existing_ids.contains(id) {
                return Err(AppError::ValidationError(format!(
                    "Image {} does not belong to product {}",
                    id, product_id
                )));
            }
        }

        // Reorder
        self.repository
            .reorder(tenant_id, product_id, &request.image_ids)
            .await?;

        tracing::info!(
            product_id = %product_id,
            tenant_id = %tenant_id,
            "Product images reordered successfully"
        );

        Ok(())
    }

    async fn set_primary_image(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        image_id: Uuid,
    ) -> Result<()> {
        // Verify image exists and belongs to product
        let image = self
            .repository
            .find_by_id(tenant_id, image_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Image not found".to_string()))?;

        if image.product_id != product_id {
            return Err(AppError::ValidationError(format!(
                "Image {} does not belong to product {}",
                image_id, product_id
            )));
        }

        self.repository
            .set_primary(tenant_id, product_id, image_id)
            .await?;

        tracing::info!(
            image_id = %image_id,
            product_id = %product_id,
            tenant_id = %tenant_id,
            "Primary product image set successfully"
        );

        Ok(())
    }
}
