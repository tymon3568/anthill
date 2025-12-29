//! Scrap Management Service Implementation
//!
//! PostgreSQL implementation of the ScrapService trait for scrap management operations.
//! Follows the 3-crate pattern: api → infra → core → shared/*

use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::dto::scrap::{
    AddScrapLinesRequest, CreateScrapRequest, PostScrapRequest, ScrapDocument,
    ScrapDocumentResponse, ScrapDocumentWithLinesResponse, ScrapLine, ScrapListQuery,
    ScrapListResponse, ScrapReasonCode, ScrapStatus,
};
use inventory_service_core::services::scrap::ScrapService;
use shared_error::AppError;

/// PostgreSQL implementation of ScrapService
pub struct PgScrapService {
    pool: Arc<PgPool>,
}

impl PgScrapService {
    /// Create a new scrap service instance
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Convert database status string to ScrapStatus enum
    fn parse_status(status: &str) -> ScrapStatus {
        match status {
            "draft" => ScrapStatus::Draft,
            "posted" => ScrapStatus::Posted,
            "cancelled" => ScrapStatus::Cancelled,
            _ => ScrapStatus::Draft,
        }
    }

    /// Convert database reason_code string to ScrapReasonCode enum
    fn parse_reason_code(code: Option<&str>) -> Option<ScrapReasonCode> {
        code.map(|c| match c {
            "damaged" => ScrapReasonCode::Damaged,
            "expired" => ScrapReasonCode::Expired,
            "lost" => ScrapReasonCode::Lost,
            "quality_fail" => ScrapReasonCode::QualityFail,
            "obsolete" => ScrapReasonCode::Obsolete,
            _ => ScrapReasonCode::Other,
        })
    }
}

/// Helper struct for scrap document SQL results
#[derive(Debug, sqlx::FromRow)]
struct ScrapDocumentRow {
    scrap_id: Uuid,
    tenant_id: Uuid,
    reference: Option<String>,
    status: String,
    scrap_location_id: Uuid,
    notes: Option<String>,
    created_by: Option<Uuid>,
    posted_by: Option<Uuid>,
    posted_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<ScrapDocumentRow> for ScrapDocument {
    fn from(row: ScrapDocumentRow) -> Self {
        Self {
            scrap_id: row.scrap_id,
            tenant_id: row.tenant_id,
            reference: row.reference,
            status: PgScrapService::parse_status(&row.status),
            scrap_location_id: row.scrap_location_id,
            notes: row.notes,
            created_by: row.created_by,
            posted_by: row.posted_by,
            posted_at: row.posted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Helper struct for scrap line SQL results
#[derive(Debug, sqlx::FromRow)]
struct ScrapLineRow {
    scrap_line_id: Uuid,
    tenant_id: Uuid,
    scrap_id: Uuid,
    product_id: Uuid,
    variant_id: Option<Uuid>,
    source_location_id: Uuid,
    lot_id: Option<Uuid>,
    serial_id: Option<Uuid>,
    qty: i64,
    reason_code: Option<String>,
    reason: Option<String>,
    posted_stock_move_id: Option<Uuid>,
    created_at: chrono::DateTime<Utc>,
}

impl ScrapLineRow {
    fn into_scrap_line(self) -> ScrapLine {
        ScrapLine {
            scrap_line_id: self.scrap_line_id,
            tenant_id: self.tenant_id,
            scrap_id: self.scrap_id,
            product_id: self.product_id,
            variant_id: self.variant_id,
            source_location_id: self.source_location_id,
            lot_id: self.lot_id,
            serial_id: self.serial_id,
            qty: self.qty,
            reason_code: PgScrapService::parse_reason_code(self.reason_code.as_deref()),
            reason: self.reason,
            posted_stock_move_id: self.posted_stock_move_id,
            created_at: self.created_at,
        }
    }
}

#[async_trait]
impl ScrapService for PgScrapService {
    async fn create_scrap(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateScrapRequest,
    ) -> Result<ScrapDocumentResponse, AppError> {
        let scrap_id = Uuid::now_v7();

        let row = sqlx::query_as::<_, ScrapDocumentRow>(
            r#"
            INSERT INTO scrap_documents (
                tenant_id, scrap_id, reference, status, scrap_location_id, notes, created_by
            )
            VALUES ($1, $2, $3, 'draft', $4, $5, $6)
            RETURNING
                scrap_id, tenant_id, reference, status, scrap_location_id, notes,
                created_by, posted_by, posted_at, created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(scrap_id)
        .bind(&request.reference)
        .bind(request.scrap_location_id)
        .bind(&request.notes)
        .bind(user_id)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create scrap document: {}", e)))?;

        Ok(ScrapDocumentResponse { scrap: row.into() })
    }

    async fn get_scrap(
        &self,
        tenant_id: Uuid,
        scrap_id: Uuid,
    ) -> Result<ScrapDocumentWithLinesResponse, AppError> {
        // Fetch document
        let doc_row = sqlx::query_as::<_, ScrapDocumentRow>(
            r#"
            SELECT
                scrap_id, tenant_id, reference, status, scrap_location_id, notes,
                created_by, posted_by, posted_at, created_at, updated_at
            FROM scrap_documents
            WHERE tenant_id = $1 AND scrap_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(scrap_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch scrap document: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Scrap document {} not found", scrap_id)))?;

        // Fetch lines
        let line_rows = sqlx::query_as::<_, ScrapLineRow>(
            r#"
            SELECT
                scrap_line_id, tenant_id, scrap_id, product_id, variant_id,
                source_location_id, lot_id, serial_id, qty, reason_code, reason,
                posted_stock_move_id, created_at
            FROM scrap_lines
            WHERE tenant_id = $1 AND scrap_id = $2
            ORDER BY created_at
            "#,
        )
        .bind(tenant_id)
        .bind(scrap_id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch scrap lines: {}", e)))?;

        let lines: Vec<ScrapLine> = line_rows.into_iter().map(|r| r.into_scrap_line()).collect();

        Ok(ScrapDocumentWithLinesResponse {
            scrap: doc_row.into(),
            lines,
        })
    }

    async fn list_scraps(
        &self,
        tenant_id: Uuid,
        query: ScrapListQuery,
    ) -> Result<ScrapListResponse, AppError> {
        let limit = query.limit.unwrap_or(50).min(100) as i64;
        let page = query.page.unwrap_or(1).max(1);
        let offset = ((page - 1) as i64) * limit;

        let status_str = query.status.map(|s| s.to_string());

        let rows = sqlx::query_as::<_, ScrapDocumentRow>(
            r#"
            SELECT
                scrap_id, tenant_id, reference, status, scrap_location_id, notes,
                created_by, posted_by, posted_at, created_at, updated_at
            FROM scrap_documents
            WHERE tenant_id = $1
              AND ($2::TEXT IS NULL OR status = $2)
              AND ($3::UUID IS NULL OR scrap_location_id = $3)
              AND ($4::TIMESTAMPTZ IS NULL OR created_at >= $4)
              AND ($5::TIMESTAMPTZ IS NULL OR created_at <= $5)
            ORDER BY created_at DESC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(tenant_id)
        .bind(&status_str)
        .bind(query.warehouse_id)
        .bind(query.from_date)
        .bind(query.to_date)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to list scrap documents: {}", e)))?;

        // Count total
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM scrap_documents
            WHERE tenant_id = $1
              AND ($2::TEXT IS NULL OR status = $2)
              AND ($3::UUID IS NULL OR scrap_location_id = $3)
              AND ($4::TIMESTAMPTZ IS NULL OR created_at >= $4)
              AND ($5::TIMESTAMPTZ IS NULL OR created_at <= $5)
            "#,
        )
        .bind(tenant_id)
        .bind(&status_str)
        .bind(query.warehouse_id)
        .bind(query.from_date)
        .bind(query.to_date)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to count scrap documents: {}", e)))?;

        let scraps: Vec<ScrapDocument> = rows.into_iter().map(|r| r.into()).collect();

        Ok(ScrapListResponse {
            scraps,
            total_count: count.0 as u64,
            page,
            page_size: limit as u32,
        })
    }

    async fn add_lines(
        &self,
        tenant_id: Uuid,
        scrap_id: Uuid,
        _user_id: Uuid,
        request: AddScrapLinesRequest,
    ) -> Result<ScrapDocumentWithLinesResponse, AppError> {
        // Verify document exists and is in draft status
        let doc_row = sqlx::query_as::<_, ScrapDocumentRow>(
            r#"
            SELECT
                scrap_id, tenant_id, reference, status, scrap_location_id, notes,
                created_by, posted_by, posted_at, created_at, updated_at
            FROM scrap_documents
            WHERE tenant_id = $1 AND scrap_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(scrap_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch scrap document: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Scrap document {} not found", scrap_id)))?;

        if doc_row.status != "draft" {
            return Err(AppError::ValidationError(
                "Lines can only be added to draft scrap documents".to_string(),
            ));
        }

        // Delete existing lines and add new ones in a transaction
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;

        // Delete existing lines
        sqlx::query(
            r#"
            DELETE FROM scrap_lines
            WHERE tenant_id = $1 AND scrap_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(scrap_id)
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
            let reason_code_str = line.reason_code.as_ref().map(|rc| rc.to_string());

            sqlx::query(
                r#"
                INSERT INTO scrap_lines (
                    tenant_id, scrap_line_id, scrap_id, product_id, variant_id,
                    source_location_id, lot_id, serial_id, qty, reason_code, reason
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                "#,
            )
            .bind(tenant_id)
            .bind(line_id)
            .bind(scrap_id)
            .bind(line.product_id)
            .bind(line.variant_id)
            .bind(line.source_location_id)
            .bind(line.lot_id)
            .bind(line.serial_id)
            .bind(line.qty)
            .bind(&reason_code_str)
            .bind(&line.reason)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to insert scrap line: {}", e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        // Return updated document with lines
        self.get_scrap(tenant_id, scrap_id).await
    }

    async fn post_scrap(
        &self,
        tenant_id: Uuid,
        scrap_id: Uuid,
        user_id: Uuid,
        _request: PostScrapRequest,
    ) -> Result<ScrapDocumentResponse, AppError> {
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;

        // Lock and verify document is in draft status
        let doc_row = sqlx::query_as::<_, ScrapDocumentRow>(
            r#"
            SELECT
                scrap_id, tenant_id, reference, status, scrap_location_id, notes,
                created_by, posted_by, posted_at, created_at, updated_at
            FROM scrap_documents
            WHERE tenant_id = $1 AND scrap_id = $2
            FOR UPDATE
            "#,
        )
        .bind(tenant_id)
        .bind(scrap_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch scrap document: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Scrap document {} not found", scrap_id)))?;

        // Idempotency check: if already posted, return success
        if doc_row.status == "posted" {
            return Ok(ScrapDocumentResponse {
                scrap: doc_row.into(),
            });
        }

        if doc_row.status != "draft" {
            return Err(AppError::ValidationError(
                "Only draft scrap documents can be posted".to_string(),
            ));
        }

        // Fetch lines
        let line_rows = sqlx::query_as::<_, ScrapLineRow>(
            r#"
            SELECT
                scrap_line_id, tenant_id, scrap_id, product_id, variant_id,
                source_location_id, lot_id, serial_id, qty, reason_code, reason,
                posted_stock_move_id, created_at
            FROM scrap_lines
            WHERE tenant_id = $1 AND scrap_id = $2
            FOR UPDATE
            "#,
        )
        .bind(tenant_id)
        .bind(scrap_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch scrap lines: {}", e)))?;

        if line_rows.is_empty() {
            return Err(AppError::ValidationError(
                "Cannot post scrap document with no lines".to_string(),
            ));
        }

        let posted_at = Utc::now();

        // Create stock moves for each line
        for line in &line_rows {
            let move_id = Uuid::now_v7();

            // Create stock move (negative quantity from source to scrap location)
            sqlx::query(
                r#"
                INSERT INTO stock_moves (
                    move_id, tenant_id, product_id, source_location_id, destination_location_id,
                    move_type, quantity, reference_type, reference_id, move_date,
                    move_reason, lot_serial_id, created_at
                )
                VALUES ($1, $2, $3, $4, $5, 'scrap', $6, 'scrap', $7, $8, $9, $10, $8)
                "#,
            )
            .bind(move_id)
            .bind(tenant_id)
            .bind(line.product_id)
            .bind(line.source_location_id)
            .bind(doc_row.scrap_location_id)
            .bind(-line.qty) // Negative to decrease source
            .bind(scrap_id)
            .bind(posted_at)
            .bind(&line.reason)
            .bind(line.lot_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create stock move: {}", e)))?;

            // Update line with stock move reference
            sqlx::query(
                r#"
                UPDATE scrap_lines
                SET posted_stock_move_id = $1
                WHERE tenant_id = $2 AND scrap_line_id = $3
                "#,
            )
            .bind(move_id)
            .bind(tenant_id)
            .bind(line.scrap_line_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to update scrap line: {}", e)))?;

            // Update inventory levels (decrease available quantity at source)
            sqlx::query(
                r#"
                UPDATE inventory_levels
                SET available_quantity = available_quantity - $1,
                    updated_at = NOW()
                WHERE tenant_id = $2
                  AND product_id = $3
                  AND warehouse_id = $4
                  AND ($5::UUID IS NULL OR lot_serial_id = $5)
                "#,
            )
            .bind(line.qty)
            .bind(tenant_id)
            .bind(line.product_id)
            .bind(line.source_location_id)
            .bind(line.lot_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to update inventory levels: {}", e))
            })?;
        }

        // Update document status to posted
        let updated_row = sqlx::query_as::<_, ScrapDocumentRow>(
            r#"
            UPDATE scrap_documents
            SET status = 'posted', posted_by = $1, posted_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND scrap_id = $4
            RETURNING
                scrap_id, tenant_id, reference, status, scrap_location_id, notes,
                created_by, posted_by, posted_at, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(posted_at)
        .bind(tenant_id)
        .bind(scrap_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update scrap document status: {}", e))
        })?;

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(ScrapDocumentResponse {
            scrap: updated_row.into(),
        })
    }

    async fn cancel_scrap(
        &self,
        tenant_id: Uuid,
        scrap_id: Uuid,
        _user_id: Uuid,
    ) -> Result<ScrapDocumentResponse, AppError> {
        // Verify document exists and is in draft status
        let doc_row = sqlx::query_as::<_, ScrapDocumentRow>(
            r#"
            SELECT
                scrap_id, tenant_id, reference, status, scrap_location_id, notes,
                created_by, posted_by, posted_at, created_at, updated_at
            FROM scrap_documents
            WHERE tenant_id = $1 AND scrap_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(scrap_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch scrap document: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Scrap document {} not found", scrap_id)))?;

        // Idempotency: if already cancelled, return success
        if doc_row.status == "cancelled" {
            return Ok(ScrapDocumentResponse {
                scrap: doc_row.into(),
            });
        }

        if doc_row.status != "draft" {
            return Err(AppError::ValidationError(
                "Only draft scrap documents can be cancelled".to_string(),
            ));
        }

        // Update status to cancelled
        let updated_row = sqlx::query_as::<_, ScrapDocumentRow>(
            r#"
            UPDATE scrap_documents
            SET status = 'cancelled', updated_at = NOW()
            WHERE tenant_id = $1 AND scrap_id = $2
            RETURNING
                scrap_id, tenant_id, reference, status, scrap_location_id, notes,
                created_by, posted_by, posted_at, created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(scrap_id)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to cancel scrap document: {}", e)))?;

        Ok(ScrapDocumentResponse {
            scrap: updated_row.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status() {
        assert_eq!(PgScrapService::parse_status("draft"), ScrapStatus::Draft);
        assert_eq!(PgScrapService::parse_status("posted"), ScrapStatus::Posted);
        assert_eq!(PgScrapService::parse_status("cancelled"), ScrapStatus::Cancelled);
        assert_eq!(PgScrapService::parse_status("unknown"), ScrapStatus::Draft);
    }

    #[test]
    fn test_parse_reason_code() {
        assert_eq!(
            PgScrapService::parse_reason_code(Some("damaged")),
            Some(ScrapReasonCode::Damaged)
        );
        assert_eq!(
            PgScrapService::parse_reason_code(Some("expired")),
            Some(ScrapReasonCode::Expired)
        );
        assert_eq!(PgScrapService::parse_reason_code(Some("lost")), Some(ScrapReasonCode::Lost));
        assert_eq!(
            PgScrapService::parse_reason_code(Some("quality_fail")),
            Some(ScrapReasonCode::QualityFail)
        );
        assert_eq!(
            PgScrapService::parse_reason_code(Some("obsolete")),
            Some(ScrapReasonCode::Obsolete)
        );
        assert_eq!(PgScrapService::parse_reason_code(Some("other")), Some(ScrapReasonCode::Other));
        assert_eq!(
            PgScrapService::parse_reason_code(Some("unknown")),
            Some(ScrapReasonCode::Other)
        );
        assert_eq!(PgScrapService::parse_reason_code(None), None);
    }
}
