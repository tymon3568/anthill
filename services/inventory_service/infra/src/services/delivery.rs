//! Delivery service implementation
//!
//! This module contains the business logic implementation for Delivery Order operations.

use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::dto::delivery::{
    PackItemsRequest, PackItemsResponse, PickItemsRequest, PickItemsResponse, ShipItemsRequest,
    ShipItemsResponse,
};
use inventory_service_core::models::{CreateStockMoveRequest, DeliveryOrderStatus};
use inventory_service_core::repositories::{
    DeliveryOrderItemRepository, DeliveryOrderRepository, InventoryLevelRepository,
    StockMoveRepository,
};
use inventory_service_core::services::delivery::DeliveryService;
use shared_error::AppError;

/// PostgreSQL implementation of the delivery service
pub struct DeliveryServiceImpl {
    delivery_repo: Arc<dyn DeliveryOrderRepository>,
    delivery_item_repo: Arc<dyn DeliveryOrderItemRepository>,
    stock_move_repo: Arc<dyn StockMoveRepository>,
    inventory_level_repo: Arc<dyn InventoryLevelRepository>,
}

impl DeliveryServiceImpl {
    /// Create a new delivery service with the given repositories
    pub fn new(
        delivery_repo: Arc<dyn DeliveryOrderRepository>,
        delivery_item_repo: Arc<dyn DeliveryOrderItemRepository>,
        stock_move_repo: Arc<dyn StockMoveRepository>,
        inventory_level_repo: Arc<dyn InventoryLevelRepository>,
    ) -> Self {
        Self {
            delivery_repo,
            delivery_item_repo,
            stock_move_repo,
            inventory_level_repo,
        }
    }
}

#[async_trait]
impl DeliveryService for DeliveryServiceImpl {
    async fn pick_items(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
        user_id: Uuid,
        request: PickItemsRequest,
    ) -> Result<PickItemsResponse, AppError> {
        // Validate request has items
        if request.items.is_empty() {
            return Err(AppError::ValidationError(
                "At least one item must be specified for picking".to_string(),
            ));
        }

        // Begin transaction
        let mut tx = self.delivery_repo.begin_transaction().await?;

        // Find the delivery order within transaction
        let mut delivery_order = self
            .delivery_repo
            .find_by_id_with_tx(&mut tx, tenant_id, delivery_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Delivery order {} not found", delivery_id))
            })?;

        // Check if the delivery order is in a valid state for picking
        if delivery_order.status != DeliveryOrderStatus::Confirmed {
            return Err(AppError::ValidationError(format!(
                "Cannot pick items for delivery order with status '{}'. Only 'Confirmed' orders can be picked.",
                delivery_order.status
            )));
        }

        let mut total_picked_quantity = 0;
        let mut updated_items_count = 0;

        // Process each item in the request
        for pick_item in &request.items {
            // Find the delivery item within transaction
            let mut delivery_item = self
                .delivery_item_repo
                .find_by_id_with_tx(&mut tx, tenant_id, pick_item.delivery_item_id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!(
                        "Delivery item {} not found",
                        pick_item.delivery_item_id
                    ))
                })?;

            // Verify the item belongs to the delivery order
            if delivery_item.delivery_id != delivery_id {
                return Err(AppError::ValidationError(format!(
                    "Delivery item {} does not belong to delivery order {}",
                    pick_item.delivery_item_id, delivery_id
                )));
            }

            // Validate picked quantity
            if pick_item.picked_quantity <= 0 {
                return Err(AppError::ValidationError(format!(
                    "Picked quantity must be positive for item {}",
                    pick_item.delivery_item_id
                )));
            }

            let remaining = delivery_item.ordered_quantity - delivery_item.picked_quantity;
            if pick_item.picked_quantity > remaining {
                return Err(AppError::ValidationError(format!(
                    "Cannot pick {} units for item {}. Only {} units remaining to pick.",
                    pick_item.picked_quantity, pick_item.delivery_item_id, remaining
                )));
            }

            // Update the picked quantity
            delivery_item.picked_quantity += pick_item.picked_quantity;
            delivery_item.updated_at = Utc::now();

            // Save the updated item within transaction
            self.delivery_item_repo
                .update_with_tx(&mut tx, &delivery_item)
                .await?;

            total_picked_quantity += pick_item.picked_quantity;
            updated_items_count += 1;
        }

        // Fetch all delivery items to check if fully picked
        let all_items = self
            .delivery_item_repo
            .find_by_delivery_id_with_tx(&mut tx, tenant_id, delivery_id)
            .await?;
        let all_fully_picked = all_items
            .iter()
            .all(|item| item.picked_quantity >= item.ordered_quantity);

        // Update the delivery order status based on full pick
        delivery_order.status = if all_fully_picked {
            DeliveryOrderStatus::Picked
        } else {
            DeliveryOrderStatus::PartiallyPicked
        };
        delivery_order.updated_by = Some(user_id);
        delivery_order.updated_at = Utc::now();

        // Save the updated delivery order within transaction
        self.delivery_repo
            .update_with_tx(&mut tx, &delivery_order)
            .await?;

        // Commit the transaction
        tx.commit().await?;

        Ok(PickItemsResponse {
            delivery_id,
            status: delivery_order.status.to_string(),
            picked_items_count: updated_items_count,
            total_picked_quantity,
        })
    }

    async fn pack_items(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
        user_id: Uuid,
        request: PackItemsRequest,
    ) -> Result<PackItemsResponse, AppError> {
        // Begin transaction
        let mut tx = self.delivery_repo.begin_transaction().await?;

        // Find the delivery order within transaction
        let mut delivery_order = self
            .delivery_repo
            .find_by_id_with_tx(&mut tx, tenant_id, delivery_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Delivery order {} not found", delivery_id))
            })?;

        // Check if the delivery order is in a valid state for packing
        if delivery_order.status != DeliveryOrderStatus::Picked {
            return Err(AppError::ValidationError(format!(
                "Cannot pack items for delivery order with status '{}'. Only 'Picked' orders can be packed.",
                delivery_order.status
            )));
        }

        let packed_at = Utc::now();

        // Update the delivery order status to Packed
        delivery_order.status = DeliveryOrderStatus::Packed;
        delivery_order.updated_by = Some(user_id);
        delivery_order.updated_at = packed_at;
        if let Some(notes) = request.notes {
            delivery_order.notes = Some(notes);
        }

        // Save the updated delivery order within transaction
        self.delivery_repo
            .update_with_tx(&mut tx, &delivery_order)
            .await?;

        // Commit the transaction
        tx.commit().await?;

        Ok(PackItemsResponse {
            delivery_id,
            status: delivery_order.status.to_string(),
            packed_at,
        })
    }

    async fn ship_items(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
        user_id: Uuid,
        request: ShipItemsRequest,
    ) -> Result<ShipItemsResponse, AppError> {
        // Begin transaction
        let mut tx = self.delivery_repo.begin_transaction().await?;

        // Find the delivery order within transaction
        let mut delivery_order = self
            .delivery_repo
            .find_by_id_with_tx(&mut tx, tenant_id, delivery_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Delivery order {} not found", delivery_id))
            })?;

        // Check if the delivery order is in a valid state for shipping
        if delivery_order.status != DeliveryOrderStatus::Packed {
            return Err(AppError::ValidationError(format!(
                "Cannot ship items for delivery order with status '{}'. Only 'Packed' orders can be shipped.",
                delivery_order.status
            )));
        }

        // Get all delivery items
        let delivery_items = self
            .delivery_item_repo
            .find_by_delivery_id_with_tx(&mut tx, tenant_id, delivery_id)
            .await?;

        let shipped_at = Utc::now();
        let mut total_cogs = 0i64;
        let mut stock_moves_created = 0;

        // Process each delivery item
        for item in &delivery_items {
            // Skip items that weren't picked (shouldn't happen in Packed status, but safety check)
            if item.picked_quantity <= 0 {
                continue;
            }

            let picked_qty = item.picked_quantity;

            // Create stock move (warehouse -> customer virtual location)
            let idempotency_key = format!("do-{}-item-{}", delivery_id, item.delivery_item_id);

            // Validate inventory level exists (needed regardless for idempotency)
            self.inventory_level_repo
                .find_by_product(tenant_id, delivery_order.warehouse_id, item.product_id)
                .await?
                .ok_or_else(|| {
                    AppError::ValidationError(format!(
                        "No inventory level found for product {}",
                        item.product_id
                    ))
                })?;

            // For deliveries, we use the item's unit_price as COGS
            // In a real system, this might come from inventory valuation
            let unit_cost = Some(item.unit_price);
            let total_cost = unit_cost.flatten().map(|cost| cost * picked_qty);

            let stock_move = CreateStockMoveRequest {
                product_id: item.product_id,
                source_location_id: Some(delivery_order.warehouse_id), // From warehouse
                destination_location_id: None, // To customer (virtual location)
                move_type: "delivery".to_string(),
                quantity: -picked_qty, // Negative for outgoing
                unit_cost: unit_cost.flatten(),
                reference_type: "do".to_string(),
                reference_id: delivery_id,
                idempotency_key: idempotency_key.clone(),
                move_reason: Some(format!("Delivery order {}", delivery_order.delivery_number)),
                batch_info: None,
                metadata: Some(serde_json::json!({
                    "delivery_item_id": item.delivery_item_id,
                    "customer_id": delivery_order.customer_id
                })),
            };

            // Create stock move idempotently within transaction
            // Returns true if created, false if already existed (no-op)
            let created = self
                .stock_move_repo
                .create_idempotent_with_tx(&mut tx, &stock_move, tenant_id)
                .await?;

            if created {
                stock_moves_created += 1;

                // Update inventory level (decrement available stock)
                self.inventory_level_repo
                    .update_available_quantity_with_tx(
                        &mut tx,
                        tenant_id,
                        delivery_order.warehouse_id,
                        item.product_id,
                        -picked_qty,
                    )
                    .await?;

                // Accumulate COGS
                if let Some(cost) = total_cost {
                    total_cogs += cost;
                }
            }
        }

        // Update the delivery order status to Shipped
        delivery_order.status = DeliveryOrderStatus::Shipped;
        delivery_order.actual_ship_date = Some(shipped_at);
        delivery_order.updated_by = Some(user_id);
        delivery_order.updated_at = shipped_at;

        // Update shipping information if provided
        if let Some(tracking_number) = request.tracking_number {
            delivery_order.tracking_number = Some(tracking_number);
        }
        if let Some(carrier) = request.carrier {
            delivery_order.carrier = Some(carrier);
        }
        if let Some(shipping_cost) = request.shipping_cost {
            delivery_order.shipping_cost = Some(shipping_cost);
        }
        if let Some(notes) = request.notes {
            delivery_order.notes = Some(notes);
        }

        // Save the updated delivery order within transaction
        self.delivery_repo
            .update_with_tx(&mut tx, &delivery_order)
            .await?;

        // TODO: Publish inventory.delivery.completed event
        // This would typically be done via an event bus/message queue

        // Commit the transaction
        tx.commit().await?;

        Ok(ShipItemsResponse {
            delivery_id,
            status: delivery_order.status.to_string(),
            shipped_at,
            stock_moves_created,
            total_cogs,
        })
    }
}
