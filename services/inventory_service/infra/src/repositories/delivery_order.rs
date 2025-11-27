use async_trait::async_trait;
use sqlx::{PgPool, Transaction};
use std::sync::Arc;
use uuid::Uuid;

/// Helper type for infra-internal transaction operations
pub type InfraTx<'a> = &'a mut Transaction<'a, sqlx::Postgres>;

use inventory_service_core::models::{DeliveryOrder, DeliveryOrderItem, DeliveryOrderStatus};
use inventory_service_core::repositories::{
    DeliveryOrderItemRepository, DeliveryOrderRepository, InventoryRepository,
};
use shared_error::AppError;

pub struct PgDeliveryOrderRepository {
    pool: Arc<PgPool>,
}

impl PgDeliveryOrderRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeliveryOrderRepository for PgDeliveryOrderRepository {
    async fn create(&self, delivery_order: &DeliveryOrder) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO delivery_orders (
                delivery_id, tenant_id, delivery_number, reference_number,
                warehouse_id, order_id, customer_id,
                status, delivery_date, expected_ship_date, actual_ship_date,
                shipping_method, carrier, tracking_number, shipping_cost,
                notes, created_by, updated_by, total_quantity, total_value, currency_code,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23
            )
            "#,
            delivery_order.delivery_id,
            delivery_order.tenant_id,
            delivery_order.delivery_number,
            delivery_order.reference_number,
            delivery_order.warehouse_id,
            delivery_order.order_id,
            delivery_order.customer_id,
            delivery_order.status.to_string(),
            delivery_order.delivery_date,
            delivery_order.expected_ship_date,
            delivery_order.actual_ship_date,
            delivery_order.shipping_method,
            delivery_order.carrier,
            delivery_order.tracking_number,
            delivery_order.shipping_cost,
            delivery_order.notes,
            delivery_order.created_by,
            delivery_order.updated_by,
            delivery_order.total_quantity,
            delivery_order.total_value,
            delivery_order.currency_code,
            delivery_order.created_at,
            delivery_order.updated_at,
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<Option<DeliveryOrder>, AppError> {
        let result = sqlx::query_as!(
            DeliveryOrder,
            r#"
            SELECT
                delivery_id, tenant_id, delivery_number, reference_number,
                warehouse_id, order_id, customer_id,
                status as "status: _",
                delivery_date, expected_ship_date, actual_ship_date,
                shipping_method, carrier, tracking_number, shipping_cost,
                notes, created_by, updated_by, total_quantity, total_value, currency_code,
                created_at, updated_at, deleted_at
            FROM delivery_orders
            WHERE tenant_id = $1 AND delivery_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            delivery_id,
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn find_by_number(
        &self,
        tenant_id: Uuid,
        delivery_number: &str,
    ) -> Result<Option<DeliveryOrder>, AppError> {
        let result = sqlx::query_as!(
            DeliveryOrder,
            r#"
            SELECT
                delivery_id, tenant_id, delivery_number, reference_number,
                warehouse_id, order_id, customer_id,
                status as "status: _",
                delivery_date, expected_ship_date, actual_ship_date,
                shipping_method, carrier, tracking_number, shipping_cost,
                notes, created_by, updated_by, total_quantity, total_value, currency_code,
                created_at, updated_at, deleted_at
            FROM delivery_orders
            WHERE tenant_id = $1 AND delivery_number = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            delivery_number
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn list(
        &self,
        tenant_id: Uuid,
        warehouse_id: Option<Uuid>,
        status: Option<DeliveryOrderStatus>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<DeliveryOrder>, AppError> {
        let status_str = status.map(|s| s.to_string());
        let rows = sqlx::query_as!(
            DeliveryOrder,
            r#"
            SELECT
                delivery_id, tenant_id, delivery_number, reference_number,
                warehouse_id, order_id, customer_id,
                status as "status: _",
                delivery_date, expected_ship_date, actual_ship_date,
                shipping_method, carrier, tracking_number, shipping_cost,
                notes, created_by, updated_by, total_quantity, total_value, currency_code,
                created_at, updated_at, deleted_at
            FROM delivery_orders
            WHERE tenant_id = $1
              AND ($2::UUID IS NULL OR warehouse_id = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
            tenant_id,
            warehouse_id,
            status_str,
            limit.unwrap_or(50) as i64,
            offset.unwrap_or(0) as i64
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    async fn update(&self, delivery_order: &DeliveryOrder) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE delivery_orders SET
                delivery_number = $3, reference_number = $4,
                warehouse_id = $5, order_id = $6, customer_id = $7, status = $8,
                delivery_date = $9, expected_ship_date = $10, actual_ship_date = $11,
                shipping_method = $12, carrier = $13, tracking_number = $14, shipping_cost = $15,
                notes = $16, updated_by = $17, total_quantity = $18, total_value = $19, currency_code = $20,
                updated_at = $21
            WHERE tenant_id = $1 AND delivery_id = $2 AND deleted_at IS NULL
            "#,
            delivery_order.tenant_id,
            delivery_order.delivery_id,
            delivery_order.delivery_number,
            delivery_order.reference_number,
            delivery_order.warehouse_id,
            delivery_order.order_id,
            delivery_order.customer_id,
            delivery_order.status.to_string(),
            delivery_order.delivery_date,
            delivery_order.expected_ship_date,
            delivery_order.actual_ship_date,
            delivery_order.shipping_method,
            delivery_order.carrier,
            delivery_order.tracking_number,
            delivery_order.shipping_cost,
            delivery_order.notes,
            delivery_order.updated_by,
            delivery_order.total_quantity,
            delivery_order.total_value,
            delivery_order.currency_code,
            delivery_order.updated_at
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<DeliveryOrder>, AppError> {
        let rows = sqlx::query_as!(
            DeliveryOrder,
            r#"
            SELECT
                delivery_id, tenant_id, delivery_number, reference_number,
                warehouse_id, order_id, customer_id,
                status as "status: _",
                delivery_date, expected_ship_date, actual_ship_date,
                shipping_method, carrier, tracking_number, shipping_cost,
                notes, created_by, updated_by, total_quantity, total_value, currency_code,
                created_at, updated_at, deleted_at
            FROM delivery_orders
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            tenant_id,
            limit.unwrap_or(50),
            offset.unwrap_or(0)
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    async fn find_by_order_id(
        &self,
        tenant_id: Uuid,
        order_id: Uuid,
    ) -> Result<Option<DeliveryOrder>, AppError> {
        let result = sqlx::query_as!(
            DeliveryOrder,
            r#"
            SELECT
                delivery_id, tenant_id, delivery_number, reference_number,
                warehouse_id, order_id, customer_id,
                status as "status: _",
                delivery_date, expected_ship_date, actual_ship_date,
                shipping_method, carrier, tracking_number, shipping_cost,
                notes, created_by, updated_by, total_quantity, total_value, currency_code,
                created_at, updated_at, deleted_at
            FROM delivery_orders
            WHERE tenant_id = $1 AND order_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            order_id
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn delete(&self, tenant_id: Uuid, delivery_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE delivery_orders SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND delivery_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            delivery_id
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}

impl PgDeliveryOrderRepository {
    // Transaction-based methods for service layer
    pub async fn begin_transaction(
        &self,
    ) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, AppError> {
        let tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;
        Ok(tx)
    }

    pub async fn find_by_id_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<Option<DeliveryOrder>, AppError> {
        let result = sqlx::query_as!(
            DeliveryOrder,
            r#"
            SELECT
                delivery_id, tenant_id, delivery_number, reference_number,
                warehouse_id, order_id, customer_id,
                status as "status: _",
                delivery_date, expected_ship_date, actual_ship_date,
                shipping_method, carrier, tracking_number, shipping_cost,
                notes, created_by, updated_by, total_quantity, total_value, currency_code,
                created_at, updated_at, deleted_at
            FROM delivery_orders
            WHERE tenant_id = $1 AND delivery_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            delivery_id,
        )
        .fetch_optional(&mut **tx)
        .await?;
        Ok(result)
    }

    pub async fn update_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        delivery_order: &DeliveryOrder,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE delivery_orders SET
                delivery_number = $3, reference_number = $4,
                warehouse_id = $5, order_id = $6, customer_id = $7, status = $8,
                delivery_date = $9, expected_ship_date = $10, actual_ship_date = $11,
                shipping_method = $12, carrier = $13, tracking_number = $14, shipping_cost = $15,
                notes = $16, updated_by = $17, total_quantity = $18, total_value = $19, currency_code = $20,
                updated_at = $21
            WHERE tenant_id = $1 AND delivery_id = $2 AND deleted_at IS NULL
            "#,
            delivery_order.tenant_id,
            delivery_order.delivery_id,
            delivery_order.delivery_number,
            delivery_order.reference_number,
            delivery_order.warehouse_id,
            delivery_order.order_id,
            delivery_order.customer_id,
            delivery_order.status.to_string(),
            delivery_order.delivery_date,
            delivery_order.expected_ship_date,
            delivery_order.actual_ship_date,
            delivery_order.shipping_method,
            delivery_order.carrier,
            delivery_order.tracking_number,
            delivery_order.shipping_cost,
            delivery_order.notes,
            delivery_order.updated_by,
            delivery_order.total_quantity,
            delivery_order.total_value,
            delivery_order.currency_code,
            delivery_order.updated_at,
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}

pub struct PgDeliveryOrderItemRepository {
    pool: Arc<PgPool>,
}

impl PgDeliveryOrderItemRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeliveryOrderItemRepository for PgDeliveryOrderItemRepository {
    async fn create(&self, delivery_item: &DeliveryOrderItem) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO delivery_order_items (
                delivery_item_id, delivery_id, tenant_id, product_id,
                ordered_quantity, picked_quantity, delivered_quantity,
                uom_id, batch_number, expiry_date,
                unit_price, line_total, notes, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            )
            "#,
            delivery_item.delivery_item_id,
            delivery_item.delivery_id,
            delivery_item.tenant_id,
            delivery_item.product_id,
            delivery_item.ordered_quantity,
            delivery_item.picked_quantity,
            delivery_item.delivered_quantity,
            delivery_item.uom_id,
            delivery_item.batch_number,
            delivery_item.expiry_date,
            delivery_item.unit_price,
            delivery_item.line_total,
            delivery_item.notes,
            delivery_item.created_at,
            delivery_item.updated_at,
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        delivery_item_id: Uuid,
    ) -> Result<Option<DeliveryOrderItem>, AppError> {
        let result = sqlx::query_as!(
            DeliveryOrderItem,
            r#"
            SELECT
                delivery_item_id, delivery_id, tenant_id, product_id,
                ordered_quantity, picked_quantity, delivered_quantity,
                uom_id, batch_number, expiry_date,
                unit_price, line_total, notes, created_at, updated_at, deleted_at
            FROM delivery_order_items
            WHERE tenant_id = $1 AND delivery_item_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            delivery_item_id,
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn find_by_delivery_id(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<Vec<DeliveryOrderItem>, AppError> {
        let result = sqlx::query_as!(
            DeliveryOrderItem,
            r#"
            SELECT
                delivery_item_id, delivery_id, tenant_id, product_id,
                ordered_quantity, picked_quantity, delivered_quantity,
                uom_id, batch_number, expiry_date,
                unit_price, line_total, notes, created_at, updated_at, deleted_at
            FROM delivery_order_items
            WHERE tenant_id = $1 AND delivery_id = $2 AND deleted_at IS NULL
            ORDER BY created_at
            "#,
            tenant_id,
            delivery_id,
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn update(&self, delivery_item: &DeliveryOrderItem) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE delivery_order_items SET
                ordered_quantity = $4, picked_quantity = $5, delivered_quantity = $6,
                uom_id = $7, batch_number = $8, expiry_date = $9,
                unit_price = $10, line_total = $11, notes = $12, updated_at = $13
            WHERE tenant_id = $1 AND delivery_item_id = $2 AND delivery_id = $3 AND deleted_at IS NULL
            "#,
            delivery_item.tenant_id,
            delivery_item.delivery_item_id,
            delivery_item.delivery_id,
            delivery_item.ordered_quantity,
            delivery_item.picked_quantity,
            delivery_item.delivered_quantity,
            delivery_item.uom_id,
            delivery_item.batch_number,
            delivery_item.expiry_date,
            delivery_item.unit_price,
            delivery_item.line_total,
            delivery_item.notes,
            delivery_item.updated_at,
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, tenant_id: Uuid, delivery_item_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE delivery_order_items SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND delivery_item_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            delivery_item_id,
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}

impl PgDeliveryOrderItemRepository {
    // Transaction-based methods for service layer
    pub async fn find_by_id_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        tenant_id: Uuid,
        delivery_item_id: Uuid,
    ) -> Result<Option<DeliveryOrderItem>, AppError> {
        let result = sqlx::query_as!(
            DeliveryOrderItem,
            r#"
            SELECT
                delivery_item_id, delivery_id, tenant_id, product_id,
                ordered_quantity, picked_quantity, delivered_quantity,
                uom_id, batch_number, expiry_date,
                unit_price, line_total, notes, created_at, updated_at, deleted_at
            FROM delivery_order_items
            WHERE tenant_id = $1 AND delivery_item_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            delivery_item_id,
        )
        .fetch_optional(&mut **tx)
        .await?;
        Ok(result)
    }

    pub async fn find_by_delivery_id_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<Vec<DeliveryOrderItem>, AppError> {
        let result = sqlx::query_as!(
            DeliveryOrderItem,
            r#"
            SELECT
                delivery_item_id, delivery_id, tenant_id, product_id,
                ordered_quantity, picked_quantity, delivered_quantity,
                uom_id, batch_number, expiry_date,
                unit_price, line_total, notes, created_at, updated_at, deleted_at
            FROM delivery_order_items
            WHERE tenant_id = $1 AND delivery_id = $2 AND deleted_at IS NULL
            ORDER BY created_at
            "#,
            tenant_id,
            delivery_id,
        )
        .fetch_all(&mut **tx)
        .await?;
        Ok(result)
    }

    pub async fn update_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        delivery_item: &DeliveryOrderItem,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE delivery_order_items SET
                ordered_quantity = $4, picked_quantity = $5, delivered_quantity = $6,
                uom_id = $7, batch_number = $8, expiry_date = $9,
                unit_price = $10, line_total = $11, notes = $12, updated_at = $13
            WHERE tenant_id = $1 AND delivery_item_id = $2 AND delivery_id = $3 AND deleted_at IS NULL
            "#,
            delivery_item.tenant_id,
            delivery_item.delivery_item_id,
            delivery_item.delivery_id,
            delivery_item.ordered_quantity,
            delivery_item.picked_quantity,
            delivery_item.delivered_quantity,
            delivery_item.uom_id,
            delivery_item.batch_number,
            delivery_item.expiry_date,
            delivery_item.unit_price,
            delivery_item.line_total,
            delivery_item.notes,
            delivery_item.updated_at,
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}

// sqlx implementations for DeliveryOrderStatus (moved from core to avoid infra deps)
pub struct PgInventoryRepository {
    pool: Arc<PgPool>,
}

impl PgInventoryRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InventoryRepository for PgInventoryRepository {
    async fn reserve_stock(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        quantity: i64,
    ) -> Result<(), AppError> {
        if quantity <= 0 {
            return Err(AppError::ValidationError(
                "Quantity to reserve must be positive".to_string(),
            ));
        }

        let res = sqlx::query!(
            r#"
            UPDATE inventory_levels
            SET available_quantity = available_quantity - $4,
                reserved_quantity = reserved_quantity + $4,
                updated_at = NOW()
            WHERE tenant_id = $1 AND product_id = $2 AND warehouse_id = $3
              AND available_quantity >= $4
              AND deleted_at IS NULL
            "#,
            tenant_id,
            product_id,
            warehouse_id,
            quantity,
        )
        .execute(&*self.pool)
        .await?;

        if res.rows_affected() == 0 {
            return Err(AppError::ValidationError(
                "Insufficient stock available for reservation".to_string(),
            ));
        }

        Ok(())
    }

    async fn release_stock(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        quantity: i64,
    ) -> Result<(), AppError> {
        if quantity <= 0 {
            return Err(AppError::ValidationError(
                "Quantity to release must be positive".to_string(),
            ));
        }

        let res = sqlx::query!(
            r#"
            UPDATE inventory_levels
            SET available_quantity = available_quantity + $4,
                reserved_quantity = reserved_quantity - $4,
                updated_at = NOW()
            WHERE tenant_id = $1 AND product_id = $2 AND warehouse_id = $3
              AND reserved_quantity >= $4
              AND deleted_at IS NULL
            "#,
            tenant_id,
            product_id,
            warehouse_id,
            quantity,
        )
        .execute(&*self.pool)
        .await?;

        if res.rows_affected() == 0 {
            return Err(AppError::ValidationError(
                "Insufficient reserved stock to release".to_string(),
            ));
        }

        Ok(())
    }

    async fn get_available_stock(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<i64, AppError> {
        let result = sqlx::query!(
            r#"
            SELECT available_quantity
            FROM inventory_levels
            WHERE tenant_id = $1 AND product_id = $2 AND warehouse_id = $3 AND deleted_at IS NULL
            "#,
            tenant_id,
            product_id,
            warehouse_id,
        )
        .fetch_optional(&*self.pool)
        .await?;

        match result {
            Some(row) => Ok(row.available_quantity),
            None => Ok(0), // No inventory record means 0 available
        }
    }
}

// sqlx implementations for DeliveryOrderStatus (moved from core to avoid infra deps)
// TODO: Move these impls to core crate or use newtype pattern to avoid orphan rules
/*
impl sqlx::Type<sqlx::Postgres> for inventory_service_core::models::DeliveryOrderStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for inventory_service_core::models::DeliveryOrderStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s.as_str() {
            "draft" => Ok(inventory_service_core::models::DeliveryOrderStatus::Draft),
            "confirmed" => Ok(inventory_service_core::models::DeliveryOrderStatus::Confirmed),
            "partially_picked" => {
                Ok(inventory_service_core::models::DeliveryOrderStatus::PartiallyPicked)
            },
            "picked" => Ok(inventory_service_core::models::DeliveryOrderStatus::Picked),
            "partially_shipped" => {
                Ok(inventory_service_core::models::DeliveryOrderStatus::PartiallyShipped)
            },
            "shipped" => Ok(inventory_service_core::models::DeliveryOrderStatus::Shipped),
            "cancelled" => Ok(inventory_service_core::models::DeliveryOrderStatus::Cancelled),
            _ => Err(format!("Unknown DeliveryOrderStatus: {}", s).into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for inventory_service_core::models::DeliveryOrderStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        <String as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.to_string(), buf)
    }
}
*/
