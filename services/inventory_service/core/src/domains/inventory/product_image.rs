//! Product Image domain entity
//!
//! Represents an image associated with a product.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Allowed MIME types for product images
pub const ALLOWED_IMAGE_MIME_TYPES: &[&str] =
    &["image/jpeg", "image/png", "image/webp", "image/gif"];

/// Maximum file size for product images (5MB)
pub const MAX_IMAGE_SIZE_BYTES: usize = 5 * 1024 * 1024;

/// Maximum number of images per product
pub const MAX_IMAGES_PER_PRODUCT: usize = 10;

/// Product Image entity
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProductImage {
    /// Primary key
    pub id: Uuid,

    /// Product this image belongs to
    pub product_id: Uuid,

    /// Multi-tenancy: Tenant identifier
    pub tenant_id: Uuid,

    /// Public URL to access the image
    pub url: String,

    /// Alternative text for accessibility
    #[validate(length(max = 255))]
    pub alt_text: Option<String>,

    /// Display position (0-based, lower = first)
    pub position: i32,

    /// Whether this is the primary/featured image
    pub is_primary: bool,

    /// File size in bytes
    pub file_size: Option<i32>,

    /// MIME type (image/jpeg, image/png, etc.)
    pub mime_type: Option<String>,

    /// Image width in pixels
    pub width: Option<i32>,

    /// Image height in pixels
    pub height: Option<i32>,

    /// S3/RustFS object key for file operations
    pub object_key: String,

    /// Audit: Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Audit: Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl ProductImage {
    /// Create a new product image
    pub fn new(
        product_id: Uuid,
        tenant_id: Uuid,
        url: String,
        object_key: String,
        position: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            product_id,
            tenant_id,
            url,
            alt_text: None,
            position,
            is_primary: position == 0, // First image is primary by default
            file_size: None,
            mime_type: None,
            width: None,
            height: None,
            object_key,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set image metadata after upload
    pub fn with_metadata(
        mut self,
        file_size: i32,
        mime_type: String,
        width: Option<i32>,
        height: Option<i32>,
    ) -> Self {
        self.file_size = Some(file_size);
        self.mime_type = Some(mime_type);
        self.width = width;
        self.height = height;
        self
    }

    /// Set alt text
    pub fn with_alt_text(mut self, alt_text: String) -> Self {
        self.alt_text = Some(alt_text);
        self
    }

    /// Mark as primary image
    pub fn set_primary(&mut self, is_primary: bool) {
        self.is_primary = is_primary;
        self.updated_at = Utc::now();
    }

    /// Update position
    pub fn set_position(&mut self, position: i32) {
        self.position = position;
        self.updated_at = Utc::now();
    }

    /// Touch updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Validate MIME type is allowed
pub fn validate_mime_type(mime_type: &str) -> bool {
    ALLOWED_IMAGE_MIME_TYPES.contains(&mime_type)
}

/// Validate file size is within limits
pub fn validate_file_size(size: usize) -> bool {
    size <= MAX_IMAGE_SIZE_BYTES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_product_image() {
        let product_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let image = ProductImage::new(
            product_id,
            tenant_id,
            "https://example.com/image.jpg".to_string(),
            "products/tenant/product/image.jpg".to_string(),
            0,
        );

        assert_eq!(image.product_id, product_id);
        assert_eq!(image.tenant_id, tenant_id);
        assert_eq!(image.position, 0);
        assert!(image.is_primary); // First image is primary
    }

    #[test]
    fn test_second_image_not_primary() {
        let image = ProductImage::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "https://example.com/image.jpg".to_string(),
            "products/tenant/product/image.jpg".to_string(),
            1, // Second position
        );

        assert!(!image.is_primary);
    }

    #[test]
    fn test_validate_mime_type() {
        assert!(validate_mime_type("image/jpeg"));
        assert!(validate_mime_type("image/png"));
        assert!(validate_mime_type("image/webp"));
        assert!(validate_mime_type("image/gif"));
        assert!(!validate_mime_type("image/bmp"));
        assert!(!validate_mime_type("application/pdf"));
    }

    #[test]
    fn test_validate_file_size() {
        assert!(validate_file_size(1024)); // 1KB
        assert!(validate_file_size(5 * 1024 * 1024)); // 5MB exactly
        assert!(!validate_file_size(5 * 1024 * 1024 + 1)); // Over 5MB
    }
}
