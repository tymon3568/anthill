//! PostgreSQL implementation of receipt repositories
//!
//! This module provides concrete implementations of receipt-related repository traits
//! using PostgreSQL as the data store. It handles all database operations for
//! Goods Receipt Notes (GRN) and related stock movements.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use inventory_service_core::dto::receipt::{
    ReceiptCreateRequest, ReceiptItemResponse, ReceiptListQuery, ReceiptListResponse,
    ReceiptResponse, ReceiptSummaryResponse,
};
use inventory_service_core::repositories::receipt::ReceiptRepository;
use shared_error::AppError;

/// PostgreSQL implementation of ReceiptRepository
///
/// Provides concrete implementations of all receipt repository operations
/// using SQLx for database interactions with PostgreSQL.
pub struct ReceiptRepositoryImpl {
    pool: PgPool,
}

impl ReceiptRepositoryImpl {
    /// Create a new ReceiptRepositoryImpl with the given database connection pool
    ///
    /// # Arguments
    /// * `pool` - PostgreSQL connection pool
    ///
    /// # Returns
    /// New ReceiptRepositoryImpl instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReceiptRepository for ReceiptRepositoryImpl {
    /// Create a new goods receipt with items in a single transaction
    async fn create_receipt(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: &ReceiptCreateRequest,
        idempotency_key: &str,
    ) -> Result<ReceiptResponse, AppError> {
        let mut tx = self.pool.begin().await?;

        // Check idempotency within transaction to prevent race condition
        let count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::BIGINT as "count!"
            FROM stock_moves
            WHERE tenant_id = $1 AND idempotency_key = $2
            "#,
            tenant_id,
            idempotency_key
        )
        .fetch_one(&mut *tx)
        .await?;

        if count > 0 {
            return Err(AppError::Conflict("Receipt already exists".to_string()));
        }

        // Generate receipt number
        let receipt_number: String = sqlx::query_scalar("SELECT generate_receipt_number()")
            .fetch_one(&mut *tx)
            .await?;

        // Create receipt
        let receipt_id = Uuid::now_v7();
        let receipt = sqlx::query!(
            r#"
            INSERT INTO goods_receipts (
                receipt_id, tenant_id, receipt_number, reference_number,
                warehouse_id, supplier_id, expected_delivery_date, notes,
                created_by, currency_code
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING receipt_id, receipt_number, reference_number,
                      warehouse_id, supplier_id, status, receipt_date,
                      expected_delivery_date, actual_delivery_date, notes,
                      created_by, total_quantity, total_value, currency_code,
                      created_at, updated_at
            "#,
            receipt_id,
            tenant_id,
            receipt_number,
            request.reference_number,
            request.warehouse_id,
            request.supplier_id,
            request.expected_delivery_date,
            request.notes,
            user_id,
            request.currency_code.clone()
        )
        .fetch_one(&mut *tx)
        .await?;

        // Create receipt items
        let mut items = Vec::new();
        for item_request in &request.items {
            let item_id = Uuid::now_v7();
            let item = sqlx::query!(
                r#"
                INSERT INTO goods_receipt_items (
                    receipt_item_id, tenant_id, receipt_id, product_id,
                    expected_quantity, received_quantity, unit_cost,
                    uom_id, lot_number, serial_numbers, expiry_date, notes
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                RETURNING receipt_item_id, product_id, expected_quantity,
                          received_quantity, unit_cost, line_total, uom_id,
                          lot_number, serial_numbers, expiry_date, notes,
                          created_at, updated_at
                "#,
                item_id,
                tenant_id,
                receipt_id,
                item_request.product_id,
                item_request.expected_quantity,
                item_request.received_quantity,
                item_request.unit_cost,
                item_request.uom_id,
                item_request.lot_number,
                item_request.serial_numbers,
                item_request.expiry_date,
                item_request.notes
            )
            .fetch_one(&mut *tx)
            .await?;

            items.push(ReceiptItemResponse {
                receipt_item_id: item.receipt_item_id,
                receipt_id,
                tenant_id,
                product_id: item.product_id,
                expected_quantity: item.expected_quantity,
                received_quantity: item.received_quantity,
                unit_cost: item.unit_cost,
                line_total: item.line_total,
                uom_id: item.uom_id,
                lot_number: item.lot_number,
                serial_numbers: item.serial_numbers,
                expiry_date: item.expiry_date,
                notes: item.notes,
                created_at: item.created_at,
                updated_at: item.updated_at,
            });
        }

        // Compute totals from items
        let total_quantity: i64 = items.iter().map(|item| item.received_quantity as i64).sum();
        let total_value: i64 = items.iter().map(|item| item.line_total as i64).sum();

        // Update receipt with computed totals
        sqlx::query!(
            r#"
            UPDATE goods_receipts
            SET total_quantity = $1, total_value = $2
            WHERE receipt_id = $3 AND tenant_id = $4
            "#,
            total_quantity,
            total_value,
            receipt_id,
            tenant_id
        )
        .execute(&mut *tx)
        .await?;

        // Create stock moves within the same transaction
        for (index, item_request) in request.items.iter().enumerate() {
            let move_id = Uuid::now_v7();
            let item_idempotency_key = format!("{}-{}", idempotency_key, index);
            sqlx::query!(
                r#"
                INSERT INTO stock_moves (
                    move_id, tenant_id, product_id, move_type, quantity,
                    unit_cost, reference_type, reference_id, idempotency_key,
                    move_date, move_reason
                )
                VALUES ($1, $2, $3, 'receipt', $4, $5, 'grn', $6, $7, NOW(), 'Goods receipt')
                "#,
                move_id,
                tenant_id,
                item_request.product_id,
                item_request.received_quantity,
                item_request.unit_cost,
                receipt_id,
                item_idempotency_key
            )
            .execute(&mut *tx)
            .await?;
        }

        // TODO: Publish receipt created event to outbox (when outbox table is implemented)
        // For now, this is a no-op until outbox pattern is fully implemented

        tx.commit().await?;
        Ok(ReceiptResponse {
            receipt_id,
            receipt_number: receipt.receipt_number,
            tenant_id,
            warehouse_id: receipt.warehouse_id,
            supplier_id: receipt.supplier_id,
            reference_number: receipt.reference_number,
            status: receipt.status,
            receipt_date: receipt.receipt_date,
            expected_delivery_date: receipt.expected_delivery_date,
            actual_delivery_date: receipt.actual_delivery_date,
            notes: receipt.notes,
            created_by: receipt.created_by,
            total_quantity,
            total_value,
            currency_code: receipt.currency_code,
            items,
            created_at: receipt.created_at,
            updated_at: receipt.updated_at,
        })
    }

    /// Get a receipt by ID with full details
    async fn get_receipt(
        &self,
        tenant_id: Uuid,
        receipt_id: Uuid,
    ) -> Result<ReceiptResponse, AppError> {
        // Get receipt
        let receipt = sqlx::query!(
            r#"
            SELECT receipt_id, receipt_number, reference_number,
                   warehouse_id, supplier_id, status, receipt_date,
                   expected_delivery_date, actual_delivery_date, notes,
                   created_by, total_quantity, total_value, currency_code,
                   created_at, updated_at
            FROM goods_receipts
            WHERE tenant_id = $1 AND receipt_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            receipt_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Receipt not found".to_string()))?;

        // Get receipt items
        let items = sqlx::query!(
            r#"
            SELECT receipt_item_id, product_id, expected_quantity,
                   received_quantity, unit_cost, line_total, uom_id,
                   lot_number, serial_numbers, expiry_date, notes,
                   created_at, updated_at
            FROM goods_receipt_items
            WHERE tenant_id = $1 AND receipt_id = $2 AND deleted_at IS NULL
            ORDER BY created_at ASC
            "#,
            tenant_id,
            receipt_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|item| ReceiptItemResponse {
            receipt_item_id: item.receipt_item_id,
            receipt_id,
            tenant_id,
            product_id: item.product_id,
            expected_quantity: item.expected_quantity,
            received_quantity: item.received_quantity,
            unit_cost: item.unit_cost,
            line_total: item.line_total,
            uom_id: item.uom_id,
            lot_number: item.lot_number,
            serial_numbers: item.serial_numbers,
            expiry_date: item.expiry_date,
            notes: item.notes,
            created_at: item.created_at,
            updated_at: item.updated_at,
        })
        .collect();

        // Compute totals from items
        let total_quantity: i64 = items.iter().map(|item| item.received_quantity as i64).sum();
        let total_value: i64 = items.iter().map(|item| item.line_total as i64).sum();

        Ok(ReceiptResponse {
            receipt_id,
            receipt_number: receipt.receipt_number,
            tenant_id,
            warehouse_id: receipt.warehouse_id,
            supplier_id: receipt.supplier_id,
            reference_number: receipt.reference_number,
            status: receipt.status,
            receipt_date: receipt.receipt_date,
            expected_delivery_date: receipt.expected_delivery_date,
            actual_delivery_date: receipt.actual_delivery_date,
            notes: receipt.notes,
            created_by: receipt.created_by,
            total_quantity,
            total_value,
            currency_code: receipt.currency_code,
            items,
            created_at: receipt.created_at,
            updated_at: receipt.updated_at,
        })
    }

    /// List receipts with pagination and filtering
    async fn list_receipts(
        &self,
        tenant_id: Uuid,
        query: ReceiptListQuery,
    ) -> Result<ReceiptListResponse, AppError> {
        let offset = (query.page - 1) * query.page_size;

        // Count query
        let count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::BIGINT as "count!"
            FROM goods_receipts
            WHERE tenant_id = $1
              AND deleted_at IS NULL
              AND ($2::UUID IS NULL OR warehouse_id = $2)
              AND ($3::UUID IS NULL OR supplier_id = $3)
              AND ($4::TEXT IS NULL OR status = $4)
              AND ($5::TEXT IS NULL OR receipt_number ILIKE '%' || $5 || '%' OR reference_number ILIKE '%' || $5 || '%')
              AND ($6::TIMESTAMPTZ IS NULL OR created_at >= $6)
              AND ($7::TIMESTAMPTZ IS NULL OR created_at <= $7)
            "#,
            tenant_id,
            query.warehouse_id,
            query.supplier_id,
            query.status,
            query.search,
            query.created_after,
            query.created_before
        )
        .fetch_one(&self.pool)
        .await?;

        // Data query
        let receipts = sqlx::query!(
            r#"
            SELECT receipt_id, receipt_number, reference_number,
                   warehouse_id, supplier_id, status, receipt_date,
                   total_quantity, total_value, currency_code,
                   created_at,
                   (
                       SELECT COUNT(*)::INTEGER
                       FROM goods_receipt_items
                       WHERE receipt_id = gr.receipt_id AND tenant_id = gr.tenant_id
                   ) as item_count
            FROM goods_receipts gr
            WHERE tenant_id = $1
              AND deleted_at IS NULL
              AND ($2::UUID IS NULL OR warehouse_id = $2)
              AND ($3::UUID IS NULL OR supplier_id = $3)
              AND ($4::TEXT IS NULL OR status = $4)
              AND ($5::TEXT IS NULL OR receipt_number ILIKE '%' || $5 || '%' OR reference_number ILIKE '%' || $5 || '%')
              AND ($6::TIMESTAMPTZ IS NULL OR created_at >= $6)
              AND ($7::TIMESTAMPTZ IS NULL OR created_at <= $7)
            ORDER BY created_at DESC
            LIMIT $8 OFFSET $9
            "#,
            tenant_id,
            query.warehouse_id,
            query.supplier_id,
            query.status,
            query.search,
            query.created_after,
            query.created_before,
            query.page_size as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| ReceiptSummaryResponse {
            receipt_id: row.receipt_id,
            receipt_number: row.receipt_number,
            warehouse_id: row.warehouse_id,
            supplier_id: row.supplier_id,
            reference_number: row.reference_number,
            status: row.status,
            receipt_date: row.receipt_date,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            currency_code: row.currency_code,
            item_count: row.item_count.unwrap_or(0),
            created_at: row.created_at,
        })
        .collect();

        let total_pages = ((count as f64) / (query.page_size as f64)).ceil() as i32;

        Ok(ReceiptListResponse {
            receipts,
            pagination: inventory_service_core::dto::receipt::PaginationInfo {
                page: query.page,
                page_size: query.page_size,
                total_items: count,
                total_pages,
                has_next: query.page < total_pages,
                has_prev: query.page > 1,
            },
        })
    }

    /// Check if a receipt exists by ID
    async fn receipt_exists(&self, tenant_id: Uuid, receipt_id: Uuid) -> Result<bool, AppError> {
        let count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::BIGINT as "count!"
            FROM goods_receipts
            WHERE tenant_id = $1 AND receipt_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            receipt_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }
}
