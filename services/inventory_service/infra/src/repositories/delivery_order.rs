use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use inventory_service_core::models::{DeliveryOrder, DeliveryOrderItem};
use inventory_service_core::repositories::{
    DeliveryOrderItemRepository, DeliveryOrderRepository, InventoryRepository,
};
use shared_error::AppError;

pub struct PgDeliveryOrderRepository {
    pool: PgPool,
}

impl PgDeliveryOrderRepository {
    pub fn new(pool: PgPool) -> Self {
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
                warehouse_id, order_id, customer_id, status,
                delivery_date, expected_ship_date, actual_ship_date,
                shipping_method, carrier, tracking_number, shipping_cost,
                notes, created_by, total_quantity, total_value, currency_code,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20, $21, $22
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
            delivery_order.total_quantity,
            delivery_order.total_value,
            delivery_order.currency_code,
            delivery_order.created_at,
            delivery_order.updated_at,
        )
        .execute(&self.pool)
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
                notes, created_by, total_quantity, total_value, currency_code,
                created_at, updated_at, deleted_at
            FROM delivery_orders
            WHERE tenant_id = $1 AND delivery_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            delivery_id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<DeliveryOrder>, AppError> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        let result = sqlx::query_as!(
            DeliveryOrder,
            r#"
            SELECT
                delivery_id, tenant_id, delivery_number, reference_number,
                warehouse_id, order_id, customer_id,
                status as "status: _",
                delivery_date, expected_ship_date, actual_ship_date,
                shipping_method, carrier, tracking_number, shipping_cost,
                notes, created_by, total_quantity, total_value, currency_code,
                created_at, updated_at, deleted_at
            FROM delivery_orders
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            tenant_id,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn update(&self, delivery_order: &DeliveryOrder) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE delivery_orders SET
                delivery_number = $3, reference_number = $4,
                warehouse_id = $5, order_id = $6, customer_id = $7, status = $8,
                delivery_date = $9, expected_ship_date = $10, actual_ship_date = $11,
                shipping_method = $12, carrier = $13, tracking_number = $14, shipping_cost = $15,
                notes = $16, total_quantity = $17, total_value = $18, currency_code = $19,
                updated_at = $20
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
            delivery_order.total_quantity,
            delivery_order.total_value,
            delivery_order.currency_code,
            delivery_order.updated_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, tenant_id: Uuid, delivery_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE delivery_orders SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND delivery_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            delivery_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
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
                notes, created_by, total_quantity, total_value, currency_code,
                created_at, updated_at, deleted_at
            FROM delivery_orders
            WHERE tenant_id = $1 AND order_id = $2 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
            tenant_id,
            order_id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }
}

pub struct PgDeliveryOrderItemRepository {
    pool: PgPool,
}

impl PgDeliveryOrderItemRepository {
    pub fn new(pool: PgPool) -> Self {
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
                unit_price, line_total, notes, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
            )
            "#,
            delivery_item.delivery_item_id,
            delivery_item.delivery_id,
            delivery_item.tenant_id,
            delivery_item.product_id,
            delivery_item.ordered_quantity,
            delivery_item.picked_quantity,
            delivery_item.delivered_quantity,
            delivery_item.unit_price,
            delivery_item.line_total,
            delivery_item.notes,
            delivery_item.created_at,
            delivery_item.updated_at,
        )
        .execute(&self.pool)
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
                unit_price, line_total, notes, created_at, updated_at, deleted_at
            FROM delivery_order_items
            WHERE tenant_id = $1 AND delivery_item_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            delivery_item_id,
        )
        .fetch_optional(&self.pool)
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
                unit_price, line_total, notes, created_at, updated_at, deleted_at
            FROM delivery_order_items
            WHERE tenant_id = $1 AND delivery_id = $2 AND deleted_at IS NULL
            ORDER BY created_at
            "#,
            tenant_id,
            delivery_id,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn update(&self, delivery_item: &DeliveryOrderItem) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE delivery_order_items SET
                ordered_quantity = $4, picked_quantity = $5, delivered_quantity = $6,
                unit_price = $7, line_total = $8, notes = $9, updated_at = $10
            WHERE tenant_id = $1 AND delivery_item_id = $2 AND delivery_id = $3 AND deleted_at IS NULL
            "#,
            delivery_item.tenant_id,
            delivery_item.delivery_item_id,
            delivery_item.delivery_id,
            delivery_item.ordered_quantity,
            delivery_item.picked_quantity,
            delivery_item.delivered_quantity,
            delivery_item.unit_price,
            delivery_item.line_total,
            delivery_item.notes,
            delivery_item.updated_at,
        )
        .execute(&self.pool)
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
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub struct PgInventoryRepository {
    pool: PgPool,
}

impl PgInventoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InventoryRepository for PgInventoryRepository {
    async fn reserve_stock(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity: i64,
    ) -> Result<(), AppError> {
        let res = sqlx::query!(
            r#"
            UPDATE inventory_levels
            SET available_quantity = available_quantity - $3,
                reserved_quantity = reserved_quantity + $3,
                updated_at = NOW()
            WHERE tenant_id = $1 AND product_id = $2
              AND available_quantity >= $3
              AND deleted_at IS NULL
            "#,
            tenant_id,
            product_id,
            quantity,
        )
        .execute(&self.pool)
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
        product_id: Uuid,
        quantity: i64,
    ) -> Result<(), AppError> {
        let res = sqlx::query!(
            r#"
            UPDATE inventory_levels
            SET available_quantity = available_quantity + $3,
                reserved_quantity = reserved_quantity - $3,
                updated_at = NOW()
            WHERE tenant_id = $1 AND product_id = $2
              AND reserved_quantity >= $3
              AND deleted_at IS NULL
            "#,
            tenant_id,
            product_id,
            quantity,
        )
        .execute(&self.pool)
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
        product_id: Uuid,
    ) -> Result<i64, AppError> {
        let result = sqlx::query!(
            r#"
            SELECT available_quantity
            FROM inventory_levels
            WHERE tenant_id = $1 AND product_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            product_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(row) => Ok(row.available_quantity),
            None => Ok(0), // No inventory record means 0 available
        }
    }
}
