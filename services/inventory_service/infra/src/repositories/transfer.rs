use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};
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

    /// Convert database string to TransferStatus enum
    fn string_to_transfer_status(s: &str) -> Result<TransferStatus, AppError> {
        match s {
            "draft" => Ok(TransferStatus::Draft),
            "confirmed" => Ok(TransferStatus::Confirmed),
            "partially_picked" => Ok(TransferStatus::PartiallyPicked),
            "picked" => Ok(TransferStatus::Picked),
            "partially_shipped" => Ok(TransferStatus::PartiallyShipped),
            "shipped" => Ok(TransferStatus::Shipped),
            "received" => Ok(TransferStatus::Received),
            "cancelled" => Ok(TransferStatus::Cancelled),
            _ => Err(AppError::DataCorruption(format!("Unknown transfer status: {}", s))),
        }
    }

    /// Convert database string to TransferType enum
    fn string_to_transfer_type(s: &str) -> Result<TransferType, AppError> {
        match s {
            "manual" => Ok(TransferType::Manual),
            "auto_replenishment" => Ok(TransferType::AutoReplenishment),
            "emergency" => Ok(TransferType::Emergency),
            "consolidation" => Ok(TransferType::Consolidation),
            _ => Err(AppError::DataCorruption(format!("Unknown transfer type: {}", s))),
        }
    }

    /// Convert database string to TransferPriority enum
    fn string_to_transfer_priority(s: &str) -> Result<TransferPriority, AppError> {
        match s {
            "low" => Ok(TransferPriority::Low),
            "normal" => Ok(TransferPriority::Normal),
            "high" => Ok(TransferPriority::High),
            "urgent" => Ok(TransferPriority::Urgent),
            _ => Err(AppError::DataCorruption(format!("Unknown transfer priority: {}", s))),
        }
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
            VALUES ($1, $2, COALESCE(NULLIF($3, ''), generate_stock_transfer_number()), $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            RETURNING transfer_id, tenant_id, transfer_number, reference_number,
                      source_warehouse_id, destination_warehouse_id, status, transfer_type, priority,
                      transfer_date, expected_ship_date, actual_ship_date,
                      expected_receive_date, actual_receive_date,
                      shipping_method, carrier, tracking_number, shipping_cost,
                      notes, reason, created_by, updated_by, approved_by, approved_at,
                      total_quantity, total_value, currency_code,
                      created_at, updated_at, deleted_at, deleted_by
            "#,
            transfer.transfer_id,
            tenant_id,
            transfer.transfer_number,
            transfer.reference_number,
            transfer.source_warehouse_id,
            transfer.destination_warehouse_id,
            match transfer.status {
                TransferStatus::Draft => "draft",
                TransferStatus::Confirmed => "confirmed",
                TransferStatus::PartiallyPicked => "partially_picked",
                TransferStatus::Picked => "picked",
                TransferStatus::PartiallyShipped => "partially_shipped",
                TransferStatus::Shipped => "shipped",
                TransferStatus::Received => "received",
                TransferStatus::Cancelled => "cancelled",
            },
            match transfer.transfer_type {
                TransferType::Manual => "manual",
                TransferType::AutoReplenishment => "auto_replenishment",
                TransferType::Emergency => "emergency",
                TransferType::Consolidation => "consolidation",
            },
            match transfer.priority {
                TransferPriority::Low => "low",
                TransferPriority::Normal => "normal",
                TransferPriority::High => "high",
                TransferPriority::Urgent => "urgent",
            },
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
            status: Self::string_to_transfer_status(&row.status)?,
            transfer_type: Self::string_to_transfer_type(&row.transfer_type)?,
            priority: Self::string_to_transfer_priority(&row.priority)?,
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
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            currency_code: row.currency_code.unwrap_or_else(|| "VND".to_string()),
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
            deleted_by: row.deleted_by,
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
                   created_at, updated_at, deleted_at, deleted_by
            FROM stock_transfers
            WHERE tenant_id = $1 AND transfer_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            transfer_id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find transfer: {}", e)))?;

        row.map(|r| -> Result<Transfer, AppError> {
            Ok(Transfer {
                transfer_id: r.transfer_id,
                tenant_id: r.tenant_id,
                transfer_number: r.transfer_number,
                reference_number: r.reference_number,
                source_warehouse_id: r.source_warehouse_id,
                destination_warehouse_id: r.destination_warehouse_id,
                status: Self::string_to_transfer_status(&r.status)?,
                transfer_type: Self::string_to_transfer_type(&r.transfer_type)?,
                priority: Self::string_to_transfer_priority(&r.priority)?,
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
                total_quantity: r.total_quantity.unwrap_or(0),
                total_value: r.total_value.unwrap_or(0),
                currency_code: r.currency_code.unwrap_or_else(|| "VND".to_string()),
                created_at: r.created_at,
                updated_at: r.updated_at,
                deleted_at: r.deleted_at,
                deleted_by: r.deleted_by,
            })
        })
        .transpose()
    }

    async fn list(
        &self,
        tenant_id: Uuid,
        source_warehouse_id: Option<Uuid>,
        destination_warehouse_id: Option<Uuid>,
        status: Option<TransferStatus>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Transfer>, AppError> {
        let status_str = status.map(|s| match s {
            TransferStatus::Draft => "draft",
            TransferStatus::Confirmed => "confirmed",
            TransferStatus::PartiallyPicked => "partially_picked",
            TransferStatus::Picked => "picked",
            TransferStatus::PartiallyShipped => "partially_shipped",
            TransferStatus::Shipped => "shipped",
            TransferStatus::Received => "received",
            TransferStatus::Cancelled => "cancelled",
        });

        let rows = sqlx::query!(
            r#"
            SELECT transfer_id, tenant_id, transfer_number, reference_number,
                   source_warehouse_id, destination_warehouse_id, status, transfer_type, priority,
                   transfer_date, expected_ship_date, actual_ship_date,
                   expected_receive_date, actual_receive_date,
                   shipping_method, carrier, tracking_number, shipping_cost,
                   notes, reason, created_by, updated_by, approved_by, approved_at,
                   total_quantity, total_value, currency_code,
                   created_at, updated_at, deleted_at, deleted_by
            FROM stock_transfers
            WHERE tenant_id = $1
              AND deleted_at IS NULL
              AND ($2::UUID IS NULL OR source_warehouse_id = $2)
              AND ($3::UUID IS NULL OR destination_warehouse_id = $3)
              AND ($4::TEXT IS NULL OR status = $4)
            ORDER BY created_at DESC
            LIMIT $5
            OFFSET $6
            "#,
            tenant_id,
            source_warehouse_id,
            destination_warehouse_id,
            status_str,
            limit.unwrap_or(50),
            offset.unwrap_or(0)
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to list transfers: {}", e)))?;

        let mut transfers = Vec::with_capacity(rows.len());
        for r in rows {
            transfers.push(Transfer {
                transfer_id: r.transfer_id,
                tenant_id: r.tenant_id,
                transfer_number: r.transfer_number,
                reference_number: r.reference_number,
                source_warehouse_id: r.source_warehouse_id,
                destination_warehouse_id: r.destination_warehouse_id,
                status: Self::string_to_transfer_status(&r.status)?,
                transfer_type: Self::string_to_transfer_type(&r.transfer_type)?,
                priority: Self::string_to_transfer_priority(&r.priority)?,
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
                total_quantity: r.total_quantity.unwrap_or(0),
                total_value: r.total_value.unwrap_or(0),
                currency_code: r.currency_code.unwrap_or_else(|| "VND".to_string()),
                created_at: r.created_at,
                updated_at: r.updated_at,
                deleted_at: r.deleted_at,
                deleted_by: r.deleted_by,
            });
        }

        Ok(transfers)
    }

    async fn count(
        &self,
        tenant_id: Uuid,
        source_warehouse_id: Option<Uuid>,
        destination_warehouse_id: Option<Uuid>,
        status: Option<TransferStatus>,
    ) -> Result<i64, AppError> {
        let status_str = status.map(|s| match s {
            TransferStatus::Draft => "draft",
            TransferStatus::Confirmed => "confirmed",
            TransferStatus::PartiallyPicked => "partially_picked",
            TransferStatus::Picked => "picked",
            TransferStatus::PartiallyShipped => "partially_shipped",
            TransferStatus::Shipped => "shipped",
            TransferStatus::Received => "received",
            TransferStatus::Cancelled => "cancelled",
        });

        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM stock_transfers
            WHERE tenant_id = $1
              AND deleted_at IS NULL
              AND ($2::UUID IS NULL OR source_warehouse_id = $2)
              AND ($3::UUID IS NULL OR destination_warehouse_id = $3)
              AND ($4::TEXT IS NULL OR status = $4)
            "#,
            tenant_id,
            source_warehouse_id,
            destination_warehouse_id,
            status_str
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to count transfers: {}", e)))?;

        Ok(row.count)
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
            match status {
                TransferStatus::Draft => "draft",
                TransferStatus::Confirmed => "confirmed",
                TransferStatus::PartiallyPicked => "partially_picked",
                TransferStatus::Picked => "picked",
                TransferStatus::PartiallyShipped => "partially_shipped",
                TransferStatus::Shipped => "shipped",
                TransferStatus::Received => "received",
                TransferStatus::Cancelled => "cancelled",
            },
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
            SET status = $1, approved_by = $2, approved_at = NOW(),
                updated_by = $3, updated_at = NOW()
            WHERE tenant_id = $4 AND transfer_id = $5 AND deleted_at IS NULL
            "#,
            "shipped",
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
            SET status = $1, actual_receive_date = NOW(),
                updated_by = $2, updated_at = NOW()
            WHERE tenant_id = $3 AND transfer_id = $4 AND deleted_at IS NULL
            "#,
            "received",
            updated_by,
            tenant_id,
            transfer_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to receive transfer: {}", e)))?;

        Ok(())
    }

    async fn cancel_transfer(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        cancelled_by: Uuid,
        reason: Option<String>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_transfers
            SET status = $1, reason = COALESCE($2, reason),
                updated_by = $3, updated_at = NOW()
            WHERE tenant_id = $4 AND transfer_id = $5 AND deleted_at IS NULL
            "#,
            "cancelled",
            reason,
            cancelled_by,
            tenant_id,
            transfer_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to cancel transfer: {}", e)))?;

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
            SET deleted_at = NOW(), deleted_by = $1, updated_at = NOW()
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
        // Fast-path: avoid hitting the DB if there is nothing to insert
        if items.is_empty() {
            return Ok(Vec::new());
        }

        // Build a single multi-values INSERT statement using QueryBuilder
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            INSERT INTO stock_transfer_items (
                transfer_item_id, tenant_id, transfer_id, product_id,
                quantity, uom_id, unit_cost, line_total, line_number,
                source_zone_id, source_location_id, destination_zone_id, destination_location_id,
                notes
            )
            "#,
        );

        query_builder.push_values(items.iter(), |mut b, item| {
            b.push_bind(item.transfer_item_id)
                .push_bind(tenant_id)
                .push_bind(item.transfer_id)
                .push_bind(item.product_id)
                .push_bind(item.quantity)
                .push_bind(item.uom_id)
                .push_bind(item.unit_cost)
                .push_bind(item.line_total)
                .push_bind(item.line_number)
                .push_bind(item.source_zone_id)
                .push_bind(item.source_location_id)
                .push_bind(item.destination_zone_id)
                .push_bind(item.destination_location_id)
                .push_bind(&item.notes);
        });

        // Return the inserted rows so the behavior matches the previous implementation
        query_builder.push(
            r#"
            RETURNING transfer_item_id, tenant_id, transfer_id, product_id,
                      quantity, uom_id, unit_cost, line_total, line_number,
                      source_zone_id, source_location_id, destination_zone_id, destination_location_id,
                      notes, created_at, updated_at, updated_by, deleted_at, deleted_by
            "#,
        );

        let query = query_builder.build_query_as::<TransferItem>();

        let created_items = query.fetch_all(&*self.pool).await?;

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
                   quantity, uom_id, unit_cost, line_total, line_number,
                   source_zone_id, source_location_id, destination_zone_id, destination_location_id,
                   notes, created_at, updated_at, updated_by, deleted_at, deleted_by
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
                line_number: r.line_number,
                source_zone_id: r.source_zone_id,
                source_location_id: r.source_location_id,
                destination_zone_id: r.destination_zone_id,
                destination_location_id: r.destination_location_id,
                notes: r.notes,
                created_at: r.created_at,
                updated_at: r.updated_at,
                updated_by: r.updated_by,
                deleted_at: r.deleted_at,
                deleted_by: r.deleted_by,
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
                   quantity, uom_id, unit_cost, line_total, line_number,
                   source_zone_id, source_location_id, destination_zone_id, destination_location_id,
                   notes, created_at, updated_at, updated_by, deleted_at, deleted_by
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
            line_number: r.line_number,
            source_zone_id: r.source_zone_id,
            source_location_id: r.source_location_id,
            destination_zone_id: r.destination_zone_id,
            destination_location_id: r.destination_location_id,
            notes: r.notes,
            created_at: r.created_at,
            updated_at: r.updated_at,
            updated_by: r.updated_by,
            deleted_at: r.deleted_at,
            deleted_by: r.deleted_by,
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
            SET quantity = $1, updated_by = $2, updated_at = NOW()
            WHERE tenant_id = $3 AND transfer_item_id = $4 AND deleted_at IS NULL
            "#,
            quantity,
            updated_by,
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
            SET deleted_at = NOW(), deleted_by = $3, updated_at = NOW()
            WHERE tenant_id = $1 AND transfer_item_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            item_id,
            deleted_by
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete transfer item: {}", e)))?;

        Ok(())
    }
}
