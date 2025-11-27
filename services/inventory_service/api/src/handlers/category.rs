//! Category HTTP handlers
//!
//! This module contains the Axum handlers for category management endpoints.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use inventory_service_core::dto::category::{
    BulkOperationResponse, CategoryCreateRequest, CategoryListQuery, CategoryListResponse,
    CategoryResponse, CategoryStatsResponse, CategoryTreeResponse, CategoryUpdateRequest,
    MoveToCategoryRequest,
};
use inventory_service_core::repositories::warehouse::WarehouseRepository;
use inventory_service_core::services::category::CategoryService;
// use inventory_service_core::services::delivery::DeliveryService;
use inventory_service_core::services::product::ProductService;
use inventory_service_core::services::receipt::ReceiptService;
use inventory_service_core::services::reconciliation::StockReconciliationService;
use inventory_service_core::services::stock_take::StockTakeService;
use inventory_service_core::services::transfer::TransferService;
use inventory_service_core::services::valuation::ValuationService;

use shared_auth::enforcer::SharedEnforcer;
use shared_auth::extractors::{AuthUser, JwtSecretProvider, KanidmClientProvider, RequireAdmin};
use shared_error::AppError;
use shared_kanidm_client::KanidmClient;

use crate::state::AppState;

/// Create the category routes with state
pub fn create_category_routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_categories).post(create_category))
        .route("/tree", get(get_category_tree))
        .route("/search", get(search_categories))
        .route("/top", get(get_top_categories))
        .route("/bulk/activate", post(bulk_activate_categories))
        .route("/bulk/deactivate", post(bulk_deactivate_categories))
        .route("/bulk/delete", post(bulk_delete_categories))
        .route(
            "/{category_id}",
            get(get_category)
                .put(update_category)
                .delete(delete_category),
        )
        .route("/{category_id}/children", get(get_children))
        .route("/{category_id}/breadcrumbs", get(get_breadcrumbs))
        .route("/{category_id}/stats", get(get_category_stats))
        .route("/{category_id}/can-delete", get(can_delete_category))
        .route("/products/move", post(move_products_to_category))
        .with_state(state)
}

/// POST /api/v1/inventory/categories - Create a new category
///
/// Creates a new product category with the provided details. The category will be
/// automatically assigned a path and level based on its parent category (if any).
/// Product counts are initialized to zero and will be updated by database triggers.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `request` - Category creation data including name, parent, display settings
///
/// # Returns
/// * `200` - Category created successfully with full category details
/// * `400` - Invalid request data or validation errors
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `409` - Category code or slug already exists
///
/// # Example
/// ```json
/// POST /api/v1/inventory/categories
/// {
///   "name": "Electronics",
///   "description": "Electronic devices and accessories",
///   "code": "ELEC",
///   "display_order": 1,
///   "is_active": true
/// }
/// ```
pub async fn create_category(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CategoryCreateRequest>,
) -> Result<Json<CategoryResponse>, AppError> {
    let category = state
        .category_service
        .create_category(auth_user.tenant_id, request)
        .await?;
    Ok(Json(CategoryResponse::from(category)))
}

/// GET /api/v1/inventory/categories - List categories with pagination and filtering
///
/// Retrieves a paginated list of categories with optional filtering and sorting.
/// Supports hierarchical filtering by parent category, level, and search terms.
/// Results are sorted by display order by default.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `parent_id` - Filter by parent category (optional)
/// * `level` - Filter by category level (0=root, optional)
/// * `is_active` - Filter by active status (optional)
/// * `is_visible` - Filter by visibility (optional)
/// * `search` - Search in name and description (optional)
/// * `page` - Page number (default: 1, min: 1)
/// * `page_size` - Items per page (default: 20, max: 100)
/// * `sort_by` - Sort field (default: display_order)
/// * `sort_dir` - Sort direction (default: asc)
///
/// # Returns
/// * `200` - Paginated list of categories with metadata
/// * `400` - Invalid query parameters
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Example
/// ```
/// GET /api/v1/inventory/categories?page=1&page_size=10&is_active=true
/// ```
pub async fn list_categories(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<CategoryListQuery>,
) -> Result<Json<CategoryListResponse>, AppError> {
    let response = state
        .category_service
        .list_categories(auth_user.tenant_id, query)
        .await?;
    Ok(Json(response))
}

/// GET /api/v1/inventory/categories/tree - Get hierarchical category tree
///
/// Returns a complete hierarchical tree structure of categories starting from
/// root categories or a specified parent. The tree includes all children
/// recursively up to the specified maximum depth.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `parent_id` - Root of the tree (optional, defaults to all roots)
/// * `max_depth` - Maximum depth to traverse (optional, unlimited if not set)
///
/// # Returns
/// * `200` - Hierarchical tree structure with category details and children
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Parent category not found
///
/// # Example
/// ```
/// GET /api/v1/inventory/categories/tree?parent_id=123e4567-e89b-12d3-a456-426614174000&max_depth=3
/// ```
pub async fn get_category_tree(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<CategoryTreeQuery>,
) -> Result<Json<Vec<CategoryTreeResponse>>, AppError> {
    let tree = state
        .category_service
        .get_category_tree(auth_user.tenant_id, params.parent_id, params.max_depth)
        .await?;
    Ok(Json(tree))
}

/// GET /api/v1/inventory/categories/search - Full-text search categories
///
/// Performs case-insensitive search across category names and descriptions.
/// Returns results ordered by name, limited to the specified number of results.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `q` - Search query string (required)
/// * `limit` - Maximum number of results (default: 50, max: 100)
///
/// # Returns
/// * `200` - Array of matching categories
/// * `400` - Missing or invalid search query
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Example
/// ```
/// GET /api/v1/inventory/categories/search?q=electronics&limit=10
/// ```
pub async fn search_categories(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<CategoryResponse>>, AppError> {
    let categories = state
        .category_service
        .search_categories(auth_user.tenant_id, &params.q, params.limit.unwrap_or(50))
        .await?;
    let responses = categories.into_iter().map(CategoryResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/v1/inventory/categories/top - Get top categories by product count
///
/// Returns root categories ordered by total product count (descending),
/// then by display order. Useful for displaying popular categories.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `limit` - Maximum number of categories to return (default: 10, max: 50)
///
/// # Returns
/// * `200` - Array of top categories with product counts
/// * `400` - Invalid limit parameter
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Example
/// ```
/// GET /api/v1/inventory/categories/top?limit=5
/// ```
pub async fn get_top_categories(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<TopCategoriesQuery>,
) -> Result<Json<Vec<CategoryResponse>>, AppError> {
    let categories = state
        .category_service
        .get_top_categories(auth_user.tenant_id, params.limit.unwrap_or(10))
        .await?;
    let responses = categories.into_iter().map(CategoryResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/v1/inventory/categories/{category_id} - Get category by ID
///
/// Retrieves a single category by its unique identifier within the tenant.
/// Includes all category details and computed fields like path and level.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `category_id` - UUID of the category to retrieve
///
/// # Returns
/// * `200` - Category details
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Category not found or belongs to different tenant
///
/// # Example
/// ```
/// GET /api/v1/inventory/categories/123e4567-e89b-12d3-a456-426614174000
/// ```
pub async fn get_category(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<Json<CategoryResponse>, AppError> {
    let category = state
        .category_service
        .get_category(auth_user.tenant_id, category_id)
        .await?;
    Ok(Json(CategoryResponse::from(category)))
}

/// PUT /api/v1/inventory/categories/{category_id} - Update category
///
/// Updates an existing category with the provided fields. Only specified fields
/// will be updated. If the parent category changes, the path and level will
/// be automatically recalculated.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `category_id` - UUID of the category to update
///
/// # Parameters
/// * `request` - Fields to update (all optional)
///
/// # Returns
/// * `200` - Updated category details
/// * `400` - Invalid request data
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Category not found
/// * `409` - Updated code or slug conflicts with existing category
///
/// # Example
/// ```json
/// PUT /api/v1/inventory/categories/123e4567-e89b-12d3-a456-426614174000
/// {
///   "name": "Updated Electronics",
///   "is_active": false
/// }
/// ```
pub async fn update_category(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
    Json(request): Json<CategoryUpdateRequest>,
) -> Result<Json<CategoryResponse>, AppError> {
    let category = state
        .category_service
        .update_category(auth_user.tenant_id, category_id, request)
        .await?;
    Ok(Json(CategoryResponse::from(category)))
}

/// DELETE /api/v1/inventory/categories/{category_id} - Soft delete category
///
/// Marks a category as deleted (soft delete). The category will no longer
/// appear in normal queries but can be restored if needed. Child categories
/// and associated products are not affected.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `category_id` - UUID of the category to delete
///
/// # Returns
/// * `204` - Category deleted successfully
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Category not found
/// * `409` - Category cannot be deleted (has children or products)
///
/// # Example
/// ```
/// DELETE /api/v1/inventory/categories/123e4567-e89b-12d3-a456-426614174000
/// ```
pub async fn delete_category(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .category_service
        .delete_category(auth_user.tenant_id, category_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/inventory/categories/{category_id}/children - Get direct children
///
/// Returns all direct child categories of the specified parent category.
/// Children are ordered by display order, then by name.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `category_id` - UUID of the parent category
///
/// # Returns
/// * `200` - Array of child categories
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Parent category not found
///
/// # Example
/// ```
/// GET /api/v1/inventory/categories/123e4567-e89b-12d3-a456-426614174000/children
/// ```
pub async fn get_children(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<Json<Vec<CategoryResponse>>, AppError> {
    let children = state
        .category_service
        .get_children(auth_user.tenant_id, category_id)
        .await?;
    let responses = children.into_iter().map(CategoryResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/v1/inventory/categories/{category_id}/breadcrumbs - Get breadcrumb path
///
/// Returns the complete path from root category to the specified category,
/// useful for navigation and displaying hierarchical context.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `category_id` - UUID of the category to get breadcrumbs for
///
/// # Returns
/// * `200` - Array of breadcrumb items from root to current category
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Category not found
///
/// # Example
/// ```
/// GET /api/v1/inventory/categories/123e4567-e89b-12d3-a456-426614174000/breadcrumbs
/// ```
///
/// Response:
/// ```json
/// [
///   {"category_id": "root-uuid", "name": "Electronics", "slug": "electronics"},
///   {"category_id": "123e4567-e89b-12d3-a456-426614174000", "name": "Smartphones", "slug": "smartphones"}
/// ]
/// ```
pub async fn get_breadcrumbs(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<Json<Vec<inventory_service_core::domains::category::CategoryBreadcrumb>>, AppError> {
    let breadcrumbs = state
        .category_service
        .get_breadcrumbs(auth_user.tenant_id, category_id)
        .await?;
    Ok(Json(breadcrumbs))
}

/// GET /api/v1/inventory/categories/{category_id}/stats - Get category statistics
///
/// Returns comprehensive statistics for a category including product counts,
/// subcategory counts, and active/inactive product breakdowns.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `category_id` - UUID of the category to get statistics for
///
/// # Returns
/// * `200` - Category statistics including counts and breakdowns
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Category not found
///
/// # Example
/// ```
/// GET /api/v1/inventory/categories/123e4567-e89b-12d3-a456-426614174000/stats
/// ```
///
/// Response:
/// ```json
/// {
///   "category_id": "123e4567-e89b-12d3-a456-426614174000",
///   "name": "Smartphones",
///   "level": 1,
///   "product_count": 15,
///   "total_product_count": 45,
///   "subcategory_count": 3,
///   "active_product_count": 42,
///   "inactive_product_count": 3
/// }
/// ```
pub async fn get_category_stats(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<Json<CategoryStatsResponse>, AppError> {
    let stats = state
        .category_service
        .get_category_stats(auth_user.tenant_id, category_id)
        .await?;
    Ok(Json(stats))
}

/// GET /api/v1/inventory/categories/{category_id}/can-delete - Check deletion safety
///
/// Determines if a category can be safely deleted by checking if it has
/// child categories or associated products.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `category_id` - UUID of the category to check
///
/// # Returns
/// * `200` - Boolean indicating if category can be deleted
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Category not found
///
/// # Example
/// ```
/// GET /api/v1/inventory/categories/123e4567-e89b-12d3-a456-426614174000/can-delete
/// ```
///
/// Response: `true` or `false`
pub async fn can_delete_category(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<Json<bool>, AppError> {
    let can_delete = state
        .category_service
        .can_delete_category(auth_user.tenant_id, category_id)
        .await?;
    Ok(Json(can_delete))
}

/// POST /api/v1/inventory/categories/bulk/activate - Bulk activate categories
///
/// Sets the `is_active` flag to true for multiple categories in a single operation.
/// Useful for enabling multiple categories at once.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `request` - List of category IDs to activate
///
/// # Returns
/// * `200` - Operation result with affected count
/// * `400` - Invalid category IDs
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Example
/// ```json
/// POST /api/v1/inventory/categories/bulk/activate
/// {
///   "category_ids": [
///     "123e4567-e89b-12d3-a456-426614174000",
///     "987fcdeb-51a2-43d7-8f9e-123456789abc"
///   ]
/// }
/// ```
pub async fn bulk_activate_categories(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<BulkCategoryIds>,
) -> Result<Json<BulkOperationResponse>, AppError> {
    let response = state
        .category_service
        .bulk_activate_categories(auth_user.tenant_id, request.category_ids)
        .await?;
    Ok(Json(response))
}

/// POST /api/v1/inventory/categories/bulk/deactivate - Bulk deactivate categories
///
/// Sets the `is_active` flag to false for multiple categories in a single operation.
/// Useful for disabling multiple categories at once.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `request` - List of category IDs to deactivate
///
/// # Returns
/// * `200` - Operation result with affected count
/// * `400` - Invalid category IDs
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Example
/// ```json
/// POST /api/v1/inventory/categories/bulk/deactivate
/// {
///   "category_ids": [
///     "123e4567-e89b-12d3-a456-426614174000",
///     "987fcdeb-51a2-43d7-8f9e-123456789abc"
///   ]
/// }
/// ```
pub async fn bulk_deactivate_categories(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<BulkCategoryIds>,
) -> Result<Json<BulkOperationResponse>, AppError> {
    let response = state
        .category_service
        .bulk_deactivate_categories(auth_user.tenant_id, request.category_ids)
        .await?;
    Ok(Json(response))
}

/// POST /api/v1/inventory/categories/bulk/delete - Bulk delete categories
///
/// Performs soft delete on multiple categories in a single operation.
/// Requires admin privileges due to the destructive nature of the operation.
///
/// # Authentication
/// Requires admin user authentication
///
/// # Parameters
/// * `request` - List of category IDs to delete
///
/// # Returns
/// * `200` - Operation result with affected count
/// * `400` - Invalid category IDs or categories cannot be deleted
/// * `401` - Authentication required
/// * `403` - Admin privileges required
///
/// # Example
/// ```json
/// POST /api/v1/inventory/categories/bulk/delete
/// {
///   "category_ids": [
///     "123e4567-e89b-12d3-a456-426614174000",
///     "987fcdeb-51a2-43d7-8f9e-123456789abc"
///   ]
/// }
/// ```
pub async fn bulk_delete_categories(
    RequireAdmin(auth_user): RequireAdmin,
    State(state): State<AppState>,
    Json(request): Json<BulkCategoryIds>,
) -> Result<Json<BulkOperationResponse>, AppError> {
    let response = state
        .category_service
        .bulk_delete_categories(auth_user.tenant_id, request.category_ids)
        .await?;
    Ok(Json(response))
}

/// POST /api/v1/inventory/categories/products/move - Move products to category
///
/// Moves multiple products from their current categories to a new target category.
/// Product counts will be automatically updated for affected categories.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Parameters
/// * `request` - Product IDs and target category ID
///
/// # Returns
/// * `200` - Operation result with number of products moved
/// * `400` - Invalid product IDs or target category
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Target category not found
///
/// # Example
/// ```json
/// POST /api/v1/inventory/categories/products/move
/// {
///   "product_ids": [
///     "123e4567-e89b-12d3-a456-426614174000",
///     "987fcdeb-51a2-43d7-8f9e-123456789abc"
///   ],
///   "category_id": "456e7890-e89b-12d3-a456-426614174001"
/// }
/// ```
pub async fn move_products_to_category(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<MoveToCategoryRequest>,
) -> Result<Json<BulkOperationResponse>, AppError> {
    let response = state
        .category_service
        .move_products_to_category(auth_user.tenant_id, request)
        .await?;
    Ok(Json(response))
}

/// Query parameters for category tree endpoint
#[derive(Deserialize)]
pub struct CategoryTreeQuery {
    pub parent_id: Option<Uuid>,
    pub max_depth: Option<i32>,
}

/// Query parameters for search endpoint
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i32>,
}

/// Query parameters for top categories endpoint
#[derive(Deserialize)]
pub struct TopCategoriesQuery {
    pub limit: Option<i32>,
}

/// Request body for bulk operations
#[derive(Deserialize)]
pub struct BulkCategoryIds {
    pub category_ids: Vec<Uuid>,
}
