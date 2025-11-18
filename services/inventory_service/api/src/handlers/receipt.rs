//! Receipt HTTP handlers
//!
//! This module contains the Axum handlers for Goods Receipt Note (GRN) operations.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use uuid::Uuid;
use validator::Validate;

use inventory_service_core::dto::receipt::{
    ReceiptCreateRequest, ReceiptListQuery, ReceiptListResponse, ReceiptResponse,
};
use inventory_service_core::services::receipt::ReceiptService;

use shared_auth::extractors::{AuthUser, RequireAdmin};
use shared_error::AppError;

/// Application state for receipt operations
#[derive(Clone)]
pub struct AppState {
    pub receipt_service: Arc<dyn ReceiptService>,
}

impl AppState {
    /// Create a new AppState with the given receipt service
    pub fn new(receipt_service: Arc<dyn ReceiptService>) -> Self {
        Self { receipt_service }
    }
}

/// Create the receipt routes with state
pub fn create_receipt_routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_receipt).get(list_receipts))
        .route("/{receipt_id}", get(get_receipt))
        .with_state(state)
}

/// POST /api/v1/inventory/receipts - Create a new Goods Receipt Note
///
/// Creates a new GRN with the provided receipt data and line items.
/// The operation includes validation, stock movement creation, and event publishing.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Idempotency
/// Supports idempotent operations via X-Idempotency-Key header (future implementation)
///
/// # Parameters
/// * `request` - Receipt creation data including warehouse, supplier, and line items
///
/// # Returns
/// * `201` - Receipt created successfully with full receipt details
/// * `400` - Invalid request data or validation errors
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `409` - Receipt already exists (idempotency)
///
/// # Example
/// ```json
/// POST /api/v1/inventory/receipts
/// {
///   "warehouse_id": "550e8400-e29b-41d4-a716-446655440000",
///   "supplier_id": "550e8400-e29b-41d4-a716-446655440001",
///   "reference_number": "PO-12345",
///   "expected_delivery_date": "2025-11-20T10:00:00Z",
///   "notes": "Urgent delivery for production line",
///   "items": [
///     {
///       "product_id": "550e8400-e29b-41d4-a716-446655440002",
///       "expected_quantity": 100,
///       "received_quantity": 95,
///       "unit_cost": 1500,
///       "uom_id": "550e8400-e29b-41d4-a716-446655440003",
///       "lot_number": "LOT-2025-001",
///       "notes": "Slightly damaged packaging"
///     }
///   ]
/// }
/// ```
pub async fn create_receipt(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<ReceiptCreateRequest>,
) -> Result<(StatusCode, Json<ReceiptResponse>), AppError> {
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let receipt = state
        .receipt_service
        .create_receipt(auth_user.tenant_id, auth_user.user_id, request)
        .await?;

    Ok((StatusCode::CREATED, Json(receipt)))
}

/// GET /api/v1/inventory/receipts - List Goods Receipt Notes with pagination and filtering
///
/// Retrieves a paginated list of GRNs with optional filtering by warehouse,
/// supplier, status, and search terms. Results are ordered by creation date (newest first).
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `warehouse_id` - Filter by warehouse UUID (optional)
/// * `supplier_id` - Filter by supplier UUID (optional)
/// * `status` - Filter by receipt status (draft/confirmed/partially_received/received/cancelled, optional)
/// * `search` - Search in receipt number or reference number (optional)
/// * `created_after` - Filter receipts created after this date (optional)
/// * `created_before` - Filter receipts created before this date (optional)
/// * `page` - Page number (default: 1, min: 1)
/// * `page_size` - Items per page (default: 20, max: 100)
///
/// # Returns
/// * `200` - Paginated list of receipts with summary information
/// * `400` - Invalid query parameters
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Example
/// ```
/// GET /api/v1/inventory/receipts?page=1&page_size=10&warehouse_id=550e8400-e29b-41d4-a716-446655440000&status=received
/// ```
pub async fn list_receipts(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ReceiptListQuery>,
) -> Result<Json<ReceiptListResponse>, AppError> {
    query
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let response = state
        .receipt_service
        .list_receipts(auth_user.tenant_id, query)
        .await?;

    Ok(Json(response))
}

/// GET /api/v1/inventory/receipts/{receipt_id} - Get Goods Receipt Note by ID
///
/// Retrieves a single GRN by its unique identifier with full details including
/// all line items, quantities, costs, and metadata.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Path Parameters
/// * `receipt_id` - UUID of the receipt to retrieve
///
/// # Returns
/// * `200` - Complete receipt details with all line items
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
/// * `404` - Receipt not found or belongs to different tenant
///
/// # Example
/// ```
/// GET /api/v1/inventory/receipts/550e8400-e29b-41d4-a716-446655440000
/// ```
///
/// Response includes:
/// ```json
/// {
///   "receipt_id": "550e8400-e29b-41d4-a716-446655440000",
///   "receipt_number": "GRN-2025-00001",
///   "warehouse_id": "550e8400-e29b-41d4-a716-446655440001",
///   "status": "confirmed",
///   "total_quantity": 95,
///   "total_value": 142500,
///   "items": [
///     {
///       "receipt_item_id": "550e8400-e29b-41d4-a716-446655440002",
///       "product_id": "550e8400-e29b-41d4-a716-446655440003",
///       "received_quantity": 95,
///       "unit_cost": 1500,
///       "line_total": 142500,
///       "lot_number": "LOT-2025-001"
///     }
///   ]
/// }
/// ```
pub async fn get_receipt(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(receipt_id): Path<Uuid>,
) -> Result<Json<ReceiptResponse>, AppError> {
    let receipt = state
        .receipt_service
        .get_receipt(auth_user.tenant_id, receipt_id)
        .await?;

    Ok(Json(receipt))
}
