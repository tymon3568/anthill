//! Product Import/Export HTTP handlers
//!
//! This module contains the Axum handlers for product bulk import and export endpoints.

use axum::{
    body::Bytes,
    extract::{Extension, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use inventory_service_core::dto::product_import::{
    ExportProductsQuery, ImportResult, ImportValidationResult,
};

use shared_auth::extractors::AuthUser;
use shared_error::AppError;

use crate::state::AppState;

/// Create the product import/export routes
pub fn create_product_import_routes() -> Router {
    Router::new()
        .route("/template", get(get_import_template))
        .route("/validate", post(validate_import))
        .route("/import", post(import_products))
        .route("/export", get(export_products))
}

/// GET /api/v1/inventory/products/import/template - Download CSV template
///
/// Downloads a CSV template file with headers and an example row.
/// Use this template as a starting point for bulk imports.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Returns
/// * `200` - CSV file download
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    get,
    path = "/api/v1/inventory/products/import/template",
    tag = "products-import",
    operation_id = "get_import_template",
    responses(
        (status = 200, description = "CSV template file", content_type = "text/csv"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_import_template(
    _auth_user: AuthUser,
    Extension(state): Extension<AppState>,
) -> Response {
    let template = state.product_import_service.get_template();

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"products_import_template.csv\"",
            ),
        ],
        template,
    )
        .into_response()
}

/// POST /api/v1/inventory/products/import/validate - Validate CSV before import
///
/// Validates a CSV file before importing. Returns validation results including
/// any errors found in the file and a preview of the first few rows.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Request Body
/// Raw CSV file content (text/csv or multipart/form-data)
///
/// # Returns
/// * `200` - Validation results
/// * `400` - Invalid CSV format
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    post,
    path = "/api/v1/inventory/products/import/validate",
    tag = "products-import",
    operation_id = "validate_import",
    request_body(content = String, content_type = "text/csv"),
    responses(
        (status = 200, description = "Validation results", body = ImportValidationResult),
        (status = 400, description = "Invalid CSV format"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn validate_import(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    body: Bytes,
) -> Result<Json<ImportValidationResult>, AppError> {
    let result = state
        .product_import_service
        .validate_csv(auth_user.tenant_id, &body)
        .await?;

    Ok(Json(result))
}

/// Query parameters for import endpoint
#[derive(Debug, serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ImportQuery {
    /// If true, existing products (by SKU) will be updated
    #[serde(default)]
    pub upsert: bool,
}

/// POST /api/v1/inventory/products/import/import - Import products from CSV
///
/// Imports products from a CSV file. If `upsert` is true, existing products
/// (matched by SKU) will be updated. If false, existing SKUs will cause errors.
///
/// Maximum of 1000 rows per import.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `upsert` - If true, update existing products by SKU (default: false)
///
/// # Request Body
/// Raw CSV file content (text/csv or multipart/form-data)
///
/// # Returns
/// * `200` - Import results with created/updated/failed counts
/// * `400` - Invalid CSV format or validation errors
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    post,
    path = "/api/v1/inventory/products/import/import",
    tag = "products-import",
    operation_id = "import_products",
    params(ImportQuery),
    request_body(content = String, content_type = "text/csv"),
    responses(
        (status = 200, description = "Import results", body = ImportResult),
        (status = 400, description = "Invalid CSV format or validation errors"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn import_products(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(query): Query<ImportQuery>,
    body: Bytes,
) -> Result<Json<ImportResult>, AppError> {
    let result = state
        .product_import_service
        .import_csv(auth_user.tenant_id, &body, query.upsert)
        .await?;

    Ok(Json(result))
}

/// GET /api/v1/inventory/products/import/export - Export products to CSV
///
/// Exports products to a CSV file. Supports optional filtering by category,
/// product type, active status, and search term.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `categoryId` - Filter by category ID (optional)
/// * `productType` - Filter by product type (optional)
/// * `isActive` - Filter by active status (optional)
/// * `search` - Search term for SKU or name (optional)
///
/// # Returns
/// * `200` - CSV file download
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    get,
    path = "/api/v1/inventory/products/import/export",
    tag = "products-import",
    operation_id = "export_products",
    params(ExportProductsQuery),
    responses(
        (status = 200, description = "CSV export file", content_type = "text/csv"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn export_products(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(query): Query<ExportProductsQuery>,
) -> Result<Response, AppError> {
    let csv_data = state
        .product_import_service
        .export_csv(auth_user.tenant_id, query)
        .await?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"products_export.csv\""),
        ],
        csv_data,
    )
        .into_response())
}
