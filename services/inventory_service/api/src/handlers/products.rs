//! Product HTTP handlers
//!
//! This module contains the Axum handlers for product management endpoints.

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};

use uuid::Uuid;
use validator::Validate;

// Import DTOs for requests/responses
use inventory_service_core::dto::product::{
    ProductCreateRequest, ProductListQuery, ProductListResponse, ProductResponse,
    ProductUpdateRequest,
};

use shared_auth::extractors::AuthUser;
use shared_error::AppError;

use crate::state::AppState;

/// Create the product routes
pub fn create_product_routes() -> Router {
    Router::new()
        .route("/", get(list_products).post(create_product))
        .route("/{product_id}", get(get_product).put(update_product).delete(delete_product))
}

/// POST /api/v1/inventory/products - Create a new product
///
/// Creates a new product with the provided details. The product will be
/// automatically assigned a UUID v7 and validated according to business rules.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `request` - Product creation data including SKU, name, type, pricing
///
/// # Returns
/// * `201` - Product created successfully with full product details
/// * `400` - Invalid request data or validation errors
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `409` - Product SKU already exists
///
/// # Example
/// ```json
/// POST /api/v1/inventory/products
/// {
///   "sku": "WIDGET-001",
///   "name": "Blue Widget",
///   "description": "A high-quality blue widget",
///   "productType": "goods",
///   "trackInventory": true,
///   "trackingMethod": "none",
///   "salePrice": 1999,
///   "costPrice": 999,
///   "currencyCode": "USD",
///   "isActive": true,
///   "isSellable": true,
///   "isPurchaseable": true
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/inventory/products/",
    tag = "products",
    operation_id = "create_product",
    request_body = ProductCreateRequest,
    responses(
        (status = 201, description = "Product created successfully", body = ProductResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 409, description = "Product SKU already exists")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_product(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<ProductCreateRequest>,
) -> Result<(StatusCode, Json<ProductResponse>), AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Create product
    let product = state
        .product_service
        .create_product(auth_user.tenant_id, request)
        .await?;

    let response = ProductResponse::from(product);
    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/v1/inventory/products - List products with pagination and filtering
///
/// Retrieves a paginated list of products with optional filtering and sorting.
/// Supports filtering by product type, active status, sellable status, and search terms.
/// Results are sorted by name by default.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `product_type` - Filter by product type (optional)
/// * `is_active` - Filter by active status (optional)
/// * `is_sellable` - Filter by sellable status (optional)
/// * `is_purchaseable` - Filter by purchaseable status (optional)
/// * `search` - Search in name, SKU, and description (optional)
/// * `page` - Page number (default: 1, min: 1)
/// * `page_size` - Items per page (default: 20, max: 100)
/// * `sort_by` - Sort field (default: name)
/// * `sort_dir` - Sort direction (default: asc)
///
/// # Returns
/// * `200` - Paginated list of products with metadata
/// * `400` - Invalid query parameters
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Example
/// ```
/// GET /api/v1/inventory/products?page=1&page_size=10&is_active=true&product_type=goods
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/inventory/products/",
    tag = "products",
    operation_id = "list_products",
    params(ProductListQuery),
    responses(
        (status = 200, description = "Paginated list of products", body = ProductListResponse),
        (status = 400, description = "Invalid query parameters"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_products(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(query): Query<ProductListQuery>,
) -> Result<Json<ProductListResponse>, AppError> {
    // Validate query parameters
    query
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let response = state
        .product_service
        .list_products(auth_user.tenant_id, query)
        .await?;
    Ok(Json(response))
}

/// GET /api/v1/inventory/products/{product_id} - Get product by ID
///
/// Retrieves a single product by its unique identifier within the tenant.
/// Includes all product details and computed fields.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `product_id` - UUID of the product to retrieve
///
/// # Returns
/// * `200` - Product details
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Product not found or belongs to different tenant
///
/// # Example
/// ```
/// GET /api/v1/inventory/products/123e4567-e89b-12d3-a456-426614174000
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/inventory/products/{product_id}",
    tag = "products",
    operation_id = "get_product",
    params(
        ("product_id" = Uuid, Path, description = "UUID of the product to retrieve")
    ),
    responses(
        (status = 200, description = "Product details", body = ProductResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Product not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_product(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<ProductResponse>, AppError> {
    let product = state
        .product_service
        .get_product(auth_user.tenant_id, product_id)
        .await?;
    Ok(Json(ProductResponse::from(product)))
}

/// PUT /api/v1/inventory/products/{product_id} - Update product
///
/// Updates an existing product with the provided fields. Only specified fields
/// will be updated. SKU uniqueness is enforced within the tenant.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `product_id` - UUID of the product to update
///
/// # Parameters
/// * `request` - Fields to update (all optional)
///
/// # Returns
/// * `200` - Updated product details
/// * `400` - Invalid request data
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Product not found
/// * `409` - Updated SKU conflicts with existing product
///
/// # Example
/// ```json
/// PUT /api/v1/inventory/products/123e4567-e89b-12d3-a456-426614174000
/// {
///   "name": "Updated Widget Name",
///   "salePrice": 2499,
///   "isActive": false
/// }
/// ```
#[utoipa::path(
    put,
    path = "/api/v1/inventory/products/{product_id}",
    tag = "products",
    operation_id = "update_product",
    params(
        ("product_id" = Uuid, Path, description = "UUID of the product to update")
    ),
    request_body = ProductUpdateRequest,
    responses(
        (status = 200, description = "Updated product details", body = ProductResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Product not found"),
        (status = 409, description = "Updated SKU conflicts with existing product")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_product(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
    Json(request): Json<ProductUpdateRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let product = state
        .product_service
        .update_product(auth_user.tenant_id, product_id, request)
        .await?;
    Ok(Json(ProductResponse::from(product)))
}

/// DELETE /api/v1/inventory/products/{product_id} - Soft delete product
///
/// Marks a product as deleted (soft delete). The product will no longer
/// appear in normal queries but can be restored if needed. Associated
/// inventory records are not affected.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `product_id` - UUID of the product to delete
///
/// # Returns
/// * `204` - Product deleted successfully
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Product not found
/// * `409` - Product cannot be deleted (has active inventory transactions)
///
/// # Example
/// ```
/// DELETE /api/v1/inventory/products/123e4567-e89b-12d3-a456-426614174000
/// ```
#[utoipa::path(
    delete,
    path = "/api/v1/inventory/products/{product_id}",
    tag = "products",
    operation_id = "delete_product",
    params(
        ("product_id" = Uuid, Path, description = "UUID of the product to delete")
    ),
    responses(
        (status = 204, description = "Product deleted successfully"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Product not found"),
        (status = 409, description = "Product cannot be deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_product(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .product_service
        .delete_product(auth_user.tenant_id, product_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
