use std::sync::Arc;

use futures::stream::StreamExt;

use inventory_service_infra::repositories::{
    PgDeliveryOrderItemRepository, PgDeliveryOrderRepository, PgInventoryRepository,
};
use shared_error::AppError;
use shared_events::{EventEnvelope, OrderConfirmedEvent};
use uuid::Uuid;

pub async fn init_event_consumers(_pool: sqlx::PgPool, _nats_url: &str) -> Result<(), AppError> {
    // Delivery service is temporarily disabled - commenting out delivery event consumer
    // let delivery_repo = Arc::new(PgDeliveryOrderRepository::new(pool.clone()));
    // let delivery_item_repo = Arc::new(PgDeliveryOrderItemRepository::new(pool.clone()));
    // let inventory_repo = Arc::new(PgInventoryRepository::new(pool.clone()));

    // start_order_confirmed_consumer(delivery_repo, delivery_item_repo, inventory_repo, pool).await;
    // TODO: Re-enable NATS initialization when delivery consumer is re-enabled
    Ok(())
}

#[allow(dead_code)]
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

#[allow(dead_code)]
async fn handle_order_confirmed(
    event: EventEnvelope<OrderConfirmedEvent>,
    _delivery_repo: Arc<PgDeliveryOrderRepository>,
    _delivery_item_repo: Arc<PgDeliveryOrderItemRepository>,
    _inventory_repo: Arc<PgInventoryRepository>,
    _pool: &sqlx::PgPool,
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
    let _system_warehouse_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let _system_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

    // Delivery consumer is temporarily disabled
    tracing::warn!("Delivery consumer disabled - order {} not processed", order_data.order_id);

    Ok(())
}

#[allow(dead_code)]
async fn generate_delivery_number(
    _tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<String, AppError> {
    // Delivery functionality is disabled
    Err(AppError::ServiceUnavailable(
        "Delivery number generation is disabled. Enable with --features delivery".to_string(),
    ))
}
