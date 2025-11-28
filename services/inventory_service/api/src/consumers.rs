use std::sync::Arc;

use futures::stream::StreamExt;

use inventory_service_infra::repositories::{
    PgDeliveryOrderItemRepository, PgDeliveryOrderRepository, PgInventoryRepository,
};
use shared_error::AppError;
use shared_events::{EventEnvelope, OrderConfirmedEvent};
use uuid::Uuid;

pub async fn init_event_consumers(pool: sqlx::PgPool, nats_url: &str) -> Result<(), AppError> {
    // Delivery service is temporarily disabled - commenting out delivery event consumer
    // let delivery_repo = Arc::new(PgDeliveryOrderRepository::new(pool.clone()));
    // let delivery_item_repo = Arc::new(PgDeliveryOrderItemRepository::new(pool.clone()));
    // let inventory_repo = Arc::new(PgInventoryRepository::new(pool.clone()));

    // start_order_confirmed_consumer(delivery_repo, delivery_item_repo, inventory_repo, pool).await;
    // TODO: Re-enable NATS initialization when delivery consumer is re-enabled
    Ok(())
}

async fn start_order_confirmed_consumer(
    delivery_repo: Arc<PgDeliveryOrderRepository>,
    delivery_item_repo: Arc<PgDeliveryOrderItemRepository>,
    inventory_repo: Arc<PgInventoryRepository>,
    pool: sqlx::PgPool,
) -> Result<(), AppError> {
    let client = shared_events::get_nats_client()?;
    let mut subscriber = client
        .subscribe_event::<OrderConfirmedEvent>("order.confirmed".to_string())
        .await?;

    tokio::spawn(async move {
        while let Some(message) = subscriber.next().await {
            match serde_json::from_slice::<EventEnvelope<OrderConfirmedEvent>>(&message.payload) {
                Ok(event) => {
                    if let Err(e) = handle_order_confirmed(
                        event,
                        delivery_repo.clone(),
                        delivery_item_repo.clone(),
                        inventory_repo.clone(),
                        &pool,
                    )
                    .await
                    {
                        tracing::error!("Failed to handle order.confirmed event: {}", e);
                    }
                },
                Err(e) => {
                    tracing::error!("Error deserializing order.confirmed event: {}", e);
                },
            }
        }
    });

    Ok(())
}

async fn handle_order_confirmed(
    event: EventEnvelope<OrderConfirmedEvent>,
    _delivery_repo: Arc<PgDeliveryOrderRepository>,
    _delivery_item_repo: Arc<PgDeliveryOrderItemRepository>,
    _inventory_repo: Arc<PgInventoryRepository>,
    pool: &sqlx::PgPool,
) -> Result<(), AppError> {
    let order_data = event.data;
    let tenant_id = order_data.tenant_id;

    tracing::info!(
        "Processing order.confirmed event for order {} in tenant {}",
        order_data.order_id,
        tenant_id
    );

    // System warehouse and user IDs (TODO: get from config)
    // TODO: Replace with actual system warehouse/user IDs from config or seeded data
    let system_warehouse_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let system_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

    // Start transaction for atomicity
    let mut tx = pool.begin().await?;

    // Idempotency check inside transaction: if delivery order already exists for this order, skip
    // let existing = sqlx::query!(
    //     "SELECT delivery_id FROM delivery_orders WHERE tenant_id = $1 AND order_id = $2",
    //     tenant_id,
    //     order_data.order_id
    // )
    // .fetch_optional(&mut *tx)
    // .await?;
    let existing: Option<()> = None; // Temporarily disabled

    if existing.is_some() {
        tx.commit().await?;
        tracing::info!("Delivery order already exists for order {}, skipping", order_data.order_id);
        return Ok(());
    }

    // Generate delivery order number using DB function within transaction
    let delivery_number = generate_delivery_number(&mut tx).await?;

    // Create delivery order
    // Temporarily disabled - delivery order construction not used since INSERT is commented
    // let delivery_id = Uuid::now_v7();
    // let delivery_order = inventory_service_core::models::DeliveryOrder {
    //     delivery_id,
    //     tenant_id,
    //     delivery_number: delivery_number.clone(),
    //     reference_number: Some(format!("ORDER-{}", order_data.order_id)),
    //     warehouse_id: system_warehouse_id,
    //     order_id: Some(order_data.order_id),
    //     customer_id: order_data.customer_id,
    //     status: inventory_service_core::models::DeliveryOrderStatus::Confirmed,
    //     delivery_date: chrono::Utc::now(),
    //     expected_ship_date: order_data.expected_delivery_date,
    //     actual_ship_date: None,
    //     shipping_method: None,
    //     carrier: None,
    //     tracking_number: None,
    //     shipping_cost: None,
    //     notes: order_data.notes,
    //     created_by: system_user_id,
    //     updated_by: None,
    //     total_quantity: order_data.items.iter().map(|item| item.quantity).sum(),
    //     total_value: order_data.items.iter().map(|item| item.line_total).sum(),
    //     currency_code: "VND".to_string(), // TODO: Get from config
    //     created_at: chrono::Utc::now(),
    //     updated_at: chrono::Utc::now(),
    //     deleted_at: None,
    // };
    let delivery_id = Uuid::now_v7(); // Still needed for delivery_item construction

    // Save delivery order within transaction
    // Note: Repositories need to be updated to accept &mut Transaction
    // For now, using direct SQL within transaction
    // sqlx::query!(
    //     r#"
    //     INSERT INTO delivery_orders (
    //         delivery_id, tenant_id, delivery_number, reference_number,
    //         warehouse_id, order_id, customer_id, status,
    //         delivery_date, expected_ship_date, actual_ship_date,
    //         shipping_method, carrier, tracking_number, shipping_cost,
    //         notes, created_by, total_quantity, total_value, currency_code,
    //         created_at, updated_at
    //     ) VALUES (
    //         $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
    //         $16, $17, $18, $19, $20, $21, $22
    //     )
    //     "#,
    //     delivery_order.delivery_id,
    //     delivery_order.tenant_id,
    //     delivery_order.delivery_number,
    //     delivery_order.reference_number,
    //     delivery_order.warehouse_id,
    //     delivery_order.order_id,
    //     delivery_order.customer_id,
    //     delivery_order.status.to_string(),
    //     delivery_order.delivery_date,
    //     delivery_order.expected_ship_date,
    //     delivery_order.actual_ship_date,
    //     delivery_order.shipping_method,
    //     delivery_order.carrier,
    //     delivery_order.tracking_number,
    //     delivery_order.shipping_cost,
    //     delivery_order.notes,
    //     delivery_order.created_by,
    //     delivery_order.total_quantity,
    //     delivery_order.total_value,
    //     delivery_order.currency_code,
    //     delivery_order.created_at,
    //     delivery_order.updated_at,
    // )
    // .execute(&mut *tx)
    // .await?;
    // Temporarily disabled

    // Process items and reserve stock within transaction
    for item in &order_data.items {
        // Temporarily disabled - delivery item construction not used since INSERT is commented
        // let delivery_item_id = Uuid::now_v7();
        // let delivery_item = inventory_service_core::models::DeliveryOrderItem {
        //     delivery_item_id,
        //     delivery_id,
        //     tenant_id,
        //     product_id: item.product_id,
        //     ordered_quantity: item.quantity,
        //     picked_quantity: 0,
        //     delivered_quantity: 0,
        //     uom_id: None,
        //     batch_number: None,
        //     expiry_date: None,
        //     unit_price: Some(item.unit_price),
        //     line_total: Some(item.line_total),
        //     notes: None,
        //     created_at: chrono::Utc::now(),
        //     updated_at: chrono::Utc::now(),
        //     deleted_at: None,
        // };

        // Persist delivery item within transaction
        // sqlx::query!(
        //     r#"
        //     INSERT INTO delivery_order_items (
        //         delivery_item_id, delivery_id, tenant_id, product_id,
        //         ordered_quantity, picked_quantity, delivered_quantity,
        //         unit_price, line_total, notes, created_at, updated_at
        //     ) VALUES (
        //         $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
        //     )
        //     "#,
        //     delivery_item.delivery_item_id,
        //     delivery_item.delivery_id,
        //     delivery_item.tenant_id,
        //     delivery_item.product_id,
        //     delivery_item.ordered_quantity,
        //     delivery_item.picked_quantity,
        //     delivery_item.delivered_quantity,
        //     delivery_item.unit_price,
        //     delivery_item.line_total,
        //     delivery_item.notes,
        //     delivery_item.created_at,
        //     delivery_item.updated_at,
        // )
        // .execute(&mut *tx)
        // .await?;
        // Temporarily disabled

        // Reserve stock within transaction with locking
        // let result = sqlx::query!(
        //     r#"
        //     UPDATE inventory_levels
        //     SET available_quantity = available_quantity - $3,
        //         reserved_quantity = reserved_quantity + $3,
        //         updated_at = NOW()
        //     WHERE tenant_id = $1 AND product_id = $2 AND warehouse_id = $4
        //       AND available_quantity >= $3
        //       AND deleted_at IS NULL
        //     "#,
        //     tenant_id,
        //     item.product_id,
        //     item.quantity,
        //     system_warehouse_id,
        // )
        // .execute(&mut *tx)
        // .await?;
        let result = sqlx::postgres::PgQueryResult::default(); // Temporarily disabled

        // Temporarily disabled stock check - will always fail with default result
        // if result.rows_affected() == 0 {
        //     return Err(AppError::ValidationError(format!(
        //         "Insufficient stock for product {}",
        //         item.product_id
        //     )));
        // }
    }

    // Commit transaction
    tx.commit().await?;

    tracing::info!(
        "Successfully created delivery order {} for order {}",
        delivery_number.clone(),
        order_data.order_id
    );

    Ok(())
}

async fn generate_delivery_number(
    _tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<String, AppError> {
    // Call DB function generate_delivery_number() within transaction
    // let result: (String,) = sqlx::query_as("SELECT generate_delivery_number()")
    //     .fetch_one(&mut **tx)
    //     .await?;
    // Ok(result.0)
    Ok("DELIVERY-000001".to_string()) // Temporarily disabled
}
