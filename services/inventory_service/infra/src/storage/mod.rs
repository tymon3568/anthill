//! S3-compatible storage client for RustFS
//!
//! Provides file upload/download capabilities using AWS SDK S3.
//! Enterprise features: retry logic, magic bytes validation, metrics,
//! image processing (resize/compress).

pub mod image_processor;

pub use image_processor::{process_product_image, ImageProcessingConfig, ProcessedImage};

use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::ObjectCannedAcl;
use aws_sdk_s3::Client;
use metrics::{counter, histogram};
use shared_error::AppError;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

/// Maximum retry attempts for S3 operations
const MAX_RETRIES: usize = 3;
/// Base delay for exponential backoff (100ms)
const RETRY_BASE_DELAY_MS: u64 = 100;

/// Supported image magic bytes signatures
const MAGIC_BYTES: &[(&str, &[u8])] = &[
    ("image/jpeg", &[0xFF, 0xD8, 0xFF]),
    ("image/png", &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
    ("image/gif", &[0x47, 0x49, 0x46, 0x38]), // GIF87a or GIF89a
    ("image/webp", &[0x52, 0x49, 0x46, 0x46]), // RIFF header (need to check WEBP after)
];

/// Storage configuration
#[derive(Clone, Debug)]
pub struct StorageConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
    pub region: String,
    pub public_url: Option<String>,
}

impl StorageConfig {
    /// Create config from environment variables
    pub fn from_env() -> Result<Self, AppError> {
        let access_key =
            std::env::var("RUSTFS_ACCESS_KEY").or_else(|_| std::env::var("S3_ACCESS_KEY"));
        let secret_key =
            std::env::var("RUSTFS_SECRET_KEY").or_else(|_| std::env::var("S3_SECRET_KEY"));

        // Warn if using default credentials (potential security risk in production)
        let (access_key, secret_key) = match (access_key, secret_key) {
            (Ok(ak), Ok(sk)) => (ak, sk),
            _ => {
                tracing::warn!(
                    "RUSTFS_ACCESS_KEY/RUSTFS_SECRET_KEY not set, using default credentials. \
                     This is insecure for production environments!"
                );
                ("rustfsadmin".to_string(), "rustfsadmin".to_string())
            },
        };

        Ok(Self {
            endpoint: std::env::var("RUSTFS_ENDPOINT")
                .or_else(|_| std::env::var("S3_ENDPOINT"))
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
            access_key,
            secret_key,
            bucket_name: std::env::var("RUSTFS_BUCKET_NAME")
                .or_else(|_| std::env::var("S3_BUCKET_NAME"))
                .unwrap_or_else(|_| "anthill-files".to_string()),
            region: std::env::var("RUSTFS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            public_url: std::env::var("RUSTFS_PUBLIC_URL").ok(),
        })
    }
}

/// Validate image by checking magic bytes
///
/// Returns the detected MIME type if valid, error otherwise
pub fn validate_image_magic_bytes(data: &[u8]) -> Result<String, AppError> {
    if data.len() < 12 {
        return Err(AppError::ValidationError("File too small to be a valid image".to_string()));
    }

    for (mime_type, magic) in MAGIC_BYTES {
        if data.starts_with(magic) {
            // Special check for WebP: RIFF header must be followed by WEBP
            if *mime_type == "image/webp" {
                if data.len() >= 12 && &data[8..12] == b"WEBP" {
                    return Ok(mime_type.to_string());
                }
                continue;
            }
            return Ok(mime_type.to_string());
        }
    }

    Err(AppError::ValidationError(
        "Invalid image file: magic bytes do not match supported formats (JPEG, PNG, GIF, WebP)"
            .to_string(),
    ))
}

/// S3 Storage client wrapper with enterprise features
pub struct StorageClient {
    client: Client,
    config: StorageConfig,
}

impl StorageClient {
    /// Create a new storage client
    pub async fn new(config: StorageConfig) -> Result<Self, AppError> {
        let credentials = aws_sdk_s3::config::Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "rustfs",
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .behavior_version_latest()
            .endpoint_url(&config.endpoint)
            .credentials_provider(credentials)
            .region(aws_sdk_s3::config::Region::new(config.region.clone()))
            .force_path_style(true) // Required for S3-compatible storage like RustFS
            .build();

        let client = Client::from_conf(s3_config);

        Ok(Self { client, config })
    }

    /// Create storage client from environment
    pub async fn from_env() -> Result<Self, AppError> {
        let config = StorageConfig::from_env()?;
        Self::new(config).await
    }

    /// Get retry strategy with exponential backoff and jitter
    fn retry_strategy() -> impl Iterator<Item = Duration> {
        ExponentialBackoff::from_millis(RETRY_BASE_DELAY_MS)
            .factor(2)
            .max_delay(Duration::from_secs(5))
            .map(jitter)
            .take(MAX_RETRIES)
    }

    /// Upload a file to storage with retry logic
    ///
    /// # Arguments
    /// * `key` - The object key (path) in the bucket
    /// * `data` - The file data as bytes
    /// * `content_type` - The MIME content type
    ///
    /// # Returns
    /// The public URL of the uploaded file
    pub async fn upload(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, AppError> {
        let start = Instant::now();
        let data_len = data.len();

        // Clone data for potential retries
        let data_clone = data.clone();
        let bucket = self.config.bucket_name.clone();
        let key_owned = key.to_string();
        let content_type_owned = content_type.to_string();
        let client = self.client.clone();

        let result = Retry::spawn(Self::retry_strategy(), || {
            let data = data_clone.clone();
            let bucket = bucket.clone();
            let key = key_owned.clone();
            let content_type = content_type_owned.clone();
            let client = client.clone();

            async move {
                let body = ByteStream::from(data);

                client
                    .put_object()
                    .bucket(&bucket)
                    .key(&key)
                    .body(body)
                    .content_type(&content_type)
                    .acl(ObjectCannedAcl::PublicRead)
                    .send()
                    .await
                    .map_err(|e| {
                        tracing::warn!("S3 upload attempt failed, may retry: {:?}", e);
                        e
                    })
            }
        })
        .await;

        let duration = start.elapsed();

        match result {
            Ok(_) => {
                // Record success metrics
                counter!("storage_upload_total", "status" => "success").increment(1);
                histogram!("storage_upload_duration_seconds").record(duration.as_secs_f64());
                histogram!("storage_upload_bytes").record(data_len as f64);

                tracing::info!(
                    key = %key,
                    size_bytes = %data_len,
                    duration_ms = %duration.as_millis(),
                    "File uploaded successfully"
                );

                let url = self.get_public_url(key);
                Ok(url)
            },
            Err(e) => {
                // Record failure metrics
                counter!("storage_upload_total", "status" => "failure").increment(1);

                tracing::error!(
                    key = %key,
                    error = %e,
                    duration_ms = %duration.as_millis(),
                    "Failed to upload file after retries"
                );

                Err(AppError::InternalError(format!(
                    "Failed to upload file after {} retries: {}",
                    MAX_RETRIES, e
                )))
            },
        }
    }

    /// Upload a validated image (validates magic bytes before upload)
    ///
    /// # Arguments
    /// * `key` - The object key (path) in the bucket
    /// * `data` - The file data as bytes
    /// * `claimed_content_type` - The MIME content type claimed by the client
    ///
    /// # Returns
    /// The public URL and detected content type
    pub async fn upload_validated_image(
        &self,
        key: &str,
        data: Vec<u8>,
        claimed_content_type: &str,
    ) -> Result<(String, String), AppError> {
        // Validate magic bytes
        let detected_type = validate_image_magic_bytes(&data)?;

        // Log if claimed type doesn't match detected type (potential attack)
        if claimed_content_type != detected_type {
            tracing::warn!(
                claimed = %claimed_content_type,
                detected = %detected_type,
                key = %key,
                "Content-Type mismatch: using detected type"
            );
            counter!("storage_content_type_mismatch").increment(1);
        }

        // Use the detected type, not the claimed one
        let url = self.upload(key, data, &detected_type).await?;
        Ok((url, detected_type))
    }

    /// Get the public URL for an object
    pub fn get_public_url(&self, key: &str) -> String {
        if let Some(public_url) = &self.config.public_url {
            format!("{}/{}/{}", public_url, self.config.bucket_name, key)
        } else {
            format!("{}/{}/{}", self.config.endpoint, self.config.bucket_name, key)
        }
    }

    /// Delete a file from storage with retry logic
    pub async fn delete(&self, key: &str) -> Result<(), AppError> {
        let start = Instant::now();
        let bucket = self.config.bucket_name.clone();
        let key_owned = key.to_string();
        let client = self.client.clone();

        let result = Retry::spawn(Self::retry_strategy(), || {
            let bucket = bucket.clone();
            let key = key_owned.clone();
            let client = client.clone();

            async move {
                client
                    .delete_object()
                    .bucket(&bucket)
                    .key(&key)
                    .send()
                    .await
                    .map_err(|e| {
                        tracing::warn!("S3 delete attempt failed, may retry: {:?}", e);
                        e
                    })
            }
        })
        .await;

        let duration = start.elapsed();

        match result {
            Ok(_) => {
                counter!("storage_delete_total", "status" => "success").increment(1);
                histogram!("storage_delete_duration_seconds").record(duration.as_secs_f64());

                tracing::info!(
                    key = %key,
                    duration_ms = %duration.as_millis(),
                    "File deleted successfully"
                );
                Ok(())
            },
            Err(e) => {
                counter!("storage_delete_total", "status" => "failure").increment(1);

                tracing::error!(
                    key = %key,
                    error = %e,
                    duration_ms = %duration.as_millis(),
                    "Failed to delete file after retries"
                );

                Err(AppError::InternalError(format!(
                    "Failed to delete file after {} retries: {}",
                    MAX_RETRIES, e
                )))
            },
        }
    }

    /// Delete a file silently (log errors but don't fail)
    /// Useful for cleanup operations where failure shouldn't block the main operation
    pub async fn delete_silent(&self, key: &str) {
        if let Err(e) = self.delete(key).await {
            tracing::warn!(
                key = %key,
                error = %e,
                "Failed to delete file (silent mode)"
            );
        }
    }

    /// Extract the object key from a full URL
    /// Returns None if the URL doesn't match our storage endpoints
    pub fn extract_key_from_url(&self, url: &str) -> Option<String> {
        // Try public URL first
        if let Some(public_url) = &self.config.public_url {
            let prefix = format!("{}/{}", public_url, self.config.bucket_name);
            if let Some(key) = url.strip_prefix(&prefix) {
                return Some(key.trim_start_matches('/').to_string());
            }
        }

        // Try endpoint URL
        let prefix = format!("{}/{}", self.config.endpoint, self.config.bucket_name);
        if let Some(key) = url.strip_prefix(&prefix) {
            return Some(key.trim_start_matches('/').to_string());
        }

        // Try just bucket path (relative)
        let bucket_path = format!("{}/", self.config.bucket_name);
        if let Some(key) = url.strip_prefix(&bucket_path) {
            return Some(key.to_string());
        }

        None
    }
}

/// Shared storage client type for dependency injection
pub type SharedStorageClient = Arc<StorageClient>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_jpeg_magic_bytes() {
        let jpeg_data = vec![
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
        ];
        let result = validate_image_magic_bytes(&jpeg_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "image/jpeg");
    }

    #[test]
    fn test_validate_png_magic_bytes() {
        let png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
        ];
        let result = validate_image_magic_bytes(&png_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "image/png");
    }

    #[test]
    fn test_validate_gif_magic_bytes() {
        let gif_data = vec![
            0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00,
        ];
        let result = validate_image_magic_bytes(&gif_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "image/gif");
    }

    #[test]
    fn test_validate_webp_magic_bytes() {
        let webp_data = vec![
            0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50,
        ];
        let result = validate_image_magic_bytes(&webp_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "image/webp");
    }

    #[test]
    fn test_validate_invalid_magic_bytes() {
        let invalid_data = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let result = validate_image_magic_bytes(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_too_small_file() {
        let small_data = vec![0xFF, 0xD8];
        let result = validate_image_magic_bytes(&small_data);
        assert!(result.is_err());
    }
}
