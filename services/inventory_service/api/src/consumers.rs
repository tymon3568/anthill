use std::sync::Arc;

use inventory_service_core::models::{DeliveryOrder, DeliveryOrderItem, DeliveryOrderStatus};
use inventory_service_core::repositories::{
    DeliveryOrderItemRepository, DeliveryOrderRepository, InventoryRepository,
};
use inventory_service_infra::repositories::{
    PgDeliveryOrderItemRepository, PgDeliveryOrderRepository, PgInventoryRepository,
};
use shared_error::AppError;
use shared_events::{EventEnvelope, OrderConfirmedEvent};
use uuid::Uuid;

pub async fn init_event_consumers(pool: sqlx::PgPool, nats_url: &str) -> Result<(), AppError> {
    let delivery_repo = Arc::new(PgDeliveryOrderRepository::new(pool.clone()));
    let delivery_item_repo = Arc::new(PgDeliveryOrderItemRepository::new(pool.clone()));
    let inventory_repo = Arc::new(PgInventoryRepository::new(pool.clone()));

    start_order_confirmed_consumer(delivery_repo, delivery_item_repo, inventory_repo, nats_url)
        .await
}

async fn start_order_confirmed_consumer(
    delivery_repo: Arc<PgDeliveryOrderRepository>,
    delivery_item_repo: Arc<PgDeliveryOrderItemRepository>,
    inventory_repo: Arc<PgInventoryRepository>,
    nats_url: &str,
) -> Result<(), AppError> {
    let client = shared_events::init_nats_client(nats_url).await?;
    let subscriber = shared_events::subscribe_event::<OrderConfirmedEvent>(
        &client,
        "order.confirmed".to_string(),
    )
    .await?;

    tokio::spawn(async move {
        while let Some(message) = subscriber.next().await {
            match message {
                Ok(event) => {
                    if let Err(e) = handle_order_confirmed(
                        event,
                        delivery_repo.clone(),
                        delivery_item_repo.clone(),
                        inventory_repo.clone(),
                    )
                    .await
                    {
                        tracing::error!("Failed to handle order.confirmed event: {}", e);
                    }
                },
                Err(e) => {
                    tracing::error!("Error receiving order.confirmed event: {}", e);
                },
            }
        }
    });

    Ok(())
}

async fn handle_order_confirmed(
    event: EventEnvelope<OrderConfirmedEvent>,
    delivery_repo: Arc<PgDeliveryOrderRepository>,
    delivery_item_repo: Arc<PgDeliveryOrderItemRepository>,
    inventory_repo: Arc<PgInventoryRepository>,
) -> Result<(), AppError> {
    let order_data = event.data;
    let tenant_id = order_data.tenant_id;

    info!(
        "Processing order.confirmed event for order {} in tenant {}",
        order_data.order_id, tenant_id
    );

    // Generate delivery order number using DB function
    let delivery_number = generate_delivery_number().await?;

    // System warehouse and user IDs (TODO: get from config)
    let system_warehouse_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let system_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

    // Create delivery order
    let delivery_id = Uuid::now_v7();
    let delivery_order = inventory_service_core::models::DeliveryOrder {
        delivery_id,
        tenant_id,
        delivery_number,
        reference_number: Some(format!("ORDER-{}", order_data.order_id)),
        warehouse_id: system_warehouse_id,
        order_id: Some(order_data.order_id),
        customer_id: order_data.customer_id,
        status: inventory_service_core::models::DeliveryOrderStatus::Confirmed,
        delivery_date: chrono::Utc::now(),
        expected_ship_date: order_data.expected_delivery_date,
        actual_ship_date: None,
        shipping_method: None,
        carrier: None,
        tracking_number: None,
        shipping_cost: None,
        notes: order_data.notes,
        created_by: system_user_id,
        total_quantity: order_data
            .items
            .iter()
            .map(|item| item.quantity as i64)
            .sum(),
        total_value: order_data.items.iter().map(|item| item.line_total).sum(),
        currency_code: "VND".to_string(), // TODO: Get from config
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
    };

    // Save delivery order
    delivery_repo.create(&delivery_order).await?;

    // Process items and reserve stock
    for item in &order_data.items {
        let delivery_item_id = Uuid::now_v7();
        let delivery_item = inventory_service_core::models::DeliveryOrderItem {
            delivery_item_id,
            delivery_id,
            tenant_id,
            product_id: item.product_id,
            ordered_quantity: item.quantity as i64,
            picked_quantity: 0,
            delivered_quantity: 0,
            unit_price: item.unit_price,
            line_total: item.line_total,
            notes: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        };

        // Persist delivery item
        delivery_item_repo.create(&delivery_item).await?;

        // Reserve stock
        inventory_repo
            .reserve_stock(tenant_id, item.product_id, item.quantity as i64)
            .await?;
    }

    info!(
        "Successfully created delivery order {} for order {}",
        delivery_number, order_data.order_id
    );

    Ok(())
}

async fn generate_delivery_number() -> Result<String, AppError> {
    // TODO: Call DB function generate_delivery_number()
    // For now, use timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    Ok(format!("DO-{}", timestamp))
}
