use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Parse a date string to DateTime<Utc>
/// Accepts both YYYY-MM-DD format and RFC3339 format
fn parse_date_to_datetime(date_str: &str) -> Result<DateTime<Utc>, String> {
    // Try RFC3339 first (e.g., "2026-01-28T10:00:00Z")
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try YYYY-MM-DD format (e.g., "2026-01-28")
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let datetime = date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        return Ok(Utc.from_utc_datetime(&datetime));
    }

    // If neither works, return an error
    Err(format!("Invalid date format: {}", date_str))
}

use inventory_service_core::domains::inventory::dto::transfer_dto::{
    CancelTransferRequest, CancelTransferResponse, ConfirmTransferRequest, ConfirmTransferResponse,
    CreateTransferRequest, CreateTransferResponse, ListTransfersParams, ListTransfersResponse,
    ReceiveTransferRequest, ReceiveTransferResponse, TransferResponse,
};
use inventory_service_core::domains::inventory::transfer::{
    Transfer, TransferItem, TransferStatus,
};
use inventory_service_core::models::CreateStockMoveRequest;
use inventory_service_core::repositories::stock::{InventoryLevelRepository, StockMoveRepository};
use inventory_service_core::repositories::transfer::{TransferItemRepository, TransferRepository};
use inventory_service_core::repositories::warehouse::WarehouseRepository;
use inventory_service_core::services::transfer::TransferService;
use shared_error::AppError;

/// PostgreSQL implementation of TransferService
pub struct PgTransferService {
    transfer_repo: Arc<dyn TransferRepository>,
    transfer_item_repo: Arc<dyn TransferItemRepository>,
    stock_move_repo: Arc<dyn StockMoveRepository>,
    inventory_repo: Arc<dyn InventoryLevelRepository>,
    warehouse_repo: Arc<dyn WarehouseRepository>,
}

impl PgTransferService {
    /// Create a new service instance
    pub fn new(
        transfer_repo: Arc<dyn TransferRepository>,
        transfer_item_repo: Arc<dyn TransferItemRepository>,
        stock_move_repo: Arc<dyn StockMoveRepository>,
        inventory_repo: Arc<dyn InventoryLevelRepository>,
        warehouse_repo: Arc<dyn WarehouseRepository>,
    ) -> Self {
        Self {
            transfer_repo,
            transfer_item_repo,
            stock_move_repo,
            inventory_repo,
            warehouse_repo,
        }
    }

    /// Get or create a default location for a warehouse
    async fn get_or_create_default_location(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Uuid, AppError> {
        use inventory_service_core::domains::inventory::dto::warehouse_dto::CreateWarehouseLocationRequest;

        // Try to get existing locations for the warehouse
        let locations = self
            .warehouse_repo
            .get_locations_by_warehouse(tenant_id, warehouse_id)
            .await?;

        if let Some(location) = locations.first() {
            return Ok(location.location_id);
        }

        // No locations exist, create a default one
        let location = self
            .warehouse_repo
            .create_location(
                tenant_id,
                warehouse_id,
                CreateWarehouseLocationRequest {
                    zone_id: None,
                    location_code: "DEFAULT".to_string(),
                    location_name: Some("Default".to_string()),
                    description: Some("Auto-created default location for transfers".to_string()),
                    location_type: "bin".to_string(),
                    coordinates: None,
                    dimensions: None,
                    capacity_info: None,
                    location_attributes: None,
                },
            )
            .await?;

        Ok(location.location_id)
    }
}

#[async_trait]
impl TransferService for PgTransferService {
    async fn list_transfers(
        &self,
        tenant_id: Uuid,
        params: ListTransfersParams,
    ) -> Result<ListTransfersResponse, AppError> {
        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * page_size;

        let items = self
            .transfer_repo
            .list(
                tenant_id,
                params.source_warehouse_id,
                params.destination_warehouse_id,
                params.status.clone(),
                Some(page_size),
                Some(offset),
            )
            .await?;

        let total = self
            .transfer_repo
            .count(
                tenant_id,
                params.source_warehouse_id,
                params.destination_warehouse_id,
                params.status,
            )
            .await?;

        let total_pages = (total + page_size - 1) / page_size;

        Ok(ListTransfersResponse {
            items,
            total,
            page,
            page_size,
            total_pages,
        })
    }

    async fn get_transfer(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
    ) -> Result<TransferResponse, AppError> {
        let transfer = self
            .transfer_repo
            .find_by_id(tenant_id, transfer_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Transfer not found".to_string()))?;

        let items = self
            .transfer_item_repo
            .find_by_transfer_id(tenant_id, transfer_id)
            .await?;

        Ok(TransferResponse { transfer, items })
    }

    async fn create_transfer(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateTransferRequest,
    ) -> Result<CreateTransferResponse, AppError> {
        // Validate warehouses are different
        if request.source_warehouse_id == request.destination_warehouse_id {
            return Err(AppError::ValidationError(
                "Source and destination warehouses must be different".to_string(),
            ));
        }

        // Parse dates - accepts both YYYY-MM-DD and RFC3339 formats
        let expected_ship_date = if let Some(date_str) = &request.expected_ship_date {
            Some(parse_date_to_datetime(date_str).map_err(|_| {
                AppError::ValidationError(
                    "Invalid expected_ship_date format. Use YYYY-MM-DD or ISO 8601 format."
                        .to_string(),
                )
            })?)
        } else {
            None
        };

        let expected_receive_date = if let Some(date_str) = &request.expected_receive_date {
            Some(parse_date_to_datetime(date_str).map_err(|_| {
                AppError::ValidationError(
                    "Invalid expected_receive_date format. Use YYYY-MM-DD or ISO 8601 format."
                        .to_string(),
                )
            })?)
        } else {
            None
        };

        // Calculate totals
        let mut total_quantity = 0i64;
        let mut total_value = 0i64;

        // Validate item quantities are positive
        for item_req in &request.items {
            if item_req.quantity <= 0 {
                return Err(AppError::ValidationError(format!(
                    "Transfer item quantity must be positive, got {}",
                    item_req.quantity
                )));
            }
        }

        // Note: Product and UOM existence/validation is handled by database constraints
        // (foreign key constraints on product_id and uom_id in stock_transfer_items table)

        let items: Vec<TransferItem> = request
            .items
            .into_iter()
            .map(|item_req| {
                let line_total = item_req.quantity * item_req.unit_cost.unwrap_or(0);
                total_quantity += item_req.quantity;
                total_value += line_total;

                TransferItem {
                    transfer_item_id: Uuid::now_v7(),
                    tenant_id,
                    transfer_id: Uuid::nil(), // Will be set after transfer creation
                    product_id: item_req.product_id,
                    quantity: item_req.quantity,
                    uom_id: item_req.uom_id,
                    unit_cost: item_req.unit_cost,
                    line_total,
                    line_number: item_req.line_number,
                    source_zone_id: item_req.source_zone_id,
                    source_location_id: item_req.source_location_id,
                    destination_zone_id: item_req.destination_zone_id,
                    destination_location_id: item_req.destination_location_id,
                    notes: item_req.notes,
                    updated_by: Some(user_id),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    deleted_at: None,
                    deleted_by: None,
                }
            })
            .collect();

        // Create transfer
        let transfer = Transfer {
            transfer_id: Uuid::now_v7(),
            tenant_id,
            transfer_number: "".to_string(), // Will be set later
            reference_number: request.reference_number,
            source_warehouse_id: request.source_warehouse_id,
            destination_warehouse_id: request.destination_warehouse_id,
            status: TransferStatus::Draft,
            transfer_type: request.transfer_type,
            priority: request.priority,
            transfer_date: Utc::now(),
            expected_ship_date,
            actual_ship_date: None,
            expected_receive_date,
            actual_receive_date: None,
            shipping_method: request.shipping_method,
            carrier: None,
            tracking_number: None,
            shipping_cost: None,
            notes: request.notes,
            reason: request.reason,
            created_by: user_id,
            updated_by: Some(user_id),
            approved_by: None,
            approved_at: None,
            total_quantity,
            total_value,
            currency_code: "VND".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            deleted_by: None,
        };

        let created_transfer = self.transfer_repo.create(tenant_id, &transfer).await?;

        // Set transfer_id for items and create them
        let items_with_transfer_id: Vec<TransferItem> = items
            .into_iter()
            .map(|mut item| {
                item.transfer_id = created_transfer.transfer_id;
                item
            })
            .collect();

        let created_items = self
            .transfer_item_repo
            .create_batch(tenant_id, &items_with_transfer_id)
            .await?;

        Ok(CreateTransferResponse {
            transfer_id: created_transfer.transfer_id,
            transfer_number: created_transfer.transfer_number,
            status: created_transfer.status,
            items_count: created_items.len(),
        })
    }

    async fn confirm_transfer(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        user_id: Uuid,
        _request: ConfirmTransferRequest,
    ) -> Result<ConfirmTransferResponse, AppError> {
        // Find transfer
        let transfer = self
            .transfer_repo
            .find_by_id(tenant_id, transfer_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Transfer not found".to_string()))?;

        if transfer.status != TransferStatus::Draft {
            return Err(AppError::ValidationError(
                "Only draft transfers can be confirmed".to_string(),
            ));
        }

        // Get items
        let items = self
            .transfer_item_repo
            .find_by_transfer_id(tenant_id, transfer_id)
            .await?;

        // Get fallback locations for items without specified locations
        let fallback_source_location_id = self
            .get_or_create_default_location(tenant_id, transfer.source_warehouse_id)
            .await?;
        let fallback_destination_location_id = self
            .get_or_create_default_location(tenant_id, transfer.destination_warehouse_id)
            .await?;

        // Create stock moves from source to destination (simplified 2-step flow)
        // Module 4.5: Now supports location-level tracking from transfer items
        for item in &items {
            // Use item's source_location_id if specified, otherwise use fallback
            let effective_source_location_id = item
                .source_location_id
                .unwrap_or(fallback_source_location_id);
            let effective_destination_location_id = item
                .destination_location_id
                .unwrap_or(fallback_destination_location_id);

            let stock_move = CreateStockMoveRequest {
                product_id: item.product_id,
                source_location_id: Some(effective_source_location_id),
                destination_location_id: Some(effective_destination_location_id),
                move_type: "transfer".to_string(),
                quantity: -item.quantity, // Outgoing from source
                unit_cost: item.unit_cost,
                reference_type: "transfer".to_string(),
                reference_id: transfer_id,
                idempotency_key: format!("transfer-{}-item-{}", transfer_id, item.transfer_item_id),
                move_reason: Some(format!("Transfer {} confirmation", transfer.transfer_number)),
                lot_serial_id: None, // TODO: Set if lot-tracked product
                batch_info: None,
                metadata: None,
            };
            self.stock_move_repo.create(&stock_move, tenant_id).await?;
        }

        // Update inventory levels (decrement source)
        // Module 4.5: Now supports location-level inventory tracking
        for item in &items {
            self.inventory_repo
                .update_available_quantity(
                    tenant_id,
                    transfer.source_warehouse_id,
                    item.source_location_id, // Use item's location if specified
                    item.product_id,
                    -item.quantity,
                )
                .await?;
        }

        // Confirm transfer (set to Shipped)
        self.transfer_repo
            .confirm_transfer(tenant_id, transfer_id, user_id, user_id)
            .await?;

        Ok(ConfirmTransferResponse {
            transfer_id,
            status: TransferStatus::Shipped,
            confirmed_at: Utc::now().to_rfc3339(),
        })
    }

    async fn receive_transfer(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        user_id: Uuid,
        _request: ReceiveTransferRequest,
    ) -> Result<ReceiveTransferResponse, AppError> {
        // Find transfer
        let transfer = self
            .transfer_repo
            .find_by_id(tenant_id, transfer_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Transfer not found".to_string()))?;

        if transfer.status != TransferStatus::Shipped {
            return Err(AppError::ValidationError(
                "Only shipped transfers can be received".to_string(),
            ));
        }

        // Get items
        let items = self
            .transfer_item_repo
            .find_by_transfer_id(tenant_id, transfer_id)
            .await?;

        // Get fallback locations for items without specified locations
        let fallback_source_location_id = self
            .get_or_create_default_location(tenant_id, transfer.source_warehouse_id)
            .await?;
        let fallback_destination_location_id = self
            .get_or_create_default_location(tenant_id, transfer.destination_warehouse_id)
            .await?;

        // Create stock moves from source to destination (complete the transfer)
        // Module 4.5: Now supports location-level tracking from transfer items
        let mut stock_moves_created = 0;
        for item in &items {
            // Use item's location if specified, otherwise use fallback
            let effective_source_location_id = item
                .source_location_id
                .unwrap_or(fallback_source_location_id);
            let effective_destination_location_id = item
                .destination_location_id
                .unwrap_or(fallback_destination_location_id);

            let stock_move = CreateStockMoveRequest {
                product_id: item.product_id,
                source_location_id: Some(effective_source_location_id),
                destination_location_id: Some(effective_destination_location_id),
                move_type: "transfer".to_string(),
                quantity: item.quantity, // Incoming to destination
                unit_cost: item.unit_cost,
                reference_type: "transfer".to_string(),
                reference_id: transfer_id,
                idempotency_key: format!(
                    "transfer-receive-{}-item-{}",
                    transfer_id, item.transfer_item_id
                ),
                move_reason: Some(format!("Transfer {} receipt", transfer.transfer_number)),
                lot_serial_id: None, // TODO: Set if lot-tracked product
                batch_info: None,
                metadata: None,
            };
            self.stock_move_repo.create(&stock_move, tenant_id).await?;
            stock_moves_created += 1;
        }

        // Update inventory levels (increment destination)
        // Module 4.5: Now supports location-level inventory tracking
        for item in &items {
            self.inventory_repo
                .update_available_quantity(
                    tenant_id,
                    transfer.destination_warehouse_id,
                    item.destination_location_id, // Use item's destination location if specified
                    item.product_id,
                    item.quantity,
                )
                .await?;
        }

        // Receive transfer
        self.transfer_repo
            .receive_transfer(tenant_id, transfer_id, user_id)
            .await?;

        // TODO: Publish inventory.transfer.completed event

        Ok(ReceiveTransferResponse {
            transfer_id,
            status: TransferStatus::Received,
            received_at: Utc::now().to_rfc3339(),
            stock_moves_created,
        })
    }

    async fn cancel_transfer(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        user_id: Uuid,
        request: CancelTransferRequest,
    ) -> Result<CancelTransferResponse, AppError> {
        // Find transfer
        let transfer = self
            .transfer_repo
            .find_by_id(tenant_id, transfer_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Transfer not found".to_string()))?;

        // Only draft or confirmed transfers can be cancelled
        if transfer.status != TransferStatus::Draft && transfer.status != TransferStatus::Confirmed
        {
            return Err(AppError::ValidationError(format!(
                "Cannot cancel transfer in '{}' status. Only draft or confirmed transfers can be cancelled.",
                match transfer.status {
                    TransferStatus::Draft => "draft",
                    TransferStatus::Confirmed => "confirmed",
                    TransferStatus::PartiallyPicked => "partially_picked",
                    TransferStatus::Picked => "picked",
                    TransferStatus::PartiallyShipped => "partially_shipped",
                    TransferStatus::Shipped => "shipped",
                    TransferStatus::Received => "received",
                    TransferStatus::Cancelled => "cancelled",
                }
            )));
        }

        // Cancel the transfer
        self.transfer_repo
            .cancel_transfer(tenant_id, transfer_id, user_id, request.reason)
            .await?;

        Ok(CancelTransferResponse {
            transfer_id,
            status: TransferStatus::Cancelled,
            cancelled_at: Utc::now().to_rfc3339(),
        })
    }
}
