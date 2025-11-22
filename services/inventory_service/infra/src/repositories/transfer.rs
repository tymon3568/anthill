use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::domains::inventory::transfer::{
    Transfer, TransferItem, TransferPriority, TransferStatus, TransferType,
};
use inventory_service_core::repositories::transfer::{TransferItemRepository, TransferRepository};
use shared_error::AppError;

/// PostgreSQL implementation of TransferRepository
pub struct PgTransferRepository {
    pool: Arc<PgPool>,
}

impl PgTransferRepository {
    /// Create a new repository instance
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransferRepository for PgTransferRepository {
    async fn create(&self, tenant_id: Uuid, transfer: &Transfer) -> Result<Transfer, AppError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO stock_transfers (
                transfer_id, tenant_id, transfer_number, reference_number,
                source_warehouse_id, destination_warehouse_id, status, transfer_type, priority,
                transfer_date, expected_ship_date, expected_receive_date,
                shipping_method, notes, reason, created_by, updated_by,
                total_quantity, total_value, currency_code
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            RETURNING transfer_id, tenant_id, transfer_number, reference_number,
                      source_warehouse_id, destination_warehouse_id, status, transfer_type, priority,
                      transfer_date, expected_ship_date, actual_ship_date,
                      expected_receive_date, actual_receive_date,
                      shipping_method, carrier, tracking_number, shipping_cost,
                      notes, reason, created_by, updated_by, approved_by, approved_at,
                      total_quantity, total_value, currency_code,
                      created_at, updated_at, deleted_at
            "#,
            transfer.transfer_id,
            tenant_id,
            transfer.transfer_number,
            transfer.reference_number,
            transfer.source_warehouse_id,
            transfer.destination_warehouse_id,
            transfer.status as TransferStatus,
            transfer.transfer_type as TransferType,
            transfer.priority as TransferPriority,
            transfer.transfer_date,
            transfer.expected_ship_date,
            transfer.expected_receive_date,
            transfer.shipping_method,
            transfer.notes,
            transfer.reason,
            transfer.created_by,
            transfer.updated_by,
            transfer.total_quantity,
            transfer.total_value,
            transfer.currency_code
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create transfer: {}", e)))?;

        Ok(Transfer {
            transfer_id: row.transfer_id,
            tenant_id: row.tenant_id,
            transfer_number: row.transfer_number,
            reference_number: row.reference_number,
            source_warehouse_id: row.source_warehouse_id,
            destination_warehouse_id: row.destination_warehouse_id,
            status: row.status,
            transfer_type: row.transfer_type,
            priority: row.priority,
            transfer_date: row.transfer_date,
            expected_ship_date: row.expected_ship_date,
            actual_ship_date: row.actual_ship_date,
            expected_receive_date: row.expected_receive_date,
            actual_receive_date: row.actual_receive_date,
            shipping_method: row.shipping_method,
            carrier: row.carrier,
            tracking_number: row.tracking_number,
            shipping_cost: row.shipping_cost,
            notes: row.notes,
            reason: row.reason,
            created_by: row.created_by,
            updated_by: row.updated_by,
            approved_by: row.approved_by,
            approved_at: row.approved_at,
            total_quantity: row.total_quantity,
            total_value: row.total_value,
            currency_code: row.currency_code,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        })
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
    ) -> Result<Option<Transfer>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT transfer_id, tenant_id, transfer_number, reference_number,
                   source_warehouse_id, destination_warehouse_id, status, transfer_type, priority,
                   transfer_date, expected_ship_date, actual_ship_date,
                   expected_receive_date, actual_receive_date,
                   shipping_method, carrier, tracking_number, shipping_cost,
                   notes, reason, created_by, updated_by, approved_by, approved_at,
                   total_quantity, total_value, currency_code,
                   created_at, updated_at, deleted_at
            FROM stock_transfers
            WHERE tenant_id = $1 AND transfer_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            transfer_id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find transfer: {}", e)))?;

        Ok(row.map(|r| Transfer {
            transfer_id: r.transfer_id,
            tenant_id: r.tenant_id,
            transfer_number: r.transfer_number,
            reference_number: r.reference_number,
            source_warehouse_id: r.source_warehouse_id,
            destination_warehouse_id: r.destination_warehouse_id,
            status: r.status,
            transfer_type: r.transfer_type,
            priority: r.priority,
            transfer_date: r.transfer_date,
            expected_ship_date: r.expected_ship_date,
            actual_ship_date: r.actual_ship_date,
            expected_receive_date: r.expected_receive_date,
            actual_receive_date: r.actual_receive_date,
            shipping_method: r.shipping_method,
            carrier: r.carrier,
            tracking_number: r.tracking_number,
            shipping_cost: r.shipping_cost,
            notes: r.notes,
            reason: r.reason,
            created_by: r.created_by,
            updated_by: r.updated_by,
            approved_by: r.approved_by,
            approved_at: r.approved_at,
            total_quantity: r.total_quantity,
            total_value: r.total_value,
            currency_code: r.currency_code,
            created_at: r.created_at,
            updated_at: r.updated_at,
            deleted_at: r.deleted_at,
        }))
    }

    async fn update_status(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        status: TransferStatus,
        updated_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_transfers
            SET status = $1, updated_by = $2, updated_at = NOW()
            WHERE tenant_id = $3 AND transfer_id = $4 AND deleted_at IS NULL
            "#,
            status as TransferStatus,
            updated_by,
            tenant_id,
            transfer_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update transfer status: {}", e)))?;

        Ok(())
    }

    async fn confirm_transfer(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        approved_by: Uuid,
        updated_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_transfers
            SET status = 'confirmed', approved_by = $1, approved_at = NOW(),
                updated_by = $2, updated_at = NOW()
            WHERE tenant_id = $3 AND transfer_id = $4 AND deleted_at IS NULL
            "#,
            approved_by,
            updated_by,
            tenant_id,
            transfer_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to confirm transfer: {}", e)))?;

        Ok(())
    }

    async fn receive_transfer(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        updated_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_transfers
            SET status = 'received', actual_receive_date = NOW(),
                updated_by = $1, updated_at = NOW()
            WHERE tenant_id = $2 AND transfer_id = $3 AND deleted_at IS NULL
            "#,
            updated_by,
            tenant_id,
            transfer_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to receive transfer: {}", e)))?;

        Ok(())
    }

    async fn delete(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_transfers
            SET deleted_at = NOW(), updated_by = $1, updated_at = NOW()
            WHERE tenant_id = $2 AND transfer_id = $3 AND deleted_at IS NULL
            "#,
            deleted_by,
            tenant_id,
            transfer_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete transfer: {}", e)))?;

        Ok(())
    }
}

/// PostgreSQL implementation of TransferItemRepository
pub struct PgTransferItemRepository {
    pool: Arc<PgPool>,
}

impl PgTransferItemRepository {
    /// Create a new repository instance
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransferItemRepository for PgTransferItemRepository {
    async fn create_batch(
        &self,
        tenant_id: Uuid,
        items: &[TransferItem],
    ) -> Result<Vec<TransferItem>, AppError> {
        let mut created_items = Vec::new();

        for item in items {
            let row = sqlx::query!(
                r#"
                INSERT INTO stock_transfer_items (
                    transfer_item_id, tenant_id, transfer_id, product_id,
                    quantity, uom_id, unit_cost, line_total, line_number, notes
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING transfer_item_id, tenant_id, transfer_id, product_id,
                          quantity, uom_id, unit_cost, line_total, line_number, notes,
                          created_at, updated_at, deleted_at
                "#,
                item.transfer_item_id,
                tenant_id,
                item.transfer_id,
                item.product_id,
                item.quantity,
                item.uom_id,
                item.unit_cost,
                item.line_total,
                item.line_number as i32,
                item.notes
            )
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to create transfer item: {}", e))
            })?;

            created_items.push(TransferItem {
                transfer_item_id: row.transfer_item_id,
                tenant_id: row.tenant_id,
                transfer_id: row.transfer_id,
                product_id: row.product_id,
                quantity: row.quantity,
                uom_id: row.uom_id,
                unit_cost: row.unit_cost,
                line_total: row.line_total,
                line_number: row.line_number as i32,
                notes: row.notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
            });
        }

        Ok(created_items)
    }

    async fn find_by_transfer_id(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
    ) -> Result<Vec<TransferItem>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT transfer_item_id, tenant_id, transfer_id, product_id,
                   quantity, uom_id, unit_cost, line_total, line_number, notes,
                   created_at, updated_at, deleted_at
            FROM stock_transfer_items
            WHERE tenant_id = $1 AND transfer_id = $2 AND deleted_at IS NULL
            ORDER BY line_number
            "#,
            tenant_id,
            transfer_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find transfer items: {}", e)))?;

        let items = rows
            .into_iter()
            .map(|r| TransferItem {
                transfer_item_id: r.transfer_item_id,
                tenant_id: r.tenant_id,
                transfer_id: r.transfer_id,
                product_id: r.product_id,
                quantity: r.quantity,
                uom_id: r.uom_id,
                unit_cost: r.unit_cost,
                line_total: r.line_total,
                line_number: r.line_number as i32,
                notes: r.notes,
                created_at: r.created_at,
                updated_at: r.updated_at,
                deleted_at: r.deleted_at,
            })
            .collect();

        Ok(items)
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
    ) -> Result<Option<TransferItem>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT transfer_item_id, tenant_id, transfer_id, product_id,
                   quantity, uom_id, unit_cost, line_total, line_number, notes,
                   created_at, updated_at, deleted_at
            FROM stock_transfer_items
            WHERE tenant_id = $1 AND transfer_item_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            item_id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find transfer item: {}", e)))?;

        Ok(row.map(|r| TransferItem {
            transfer_item_id: r.transfer_item_id,
            tenant_id: r.tenant_id,
            transfer_id: r.transfer_id,
            product_id: r.product_id,
            quantity: r.quantity,
            uom_id: r.uom_id,
            unit_cost: r.unit_cost,
            line_total: r.line_total,
            line_number: r.line_number as i32,
            notes: r.notes,
            created_at: r.created_at,
            updated_at: r.updated_at,
            deleted_at: r.deleted_at,
        }))
    }

    async fn update_quantity(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
        quantity: i64,
        updated_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_transfer_items
            SET quantity = $1, updated_at = NOW()
            WHERE tenant_id = $2 AND transfer_item_id = $3 AND deleted_at IS NULL
            "#,
            quantity,
            tenant_id,
            item_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update transfer item quantity: {}", e))
        })?;

        Ok(())
    }

    async fn delete(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_transfer_items
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND transfer_item_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            item_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete transfer item: {}", e)))?;

        Ok(())
    }
}
