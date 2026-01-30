//! Image processing module for product images
//!
//! Provides resize, compression for product images.
//! Based on the user_service implementation.
//!
//! Security: Uses decoding limits to prevent memory exhaustion attacks
//! via malformed images (ZIP bomb prevention).

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::Limits;
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader};
use metrics::{counter, histogram};
use shared_error::AppError;
use std::io::Cursor;
use std::time::Instant;

/// Maximum dimension for product images (resize if larger)
const MAX_IMAGE_DIMENSION: u32 = 2048;

/// JPEG quality for compression (1-100)
const JPEG_QUALITY: u8 = 85;

/// Maximum allowed dimension during decoding (before resize)
/// This prevents memory exhaustion from images claiming absurd dimensions
const MAX_DECODE_DIMENSION: u32 = 16384; // 16K x 16K max

/// Maximum memory allocation during decoding (256 MB)
/// Prevents ZIP bomb attacks via compressed images
const MAX_DECODE_ALLOC_BYTES: u64 = 256 * 1024 * 1024;

/// Image processing configuration
#[derive(Clone, Debug)]
pub struct ImageProcessingConfig {
    /// Maximum allowed dimension
    pub max_dimension: u32,
    /// JPEG quality (1-100)
    pub jpeg_quality: u8,
}

impl Default for ImageProcessingConfig {
    fn default() -> Self {
        Self {
            max_dimension: MAX_IMAGE_DIMENSION,
            jpeg_quality: JPEG_QUALITY,
        }
    }
}

/// Result of image processing operation
pub struct ProcessedImage {
    /// Processed image data
    pub data: Vec<u8>,
    /// MIME type of the processed image
    pub content_type: String,
    /// Original dimensions (width, height)
    pub original_dimensions: (u32, u32),
    /// Final dimensions (width, height)
    pub final_dimensions: (u32, u32),
    /// File size in bytes
    pub file_size: usize,
    /// Whether the image was resized
    pub was_resized: bool,
}

/// Process a product image: resize if too large and compress
///
/// - Only resizes if image exceeds max dimensions
/// - Maintains original aspect ratio
/// - Compresses output based on format
pub fn process_product_image(
    data: &[u8],
    detected_mime: &str,
    config: &ImageProcessingConfig,
) -> Result<ProcessedImage, AppError> {
    let start = Instant::now();

    let img = load_image(data)?;
    let (orig_w, orig_h) = img.dimensions();

    // Only resize if exceeds max dimensions
    let processed = if orig_w > config.max_dimension || orig_h > config.max_dimension {
        resize_to_fit(&img, config.max_dimension, config.max_dimension)
    } else {
        img
    };

    let (final_w, final_h) = processed.dimensions();

    // Encode with compression
    let (output_data, content_type) = encode_image(&processed, detected_mime, config)?;

    let duration = start.elapsed();
    let was_resized = orig_w != final_w || orig_h != final_h;
    let file_size = output_data.len();

    // Record metrics
    counter!("image_processing_total", "operation" => "product").increment(1);
    histogram!("image_processing_duration_seconds").record(duration.as_secs_f64());

    if was_resized {
        counter!("image_processing_resized").increment(1);
    }

    tracing::info!(
        original_size = %data.len(),
        output_size = %file_size,
        original_dimensions = %format!("{}x{}", orig_w, orig_h),
        final_dimensions = %format!("{}x{}", final_w, final_h),
        duration_ms = %duration.as_millis(),
        "Product image processed"
    );

    Ok(ProcessedImage {
        data: output_data,
        content_type,
        original_dimensions: (orig_w, orig_h),
        final_dimensions: (final_w, final_h),
        file_size,
        was_resized,
    })
}

/// Load image from bytes with security limits
///
/// Applies decoding limits to prevent:
/// - Memory exhaustion from malformed images with absurd dimensions
/// - ZIP bomb attacks via highly compressed image data
fn load_image(data: &[u8]) -> Result<DynamicImage, AppError> {
    // Configure decoding limits for security
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DECODE_DIMENSION);
    limits.max_image_height = Some(MAX_DECODE_DIMENSION);
    limits.max_alloc = Some(MAX_DECODE_ALLOC_BYTES);

    let mut reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| AppError::ValidationError(format!("Failed to detect image format: {}", e)))?;

    // Apply security limits before decoding
    reader.limits(limits);

    reader.decode().map_err(|e| {
        // Check if this was a limit error for better error messaging
        let error_str = e.to_string();
        if error_str.contains("limit") || error_str.contains("Limit") {
            tracing::warn!(error = %e, "Image decoding rejected due to security limits");
            AppError::ValidationError(
                "Image exceeds processing limits. Maximum dimensions: 16384x16384, maximum memory: 256MB".to_string()
            )
        } else {
            AppError::ValidationError(format!("Failed to decode image: {}", e))
        }
    })
}

/// Resize image to fit within max dimensions while maintaining aspect ratio
fn resize_to_fit(img: &DynamicImage, max_width: u32, max_height: u32) -> DynamicImage {
    img.resize(max_width, max_height, image::imageops::FilterType::Lanczos3)
}

/// Encode image to bytes with compression
fn encode_image(
    img: &DynamicImage,
    mime_type: &str,
    config: &ImageProcessingConfig,
) -> Result<(Vec<u8>, String), AppError> {
    let mut output = Vec::new();

    match mime_type {
        "image/jpeg" => {
            let encoder = JpegEncoder::new_with_quality(&mut output, config.jpeg_quality);
            img.write_with_encoder(encoder)
                .map_err(|e| AppError::InternalError(format!("Failed to encode JPEG: {}", e)))?;
            Ok((output, "image/jpeg".to_string()))
        },
        "image/png" => {
            let encoder = PngEncoder::new_with_quality(
                &mut output,
                image::codecs::png::CompressionType::Default,
                image::codecs::png::FilterType::Adaptive,
            );
            img.write_with_encoder(encoder)
                .map_err(|e| AppError::InternalError(format!("Failed to encode PNG: {}", e)))?;
            Ok((output, "image/png".to_string()))
        },
        "image/webp" => {
            let encoder = WebPEncoder::new_lossless(&mut output);
            img.write_with_encoder(encoder)
                .map_err(|e| AppError::InternalError(format!("Failed to encode WebP: {}", e)))?;
            Ok((output, "image/webp".to_string()))
        },
        "image/gif" => {
            // GIF doesn't support quality settings, encode as-is
            let mut cursor = Cursor::new(&mut output);
            img.write_to(&mut cursor, ImageFormat::Gif)
                .map_err(|e| AppError::InternalError(format!("Failed to encode GIF: {}", e)))?;
            Ok((output, "image/gif".to_string()))
        },
        _ => {
            // Default to JPEG for unknown formats
            let encoder = JpegEncoder::new_with_quality(&mut output, config.jpeg_quality);
            img.write_with_encoder(encoder)
                .map_err(|e| AppError::InternalError(format!("Failed to encode image: {}", e)))?;
            Ok((output, "image/jpeg".to_string()))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_jpeg() -> Vec<u8> {
        // Create a simple 100x100 red image
        let img = DynamicImage::new_rgb8(100, 100);
        let mut output = Vec::new();
        let encoder = JpegEncoder::new_with_quality(&mut output, 85);
        img.write_with_encoder(encoder).unwrap();
        output
    }

    fn create_test_png() -> Vec<u8> {
        let img = DynamicImage::new_rgba8(100, 100);
        let mut output = Vec::new();
        img.write_to(&mut Cursor::new(&mut output), ImageFormat::Png)
            .unwrap();
        output
    }

    #[test]
    fn test_process_product_image_jpeg() {
        let data = create_test_jpeg();
        let config = ImageProcessingConfig::default();

        let result = process_product_image(&data, "image/jpeg", &config);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.content_type, "image/jpeg");
        // 100x100 is below max_dimension (2048), so no resize
        assert!(!processed.was_resized);
    }

    #[test]
    fn test_process_product_image_png() {
        let data = create_test_png();
        let config = ImageProcessingConfig::default();

        let result = process_product_image(&data, "image/png", &config);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.content_type, "image/png");
    }

    #[test]
    fn test_invalid_image_data() {
        let invalid_data = vec![0, 1, 2, 3, 4, 5];
        let config = ImageProcessingConfig::default();

        let result = process_product_image(&invalid_data, "image/jpeg", &config);
        assert!(result.is_err());
    }
}
