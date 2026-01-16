//! Landed cost HTTP handlers
//!
//! This module contains the Axum handlers for landed cost endpoints.

use axum::{
    extract::{Extension, Path, Query},
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::landed_cost_dto::{
    AllocationPreviewResponse, CreateLandedCostDocumentRequest, CreateLandedCostLineRequest,
    LandedCostDocumentListResponse, LandedCostDocumentWithLinesDto, LandedCostLineDto,
    PostLandedCostResponse, UpdateLandedCostDocumentRequest, UpdateLandedCostLineRequest,
};
use inventory_service_core::domains::inventory::landed_cost::LandedCostDocument;

use crate::state::AppState;
use shared_auth::extractors::AuthUser;
use shared_error::AppError;

/// Query parameters for listing documents
#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    /// Filter by status (draft, posted, cancelled)
    pub status: Option<String>,
    /// Filter by receipt ID
    pub receipt_id: Option<Uuid>,
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: i32,
    /// Items per page
    #[serde(default = "default_page_size")]
    pub page_size: i32,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

/// Create the landed cost routes
pub fn create_landed_cost_routes() -> Router {
    Router::new()
        // Document routes
        .route("/", post(create_document))
        .route("/", get(list_documents))
        .route("/{document_id}", get(get_document))
        .route("/{document_id}", put(update_document))
        .route("/{document_id}", delete(delete_document))
        // Line routes
        .route("/{document_id}/lines", post(add_line))
        .route("/{document_id}/lines/{line_id}", put(update_line))
        .route("/{document_id}/lines/{line_id}", delete(delete_line))
        // Workflow routes
        .route("/{document_id}/preview", get(get_allocation_preview))
        .route("/{document_id}/post", post(post_document))
        .route("/{document_id}/cancel", post(cancel_document))
}

/// POST /api/v1/inventory/landed-costs - Create a new landed cost document
///
/// Creates a new landed cost document in draft status.
/// Can optionally include initial cost lines.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Request Body
/// ```json
/// {
///   "receipt_id": "uuid",
///   "reference_number": "optional string",
///   "allocation_method": "by_value" | "by_quantity" | "equal",
///   "currency_code": "VND",
///   "notes": "optional string",
///   "lines": [...]
/// }
/// ```
///
/// # Returns
/// * `201` - Created document with lines
/// * `400` - Invalid request
/// * `401` - Authentication required
#[utoipa::path(
    post,
    path = "/api/v1/inventory/landed-costs",
    tag = "landed_costs",
    operation_id = "create_landed_cost_document",
    request_body = CreateLandedCostDocumentRequest,
    responses(
        (status = 201, description = "Created landed cost document", body = LandedCostDocumentWithLinesDto),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Authentication required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_document(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Json(request): Json<CreateLandedCostDocumentRequest>,
) -> Result<(axum::http::StatusCode, Json<LandedCostDocumentWithLinesDto>), AppError> {
    let document = state
        .landed_cost_service
        .create_document(auth_user.tenant_id, auth_user.user_id, request)
        .await?;

    Ok((axum::http::StatusCode::CREATED, Json(document)))
}

/// GET /api/v1/inventory/landed-costs - List landed cost documents
///
/// Returns a paginated list of landed cost documents with optional filters.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `status` - Filter by status (draft, posted, cancelled)
/// * `receipt_id` - Filter by receipt ID
/// * `page` - Page number (default: 1)
/// * `page_size` - Items per page (default: 20)
///
/// # Returns
/// * `200` - List of documents with pagination
/// * `401` - Authentication required
#[utoipa::path(
    get,
    path = "/api/v1/inventory/landed-costs",
    tag = "landed_costs",
    operation_id = "list_landed_cost_documents",
    params(
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("receipt_id" = Option<Uuid>, Query, description = "Filter by receipt ID"),
        ("page" = i32, Query, description = "Page number"),
        ("page_size" = i32, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List of landed cost documents", body = LandedCostDocumentListResponse),
        (status = 401, description = "Authentication required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_documents(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<LandedCostDocumentListResponse>, AppError> {
    let documents = state
        .landed_cost_service
        .list_documents(
            auth_user.tenant_id,
            query.status,
            query.receipt_id,
            query.page,
            query.page_size,
        )
        .await?;

    Ok(Json(documents))
}

/// GET /api/v1/inventory/landed-costs/{document_id} - Get a landed cost document
///
/// Returns a single landed cost document with its lines and allocations.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `document_id` - Document UUID
///
/// # Returns
/// * `200` - Document with lines
/// * `404` - Document not found
/// * `401` - Authentication required
#[utoipa::path(
    get,
    path = "/api/v1/inventory/landed-costs/{document_id}",
    tag = "landed_costs",
    operation_id = "get_landed_cost_document",
    params(
        ("document_id" = Uuid, Path, description = "Document ID")
    ),
    responses(
        (status = 200, description = "Landed cost document with lines", body = LandedCostDocumentWithLinesDto),
        (status = 404, description = "Document not found"),
        (status = 401, description = "Authentication required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_document(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<LandedCostDocumentWithLinesDto>, AppError> {
    let document = state
        .landed_cost_service
        .get_document(auth_user.tenant_id, document_id)
        .await?;

    Ok(Json(document))
}

/// PUT /api/v1/inventory/landed-costs/{document_id} - Update a landed cost document
///
/// Updates a landed cost document. Only draft documents can be updated.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `document_id` - Document UUID
///
/// # Request Body
/// ```json
/// {
///   "reference_number": "optional string",
///   "allocation_method": "by_value" | "by_quantity" | "equal",
///   "notes": "optional string"
/// }
/// ```
///
/// # Returns
/// * `200` - Updated document
/// * `400` - Cannot update non-draft document
/// * `404` - Document not found
/// * `401` - Authentication required
#[utoipa::path(
    put,
    path = "/api/v1/inventory/landed-costs/{document_id}",
    tag = "landed_costs",
    operation_id = "update_landed_cost_document",
    params(
        ("document_id" = Uuid, Path, description = "Document ID")
    ),
    request_body = UpdateLandedCostDocumentRequest,
    responses(
        (status = 200, description = "Updated document", body = LandedCostDocument),
        (status = 400, description = "Cannot update non-draft document"),
        (status = 404, description = "Document not found"),
        (status = 401, description = "Authentication required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_document(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(document_id): Path<Uuid>,
    Json(request): Json<UpdateLandedCostDocumentRequest>,
) -> Result<Json<LandedCostDocument>, AppError> {
    let document = state
        .landed_cost_service
        .update_document(auth_user.tenant_id, document_id, request)
        .await?;

    Ok(Json(document))
}

/// DELETE /api/v1/inventory/landed-costs/{document_id} - Delete a landed cost document
///
/// Deletes a landed cost document. Only draft documents can be deleted.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `document_id` - Document UUID
///
/// # Returns
/// * `204` - Document deleted
/// * `400` - Cannot delete non-draft document
/// * `404` - Document not found
/// * `401` - Authentication required
#[utoipa::path(
    delete,
    path = "/api/v1/inventory/landed-costs/{document_id}",
    tag = "landed_costs",
    operation_id = "delete_landed_cost_document",
    params(
        ("document_id" = Uuid, Path, description = "Document ID")
    ),
    responses(
        (status = 204, description = "Document deleted"),
        (status = 400, description = "Cannot delete non-draft document"),
        (status = 404, description = "Document not found"),
        (status = 401, description = "Authentication required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_document(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(document_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    state
        .landed_cost_service
        .delete_document(auth_user.tenant_id, document_id)
        .await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// POST /api/v1/inventory/landed-costs/{document_id}/lines - Add a cost line
///
/// Adds a cost line to a landed cost document. Only draft documents can have lines added.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `document_id` - Document UUID
///
/// # Request Body
/// ```json
/// {
///   "cost_type": "freight" | "customs" | "handling" | "insurance" | "other",
///   "description": "optional string",
///   "amount": 100000,
///   "vendor_reference": "optional string"
/// }
/// ```
///
/// # Returns
/// * `201` - Created line
/// * `400` - Cannot add lines to non-draft document
/// * `404` - Document not found
/// * `401` - Authentication required
#[utoipa::path(
    post,
    path = "/api/v1/inventory/landed-costs/{document_id}/lines",
    tag = "landed_costs",
    operation_id = "add_landed_cost_line",
    params(
        ("document_id" = Uuid, Path, description = "Document ID")
    ),
    request_body = CreateLandedCostLineRequest,
    responses(
        (status = 201, description = "Created line", body = LandedCostLineDto),
        (status = 400, description = "Cannot add lines to non-draft document"),
        (status = 404, description = "Document not found"),
        (status = 401, description = "Authentication required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn add_line(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(document_id): Path<Uuid>,
    Json(request): Json<CreateLandedCostLineRequest>,
) -> Result<(axum::http::StatusCode, Json<LandedCostLineDto>), AppError> {
    let line = state
        .landed_cost_service
        .add_line(auth_user.tenant_id, document_id, request)
        .await?;

    Ok((axum::http::StatusCode::CREATED, Json(line)))
}

/// Path parameters for line operations
#[derive(Debug, Deserialize)]
pub struct LinePathParams {
    pub document_id: Uuid,
    pub line_id: Uuid,
}

/// PUT /api/v1/inventory/landed-costs/{document_id}/lines/{line_id} - Update a cost line
///
/// Updates a cost line. Only lines on draft documents can be updated.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `document_id` - Document UUID
/// * `line_id` - Line UUID
///
/// # Request Body
/// ```json
/// {
///   "cost_type": "optional string",
///   "description": "optional string",
///   "amount": optional number,
///   "vendor_reference": "optional string"
/// }
/// ```
///
/// # Returns
/// * `200` - Updated line
/// * `400` - Cannot update lines on non-draft document
/// * `404` - Line not found
/// * `401` - Authentication required
#[utoipa::path(
    put,
    path = "/api/v1/inventory/landed-costs/{document_id}/lines/{line_id}",
    tag = "landed_costs",
    operation_id = "update_landed_cost_line",
    params(
        ("document_id" = Uuid, Path, description = "Document ID"),
        ("line_id" = Uuid, Path, description = "Line ID")
    ),
    request_body = UpdateLandedCostLineRequest,
    responses(
        (status = 200, description = "Updated line", body = LandedCostLineDto),
        (status = 400, description = "Cannot update lines on non-draft document"),
        (status = 404, description = "Line not found"),
        (status = 401, description = "Authentication required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_line(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(params): Path<LinePathParams>,
    Json(request): Json<UpdateLandedCostLineRequest>,
) -> Result<Json<LandedCostLineDto>, AppError> {
    let line = state
        .landed_cost_service
        .update_line(auth_user.tenant_id, params.document_id, params.line_id, request)
        .await?;

    Ok(Json(line))
}

/// DELETE /api/v1/inventory/landed-costs/{document_id}/lines/{line_id} - Delete a cost line
///
/// Deletes a cost line. Only lines on draft documents can be deleted.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `document_id` - Document UUID
/// * `line_id` - Line UUID
///
/// # Returns
/// * `204` - Line deleted
/// * `400` - Cannot delete lines on non-draft document
/// * `404` - Line not found
/// * `401` - Authentication required
#[utoipa::path(
    delete,
    path = "/api/v1/inventory/landed-costs/{document_id}/lines/{line_id}",
    tag = "landed_costs",
    operation_id = "delete_landed_cost_line",
    params(
        ("document_id" = Uuid, Path, description = "Document ID"),
        ("line_id" = Uuid, Path, description = "Line ID")
    ),
    responses(
        (status = 204, description = "Line deleted"),
        (status = 400, description = "Cannot delete lines on non-draft document"),
        (status = 404, description = "Line not found"),
        (status = 401, description = "Authentication required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_line(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(params): Path<LinePathParams>,
) -> Result<axum::http::StatusCode, AppError> {
    state
        .landed_cost_service
        .delete_line(auth_user.tenant_id, params.document_id, params.line_id)
        .await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// GET /api/v1/inventory/landed-costs/{document_id}/preview - Get allocation preview
///
/// Returns a preview of how costs would be allocated without actually posting.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `document_id` - Document UUID
///
/// # Returns
/// * `200` - Allocation preview
/// * `400` - Document has no cost lines
/// * `404` - Document not found
/// * `401` - Authentication required
#[utoipa::path(
    get,
    path = "/api/v1/inventory/landed-costs/{document_id}/preview",
    tag = "landed_costs",
    operation_id = "get_landed_cost_allocation_preview",
    params(
        ("document_id" = Uuid, Path, description = "Document ID")
    ),
    responses(
        (status = 200, description = "Allocation preview", body = AllocationPreviewResponse),
        (status = 400, description = "Document has no cost lines"),
        (status = 404, description = "Document not found"),
        (status = 401, description = "Authentication required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_allocation_preview(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<AllocationPreviewResponse>, AppError> {
    let preview = state
        .landed_cost_service
        .get_allocation_preview(auth_user.tenant_id, document_id)
        .await?;

    Ok(Json(preview))
}

/// POST /api/v1/inventory/landed-costs/{document_id}/post - Post a landed cost document
///
/// Posts a landed cost document, allocating costs to receipt items.
/// Only draft documents with at least one cost line can be posted.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `document_id` - Document UUID
///
/// # Returns
/// * `200` - Post result with allocations
/// * `400` - Document is not in draft status or has no lines
/// * `404` - Document not found
/// * `401` - Authentication required
#[utoipa::path(
    post,
    path = "/api/v1/inventory/landed-costs/{document_id}/post",
    tag = "landed_costs",
    operation_id = "post_landed_cost_document",
    params(
        ("document_id" = Uuid, Path, description = "Document ID")
    ),
    responses(
        (status = 200, description = "Post result with allocations", body = PostLandedCostResponse),
        (status = 400, description = "Document is not in draft status or has no lines"),
        (status = 404, description = "Document not found"),
        (status = 401, description = "Authentication required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn post_document(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<PostLandedCostResponse>, AppError> {
    let result = state
        .landed_cost_service
        .post_document(auth_user.tenant_id, document_id)
        .await?;

    Ok(Json(result))
}

/// POST /api/v1/inventory/landed-costs/{document_id}/cancel - Cancel a posted document
///
/// Cancels a posted landed cost document, reversing the allocations.
/// Only posted documents can be cancelled.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `document_id` - Document UUID
///
/// # Returns
/// * `204` - Document cancelled
/// * `400` - Document is not in posted status
/// * `404` - Document not found
/// * `401` - Authentication required
#[utoipa::path(
    post,
    path = "/api/v1/inventory/landed-costs/{document_id}/cancel",
    tag = "landed_costs",
    operation_id = "cancel_landed_cost_document",
    params(
        ("document_id" = Uuid, Path, description = "Document ID")
    ),
    responses(
        (status = 204, description = "Document cancelled"),
        (status = 400, description = "Document is not in posted status"),
        (status = 404, description = "Document not found"),
        (status = 401, description = "Authentication required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn cancel_document(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Path(document_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    state
        .landed_cost_service
        .cancel_document(auth_user.tenant_id, document_id)
        .await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
