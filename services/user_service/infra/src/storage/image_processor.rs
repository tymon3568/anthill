//! Image processing module for avatars and uploads
//!
//! Provides resize, compression, and thumbnail generation for uploaded images.

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader};
use metrics::{counter, histogram};
use shared_error::AppError;
use std::io::Cursor;
use std::time::Instant;

/// Default avatar dimensions (square)
const DEFAULT_AVATAR_SIZE: u32 = 256;

/// Thumbnail dimensions for list views
const THUMBNAIL_SIZE: u32 = 64;

/// Maximum dimension for any uploaded image
const MAX_IMAGE_DIMENSION: u32 = 2048;

/// JPEG quality for compression (1-100)
const JPEG_QUALITY: u8 = 85;

/// WebP quality for compression (1-100)
const WEBP_QUALITY: u8 = 85;

/// Image processing configuration
#[derive(Clone, Debug)]
pub struct ImageProcessingConfig {
    /// Target size for avatars (width = height)
    pub avatar_size: u32,
    /// Target size for thumbnails
    pub thumbnail_size: u32,
    /// Maximum allowed dimension
    pub max_dimension: u32,
    /// JPEG quality (1-100)
    pub jpeg_quality: u8,
    /// WebP quality (1-100)
    pub webp_quality: u8,
}

impl Default for ImageProcessingConfig {
    fn default() -> Self {
        Self {
            avatar_size: DEFAULT_AVATAR_SIZE,
            thumbnail_size: THUMBNAIL_SIZE,
            max_dimension: MAX_IMAGE_DIMENSION,
            jpeg_quality: JPEG_QUALITY,
            webp_quality: WEBP_QUALITY,
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
    /// Whether the image was resized
    pub was_resized: bool,
}

/// Process an avatar image: resize to standard dimensions and compress
///
/// - Resizes to square avatar dimensions (default 256x256)
/// - Maintains aspect ratio with center crop
/// - Compresses output based on format
pub fn process_avatar(
    data: &[u8],
    detected_mime: &str,
    config: &ImageProcessingConfig,
) -> Result<ProcessedImage, AppError> {
    let start = Instant::now();

    let img = load_image(data)?;
    let (orig_w, orig_h) = img.dimensions();

    // Resize and crop to square for avatar
    let processed = resize_and_crop_square(&img, config.avatar_size);
    let (final_w, final_h) = processed.dimensions();

    // Encode with compression
    let (output_data, content_type) = encode_image(&processed, detected_mime, config)?;

    let duration = start.elapsed();
    let was_resized = orig_w != final_w || orig_h != final_h;

    // Record metrics
    counter!("image_processing_total", "operation" => "avatar").increment(1);
    histogram!("image_processing_duration_seconds").record(duration.as_secs_f64());
    histogram!("image_processing_input_bytes").record(data.len() as f64);
    histogram!("image_processing_output_bytes").record(output_data.len() as f64);

    if was_resized {
        counter!("image_processing_resized").increment(1);
    }

    let compression_ratio = if !output_data.is_empty() {
        data.len() as f64 / output_data.len() as f64
    } else {
        1.0
    };

    tracing::info!(
        original_size = %data.len(),
        output_size = %output_data.len(),
        compression_ratio = %format!("{:.2}", compression_ratio),
        original_dimensions = %format!("{}x{}", orig_w, orig_h),
        final_dimensions = %format!("{}x{}", final_w, final_h),
        duration_ms = %duration.as_millis(),
        "Avatar image processed"
    );

    Ok(ProcessedImage {
        data: output_data,
        content_type,
        original_dimensions: (orig_w, orig_h),
        final_dimensions: (final_w, final_h),
        was_resized,
    })
}

/// Process a general image: resize if too large and compress
///
/// - Only resizes if image exceeds max dimensions
/// - Maintains original aspect ratio
/// - Compresses output based on format
pub fn process_image(
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

    // Record metrics
    counter!("image_processing_total", "operation" => "general").increment(1);
    histogram!("image_processing_duration_seconds").record(duration.as_secs_f64());

    tracing::info!(
        original_size = %data.len(),
        output_size = %output_data.len(),
        original_dimensions = %format!("{}x{}", orig_w, orig_h),
        final_dimensions = %format!("{}x{}", final_w, final_h),
        duration_ms = %duration.as_millis(),
        "Image processed"
    );

    Ok(ProcessedImage {
        data: output_data,
        content_type,
        original_dimensions: (orig_w, orig_h),
        final_dimensions: (final_w, final_h),
        was_resized,
    })
}

/// Generate a thumbnail from image data
pub fn generate_thumbnail(
    data: &[u8],
    detected_mime: &str,
    config: &ImageProcessingConfig,
) -> Result<ProcessedImage, AppError> {
    let start = Instant::now();

    let img = load_image(data)?;
    let (orig_w, orig_h) = img.dimensions();

    // Resize and crop to square thumbnail
    let processed = resize_and_crop_square(&img, config.thumbnail_size);
    let (final_w, final_h) = processed.dimensions();

    // Encode with compression
    let (output_data, content_type) = encode_image(&processed, detected_mime, config)?;

    let duration = start.elapsed();

    counter!("image_processing_total", "operation" => "thumbnail").increment(1);
    histogram!("image_processing_duration_seconds").record(duration.as_secs_f64());

    tracing::debug!(
        original_dimensions = %format!("{}x{}", orig_w, orig_h),
        final_dimensions = %format!("{}x{}", final_w, final_h),
        duration_ms = %duration.as_millis(),
        "Thumbnail generated"
    );

    Ok(ProcessedImage {
        data: output_data,
        content_type,
        original_dimensions: (orig_w, orig_h),
        final_dimensions: (final_w, final_h),
        was_resized: true,
    })
}

/// Load image from bytes
fn load_image(data: &[u8]) -> Result<DynamicImage, AppError> {
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| AppError::ValidationError(format!("Failed to detect image format: {}", e)))?;

    reader
        .decode()
        .map_err(|e| AppError::ValidationError(format!("Failed to decode image: {}", e)))
}

/// Resize image to fit within max dimensions while maintaining aspect ratio
fn resize_to_fit(img: &DynamicImage, max_width: u32, max_height: u32) -> DynamicImage {
    img.resize(max_width, max_height, image::imageops::FilterType::Lanczos3)
}

/// Resize and crop image to a square of given size
fn resize_and_crop_square(img: &DynamicImage, size: u32) -> DynamicImage {
    // First resize so the smaller dimension matches target size
    let (w, h) = img.dimensions();
    let scale = if w < h {
        size as f64 / w as f64
    } else {
        size as f64 / h as f64
    };

    let new_w = (w as f64 * scale).ceil() as u32;
    let new_h = (h as f64 * scale).ceil() as u32;

    let resized = img.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3);

    // Then crop to square from center
    let (rw, rh) = resized.dimensions();
    let x = (rw.saturating_sub(size)) / 2;
    let y = (rh.saturating_sub(size)) / 2;

    resized.crop_imm(x, y, size.min(rw), size.min(rh))
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
    fn test_process_avatar_jpeg() {
        let data = create_test_jpeg();
        let config = ImageProcessingConfig::default();

        let result = process_avatar(&data, "image/jpeg", &config);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.content_type, "image/jpeg");
        assert_eq!(processed.final_dimensions, (256, 256));
        assert!(processed.was_resized);
    }

    #[test]
    fn test_process_avatar_png() {
        let data = create_test_png();
        let config = ImageProcessingConfig::default();

        let result = process_avatar(&data, "image/png", &config);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.content_type, "image/png");
        assert_eq!(processed.final_dimensions, (256, 256));
    }

    #[test]
    fn test_process_image_no_resize_needed() {
        let data = create_test_jpeg();
        let config = ImageProcessingConfig::default();

        let result = process_image(&data, "image/jpeg", &config);
        assert!(result.is_ok());

        let processed = result.unwrap();
        // 100x100 is below max_dimension (2048), so no resize
        assert!(!processed.was_resized);
    }

    #[test]
    fn test_generate_thumbnail() {
        let data = create_test_jpeg();
        let config = ImageProcessingConfig::default();

        let result = generate_thumbnail(&data, "image/jpeg", &config);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.final_dimensions, (64, 64));
        assert!(processed.was_resized);
    }

    #[test]
    fn test_custom_config() {
        let data = create_test_jpeg();
        let config = ImageProcessingConfig {
            avatar_size: 128,
            thumbnail_size: 32,
            max_dimension: 1024,
            jpeg_quality: 90,
            webp_quality: 90,
        };

        let result = process_avatar(&data, "image/jpeg", &config);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.final_dimensions, (128, 128));
    }

    #[test]
    fn test_invalid_image_data() {
        let invalid_data = vec![0, 1, 2, 3, 4, 5];
        let config = ImageProcessingConfig::default();

        let result = process_avatar(&invalid_data, "image/jpeg", &config);
        assert!(result.is_err());
    }
}
