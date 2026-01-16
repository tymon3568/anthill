//! Landed Cost Repository implementations.
//!
//! PostgreSQL implementations of the landed cost repository traits.

use async_trait::async_trait;
use chrono::Utc;
use shared_error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

use inventory_service_core::domains::inventory::landed_cost::{
    AllocationMethod, LandedCostAllocation, LandedCostDocument, LandedCostLine, LandedCostStatus,
};
use inventory_service_core::repositories::landed_cost::{
    LandedCostAllocationRepository, LandedCostDocumentRepository, LandedCostLineRepository,
};

/// PostgreSQL implementation of `LandedCostDocumentRepository`.
pub struct LandedCostDocumentRepositoryImpl {
    pool: PgPool,
}

impl LandedCostDocumentRepositoryImpl {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn status_to_string(status: &LandedCostStatus) -> &'static str {
        status.as_str()
    }

    fn string_to_status(s: &str) -> LandedCostStatus {
        LandedCostStatus::parse(s).unwrap_or(LandedCostStatus::Draft)
    }

    fn method_to_string(method: &AllocationMethod) -> &'static str {
        method.as_str()
    }

    fn string_to_method(s: &str) -> AllocationMethod {
        AllocationMethod::parse(s).unwrap_or(AllocationMethod::ByValue)
    }
}

#[async_trait]
impl LandedCostDocumentRepository for LandedCostDocumentRepositoryImpl {
    async fn create(
        &self,
        tenant_id: Uuid,
        receipt_id: Uuid,
        document_number: String,
        reference_number: Option<String>,
        allocation_method: AllocationMethod,
        currency_code: String,
        notes: Option<String>,
        created_by: Uuid,
    ) -> Result<LandedCostDocument, AppError> {
        let document_id = Uuid::now_v7();
        let now = Utc::now();
        let method_str = Self::method_to_string(&allocation_method);

        let row = sqlx::query!(
            r#"
            INSERT INTO landed_cost_documents (
                document_id, tenant_id, document_number, reference_number,
                status, receipt_id, total_cost_amount, currency_code,
                allocation_method, document_date, notes, created_by,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, 'draft', $5, 0, $6, $7, $8, $9, $10, $11, $12)
            RETURNING
                document_id, tenant_id, document_number, reference_number,
                status, receipt_id, total_cost_amount, currency_code,
                allocation_method, document_date, posted_at, notes,
                created_by, created_at, updated_at
            "#,
            document_id,
            tenant_id,
            document_number,
            reference_number,
            receipt_id,
            currency_code,
            method_str,
            now,
            notes,
            created_by,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to create landed cost document: {e}"))
        })?;

        Ok(LandedCostDocument {
            document_id: row.document_id,
            tenant_id: row.tenant_id,
            document_number: row.document_number,
            reference_number: row.reference_number,
            status: Self::string_to_status(&row.status),
            receipt_id: row.receipt_id,
            total_cost_amount: row.total_cost_amount,
            currency_code: row.currency_code,
            allocation_method: Self::string_to_method(&row.allocation_method),
            document_date: row.document_date,
            posted_at: row.posted_at,
            notes: row.notes,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<Option<LandedCostDocument>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT
                document_id, tenant_id, document_number, reference_number,
                status, receipt_id, total_cost_amount, currency_code,
                allocation_method, document_date, posted_at, notes,
                created_by, created_at, updated_at
            FROM landed_cost_documents
            WHERE tenant_id = $1 AND document_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            document_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to find landed cost document: {e}"))
        })?;

        Ok(row.map(|r| LandedCostDocument {
            document_id: r.document_id,
            tenant_id: r.tenant_id,
            document_number: r.document_number,
            reference_number: r.reference_number,
            status: Self::string_to_status(&r.status),
            receipt_id: r.receipt_id,
            total_cost_amount: r.total_cost_amount,
            currency_code: r.currency_code,
            allocation_method: Self::string_to_method(&r.allocation_method),
            document_date: r.document_date,
            posted_at: r.posted_at,
            notes: r.notes,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn find_by_receipt_id(
        &self,
        tenant_id: Uuid,
        receipt_id: Uuid,
    ) -> Result<Vec<LandedCostDocument>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                document_id, tenant_id, document_number, reference_number,
                status, receipt_id, total_cost_amount, currency_code,
                allocation_method, document_date, posted_at, notes,
                created_by, created_at, updated_at
            FROM landed_cost_documents
            WHERE tenant_id = $1 AND receipt_id = $2 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
            tenant_id,
            receipt_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to find landed cost documents: {e}"))
        })?;

        Ok(rows
            .into_iter()
            .map(|r| LandedCostDocument {
                document_id: r.document_id,
                tenant_id: r.tenant_id,
                document_number: r.document_number,
                reference_number: r.reference_number,
                status: Self::string_to_status(&r.status),
                receipt_id: r.receipt_id,
                total_cost_amount: r.total_cost_amount,
                currency_code: r.currency_code,
                allocation_method: Self::string_to_method(&r.allocation_method),
                document_date: r.document_date,
                posted_at: r.posted_at,
                notes: r.notes,
                created_by: r.created_by,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    async fn list(
        &self,
        tenant_id: Uuid,
        status: Option<LandedCostStatus>,
        receipt_id: Option<Uuid>,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<LandedCostDocument>, i64), AppError> {
        let offset = (page - 1) * page_size;
        let status_str = status.map(|s| s.as_str().to_string());

        // Count query
        let count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM landed_cost_documents
            WHERE tenant_id = $1
              AND deleted_at IS NULL
              AND ($2::TEXT IS NULL OR status = $2)
              AND ($3::UUID IS NULL OR receipt_id = $3)
            "#,
            tenant_id,
            status_str,
            receipt_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to count landed cost documents: {e}"))
        })?;

        // Data query
        let rows = sqlx::query!(
            r#"
            SELECT
                document_id, tenant_id, document_number, reference_number,
                status, receipt_id, total_cost_amount, currency_code,
                allocation_method, document_date, posted_at, notes,
                created_by, created_at, updated_at
            FROM landed_cost_documents
            WHERE tenant_id = $1
              AND deleted_at IS NULL
              AND ($2::TEXT IS NULL OR status = $2)
              AND ($3::UUID IS NULL OR receipt_id = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
            tenant_id,
            status_str,
            receipt_id,
            page_size as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to list landed cost documents: {e}"))
        })?;

        let documents = rows
            .into_iter()
            .map(|r| LandedCostDocument {
                document_id: r.document_id,
                tenant_id: r.tenant_id,
                document_number: r.document_number,
                reference_number: r.reference_number,
                status: Self::string_to_status(&r.status),
                receipt_id: r.receipt_id,
                total_cost_amount: r.total_cost_amount,
                currency_code: r.currency_code,
                allocation_method: Self::string_to_method(&r.allocation_method),
                document_date: r.document_date,
                posted_at: r.posted_at,
                notes: r.notes,
                created_by: r.created_by,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok((documents, count))
    }

    async fn update(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        reference_number: Option<String>,
        allocation_method: Option<AllocationMethod>,
        notes: Option<String>,
    ) -> Result<LandedCostDocument, AppError> {
        let method_str = allocation_method.map(|m| m.as_str().to_string());

        let row = sqlx::query!(
            r#"
            UPDATE landed_cost_documents
            SET
                reference_number = COALESCE($3, reference_number),
                allocation_method = COALESCE($4, allocation_method),
                notes = COALESCE($5, notes),
                updated_at = NOW()
            WHERE tenant_id = $1 AND document_id = $2 AND deleted_at IS NULL
            RETURNING
                document_id, tenant_id, document_number, reference_number,
                status, receipt_id, total_cost_amount, currency_code,
                allocation_method, document_date, posted_at, notes,
                created_by, created_at, updated_at
            "#,
            tenant_id,
            document_id,
            reference_number,
            method_str,
            notes
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update landed cost document: {e}"))
        })?
        .ok_or_else(|| AppError::NotFound("Landed cost document not found".to_string()))?;

        Ok(LandedCostDocument {
            document_id: row.document_id,
            tenant_id: row.tenant_id,
            document_number: row.document_number,
            reference_number: row.reference_number,
            status: Self::string_to_status(&row.status),
            receipt_id: row.receipt_id,
            total_cost_amount: row.total_cost_amount,
            currency_code: row.currency_code,
            allocation_method: Self::string_to_method(&row.allocation_method),
            document_date: row.document_date,
            posted_at: row.posted_at,
            notes: row.notes,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn update_status(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        status: LandedCostStatus,
    ) -> Result<LandedCostDocument, AppError> {
        let status_str = Self::status_to_string(&status);
        let posted_at = if status == LandedCostStatus::Posted {
            Some(Utc::now())
        } else {
            None
        };

        let row = sqlx::query!(
            r#"
            UPDATE landed_cost_documents
            SET
                status = $3,
                posted_at = COALESCE($4, posted_at),
                updated_at = NOW()
            WHERE tenant_id = $1 AND document_id = $2 AND deleted_at IS NULL
            RETURNING
                document_id, tenant_id, document_number, reference_number,
                status, receipt_id, total_cost_amount, currency_code,
                allocation_method, document_date, posted_at, notes,
                created_by, created_at, updated_at
            "#,
            tenant_id,
            document_id,
            status_str,
            posted_at
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update document status: {e}")))?
        .ok_or_else(|| AppError::NotFound("Landed cost document not found".to_string()))?;

        Ok(LandedCostDocument {
            document_id: row.document_id,
            tenant_id: row.tenant_id,
            document_number: row.document_number,
            reference_number: row.reference_number,
            status: Self::string_to_status(&row.status),
            receipt_id: row.receipt_id,
            total_cost_amount: row.total_cost_amount,
            currency_code: row.currency_code,
            allocation_method: Self::string_to_method(&row.allocation_method),
            document_date: row.document_date,
            posted_at: row.posted_at,
            notes: row.notes,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn update_total_cost(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        total_cost_amount: i64,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE landed_cost_documents
            SET total_cost_amount = $3, updated_at = NOW()
            WHERE tenant_id = $1 AND document_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            document_id,
            total_cost_amount
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update total cost: {e}")))?;

        Ok(())
    }

    async fn delete(&self, tenant_id: Uuid, document_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE landed_cost_documents
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND document_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            document_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to delete landed cost document: {e}"))
        })?;

        Ok(())
    }

    async fn generate_document_number(&self, _tenant_id: Uuid) -> Result<String, AppError> {
        let year = Utc::now().format("%Y");
        let seq: i64 = sqlx::query_scalar!("SELECT nextval('landed_cost_document_number_seq')")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to generate document number: {e}"))
            })?
            .unwrap_or(1);

        Ok(format!("LC-{year}-{seq:05}"))
    }
}

/// PostgreSQL implementation of `LandedCostLineRepository`.
pub struct LandedCostLineRepositoryImpl {
    pool: PgPool,
}

impl LandedCostLineRepositoryImpl {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LandedCostLineRepository for LandedCostLineRepositoryImpl {
    async fn create(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        cost_type: String,
        description: Option<String>,
        amount: i64,
        vendor_reference: Option<String>,
    ) -> Result<LandedCostLine, AppError> {
        let line_id = Uuid::now_v7();
        let now = Utc::now();

        let row = sqlx::query!(
            r#"
            INSERT INTO landed_cost_lines (
                line_id, tenant_id, document_id, cost_type,
                description, amount, vendor_reference, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING
                line_id, tenant_id, document_id, cost_type,
                description, amount, vendor_reference, created_at, updated_at
            "#,
            line_id,
            tenant_id,
            document_id,
            cost_type,
            description,
            amount,
            vendor_reference,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create landed cost line: {e}")))?;

        Ok(LandedCostLine {
            line_id: row.line_id,
            tenant_id: row.tenant_id,
            document_id: row.document_id,
            cost_type: row.cost_type,
            description: row.description,
            amount: row.amount,
            vendor_reference: row.vendor_reference,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn find_by_document_id(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<Vec<LandedCostLine>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                line_id, tenant_id, document_id, cost_type,
                description, amount, vendor_reference, created_at, updated_at
            FROM landed_cost_lines
            WHERE tenant_id = $1 AND document_id = $2
            ORDER BY created_at ASC
            "#,
            tenant_id,
            document_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find landed cost lines: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| LandedCostLine {
                line_id: r.line_id,
                tenant_id: r.tenant_id,
                document_id: r.document_id,
                cost_type: r.cost_type,
                description: r.description,
                amount: r.amount,
                vendor_reference: r.vendor_reference,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        line_id: Uuid,
    ) -> Result<Option<LandedCostLine>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT
                line_id, tenant_id, document_id, cost_type,
                description, amount, vendor_reference, created_at, updated_at
            FROM landed_cost_lines
            WHERE tenant_id = $1 AND line_id = $2
            "#,
            tenant_id,
            line_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find landed cost line: {e}")))?;

        Ok(row.map(|r| LandedCostLine {
            line_id: r.line_id,
            tenant_id: r.tenant_id,
            document_id: r.document_id,
            cost_type: r.cost_type,
            description: r.description,
            amount: r.amount,
            vendor_reference: r.vendor_reference,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn update(
        &self,
        tenant_id: Uuid,
        line_id: Uuid,
        cost_type: Option<String>,
        description: Option<String>,
        amount: Option<i64>,
        vendor_reference: Option<String>,
    ) -> Result<LandedCostLine, AppError> {
        let row = sqlx::query!(
            r#"
            UPDATE landed_cost_lines
            SET
                cost_type = COALESCE($3, cost_type),
                description = COALESCE($4, description),
                amount = COALESCE($5, amount),
                vendor_reference = COALESCE($6, vendor_reference),
                updated_at = NOW()
            WHERE tenant_id = $1 AND line_id = $2
            RETURNING
                line_id, tenant_id, document_id, cost_type,
                description, amount, vendor_reference, created_at, updated_at
            "#,
            tenant_id,
            line_id,
            cost_type,
            description,
            amount,
            vendor_reference
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update landed cost line: {e}")))?
        .ok_or_else(|| AppError::NotFound("Landed cost line not found".to_string()))?;

        Ok(LandedCostLine {
            line_id: row.line_id,
            tenant_id: row.tenant_id,
            document_id: row.document_id,
            cost_type: row.cost_type,
            description: row.description,
            amount: row.amount,
            vendor_reference: row.vendor_reference,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn delete(&self, tenant_id: Uuid, line_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            DELETE FROM landed_cost_lines
            WHERE tenant_id = $1 AND line_id = $2
            "#,
            tenant_id,
            line_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete landed cost line: {e}")))?;

        Ok(())
    }

    async fn delete_by_document_id(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<i64, AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM landed_cost_lines
            WHERE tenant_id = $1 AND document_id = $2
            "#,
            tenant_id,
            document_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete landed cost lines: {e}")))?;

        Ok(result.rows_affected() as i64)
    }
}

/// PostgreSQL implementation of `LandedCostAllocationRepository`.
pub struct LandedCostAllocationRepositoryImpl {
    pool: PgPool,
}

impl LandedCostAllocationRepositoryImpl {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LandedCostAllocationRepository for LandedCostAllocationRepositoryImpl {
    async fn create_batch(
        &self,
        allocations: Vec<LandedCostAllocation>,
    ) -> Result<Vec<LandedCostAllocation>, AppError> {
        if allocations.is_empty() {
            return Ok(vec![]);
        }

        let mut created = Vec::with_capacity(allocations.len());

        for alloc in allocations {
            let row = sqlx::query!(
                r#"
                INSERT INTO landed_cost_allocations (
                    allocation_id, tenant_id, document_id, receipt_item_id,
                    allocated_amount, original_unit_cost, new_unit_cost, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING
                    allocation_id, tenant_id, document_id, receipt_item_id,
                    allocated_amount, original_unit_cost, new_unit_cost, created_at
                "#,
                alloc.allocation_id,
                alloc.tenant_id,
                alloc.document_id,
                alloc.receipt_item_id,
                alloc.allocated_amount,
                alloc.original_unit_cost,
                alloc.new_unit_cost,
                alloc.created_at
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create allocation: {e}")))?;

            created.push(LandedCostAllocation {
                allocation_id: row.allocation_id,
                tenant_id: row.tenant_id,
                document_id: row.document_id,
                receipt_item_id: row.receipt_item_id,
                allocated_amount: row.allocated_amount,
                original_unit_cost: row.original_unit_cost,
                new_unit_cost: row.new_unit_cost,
                created_at: row.created_at,
            });
        }

        Ok(created)
    }

    async fn find_by_document_id(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<Vec<LandedCostAllocation>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                allocation_id, tenant_id, document_id, receipt_item_id,
                allocated_amount, original_unit_cost, new_unit_cost, created_at
            FROM landed_cost_allocations
            WHERE tenant_id = $1 AND document_id = $2
            ORDER BY created_at ASC
            "#,
            tenant_id,
            document_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find allocations: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| LandedCostAllocation {
                allocation_id: r.allocation_id,
                tenant_id: r.tenant_id,
                document_id: r.document_id,
                receipt_item_id: r.receipt_item_id,
                allocated_amount: r.allocated_amount,
                original_unit_cost: r.original_unit_cost,
                new_unit_cost: r.new_unit_cost,
                created_at: r.created_at,
            })
            .collect())
    }

    async fn find_by_receipt_item_id(
        &self,
        tenant_id: Uuid,
        receipt_item_id: Uuid,
    ) -> Result<Vec<LandedCostAllocation>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                allocation_id, tenant_id, document_id, receipt_item_id,
                allocated_amount, original_unit_cost, new_unit_cost, created_at
            FROM landed_cost_allocations
            WHERE tenant_id = $1 AND receipt_item_id = $2
            ORDER BY created_at ASC
            "#,
            tenant_id,
            receipt_item_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find allocations: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| LandedCostAllocation {
                allocation_id: r.allocation_id,
                tenant_id: r.tenant_id,
                document_id: r.document_id,
                receipt_item_id: r.receipt_item_id,
                allocated_amount: r.allocated_amount,
                original_unit_cost: r.original_unit_cost,
                new_unit_cost: r.new_unit_cost,
                created_at: r.created_at,
            })
            .collect())
    }

    async fn delete_by_document_id(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<i64, AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM landed_cost_allocations
            WHERE tenant_id = $1 AND document_id = $2
            "#,
            tenant_id,
            document_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete allocations: {e}")))?;

        Ok(result.rows_affected() as i64)
    }
}
