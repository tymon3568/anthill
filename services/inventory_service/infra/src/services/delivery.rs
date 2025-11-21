//! Delivery service implementation
//!
//! This module contains the business logic implementation for Delivery Order operations.

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::dto::delivery::{PickItemsRequest, PickItemsResponse};
use inventory_service_core::models::{DeliveryOrder, DeliveryOrderItem, DeliveryOrderStatus};
use inventory_service_core::repositories::{DeliveryOrderItemRepository, DeliveryOrderRepository};
use inventory_service_core::services::delivery::DeliveryService;
use shared_error::AppError;

/// PostgreSQL implementation of the delivery service
pub struct DeliveryServiceImpl {
    delivery_repo: Arc<dyn DeliveryOrderRepository>,
    delivery_item_repo: Arc<dyn DeliveryOrderItemRepository>,
}

impl DeliveryServiceImpl {
    /// Create a new delivery service with the given repositories
    pub fn new(
        delivery_repo: Arc<dyn DeliveryOrderRepository>,
        delivery_item_repo: Arc<dyn DeliveryOrderItemRepository>,
    ) -> Self {
        Self {
            delivery_repo,
            delivery_item_repo,
        }
    }
}

#[async_trait]
impl DeliveryService for PgDeliveryService {
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

        // Find the delivery order
        let mut delivery_order = self
            .delivery_repo
            .find_by_id(tenant_id, delivery_id)
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
            // Find the delivery item
            let mut delivery_item = self
                .delivery_item_repo
                .find_by_id(tenant_id, pick_item.delivery_item_id)
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

            if pick_item.picked_quantity
                > delivery_item.ordered_quantity - delivery_item.picked_quantity
            {
                return Err(AppError::ValidationError(format!(
                    "Cannot pick {} units for item {}. Only {} units remaining to pick.",
                    pick_item.picked_quantity,
                    pick_item.delivery_item_id,
                    delivery_item.ordered_quantity - delivery_item.picked_quantity
                )));
            }

            // Update the picked quantity
            delivery_item.picked_quantity += pick_item.picked_quantity;
            delivery_item.updated_at = chrono::Utc::now();

            // Save the updated item
            self.delivery_item_repo.update(&delivery_item).await?;

            total_picked_quantity += pick_item.picked_quantity;
            updated_items_count += 1;
        }

        // Update the delivery order status to Picked
        delivery_order.status = DeliveryOrderStatus::Picked;
        delivery_order.updated_at = chrono::Utc::now();

        self.delivery_repo.update(&delivery_order).await?;

        Ok(PickItemsResponse {
            delivery_id,
            status: delivery_order.status.to_string(),
            picked_items_count: updated_items_count,
            total_picked_quantity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_service_core::models::{DeliveryOrder, DeliveryOrderItem};
    use inventory_service_core::repositories::{
        MockDeliveryOrderItemRepository, MockDeliveryOrderRepository,
    };
    use std::sync::Arc;

    // TODO: Add comprehensive unit tests
    // - Test successful picking
    // - Test validation errors (wrong status, invalid quantities, etc.)
    // - Test not found errors
    // - Test transaction rollback on errors
}
