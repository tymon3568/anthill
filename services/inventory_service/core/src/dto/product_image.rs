//! Product Image Data Transfer Objects
//!
//! DTOs for product image API operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::product_image::ProductImage;

/// Response DTO for a product image
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProductImageResponse {
    /// Image ID
    pub id: Uuid,

    /// Product ID this image belongs to
    pub product_id: Uuid,

    /// Public URL to access the image
    pub url: String,

    /// Alternative text for accessibility
    pub alt_text: Option<String>,

    /// Display position (0-based)
    pub position: i32,

    /// Whether this is the primary/featured image
    pub is_primary: bool,

    /// File size in bytes
    pub file_size: Option<i32>,

    /// MIME type
    pub mime_type: Option<String>,

    /// Image width in pixels
    pub width: Option<i32>,

    /// Image height in pixels
    pub height: Option<i32>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl From<ProductImage> for ProductImageResponse {
    fn from(image: ProductImage) -> Self {
        Self {
            id: image.id,
            product_id: image.product_id,
            url: image.url,
            alt_text: image.alt_text,
            position: image.position,
            is_primary: image.is_primary,
            file_size: image.file_size,
            mime_type: image.mime_type,
            width: image.width,
            height: image.height,
            created_at: image.created_at,
        }
    }
}

/// Request DTO for updating image metadata
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductImageRequest {
    /// Alternative text for accessibility
    #[validate(length(max = 255))]
    pub alt_text: Option<String>,
}

/// Request DTO for reordering images
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ReorderImagesRequest {
    /// Ordered list of image IDs (first = position 0)
    #[validate(length(min = 1, max = 10))]
    pub image_ids: Vec<Uuid>,
}

/// Response DTO for image list
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProductImagesListResponse {
    /// List of product images
    pub images: Vec<ProductImageResponse>,

    /// Total count
    pub total: usize,
}

/// Response DTO for upload result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct UploadImageResponse {
    /// The uploaded image
    pub image: ProductImageResponse,

    /// Success message
    pub message: String,
}

/// Response DTO for delete result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct DeleteImageResponse {
    /// Whether deletion was successful
    pub success: bool,

    /// Message
    pub message: String,
}
