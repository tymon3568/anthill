//! Product Variant HTTP handlers
//!
//! This module contains the Axum handlers for product variant management endpoints.

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};

use uuid::Uuid;
use validator::Validate;

// Import DTOs for requests/responses
use inventory_service_core::dto::product_variant::{
    BulkVariantIds, BulkVariantOperationResponse, VariantCreateRequest, VariantListQuery,
    VariantListResponse, VariantResponse, VariantUpdateRequest,
};

use shared_auth::extractors::AuthUser;
use shared_error::AppError;

use crate::state::AppState;

/// Create the product variant routes
pub fn create_variant_routes() -> Router {
    Router::new()
        .route("/", get(list_variants).post(create_variant))
        .route("/{variant_id}", get(get_variant).put(update_variant).delete(delete_variant))
        .route("/by-sku/{sku}", get(get_variant_by_sku))
        .route("/by-barcode/{barcode}", get(get_variant_by_barcode))
        .route("/bulk/activate", post(bulk_activate))
        .route("/bulk/deactivate", post(bulk_deactivate))
        .route("/bulk/delete", post(bulk_delete))
}

/// POST /api/v1/inventory/variants - Create a new product variant
///
/// Creates a new product variant with the provided details. The variant will be
/// automatically assigned a UUID v7 and validated according to business rules.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `request` - Variant creation data including parent product ID, SKU, attributes
///
/// # Returns
/// * `201` - Variant created successfully with full variant details
/// * `400` - Invalid request data or validation errors
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Parent product not found
/// * `409` - Variant SKU or attributes already exists
#[utoipa::path(
    post,
    path = "/api/v1/inventory/variants/",
    tag = "variants",
    operation_id = "create_variant",
    request_body = VariantCreateRequest,
    responses(
        (status = 201, description = "Variant created successfully", body = VariantResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Parent product not found"),
        (status = 409, description = "Variant SKU or attributes already exists")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_variant(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<VariantCreateRequest>,
) -> Result<(StatusCode, Json<VariantResponse>), AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Create variant
    let variant = state
        .variant_service
        .create_variant(auth_user.tenant_id, request)
        .await?;

    Ok((StatusCode::CREATED, Json(variant)))
}

/// GET /api/v1/inventory/variants - List variants with pagination and filtering
///
/// Retrieves a paginated list of product variants with optional filtering and sorting.
/// Supports filtering by parent product, active status, and search terms.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `parent_product_id` - Filter by parent product (optional)
/// * `is_active` - Filter by active status (optional)
/// * `search` - Search in SKU, barcode, and parent product name (optional)
/// * `page` - Page number (default: 1, min: 1)
/// * `page_size` - Items per page (default: 20, max: 100)
/// * `sort_by` - Sort field (default: sku)
/// * `sort_dir` - Sort direction (default: asc)
///
/// # Returns
/// * `200` - Paginated list of variants with metadata
/// * `400` - Invalid query parameters
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    get,
    path = "/api/v1/inventory/variants/",
    tag = "variants",
    operation_id = "list_variants",
    params(VariantListQuery),
    responses(
        (status = 200, description = "Paginated list of variants", body = VariantListResponse),
        (status = 400, description = "Invalid query parameters"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_variants(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(query): Query<VariantListQuery>,
) -> Result<Json<VariantListResponse>, AppError> {
    // Validate query parameters
    query
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let response = state
        .variant_service
        .list_variants(auth_user.tenant_id, query)
        .await?;
    Ok(Json(response))
}

/// GET /api/v1/inventory/variants/{variant_id} - Get variant by ID
///
/// Retrieves a single product variant by its unique identifier within the tenant.
/// Includes parent product information.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `variant_id` - UUID of the variant to retrieve
///
/// # Returns
/// * `200` - Variant details
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Variant not found
#[utoipa::path(
    get,
    path = "/api/v1/inventory/variants/{variant_id}",
    tag = "variants",
    operation_id = "get_variant",
    params(
        ("variant_id" = Uuid, Path, description = "UUID of the variant to retrieve")
    ),
    responses(
        (status = 200, description = "Variant details", body = VariantResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Variant not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_variant(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(variant_id): Path<Uuid>,
) -> Result<Json<VariantResponse>, AppError> {
    let variant = state
        .variant_service
        .get_variant(auth_user.tenant_id, variant_id)
        .await?;
    Ok(Json(variant))
}

/// GET /api/v1/inventory/variants/by-sku/{sku} - Get variant by SKU
///
/// Retrieves a single product variant by its SKU within the tenant.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `sku` - SKU of the variant to retrieve
///
/// # Returns
/// * `200` - Variant details
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Variant not found
#[utoipa::path(
    get,
    path = "/api/v1/inventory/variants/by-sku/{sku}",
    tag = "variants",
    operation_id = "get_variant_by_sku",
    params(
        ("sku" = String, Path, description = "SKU of the variant to retrieve")
    ),
    responses(
        (status = 200, description = "Variant details", body = VariantResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Variant not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_variant_by_sku(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(sku): Path<String>,
) -> Result<Json<VariantResponse>, AppError> {
    let variant = state
        .variant_service
        .get_variant_by_sku(auth_user.tenant_id, &sku)
        .await?;
    Ok(Json(variant))
}

/// GET /api/v1/inventory/variants/by-barcode/{barcode} - Get variant by barcode
///
/// Retrieves a single product variant by its barcode within the tenant.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `barcode` - Barcode of the variant to retrieve
///
/// # Returns
/// * `200` - Variant details
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Variant not found
#[utoipa::path(
    get,
    path = "/api/v1/inventory/variants/by-barcode/{barcode}",
    tag = "variants",
    operation_id = "get_variant_by_barcode",
    params(
        ("barcode" = String, Path, description = "Barcode of the variant to retrieve")
    ),
    responses(
        (status = 200, description = "Variant details", body = VariantResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Variant not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_variant_by_barcode(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(barcode): Path<String>,
) -> Result<Json<VariantResponse>, AppError> {
    let variant = state
        .variant_service
        .get_variant_by_barcode(auth_user.tenant_id, &barcode)
        .await?;
    Ok(Json(variant))
}

/// PUT /api/v1/inventory/variants/{variant_id} - Update variant
///
/// Updates an existing product variant with the provided fields.
/// Only specified fields will be updated.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `variant_id` - UUID of the variant to update
///
/// # Parameters
/// * `request` - Fields to update (all optional)
///
/// # Returns
/// * `200` - Updated variant details
/// * `400` - Invalid request data
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Variant not found
/// * `409` - Updated SKU or attributes conflicts
#[utoipa::path(
    put,
    path = "/api/v1/inventory/variants/{variant_id}",
    tag = "variants",
    operation_id = "update_variant",
    params(
        ("variant_id" = Uuid, Path, description = "UUID of the variant to update")
    ),
    request_body = VariantUpdateRequest,
    responses(
        (status = 200, description = "Updated variant details", body = VariantResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Variant not found"),
        (status = 409, description = "Updated SKU or attributes conflicts")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_variant(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(variant_id): Path<Uuid>,
    Json(request): Json<VariantUpdateRequest>,
) -> Result<Json<VariantResponse>, AppError> {
    // Validate request
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let variant = state
        .variant_service
        .update_variant(auth_user.tenant_id, variant_id, request)
        .await?;
    Ok(Json(variant))
}

/// DELETE /api/v1/inventory/variants/{variant_id} - Soft delete variant
///
/// Marks a product variant as deleted (soft delete).
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `variant_id` - UUID of the variant to delete
///
/// # Returns
/// * `204` - Variant deleted successfully
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Variant not found
#[utoipa::path(
    delete,
    path = "/api/v1/inventory/variants/{variant_id}",
    tag = "variants",
    operation_id = "delete_variant",
    params(
        ("variant_id" = Uuid, Path, description = "UUID of the variant to delete")
    ),
    responses(
        (status = 204, description = "Variant deleted successfully"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Variant not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_variant(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(variant_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .variant_service
        .delete_variant(auth_user.tenant_id, variant_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/inventory/variants/bulk/activate - Bulk activate variants
///
/// Activates multiple product variants at once.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `request` - List of variant IDs to activate
///
/// # Returns
/// * `200` - Bulk operation result
/// * `400` - Invalid request data
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    post,
    path = "/api/v1/inventory/variants/bulk/activate",
    tag = "variants",
    operation_id = "bulk_activate_variants",
    request_body = BulkVariantIds,
    responses(
        (status = 200, description = "Bulk operation result", body = BulkVariantOperationResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn bulk_activate(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<BulkVariantIds>,
) -> Result<Json<BulkVariantOperationResponse>, AppError> {
    let response = state
        .variant_service
        .bulk_activate(auth_user.tenant_id, request.variant_ids)
        .await?;
    Ok(Json(response))
}

/// POST /api/v1/inventory/variants/bulk/deactivate - Bulk deactivate variants
///
/// Deactivates multiple product variants at once.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `request` - List of variant IDs to deactivate
///
/// # Returns
/// * `200` - Bulk operation result
/// * `400` - Invalid request data
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    post,
    path = "/api/v1/inventory/variants/bulk/deactivate",
    tag = "variants",
    operation_id = "bulk_deactivate_variants",
    request_body = BulkVariantIds,
    responses(
        (status = 200, description = "Bulk operation result", body = BulkVariantOperationResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn bulk_deactivate(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<BulkVariantIds>,
) -> Result<Json<BulkVariantOperationResponse>, AppError> {
    let response = state
        .variant_service
        .bulk_deactivate(auth_user.tenant_id, request.variant_ids)
        .await?;
    Ok(Json(response))
}

/// POST /api/v1/inventory/variants/bulk/delete - Bulk delete variants
///
/// Soft deletes multiple product variants at once.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `request` - List of variant IDs to delete
///
/// # Returns
/// * `200` - Bulk operation result
/// * `400` - Invalid request data
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    post,
    path = "/api/v1/inventory/variants/bulk/delete",
    tag = "variants",
    operation_id = "bulk_delete_variants",
    request_body = BulkVariantIds,
    responses(
        (status = 200, description = "Bulk operation result", body = BulkVariantOperationResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn bulk_delete(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<BulkVariantIds>,
) -> Result<Json<BulkVariantOperationResponse>, AppError> {
    let response = state
        .variant_service
        .bulk_delete(auth_user.tenant_id, request.variant_ids)
        .await?;
    Ok(Json(response))
}
