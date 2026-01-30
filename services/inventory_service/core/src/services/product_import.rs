//! Product Import/Export Service Trait
//!
//! Defines the interface for bulk import and export of products via CSV.

use async_trait::async_trait;
use shared_error::AppError;
use uuid::Uuid;

use crate::dto::product_import::{ExportProductsQuery, ImportResult, ImportValidationResult};

/// Service for bulk import and export of products
#[async_trait]
pub trait ProductImportService: Send + Sync {
    /// Validate a CSV file before importing
    ///
    /// Returns validation results including any errors found in the file.
    async fn validate_csv(
        &self,
        tenant_id: Uuid,
        data: &[u8],
    ) -> Result<ImportValidationResult, AppError>;

    /// Import products from a CSV file
    ///
    /// If `upsert` is true, existing products (by SKU) will be updated.
    /// If `upsert` is false, existing SKUs will cause an error.
    async fn import_csv(
        &self,
        tenant_id: Uuid,
        data: &[u8],
        upsert: bool,
    ) -> Result<ImportResult, AppError>;

    /// Get a CSV template with headers and example row
    fn get_template(&self) -> Vec<u8>;

    /// Export products to CSV format
    ///
    /// Returns the CSV file contents as bytes.
    async fn export_csv(
        &self,
        tenant_id: Uuid,
        query: ExportProductsQuery,
    ) -> Result<Vec<u8>, AppError>;
}
