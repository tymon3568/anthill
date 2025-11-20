//! NATS Event Consumers for Inventory Service
//!
//! This module handles event-driven operations for the inventory service,
//! specifically processing order confirmation events to create delivery orders.

use inventory_service_core::repositories::{DeliveryOrderRepository, InventoryRepository};
use inventory_service_infra::repositories::{PgDeliveryOrderRepository, PgInventoryRepository};
use shared_error::AppError;
use shared_events::{get_nats_client, init_nats_client, EventEnvelope, OrderConfirmedEvent};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Initialize NATS client and start event consumers
pub async fn init_event_consumers(pool: PgPool, nats_url: &str) -> Result<(), AppError> {
    // Initialize NATS client
    init_nats_client(nats_url).await?;

    // Get repositories
    let delivery_repo = Arc::new(PgDeliveryOrderRepository::new(pool.clone()));
    let inventory_repo = Arc::new(PgInventoryRepository::new(pool));

    // Start order confirmed consumer
    start_order_confirmed_consumer(delivery_repo, inventory_repo).await?;

    info!("Event consumers initialized successfully");
    Ok(())
}

/// Start consumer for order.confirmed events
async fn start_order_confirmed_consumer(
    delivery_repo: Arc<PgDeliveryOrderRepository>,
    inventory_repo: Arc<PgInventoryRepository>,
) -> Result<(), AppError> {
    let nats_client = get_nats_client().await?;

    nats_client
        .subscribe_event("order.confirmed", move |event: EventEnvelope<OrderConfirmedEvent>| {
            let delivery_repo = Arc::clone(&delivery_repo);
            let inventory_repo = Arc::clone(&inventory_repo);

            async move {
                if let Err(e) = handle_order_confirmed(event, delivery_repo, inventory_repo).await {
                    error!("Failed to handle order.confirmed event: {}", e);
                }
            }
        })
        .await?;

    info!("Subscribed to order.confirmed events");
    Ok(())
}

/// Handle order.confirmed event
async fn handle_order_confirmed(
    event: EventEnvelope<OrderConfirmedEvent>,
    delivery_repo: Arc<PgDeliveryOrderRepository>,
    inventory_repo: Arc<PgInventoryRepository>,
) -> Result<(), AppError> {
    let order_data = event.data;
    let tenant_id = order_data.tenant_id;

    info!(
        "Processing order.confirmed event for order {} in tenant {}",
        order_data.order_id, tenant_id
    );

    // Generate delivery order number
    let delivery_number = generate_delivery_number().await?;

    // Create delivery order
    let delivery_id = Uuid::now_v7();
    let delivery_order = inventory_service_core::models::DeliveryOrder {
        delivery_id,
        tenant_id,
        delivery_number,
        reference_number: Some(format!("ORDER-{}", order_data.order_id)),
        warehouse_id: Uuid::nil(), // TODO: Get from config or order
        order_id: Some(order_data.order_id),
        customer_id: order_data.customer_id,
        status: inventory_service_core::models::DeliveryOrderStatus::Reserved,
        delivery_date: chrono::Utc::now().date_naive(),
        expected_ship_date: order_data.expected_delivery_date.map(|dt| dt.date_naive()),
        actual_ship_date: None,
        shipping_method: None,
        carrier: None,
        tracking_number: None,
        shipping_cost: None,
        notes: order_data.notes,
        created_by: Uuid::nil(), // TODO: Get from system user
        total_quantity: order_data.items.iter().map(|item| item.quantity).sum(),
        total_value: order_data.items.iter().map(|item| item.line_total).sum(),
        currency_code: "VND".to_string(), // TODO: Get from config
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
    };

    // Save delivery order
    delivery_repo.create(&delivery_order).await?;
    info!("Created delivery order {} for order {}", delivery_number, order_data.order_id);

    // Create delivery order items and reserve stock
    for item in &order_data.items {
        let delivery_item_id = Uuid::now_v7();
        let delivery_item = inventory_service_core::models::DeliveryOrderItem {
            delivery_item_id,
            tenant_id,
            delivery_id,
            product_id: item.product_id,
            ordered_quantity: item.quantity,
            picked_quantity: 0,
            delivered_quantity: 0,
            unit_price: item.unit_price,
            line_total: item.line_total,
            batch_number: None,
            expiry_date: None,
            uom_id: None, // TODO: Get from product
            notes: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        };

        // TODO: Create delivery item repository method
        // delivery_item_repo.create(&delivery_item).await?;

        // Reserve stock in inventory_levels
        inventory_repo
            .reserve_stock(tenant_id, item.product_id, item.quantity)
            .await?;
        info!(
            "Reserved {} units of product {} for delivery order {}",
            item.quantity, item.product_id, delivery_number
        );
    }

    info!(
        "Successfully processed order.confirmed event for order {} - created DO {}",
        order_data.order_id, delivery_number
    );

    Ok(())
}

/// Generate delivery order number
async fn generate_delivery_number() -> Result<String, AppError> {
    // TODO: Implement proper sequence-based numbering
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    Ok(format!("DO-{}", timestamp))
}
