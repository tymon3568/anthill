//! Stock Adjustment Service Implementation
//!
//! PostgreSQL implementation of the AdjustmentService trait for stock adjustment operations.
//! Follows the 3-crate pattern: api → infra → core → shared/*

use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::dto::adjustment::{
    AddAdjustmentLinesRequest, AdjustmentDocument, AdjustmentDocumentResponse,
    AdjustmentDocumentWithLinesResponse, AdjustmentLine, AdjustmentListQuery,
    AdjustmentListResponse, AdjustmentReasonCode, AdjustmentStatus, AdjustmentSummary,
    AdjustmentType, CreateAdjustmentRequest, PostAdjustmentRequest,
};
use inventory_service_core::services::adjustment::AdjustmentService;
use shared_error::AppError;

/// PostgreSQL implementation of AdjustmentService
pub struct PgAdjustmentService {
    pool: Arc<PgPool>,
}

impl PgAdjustmentService {
    /// Create a new adjustment service instance
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Convert database status string to AdjustmentStatus enum
    fn parse_status(status: &str) -> AdjustmentStatus {
        match status {
            "draft" => AdjustmentStatus::Draft,
            "posted" => AdjustmentStatus::Posted,
            "cancelled" => AdjustmentStatus::Cancelled,
            _ => AdjustmentStatus::Draft,
        }
    }

    /// Convert database adjustment_type string to AdjustmentType enum
    fn parse_adjustment_type(adj_type: &str) -> AdjustmentType {
        match adj_type {
            "increase" => AdjustmentType::Increase,
            "decrease" => AdjustmentType::Decrease,
            _ => AdjustmentType::Decrease,
        }
    }

    /// Convert database reason_code string to AdjustmentReasonCode enum
    fn parse_reason_code(code: &str) -> AdjustmentReasonCode {
        match code {
            "damaged" => AdjustmentReasonCode::Damaged,
            "lost" => AdjustmentReasonCode::Lost,
            "found" => AdjustmentReasonCode::Found,
            "count_correction" => AdjustmentReasonCode::CountCorrection,
            "system_correction" => AdjustmentReasonCode::SystemCorrection,
            "expired" => AdjustmentReasonCode::Expired,
            "theft" => AdjustmentReasonCode::Theft,
            "promotion" => AdjustmentReasonCode::Promotion,
            "return_to_stock" => AdjustmentReasonCode::ReturnToStock,
            _ => AdjustmentReasonCode::Other,
        }
    }
}

/// Helper struct for adjustment document SQL results
#[derive(Debug, sqlx::FromRow)]
struct AdjustmentDocumentRow {
    adjustment_id: Uuid,
    tenant_id: Uuid,
    reference: Option<String>,
    status: String,
    warehouse_id: Uuid,
    notes: Option<String>,
    created_by: Option<Uuid>,
    posted_by: Option<Uuid>,
    posted_at: Option<chrono::DateTime<Utc>>,
    cancelled_by: Option<Uuid>,
    cancelled_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<AdjustmentDocumentRow> for AdjustmentDocument {
    fn from(row: AdjustmentDocumentRow) -> Self {
        Self {
            adjustment_id: row.adjustment_id,
            tenant_id: row.tenant_id,
            reference: row.reference,
            status: PgAdjustmentService::parse_status(&row.status),
            warehouse_id: row.warehouse_id,
            notes: row.notes,
            created_by: row.created_by,
            posted_by: row.posted_by,
            posted_at: row.posted_at,
            cancelled_by: row.cancelled_by,
            cancelled_at: row.cancelled_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Helper struct for adjustment line SQL results
#[derive(Debug, sqlx::FromRow)]
struct AdjustmentLineRow {
    adjustment_line_id: Uuid,
    tenant_id: Uuid,
    adjustment_id: Uuid,
    product_id: Uuid,
    variant_id: Option<Uuid>,
    adjustment_type: String,
    qty: i64,
    reason_code: String,
    reason_notes: Option<String>,
    location_id: Option<Uuid>,
    lot_id: Option<Uuid>,
    serial_id: Option<Uuid>,
    posted_stock_move_id: Option<Uuid>,
    created_at: chrono::DateTime<Utc>,
}

impl AdjustmentLineRow {
    fn into_adjustment_line(self) -> AdjustmentLine {
        AdjustmentLine {
            adjustment_line_id: self.adjustment_line_id,
            tenant_id: self.tenant_id,
            adjustment_id: self.adjustment_id,
            product_id: self.product_id,
            variant_id: self.variant_id,
            adjustment_type: PgAdjustmentService::parse_adjustment_type(&self.adjustment_type),
            qty: self.qty,
            reason_code: PgAdjustmentService::parse_reason_code(&self.reason_code),
            reason_notes: self.reason_notes,
            location_id: self.location_id,
            lot_id: self.lot_id,
            serial_id: self.serial_id,
            posted_stock_move_id: self.posted_stock_move_id,
            created_at: self.created_at,
        }
    }
}

#[async_trait]
impl AdjustmentService for PgAdjustmentService {
    async fn create_adjustment(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateAdjustmentRequest,
    ) -> Result<AdjustmentDocumentWithLinesResponse, AppError> {
        let adjustment_id = Uuid::now_v7();

        let mut tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;

        // Create the adjustment document
        let row = sqlx::query_as::<_, AdjustmentDocumentRow>(
            r#"
            INSERT INTO adjustment_documents (
                tenant_id, adjustment_id, reference, status, warehouse_id, notes, created_by
            )
            VALUES ($1, $2, $3, 'draft', $4, $5, $6)
            RETURNING
                adjustment_id, tenant_id, reference, status, warehouse_id, notes,
                created_by, posted_by, posted_at, cancelled_by, cancelled_at,
                created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(adjustment_id)
        .bind(&request.reference)
        .bind(request.warehouse_id)
        .bind(&request.notes)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to create adjustment document: {}", e))
        })?;

        // Insert initial lines if provided
        let mut lines: Vec<AdjustmentLine> = Vec::new();
        if let Some(input_lines) = &request.lines {
            for line in input_lines {
                // Validate qty > 0
                if line.qty <= 0 {
                    return Err(AppError::ValidationError(
                        "Line quantity must be greater than 0".to_string(),
                    ));
                }

                let line_id = Uuid::now_v7();
                let adj_type_str = line.adjustment_type.to_string();
                let reason_code_str = line.reason_code.to_string();

                let line_row = sqlx::query_as::<_, AdjustmentLineRow>(
                    r#"
                    INSERT INTO adjustment_lines (
                        tenant_id, adjustment_line_id, adjustment_id, product_id, variant_id,
                        adjustment_type, qty, reason_code, reason_notes, location_id,
                        lot_id, serial_id
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                    RETURNING
                        adjustment_line_id, tenant_id, adjustment_id, product_id, variant_id,
                        adjustment_type, qty, reason_code, reason_notes, location_id,
                        lot_id, serial_id, posted_stock_move_id, created_at
                    "#,
                )
                .bind(tenant_id)
                .bind(line_id)
                .bind(adjustment_id)
                .bind(line.product_id)
                .bind(line.variant_id)
                .bind(&adj_type_str)
                .bind(line.qty)
                .bind(&reason_code_str)
                .bind(&line.reason_notes)
                .bind(line.location_id)
                .bind(line.lot_id)
                .bind(line.serial_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to insert adjustment line: {}", e))
                })?;

                lines.push(line_row.into_adjustment_line());
            }
        }

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(AdjustmentDocumentWithLinesResponse {
            adjustment: row.into(),
            lines,
        })
    }

    async fn get_adjustment(
        &self,
        tenant_id: Uuid,
        adjustment_id: Uuid,
    ) -> Result<AdjustmentDocumentWithLinesResponse, AppError> {
        // Fetch document
        let doc_row = sqlx::query_as::<_, AdjustmentDocumentRow>(
            r#"
            SELECT
                adjustment_id, tenant_id, reference, status, warehouse_id, notes,
                created_by, posted_by, posted_at, cancelled_by, cancelled_at,
                created_at, updated_at
            FROM adjustment_documents
            WHERE tenant_id = $1 AND adjustment_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(adjustment_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch adjustment document: {}", e))
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!("Adjustment document {} not found", adjustment_id))
        })?;

        // Fetch lines
        let line_rows = sqlx::query_as::<_, AdjustmentLineRow>(
            r#"
            SELECT
                adjustment_line_id, tenant_id, adjustment_id, product_id, variant_id,
                adjustment_type, qty, reason_code, reason_notes, location_id,
                lot_id, serial_id, posted_stock_move_id, created_at
            FROM adjustment_lines
            WHERE tenant_id = $1 AND adjustment_id = $2
            ORDER BY created_at
            "#,
        )
        .bind(tenant_id)
        .bind(adjustment_id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch adjustment lines: {}", e)))?;

        let lines: Vec<AdjustmentLine> = line_rows
            .into_iter()
            .map(|r| r.into_adjustment_line())
            .collect();

        Ok(AdjustmentDocumentWithLinesResponse {
            adjustment: doc_row.into(),
            lines,
        })
    }

    async fn list_adjustments(
        &self,
        tenant_id: Uuid,
        query: AdjustmentListQuery,
    ) -> Result<AdjustmentListResponse, AppError> {
        let limit = query.limit.unwrap_or(50).min(100) as i64;
        let page = query.page.unwrap_or(1).max(1);
        let offset = ((page - 1) as i64) * limit;

        let status_str = query.status.map(|s| s.to_string());
        let search_pattern = query.search.map(|s| format!("%{}%", s));

        let rows = sqlx::query_as::<_, AdjustmentDocumentRow>(
            r#"
            SELECT
                adjustment_id, tenant_id, reference, status, warehouse_id, notes,
                created_by, posted_by, posted_at, cancelled_by, cancelled_at,
                created_at, updated_at
            FROM adjustment_documents
            WHERE tenant_id = $1
              AND ($2::TEXT IS NULL OR status = $2)
              AND ($3::UUID IS NULL OR warehouse_id = $3)
              AND ($4::TIMESTAMPTZ IS NULL OR created_at >= $4)
              AND ($5::TIMESTAMPTZ IS NULL OR created_at <= $5)
              AND ($6::TEXT IS NULL OR reference ILIKE $6 OR notes ILIKE $6)
            ORDER BY created_at DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(tenant_id)
        .bind(&status_str)
        .bind(query.warehouse_id)
        .bind(query.from_date)
        .bind(query.to_date)
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to list adjustment documents: {}", e))
        })?;

        // Count total
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM adjustment_documents
            WHERE tenant_id = $1
              AND ($2::TEXT IS NULL OR status = $2)
              AND ($3::UUID IS NULL OR warehouse_id = $3)
              AND ($4::TIMESTAMPTZ IS NULL OR created_at >= $4)
              AND ($5::TIMESTAMPTZ IS NULL OR created_at <= $5)
              AND ($6::TEXT IS NULL OR reference ILIKE $6 OR notes ILIKE $6)
            "#,
        )
        .bind(tenant_id)
        .bind(&status_str)
        .bind(query.warehouse_id)
        .bind(query.from_date)
        .bind(query.to_date)
        .bind(&search_pattern)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to count adjustment documents: {}", e))
        })?;

        let adjustments: Vec<AdjustmentDocument> = rows.into_iter().map(|r| r.into()).collect();

        Ok(AdjustmentListResponse {
            adjustments,
            total_count: count.0 as u64,
            page,
            page_size: limit as u32,
        })
    }

    async fn get_adjustment_summary(
        &self,
        tenant_id: Uuid,
        warehouse_id: Option<Uuid>,
    ) -> Result<AdjustmentSummary, AppError> {
        let row: (i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(DISTINCT ad.adjustment_id) as total_adjustments,
                COALESCE(SUM(CASE WHEN al.adjustment_type = 'increase' THEN 1 ELSE 0 END), 0) as total_increases,
                COALESCE(SUM(CASE WHEN al.adjustment_type = 'decrease' THEN 1 ELSE 0 END), 0) as total_decreases,
                COALESCE(SUM(CASE
                    WHEN al.adjustment_type = 'increase' THEN al.qty
                    WHEN al.adjustment_type = 'decrease' THEN -al.qty
                    ELSE 0
                END), 0) as net_change
            FROM adjustment_documents ad
            LEFT JOIN adjustment_lines al ON ad.tenant_id = al.tenant_id AND ad.adjustment_id = al.adjustment_id
            WHERE ad.tenant_id = $1
              AND ad.status = 'posted'
              AND ($2::UUID IS NULL OR ad.warehouse_id = $2)
            "#,
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to get adjustment summary: {}", e))
        })?;

        Ok(AdjustmentSummary {
            total_adjustments: row.0 as u64,
            total_increases: row.1 as u64,
            total_decreases: row.2 as u64,
            net_change: row.3,
        })
    }

    async fn add_lines(
        &self,
        tenant_id: Uuid,
        adjustment_id: Uuid,
        _user_id: Uuid,
        request: AddAdjustmentLinesRequest,
    ) -> Result<AdjustmentDocumentWithLinesResponse, AppError> {
        // Start transaction first to prevent race conditions
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;

        // Verify document exists and is in draft status with FOR UPDATE lock
        let doc_row = sqlx::query_as::<_, AdjustmentDocumentRow>(
            r#"
            SELECT
                adjustment_id, tenant_id, reference, status, warehouse_id, notes,
                created_by, posted_by, posted_at, cancelled_by, cancelled_at,
                created_at, updated_at
            FROM adjustment_documents
            WHERE tenant_id = $1 AND adjustment_id = $2
            FOR UPDATE
            "#,
        )
        .bind(tenant_id)
        .bind(adjustment_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch adjustment document: {}", e))
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!("Adjustment document {} not found", adjustment_id))
        })?;

        if doc_row.status != "draft" {
            return Err(AppError::ValidationError(
                "Lines can only be added to draft adjustment documents".to_string(),
            ));
        }

        // Delete existing lines (replace strategy like scrap)
        sqlx::query(
            r#"
            DELETE FROM adjustment_lines
            WHERE tenant_id = $1 AND adjustment_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(adjustment_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete existing lines: {}", e)))?;

        // Insert new lines
        for line in &request.lines {
            // Validate qty > 0
            if line.qty <= 0 {
                return Err(AppError::ValidationError(
                    "Line quantity must be greater than 0".to_string(),
                ));
            }

            let line_id = Uuid::now_v7();
            let adj_type_str = line.adjustment_type.to_string();
            let reason_code_str = line.reason_code.to_string();

            sqlx::query(
                r#"
                INSERT INTO adjustment_lines (
                    tenant_id, adjustment_line_id, adjustment_id, product_id, variant_id,
                    adjustment_type, qty, reason_code, reason_notes, location_id,
                    lot_id, serial_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                "#,
            )
            .bind(tenant_id)
            .bind(line_id)
            .bind(adjustment_id)
            .bind(line.product_id)
            .bind(line.variant_id)
            .bind(&adj_type_str)
            .bind(line.qty)
            .bind(&reason_code_str)
            .bind(&line.reason_notes)
            .bind(line.location_id)
            .bind(line.lot_id)
            .bind(line.serial_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to insert adjustment line: {}", e))
            })?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        // Return updated document with lines
        self.get_adjustment(tenant_id, adjustment_id).await
    }

    async fn post_adjustment(
        &self,
        tenant_id: Uuid,
        adjustment_id: Uuid,
        user_id: Uuid,
        _request: PostAdjustmentRequest,
    ) -> Result<AdjustmentDocumentResponse, AppError> {
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;

        // Lock and verify document is in draft status
        let doc_row = sqlx::query_as::<_, AdjustmentDocumentRow>(
            r#"
            SELECT
                adjustment_id, tenant_id, reference, status, warehouse_id, notes,
                created_by, posted_by, posted_at, cancelled_by, cancelled_at,
                created_at, updated_at
            FROM adjustment_documents
            WHERE tenant_id = $1 AND adjustment_id = $2
            FOR UPDATE
            "#,
        )
        .bind(tenant_id)
        .bind(adjustment_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch adjustment document: {}", e))
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!("Adjustment document {} not found", adjustment_id))
        })?;

        // Idempotency check: if already posted, return success
        if doc_row.status == "posted" {
            return Ok(AdjustmentDocumentResponse {
                adjustment: doc_row.into(),
            });
        }

        if doc_row.status != "draft" {
            return Err(AppError::ValidationError(
                "Only draft adjustment documents can be posted".to_string(),
            ));
        }

        // Fetch lines
        let line_rows = sqlx::query_as::<_, AdjustmentLineRow>(
            r#"
            SELECT
                adjustment_line_id, tenant_id, adjustment_id, product_id, variant_id,
                adjustment_type, qty, reason_code, reason_notes, location_id,
                lot_id, serial_id, posted_stock_move_id, created_at
            FROM adjustment_lines
            WHERE tenant_id = $1 AND adjustment_id = $2
            FOR UPDATE
            "#,
        )
        .bind(tenant_id)
        .bind(adjustment_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch adjustment lines: {}", e)))?;

        if line_rows.is_empty() {
            return Err(AppError::ValidationError(
                "Cannot post adjustment document with no lines".to_string(),
            ));
        }

        let posted_at = Utc::now();

        // Create stock moves for each line
        for line in &line_rows {
            let move_id = Uuid::now_v7();

            // Determine quantity sign based on adjustment type
            let move_qty = match line.adjustment_type.as_str() {
                "increase" => line.qty,
                "decrease" => -line.qty,
                _ => -line.qty,
            };

            // Determine source/destination based on adjustment type
            // For increase: virtual "adjustment" location -> warehouse location
            // For decrease: warehouse location -> virtual "adjustment" location
            // Using warehouse_id as destination for simplicity (location_id is optional)
            let (source_loc, dest_loc) = if line.adjustment_type == "increase" {
                // From virtual adjustment location to warehouse
                (doc_row.warehouse_id, line.location_id.unwrap_or(doc_row.warehouse_id))
            } else {
                // From warehouse to virtual adjustment location
                (line.location_id.unwrap_or(doc_row.warehouse_id), doc_row.warehouse_id)
            };

            // Create stock move
            sqlx::query(
                r#"
                INSERT INTO stock_moves (
                    move_id, tenant_id, product_id, source_location_id, destination_location_id,
                    move_type, quantity, reference_type, reference_id, move_date,
                    move_reason, lot_serial_id, created_at
                )
                VALUES ($1, $2, $3, $4, $5, 'adjustment', $6, 'adjustment', $7, $8, $9, $10, $8)
                "#,
            )
            .bind(move_id)
            .bind(tenant_id)
            .bind(line.product_id)
            .bind(source_loc)
            .bind(dest_loc)
            .bind(move_qty)
            .bind(adjustment_id)
            .bind(posted_at)
            .bind(&line.reason_notes)
            .bind(line.lot_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create stock move: {}", e)))?;

            // Update line with stock move reference
            sqlx::query(
                r#"
                UPDATE adjustment_lines
                SET posted_stock_move_id = $1
                WHERE tenant_id = $2 AND adjustment_line_id = $3
                "#,
            )
            .bind(move_id)
            .bind(tenant_id)
            .bind(line.adjustment_line_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to update adjustment line: {}", e))
            })?;

            // Update inventory levels
            // For increase: add quantity
            // For decrease: subtract quantity
            let location_id = line.location_id.unwrap_or(doc_row.warehouse_id);
            let qty_change = match line.adjustment_type.as_str() {
                "increase" => line.qty,
                "decrease" => -line.qty,
                _ => -line.qty,
            };

            sqlx::query(
                r#"
                UPDATE inventory_levels
                SET available_quantity = available_quantity + $1,
                    updated_at = NOW()
                WHERE tenant_id = $2
                  AND product_id = $3
                  AND location_id = $4
                "#,
            )
            .bind(qty_change)
            .bind(tenant_id)
            .bind(line.product_id)
            .bind(location_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to update inventory levels: {}", e))
            })?;
        }

        // Update document status to posted
        let updated_row = sqlx::query_as::<_, AdjustmentDocumentRow>(
            r#"
            UPDATE adjustment_documents
            SET status = 'posted', posted_by = $1, posted_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND adjustment_id = $4
            RETURNING
                adjustment_id, tenant_id, reference, status, warehouse_id, notes,
                created_by, posted_by, posted_at, cancelled_by, cancelled_at,
                created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(posted_at)
        .bind(tenant_id)
        .bind(adjustment_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update adjustment document status: {}", e))
        })?;

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(AdjustmentDocumentResponse {
            adjustment: updated_row.into(),
        })
    }

    async fn cancel_adjustment(
        &self,
        tenant_id: Uuid,
        adjustment_id: Uuid,
        user_id: Uuid,
    ) -> Result<AdjustmentDocumentResponse, AppError> {
        // Use transaction with FOR UPDATE lock to prevent race conditions
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;

        // Verify document exists and lock the row
        let doc_row = sqlx::query_as::<_, AdjustmentDocumentRow>(
            r#"
            SELECT
                adjustment_id, tenant_id, reference, status, warehouse_id, notes,
                created_by, posted_by, posted_at, cancelled_by, cancelled_at,
                created_at, updated_at
            FROM adjustment_documents
            WHERE tenant_id = $1 AND adjustment_id = $2
            FOR UPDATE
            "#,
        )
        .bind(tenant_id)
        .bind(adjustment_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch adjustment document: {}", e))
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!("Adjustment document {} not found", adjustment_id))
        })?;

        // Idempotency: if already cancelled, return success
        if doc_row.status == "cancelled" {
            // Rollback is automatic on drop, but let's be explicit
            tx.rollback().await.ok();
            return Ok(AdjustmentDocumentResponse {
                adjustment: doc_row.into(),
            });
        }

        if doc_row.status != "draft" {
            tx.rollback().await.ok();
            return Err(AppError::ValidationError(
                "Only draft adjustment documents can be cancelled".to_string(),
            ));
        }

        let cancelled_at = Utc::now();

        // Update status to cancelled
        let updated_row = sqlx::query_as::<_, AdjustmentDocumentRow>(
            r#"
            UPDATE adjustment_documents
            SET status = 'cancelled', cancelled_by = $1, cancelled_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND adjustment_id = $4
            RETURNING
                adjustment_id, tenant_id, reference, status, warehouse_id, notes,
                created_by, posted_by, posted_at, cancelled_by, cancelled_at,
                created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(cancelled_at)
        .bind(tenant_id)
        .bind(adjustment_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to cancel adjustment document: {}", e))
        })?;

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(AdjustmentDocumentResponse {
            adjustment: updated_row.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status() {
        assert_eq!(PgAdjustmentService::parse_status("draft"), AdjustmentStatus::Draft);
        assert_eq!(PgAdjustmentService::parse_status("posted"), AdjustmentStatus::Posted);
        assert_eq!(PgAdjustmentService::parse_status("cancelled"), AdjustmentStatus::Cancelled);
        assert_eq!(PgAdjustmentService::parse_status("unknown"), AdjustmentStatus::Draft);
    }

    #[test]
    fn test_parse_adjustment_type() {
        assert_eq!(
            PgAdjustmentService::parse_adjustment_type("increase"),
            AdjustmentType::Increase
        );
        assert_eq!(
            PgAdjustmentService::parse_adjustment_type("decrease"),
            AdjustmentType::Decrease
        );
        assert_eq!(PgAdjustmentService::parse_adjustment_type("unknown"), AdjustmentType::Decrease);
    }

    #[test]
    fn test_parse_reason_code() {
        assert_eq!(
            PgAdjustmentService::parse_reason_code("damaged"),
            AdjustmentReasonCode::Damaged
        );
        assert_eq!(PgAdjustmentService::parse_reason_code("lost"), AdjustmentReasonCode::Lost);
        assert_eq!(PgAdjustmentService::parse_reason_code("found"), AdjustmentReasonCode::Found);
        assert_eq!(
            PgAdjustmentService::parse_reason_code("count_correction"),
            AdjustmentReasonCode::CountCorrection
        );
        assert_eq!(
            PgAdjustmentService::parse_reason_code("system_correction"),
            AdjustmentReasonCode::SystemCorrection
        );
        assert_eq!(
            PgAdjustmentService::parse_reason_code("expired"),
            AdjustmentReasonCode::Expired
        );
        assert_eq!(PgAdjustmentService::parse_reason_code("theft"), AdjustmentReasonCode::Theft);
        assert_eq!(
            PgAdjustmentService::parse_reason_code("promotion"),
            AdjustmentReasonCode::Promotion
        );
        assert_eq!(
            PgAdjustmentService::parse_reason_code("return_to_stock"),
            AdjustmentReasonCode::ReturnToStock
        );
        assert_eq!(PgAdjustmentService::parse_reason_code("other"), AdjustmentReasonCode::Other);
        assert_eq!(PgAdjustmentService::parse_reason_code("unknown"), AdjustmentReasonCode::Other);
    }
}
