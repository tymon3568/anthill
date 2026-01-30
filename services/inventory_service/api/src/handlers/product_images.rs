//! Product Image HTTP handlers
//!
//! This module contains the Axum handlers for product image management endpoints.

use axum::{
    extract::{Extension, Multipart, Path},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use uuid::Uuid;

use inventory_service_core::dto::product_image::{
    DeleteImageResponse, ProductImageResponse, ProductImagesListResponse, ReorderImagesRequest,
    UpdateProductImageRequest, UploadImageResponse,
};

use shared_auth::extractors::AuthUser;
use shared_error::AppError;

use crate::state::AppState;

/// Create the product image routes
pub fn create_product_image_routes() -> Router {
    Router::new()
        .route("/", get(list_images).post(upload_image))
        .route("/{image_id}", get(get_image).put(update_image).delete(delete_image))
        .route("/reorder", post(reorder_images))
        .route("/{image_id}/set-primary", post(set_primary_image))
}

/// GET /api/v1/inventory/products/{product_id}/images - List all images for a product
///
/// Returns all images associated with a product, ordered by position.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `product_id` - UUID of the product
///
/// # Returns
/// * `200` - List of product images
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    get,
    path = "/api/v1/inventory/products/{product_id}/images/",
    tag = "product_images",
    operation_id = "list_product_images",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    responses(
        (status = 200, description = "List of product images", body = ProductImagesListResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_images(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<ProductImagesListResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;

    let response = state
        .product_image_service
        .list_images(tenant_id, product_id)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/products/{product_id}/images - Upload a new product image
///
/// Uploads a new image for the specified product. The image will be processed
/// (resized if too large) and stored in RustFS.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `product_id` - UUID of the product
/// * `file` - Image file (multipart/form-data)
///
/// # Constraints
/// * Maximum 10 images per product
/// * Maximum 5MB file size
/// * Only JPEG, PNG, WebP, GIF allowed
///
/// # Returns
/// * `201` - Image uploaded successfully
/// * `400` - Invalid file or validation errors
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `422` - Business rule violation (max images exceeded)
#[utoipa::path(
    post,
    path = "/api/v1/inventory/products/{product_id}/images/",
    tag = "product_images",
    operation_id = "upload_product_image",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    request_body(content_type = "multipart/form-data", content = inline(ImageUploadForm)),
    responses(
        (status = 201, description = "Image uploaded successfully", body = UploadImageResponse),
        (status = 400, description = "Invalid file or validation errors"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 422, description = "Maximum images exceeded")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn upload_image(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadImageResponse>), AppError> {
    let tenant_id = auth_user.tenant_id;

    // Extract file from multipart
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::ValidationError(format!("Failed to parse multipart: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" || name == "image" {
            file_name = field.file_name().map(|s| s.to_string());
            content_type = field.content_type().map(|s| s.to_string());

            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::ValidationError(format!("Failed to read file: {}", e)))?;

            file_data = Some(data.to_vec());
            break;
        }
    }

    let file_data = file_data.ok_or_else(|| {
        AppError::ValidationError("No file provided. Use 'file' or 'image' field name.".to_string())
    })?;

    let file_name = file_name.unwrap_or_else(|| "image.jpg".to_string());
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());

    let response = state
        .product_image_service
        .upload_image(tenant_id, product_id, file_data, &file_name, &content_type)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// OpenAPI schema for image upload form
#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
struct ImageUploadForm {
    /// The image file to upload
    #[schema(value_type = String, format = Binary)]
    file: Vec<u8>,
}

/// GET /api/v1/inventory/products/{product_id}/images/{image_id} - Get a single image
///
/// Returns details for a specific product image.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `product_id` - UUID of the product
/// * `image_id` - UUID of the image
///
/// # Returns
/// * `200` - Image details
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Image not found
#[utoipa::path(
    get,
    path = "/api/v1/inventory/products/{product_id}/images/{image_id}",
    tag = "product_images",
    operation_id = "get_product_image",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID"),
        ("image_id" = Uuid, Path, description = "Image UUID")
    ),
    responses(
        (status = 200, description = "Image details", body = ProductImageResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Image not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_image(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path((product_id, image_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ProductImageResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;

    let image = state
        .product_image_service
        .get_image(tenant_id, image_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Image not found".to_string()))?;

    // Verify image belongs to product
    if image.product_id != product_id {
        return Err(AppError::NotFound("Image not found".to_string()));
    }

    Ok(Json(ProductImageResponse::from(image)))
}

/// PUT /api/v1/inventory/products/{product_id}/images/{image_id} - Update image metadata
///
/// Updates the alt text for a product image.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `product_id` - UUID of the product
/// * `image_id` - UUID of the image
/// * `request` - Update data
///
/// # Returns
/// * `200` - Image updated successfully
/// * `400` - Invalid request data
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Image not found
#[utoipa::path(
    put,
    path = "/api/v1/inventory/products/{product_id}/images/{image_id}",
    tag = "product_images",
    operation_id = "update_product_image",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID"),
        ("image_id" = Uuid, Path, description = "Image UUID")
    ),
    request_body = UpdateProductImageRequest,
    responses(
        (status = 200, description = "Image updated successfully", body = ProductImageResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Image not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_image(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path((product_id, image_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateProductImageRequest>,
) -> Result<Json<ProductImageResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;

    // Verify image belongs to product first
    let existing = state
        .product_image_service
        .get_image(tenant_id, image_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Image not found".to_string()))?;

    if existing.product_id != product_id {
        return Err(AppError::NotFound("Image not found".to_string()));
    }

    let updated = state
        .product_image_service
        .update_image(tenant_id, image_id, request)
        .await?;

    Ok(Json(ProductImageResponse::from(updated)))
}

/// DELETE /api/v1/inventory/products/{product_id}/images/{image_id} - Delete an image
///
/// Deletes a product image from storage and database.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `product_id` - UUID of the product
/// * `image_id` - UUID of the image
///
/// # Returns
/// * `200` - Image deleted successfully
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Image not found
#[utoipa::path(
    delete,
    path = "/api/v1/inventory/products/{product_id}/images/{image_id}",
    tag = "product_images",
    operation_id = "delete_product_image",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID"),
        ("image_id" = Uuid, Path, description = "Image UUID")
    ),
    responses(
        (status = 200, description = "Image deleted successfully", body = DeleteImageResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Image not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_image(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path((product_id, image_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DeleteImageResponse>, AppError> {
    let tenant_id = auth_user.tenant_id;

    // Verify image belongs to product first
    let existing = state
        .product_image_service
        .get_image(tenant_id, image_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Image not found".to_string()))?;

    if existing.product_id != product_id {
        return Err(AppError::NotFound("Image not found".to_string()));
    }

    let response = state
        .product_image_service
        .delete_image(tenant_id, image_id)
        .await?;

    Ok(Json(response))
}

/// POST /api/v1/inventory/products/{product_id}/images/reorder - Reorder images
///
/// Reorders all images for a product. The position is determined by the order
/// of image IDs in the request.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `product_id` - UUID of the product
/// * `request` - Ordered list of image IDs
///
/// # Returns
/// * `204` - Images reordered successfully
/// * `400` - Invalid request data
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
#[utoipa::path(
    post,
    path = "/api/v1/inventory/products/{product_id}/images/reorder",
    tag = "product_images",
    operation_id = "reorder_product_images",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    request_body = ReorderImagesRequest,
    responses(
        (status = 204, description = "Images reordered successfully"),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn reorder_images(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(product_id): Path<Uuid>,
    Json(request): Json<ReorderImagesRequest>,
) -> Result<StatusCode, AppError> {
    let tenant_id = auth_user.tenant_id;

    state
        .product_image_service
        .reorder_images(tenant_id, product_id, request)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/inventory/products/{product_id}/images/{image_id}/set-primary - Set as primary
///
/// Sets the specified image as the primary image for the product.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `product_id` - UUID of the product
/// * `image_id` - UUID of the image to set as primary
///
/// # Returns
/// * `204` - Primary image set successfully
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Image not found
#[utoipa::path(
    post,
    path = "/api/v1/inventory/products/{product_id}/images/{image_id}/set-primary",
    tag = "product_images",
    operation_id = "set_primary_product_image",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID"),
        ("image_id" = Uuid, Path, description = "Image UUID to set as primary")
    ),
    responses(
        (status = 204, description = "Primary image set successfully"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Image not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn set_primary_image(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path((product_id, image_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let tenant_id = auth_user.tenant_id;

    state
        .product_image_service
        .set_primary_image(tenant_id, product_id, image_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
