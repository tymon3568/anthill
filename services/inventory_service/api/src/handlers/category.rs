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
use inventory_service_core::services::category::CategoryService;

use shared_auth::enforcer::SharedEnforcer;
use shared_auth::extractors::{AuthUser, JwtSecretProvider, KanidmClientProvider, RequireAdmin};
use shared_error::AppError;
use shared_kanidm_client::KanidmClient;

/// Application state for inventory service
#[derive(Clone)]
pub struct AppState {
    pub category_service: Arc<dyn CategoryService>,
    pub enforcer: SharedEnforcer,
    pub jwt_secret: String,
    pub kanidm_client: KanidmClient,
}

impl JwtSecretProvider for AppState {
    fn get_jwt_secret(&self) -> &str {
        &self.jwt_secret
    }
}

impl KanidmClientProvider for AppState {
    fn get_kanidm_client(&self) -> &KanidmClient {
        &self.kanidm_client
    }
}

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

/// GET /api/v1/inventory/categories - List categories with pagination
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

/// GET /api/v1/inventory/categories/tree - Get category tree
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

/// GET /api/v1/inventory/categories/search - Search categories
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

/// DELETE /api/v1/inventory/categories/{category_id} - Delete category
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

/// GET /api/v1/inventory/categories/{category_id}/children - Get category children
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

/// GET /api/v1/inventory/categories/{category_id}/breadcrumbs - Get category breadcrumbs
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

/// GET /api/v1/inventory/categories/{category_id}/can-delete - Check if category can be deleted
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
