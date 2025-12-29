//! Cycle Counting Service Implementation
//!
//! PostgreSQL implementation of the CycleCountingService trait.
//! Provides cycle counting workflow with as-of snapshot semantics.
//!
//! Uses runtime query building instead of sqlx macros to avoid
//! offline mode compilation issues.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::ops::DerefMut;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::dto::cycle_count::{
    calculate_summary, CountType, CreateCycleCountRequest, CycleCountLine, CycleCountLineStatus,
    CycleCountListQuery, CycleCountListResponse, CycleCountResponse, CycleCountSession,
    CycleCountStatus, CycleCountWithLinesResponse, GenerateLinesRequest, LineAdjustment,
    ReconcileRequest, ReconcileResponse, SkipLinesRequest, SubmitCountsRequest,
};
use inventory_service_core::models::CreateStockMoveRequest;
use inventory_service_core::services::cycle_count::CycleCountingService;
use shared_error::AppError;

use crate::repositories::stock::{PgInventoryLevelRepository, PgStockMoveRepository};

/// PostgreSQL implementation of CycleCountingService
pub struct PgCycleCountingService {
    pool: Arc<PgPool>,
    stock_move_repo: Arc<PgStockMoveRepository>,
    inventory_repo: Arc<PgInventoryLevelRepository>,
}

impl PgCycleCountingService {
    /// Create a new cycle counting service instance
    pub fn new(
        pool: Arc<PgPool>,
        stock_move_repo: Arc<PgStockMoveRepository>,
        inventory_repo: Arc<PgInventoryLevelRepository>,
    ) -> Self {
        Self {
            pool,
            stock_move_repo,
            inventory_repo,
        }
    }

    /// Convert database status string to CycleCountStatus enum
    fn string_to_status(s: &str) -> Result<CycleCountStatus, AppError> {
        match s {
            "draft" => Ok(CycleCountStatus::Draft),
            "in_progress" => Ok(CycleCountStatus::InProgress),
            "ready_to_reconcile" => Ok(CycleCountStatus::ReadyToReconcile),
            "reconciled" => Ok(CycleCountStatus::Reconciled),
            "cancelled" => Ok(CycleCountStatus::Cancelled),
            _ => Err(AppError::DataCorruption(format!("Unknown cycle count status: {}", s))),
        }
    }

    /// Convert CycleCountStatus enum to database string
    fn status_to_string(status: &CycleCountStatus) -> &'static str {
        match status {
            CycleCountStatus::Draft => "draft",
            CycleCountStatus::InProgress => "in_progress",
            CycleCountStatus::ReadyToReconcile => "ready_to_reconcile",
            CycleCountStatus::Reconciled => "reconciled",
            CycleCountStatus::Cancelled => "cancelled",
        }
    }

    /// Convert database line status string to CycleCountLineStatus enum
    fn string_to_line_status(s: &str) -> Result<CycleCountLineStatus, AppError> {
        match s {
            "open" => Ok(CycleCountLineStatus::Open),
            "counted" => Ok(CycleCountLineStatus::Counted),
            "skipped" => Ok(CycleCountLineStatus::Skipped),
            _ => Err(AppError::DataCorruption(format!("Unknown line status: {}", s))),
        }
    }

    /// Convert database count type string to CountType enum
    fn string_to_count_type(s: &str) -> Result<CountType, AppError> {
        match s {
            "full" => Ok(CountType::Full),
            "cycle" => Ok(CountType::Cycle),
            "spot" => Ok(CountType::Spot),
            _ => Err(AppError::DataCorruption(format!("Unknown count type: {}", s))),
        }
    }

    /// Convert CountType enum to database string
    fn count_type_to_string(count_type: &CountType) -> &'static str {
        match count_type {
            CountType::Full => "full",
            CountType::Cycle => "cycle",
            CountType::Spot => "spot",
        }
    }

    /// Generate a unique session number
    fn generate_session_number() -> String {
        format!("CC-{}", Utc::now().format("%Y%m%d-%H%M%S"))
    }

    /// Get cycle count lines for a session
    async fn get_lines(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
    ) -> Result<Vec<CycleCountLine>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT
                l.line_id,
                l.tenant_id,
                l.stock_take_id as cycle_count_id,
                l.product_id,
                l.variant_id,
                l.location_id,
                l.lot_id,
                l.serial_id,
                l.expected_quantity as expected_qty,
                l.actual_quantity as counted_qty,
                l.difference_quantity as difference_qty,
                l.line_status,
                l.counted_by,
                l.counted_at,
                l.notes,
                l.created_at,
                l.updated_at,
                s.warehouse_id as session_warehouse_id
            FROM stock_take_lines l
            JOIN stock_takes s ON l.tenant_id = s.tenant_id AND l.stock_take_id = s.stock_take_id
            WHERE l.tenant_id = $1 AND l.stock_take_id = $2 AND l.deleted_at IS NULL
            ORDER BY l.created_at ASC
            "#,
        )
        .bind(tenant_id)
        .bind(cycle_count_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch lines: {}", e)))?;

        let mut lines = Vec::with_capacity(rows.len());
        for row in rows {
            let line_status_str: String = row.get("line_status");
            let location_id_opt: Option<Uuid> = row.get("location_id");
            let warehouse_id: Uuid = row.get("session_warehouse_id");

            lines.push(CycleCountLine {
                line_id: row.get("line_id"),
                tenant_id: row.get("tenant_id"),
                cycle_count_id: row.get("cycle_count_id"),
                product_id: row.get("product_id"),
                variant_id: row.get("variant_id"),
                location_id: location_id_opt.unwrap_or(warehouse_id),
                lot_id: row.get("lot_id"),
                serial_id: row.get("serial_id"),
                expected_qty: row.get("expected_qty"),
                counted_qty: row.get("counted_qty"),
                difference_qty: row.get("difference_qty"),
                line_status: Self::string_to_line_status(&line_status_str)?,
                counted_by: row.get("counted_by"),
                counted_at: row.get("counted_at"),
                notes: row.get("notes"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(lines)
    }

    /// Get session by ID (internal helper)
    async fn get_session_internal(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
    ) -> Result<CycleCountSession, AppError> {
        let row = sqlx::query(
            r#"
            SELECT
                stock_take_id as cycle_count_id,
                tenant_id,
                stock_take_number as session_number,
                schedule_id,
                warehouse_id,
                location_id,
                as_of,
                status,
                count_type,
                notes,
                created_by,
                closed_by,
                completed_at as closed_at,
                adjustment_id,
                created_at,
                updated_at
            FROM stock_takes
            WHERE tenant_id = $1 AND stock_take_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(cycle_count_id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch session: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Cycle count session not found".to_string()))?;

        let status_str: String = row.get("status");
        let count_type_str: String = row.get("count_type");
        let as_of_opt: Option<DateTime<Utc>> = row.get("as_of");

        Ok(CycleCountSession {
            cycle_count_id: row.get("cycle_count_id"),
            tenant_id: row.get("tenant_id"),
            session_number: row.get("session_number"),
            schedule_id: row.get("schedule_id"),
            warehouse_id: row.get("warehouse_id"),
            location_id: row.get("location_id"),
            as_of: as_of_opt.unwrap_or_else(Utc::now),
            status: Self::string_to_status(&status_str)?,
            count_type: Self::string_to_count_type(&count_type_str)?,
            notes: row.get("notes"),
            created_by: row.get("created_by"),
            closed_by: row.get("closed_by"),
            closed_at: row.get("closed_at"),
            adjustment_id: row.get("adjustment_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Update session status
    async fn update_session_status(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
        new_status: CycleCountStatus,
        user_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let status_str = Self::status_to_string(&new_status);

        let result = sqlx::query(
            r#"
            UPDATE stock_takes
            SET status = $1,
                closed_by = COALESCE($4, closed_by),
                completed_at = CASE WHEN $1 IN ('ready_to_reconcile', 'reconciled', 'cancelled') THEN NOW() ELSE completed_at END,
                updated_at = NOW()
            WHERE tenant_id = $2 AND stock_take_id = $3 AND deleted_at IS NULL
            "#,
        )
        .bind(status_str)
        .bind(tenant_id)
        .bind(cycle_count_id)
        .bind(user_id)
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update status: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Cycle count session not found".to_string()));
        }

        Ok(())
    }

    /// Check for stock movements after as_of timestamp
    async fn check_movements_after_as_of(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        location_id: Option<Uuid>,
        as_of: DateTime<Utc>,
        product_ids: &[Uuid],
    ) -> Result<bool, AppError> {
        if product_ids.is_empty() {
            return Ok(false);
        }

        let location = location_id.unwrap_or(warehouse_id);

        let row = sqlx::query(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM stock_moves
                WHERE tenant_id = $1
                  AND (source_location_id = $2 OR destination_location_id = $2)
                  AND product_id = ANY($3)
                  AND move_date > $4
            ) as exists
            "#,
        )
        .bind(tenant_id)
        .bind(location)
        .bind(product_ids)
        .bind(as_of)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to check movements: {}", e)))?;

        let exists: bool = row.get("exists");
        Ok(exists)
    }
}

#[async_trait]
impl CycleCountingService for PgCycleCountingService {
    async fn create_session(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateCycleCountRequest,
    ) -> Result<CycleCountResponse, AppError> {
        let cycle_count_id = Uuid::now_v7();
        let session_number = Self::generate_session_number();
        let as_of = request.as_of.unwrap_or_else(Utc::now);
        let count_type = request.count_type;
        let status = CycleCountStatus::Draft;

        sqlx::query(
            r#"
            INSERT INTO stock_takes (
                stock_take_id, tenant_id, stock_take_number, warehouse_id,
                location_id, schedule_id, as_of, status, count_type,
                notes, created_by, started_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
            "#,
        )
        .bind(cycle_count_id)
        .bind(tenant_id)
        .bind(&session_number)
        .bind(request.warehouse_id)
        .bind(request.location_id)
        .bind(request.schedule_id)
        .bind(as_of)
        .bind(Self::status_to_string(&status))
        .bind(Self::count_type_to_string(&count_type))
        .bind(&request.notes)
        .bind(user_id)
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create session: {}", e)))?;

        let session = self.get_session_internal(tenant_id, cycle_count_id).await?;

        Ok(CycleCountResponse {
            cycle_count: session,
        })
    }

    async fn get_session(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
    ) -> Result<CycleCountWithLinesResponse, AppError> {
        let session = self.get_session_internal(tenant_id, cycle_count_id).await?;
        let lines = self.get_lines(tenant_id, cycle_count_id).await?;
        let summary = calculate_summary(&lines);

        Ok(CycleCountWithLinesResponse {
            cycle_count: session,
            lines,
            summary,
        })
    }

    async fn list_sessions(
        &self,
        tenant_id: Uuid,
        query: CycleCountListQuery,
    ) -> Result<CycleCountListResponse, AppError> {
        let limit = query.limit.unwrap_or(50).clamp(1, 100) as i64;
        let page = query.page.unwrap_or(1).max(1) as i64;
        let offset = (page - 1) * limit;

        // Build dynamic WHERE clause
        let status_filter = query
            .status
            .as_ref()
            .map(|s| Self::status_to_string(s).to_string());
        let count_type_filter = query
            .count_type
            .as_ref()
            .map(|ct| Self::count_type_to_string(ct).to_string());

        let rows = sqlx::query(
            r#"
            SELECT
                stock_take_id as cycle_count_id,
                tenant_id,
                stock_take_number as session_number,
                schedule_id,
                warehouse_id,
                location_id,
                as_of,
                status,
                count_type,
                notes,
                created_by,
                closed_by,
                completed_at as closed_at,
                adjustment_id,
                created_at,
                updated_at
            FROM stock_takes
            WHERE tenant_id = $1
              AND deleted_at IS NULL
              AND ($2::uuid IS NULL OR warehouse_id = $2)
              AND ($3::text IS NULL OR status = $3)
              AND ($4::text IS NULL OR count_type = $4)
              AND ($5::timestamptz IS NULL OR created_at >= $5)
              AND ($6::timestamptz IS NULL OR created_at <= $6)
            ORDER BY created_at DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(tenant_id)
        .bind(query.warehouse_id)
        .bind(&status_filter)
        .bind(&count_type_filter)
        .bind(query.from_date)
        .bind(query.to_date)
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to list sessions: {}", e)))?;

        let mut cycle_counts = Vec::with_capacity(rows.len());
        for row in rows {
            let status_str: String = row.get("status");
            let count_type_str: String = row.get("count_type");
            let as_of_opt: Option<DateTime<Utc>> = row.get("as_of");

            cycle_counts.push(CycleCountSession {
                cycle_count_id: row.get("cycle_count_id"),
                tenant_id: row.get("tenant_id"),
                session_number: row.get("session_number"),
                schedule_id: row.get("schedule_id"),
                warehouse_id: row.get("warehouse_id"),
                location_id: row.get("location_id"),
                as_of: as_of_opt.unwrap_or_else(Utc::now),
                status: Self::string_to_status(&status_str)?,
                count_type: Self::string_to_count_type(&count_type_str)?,
                notes: row.get("notes"),
                created_by: row.get("created_by"),
                closed_by: row.get("closed_by"),
                closed_at: row.get("closed_at"),
                adjustment_id: row.get("adjustment_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        // Get total count
        let count_row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM stock_takes
            WHERE tenant_id = $1
              AND deleted_at IS NULL
              AND ($2::uuid IS NULL OR warehouse_id = $2)
              AND ($3::text IS NULL OR status = $3)
              AND ($4::text IS NULL OR count_type = $4)
              AND ($5::timestamptz IS NULL OR created_at >= $5)
              AND ($6::timestamptz IS NULL OR created_at <= $6)
            "#,
        )
        .bind(tenant_id)
        .bind(query.warehouse_id)
        .bind(&status_filter)
        .bind(&count_type_filter)
        .bind(query.from_date)
        .bind(query.to_date)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to count sessions: {}", e)))?;

        let total_count: i64 = count_row.get("count");

        Ok(CycleCountListResponse {
            cycle_counts,
            total_count: total_count as u64,
            page: page as u32,
            page_size: limit as u32,
        })
    }

    async fn generate_lines(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
        _user_id: Uuid,
        request: GenerateLinesRequest,
    ) -> Result<CycleCountWithLinesResponse, AppError> {
        // Verify session exists and is in correct status
        let session = self.get_session_internal(tenant_id, cycle_count_id).await?;

        if !session.status.allows_line_edits() {
            return Err(AppError::ValidationError(format!(
                "Cannot generate lines when session is in {} status",
                session.status
            )));
        }

        // Start transaction
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;

        // Optionally delete existing lines
        if request.replace_existing {
            sqlx::query(
                r#"
                DELETE FROM stock_take_lines
                WHERE tenant_id = $1 AND stock_take_id = $2
                "#,
            )
            .bind(tenant_id)
            .bind(cycle_count_id)
            .execute(tx.deref_mut())
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to delete lines: {}", e)))?;
        }

        // Generate lines from inventory levels as of the session's as_of timestamp
        // For MVP, we use current inventory levels (snapshot at as_of would require historical tracking)
        let location = session.location_id.unwrap_or(session.warehouse_id);

        sqlx::query(
            r#"
            INSERT INTO stock_take_lines (
                line_id, tenant_id, stock_take_id, product_id, location_id,
                expected_quantity, line_status
            )
            SELECT
                gen_random_uuid(),
                $1,
                $2,
                il.product_id,
                $3,
                il.available_quantity,
                'open'
            FROM inventory_levels il
            WHERE il.tenant_id = $1
              AND il.warehouse_id = $4
              AND il.deleted_at IS NULL
              AND il.available_quantity > 0
              AND ($5::uuid IS NULL OR il.product_id = $5)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(tenant_id)
        .bind(cycle_count_id)
        .bind(location)
        .bind(session.warehouse_id)
        .bind(request.product_id)
        .execute(tx.deref_mut())
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to generate lines: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit: {}", e)))?;

        // Return updated session with lines
        self.get_session(tenant_id, cycle_count_id).await
    }

    async fn submit_counts(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
        user_id: Uuid,
        request: SubmitCountsRequest,
    ) -> Result<CycleCountWithLinesResponse, AppError> {
        // Verify session exists and allows counting
        let session = self.get_session_internal(tenant_id, cycle_count_id).await?;

        if !session.status.allows_counting() {
            return Err(AppError::ValidationError(format!(
                "Cannot submit counts when session is in {} status",
                session.status
            )));
        }

        // Validate all count submissions
        for submission in &request.counts {
            if submission.counted_qty < 0 {
                return Err(AppError::ValidationError(format!(
                    "Counted quantity cannot be negative for line {}",
                    submission.line_id
                )));
            }
        }

        // Start transaction
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;

        // Update counts for each line
        for submission in &request.counts {
            let result = sqlx::query(
                r#"
                UPDATE stock_take_lines
                SET actual_quantity = $4,
                    difference_quantity = $4 - expected_quantity,
                    line_status = 'counted',
                    counted_by = $5,
                    counted_at = NOW(),
                    notes = COALESCE($6, notes),
                    updated_at = NOW()
                WHERE tenant_id = $1 AND stock_take_id = $2 AND line_id = $3 AND deleted_at IS NULL
                "#,
            )
            .bind(tenant_id)
            .bind(cycle_count_id)
            .bind(submission.line_id)
            .bind(submission.counted_qty)
            .bind(user_id)
            .bind(&submission.notes)
            .execute(tx.deref_mut())
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to update count: {}", e)))?;

            if result.rows_affected() == 0 {
                return Err(AppError::NotFound(format!(
                    "Line {} not found in this session",
                    submission.line_id
                )));
            }
        }

        // Transition from Draft to InProgress if needed
        if session.status == CycleCountStatus::Draft {
            sqlx::query(
                r#"
                UPDATE stock_takes
                SET status = 'in_progress', updated_at = NOW()
                WHERE tenant_id = $1 AND stock_take_id = $2 AND deleted_at IS NULL
                "#,
            )
            .bind(tenant_id)
            .bind(cycle_count_id)
            .execute(tx.deref_mut())
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to update status: {}", e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit: {}", e)))?;

        self.get_session(tenant_id, cycle_count_id).await
    }

    async fn skip_lines(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
        _user_id: Uuid,
        request: SkipLinesRequest,
    ) -> Result<CycleCountWithLinesResponse, AppError> {
        // Verify session exists and allows counting
        let session = self.get_session_internal(tenant_id, cycle_count_id).await?;

        if !session.status.allows_counting() {
            return Err(AppError::ValidationError(format!(
                "Cannot skip lines when session is in {} status",
                session.status
            )));
        }

        // Update status for all specified lines
        let result = sqlx::query(
            r#"
            UPDATE stock_take_lines
            SET line_status = 'skipped',
                notes = COALESCE($4, notes),
                updated_at = NOW()
            WHERE tenant_id = $1 AND stock_take_id = $2 AND line_id = ANY($3) AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(cycle_count_id)
        .bind(&request.line_ids)
        .bind(&request.reason)
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to skip lines: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("No matching lines found to skip".to_string()));
        }

        self.get_session(tenant_id, cycle_count_id).await
    }

    async fn close_session(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
        user_id: Uuid,
    ) -> Result<CycleCountResponse, AppError> {
        // Verify session exists and is in InProgress status
        let session = self.get_session_internal(tenant_id, cycle_count_id).await?;

        if session.status != CycleCountStatus::InProgress {
            return Err(AppError::ValidationError(format!(
                "Can only close sessions in InProgress status, current status: {}",
                session.status
            )));
        }

        // Check for uncounted lines (not skipped)
        let count_row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM stock_take_lines
            WHERE tenant_id = $1 AND stock_take_id = $2 AND line_status = 'open' AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(cycle_count_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to check uncounted lines: {}", e))
        })?;

        let uncounted_count: i64 = count_row.get("count");

        if uncounted_count > 0 {
            return Err(AppError::ValidationError(format!(
                "{} lines have not been counted or skipped",
                uncounted_count
            )));
        }

        // Update status to ReadyToReconcile
        self.update_session_status(
            tenant_id,
            cycle_count_id,
            CycleCountStatus::ReadyToReconcile,
            Some(user_id),
        )
        .await?;

        let session = self.get_session_internal(tenant_id, cycle_count_id).await?;

        Ok(CycleCountResponse {
            cycle_count: session,
        })
    }

    async fn reconcile(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
        user_id: Uuid,
        request: ReconcileRequest,
    ) -> Result<ReconcileResponse, AppError> {
        // Verify session exists
        let session = self.get_session_internal(tenant_id, cycle_count_id).await?;

        // Check if already reconciled (idempotency)
        if session.status == CycleCountStatus::Reconciled {
            return Ok(ReconcileResponse {
                cycle_count: session.clone(),
                adjustment_id: session.adjustment_id.unwrap_or_else(Uuid::now_v7),
                moves_created: 0,           // Already done
                lines_adjusted: Vec::new(), // Already processed
            });
        }

        // Verify session is ready for reconciliation
        if session.status != CycleCountStatus::ReadyToReconcile {
            return Err(AppError::ValidationError(format!(
                "Session must be in ReadyToReconcile status, current status: {}",
                session.status
            )));
        }

        // Get lines with variances
        let lines = self.get_lines(tenant_id, cycle_count_id).await?;
        let lines_with_variance: Vec<&CycleCountLine> = lines
            .iter()
            .filter(|l| {
                l.line_status == CycleCountLineStatus::Counted && l.difference_qty.unwrap_or(0) != 0
            })
            .collect();

        // Check for movements after as_of (if force is not set)
        if !request.force {
            let product_ids: Vec<Uuid> = lines_with_variance.iter().map(|l| l.product_id).collect();

            let has_movements = self
                .check_movements_after_as_of(
                    tenant_id,
                    session.warehouse_id,
                    session.location_id,
                    session.as_of,
                    &product_ids,
                )
                .await?;

            if has_movements {
                return Err(AppError::ValidationError(
                    "Stock movements detected after as_of timestamp. Use force=true to proceed anyway.".to_string()
                ));
            }
        }

        // Start transaction for reconciliation
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;

        // Generate a unique adjustment ID
        let adjustment_id = Uuid::now_v7();
        let mut line_adjustments = Vec::new();
        let mut moves_created = 0u32;

        // Create stock moves for each variance
        for line in &lines_with_variance {
            let difference = line.difference_qty.unwrap_or(0);

            // Create stock move for the adjustment
            let idempotency_key = format!("cc-{}-line-{}", cycle_count_id, line.line_id);
            let location = line.location_id;
            let stock_move = CreateStockMoveRequest {
                product_id: line.product_id,
                source_location_id: Some(location),
                destination_location_id: Some(location),
                move_type: "adjustment".to_string(),
                quantity: difference,
                unit_cost: None,
                reference_type: "cycle_count".to_string(),
                reference_id: cycle_count_id,
                idempotency_key,
                move_reason: Some(format!("Cycle count {} adjustment", session.session_number)),
                lot_serial_id: line.lot_id.or(line.serial_id),
                batch_info: None,
                metadata: None,
            };

            let (move_id, new_tx) = self
                .stock_move_repo
                .create_with_tx(tx, stock_move, tenant_id)
                .await?;
            tx = new_tx;
            moves_created += 1;

            // Update inventory level
            tx = self
                .inventory_repo
                .update_available_quantity_with_tx(
                    tx,
                    tenant_id,
                    session.warehouse_id,
                    line.product_id,
                    difference,
                )
                .await?;

            line_adjustments.push(LineAdjustment {
                line_id: line.line_id,
                product_id: line.product_id,
                location_id: line.location_id,
                expected_qty: line.expected_qty,
                counted_qty: line.counted_qty.unwrap_or(0),
                adjustment_qty: difference,
                stock_move_id: move_id,
            });
        }

        // Update session status to Reconciled
        sqlx::query(
            r#"
            UPDATE stock_takes
            SET status = 'reconciled',
                adjustment_id = $3,
                closed_by = $4,
                completed_at = NOW(),
                updated_at = NOW()
            WHERE tenant_id = $1 AND stock_take_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(cycle_count_id)
        .bind(adjustment_id)
        .bind(user_id)
        .execute(tx.deref_mut())
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update session: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit: {}", e)))?;

        // Fetch updated session
        let session = self.get_session_internal(tenant_id, cycle_count_id).await?;

        Ok(ReconcileResponse {
            cycle_count: session,
            adjustment_id,
            moves_created,
            lines_adjusted: line_adjustments,
        })
    }

    async fn cancel_session(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
        user_id: Uuid,
    ) -> Result<CycleCountResponse, AppError> {
        // Verify session exists
        let session = self.get_session_internal(tenant_id, cycle_count_id).await?;

        // Check if session can be cancelled
        if !session
            .status
            .can_transition_to(CycleCountStatus::Cancelled)
        {
            return Err(AppError::ValidationError(format!(
                "Cannot cancel session in {} status",
                session.status
            )));
        }

        // Update status to Cancelled
        self.update_session_status(
            tenant_id,
            cycle_count_id,
            CycleCountStatus::Cancelled,
            Some(user_id),
        )
        .await?;

        let session = self.get_session_internal(tenant_id, cycle_count_id).await?;

        Ok(CycleCountResponse {
            cycle_count: session,
        })
    }
}
