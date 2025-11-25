use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::domains::inventory::reconciliation::{
    CycleType, ReconciliationStatus, StockReconciliation, StockReconciliationItem,
};
use inventory_service_core::repositories::reconciliation::{
    ReconciliationItemCountUpdate, StockReconciliationItemRepository,
    StockReconciliationRepository, VarianceAnalysisResult,
};
use shared_error::AppError;

/// PostgreSQL implementation of StockReconciliationRepository
pub struct PgStockReconciliationRepository {
    pool: Arc<PgPool>,
}

impl PgStockReconciliationRepository {
    /// Create a new repository instance
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Convert database string to ReconciliationStatus enum
    fn string_to_reconciliation_status(s: &str) -> Result<ReconciliationStatus, AppError> {
        match s {
            "draft" => Ok(ReconciliationStatus::Draft),
            "in_progress" => Ok(ReconciliationStatus::InProgress),
            "completed" => Ok(ReconciliationStatus::Completed),
            "cancelled" => Ok(ReconciliationStatus::Cancelled),
            _ => Err(AppError::DataCorruption(format!("Unknown reconciliation status: {}", s))),
        }
    }

    /// Convert database string to CycleType enum
    fn string_to_cycle_type(s: &str) -> Result<CycleType, AppError> {
        match s {
            "full" => Ok(CycleType::Full),
            "abc_a" => Ok(CycleType::AbcA),
            "abc_b" => Ok(CycleType::AbcB),
            "abc_c" => Ok(CycleType::AbcC),
            "location" => Ok(CycleType::Location),
            "random" => Ok(CycleType::Random),
            _ => Err(AppError::DataCorruption(format!("Unknown cycle type: {}", s))),
        }
    }

    /// Convert BIGINT cents to Decimal
    fn cents_to_decimal(cents: i64) -> Decimal {
        Decimal::new(cents, 2) // 2 decimal places
    }

    /// Convert Decimal to BIGINT cents
    fn decimal_to_cents(decimal: Decimal) -> i64 {
        (decimal * Decimal::new(100, 0))
            .round()
            .to_i64()
            .unwrap_or(0)
    }

    /// Convert f64 to Decimal
    fn f64_to_decimal(f: f64) -> Decimal {
        Decimal::from_f64(f).unwrap_or(Decimal::ZERO)
    }
}

#[async_trait]
impl StockReconciliationRepository for PgStockReconciliationRepository {
    async fn create(
        &self,
        tenant_id: Uuid,
        reconciliation: &StockReconciliation,
    ) -> Result<StockReconciliation, AppError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO stock_reconciliations (
                reconciliation_id, tenant_id, reconciliation_number, name, description, status,
                cycle_type, warehouse_id, location_filter, product_filter, total_items,
                counted_items, total_variance, created_by, created_at, updated_at,
                started_at, completed_at, approved_by, approved_at, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            RETURNING reconciliation_id, tenant_id, reconciliation_number, name, description, status,
                      cycle_type, warehouse_id, location_filter, product_filter, total_items,
                      counted_items, total_variance, created_by, created_at, updated_at,
                      started_at, completed_at, approved_by, approved_at, notes,
                      deleted_at, deleted_by
            "#,
            reconciliation.reconciliation_id,
            tenant_id,
            reconciliation.reconciliation_number,
            reconciliation.name,
            reconciliation.description,
            match reconciliation.status {
                ReconciliationStatus::Draft => "draft",
                ReconciliationStatus::InProgress => "in_progress",
                ReconciliationStatus::Completed => "completed",
                ReconciliationStatus::Cancelled => "cancelled",
            },
            match reconciliation.cycle_type {
                CycleType::Full => "full",
                CycleType::AbcA => "abc_a",
                CycleType::AbcB => "abc_b",
                CycleType::AbcC => "abc_c",
                CycleType::Location => "location",
                CycleType::Random => "random",
            },
            reconciliation.warehouse_id,
            reconciliation.location_filter,
            reconciliation.product_filter,
            reconciliation.total_items,
            reconciliation.counted_items,
            reconciliation.total_variance,
            reconciliation.created_by,
            reconciliation.created_at,
            reconciliation.updated_at,
            reconciliation.started_at,
            reconciliation.completed_at,
            reconciliation.approved_by,
            reconciliation.approved_at,
            reconciliation.notes
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create reconciliation: {}", e)))?;

        Ok(StockReconciliation {
            reconciliation_id: row.reconciliation_id,
            tenant_id: row.tenant_id,
            reconciliation_number: row.reconciliation_number,
            name: row.name,
            description: row.description,
            status: Self::string_to_reconciliation_status(&row.status)?,
            cycle_type: Self::string_to_cycle_type(&row.cycle_type)?,
            warehouse_id: row.warehouse_id,
            location_filter: row.location_filter,
            product_filter: row.product_filter,
            total_items: row.total_items,
            counted_items: row.counted_items,
            total_variance: row.total_variance,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
            started_at: row.started_at,
            completed_at: row.completed_at,
            approved_by: row.approved_by,
            approved_at: row.approved_at,
            notes: row.notes,
        })
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
    ) -> Result<Option<StockReconciliation>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT reconciliation_id, tenant_id, reconciliation_number, name, description, status,
                   cycle_type, warehouse_id, location_filter, product_filter, total_items,
                   counted_items, total_variance, created_by, created_at, updated_at,
                   started_at, completed_at, approved_by, approved_at, notes
            FROM stock_reconciliations
            WHERE tenant_id = $1 AND reconciliation_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            reconciliation_id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find reconciliation: {}", e)))?;

        row.map(|r| -> Result<StockReconciliation, AppError> {
            Ok(StockReconciliation {
                reconciliation_id: r.reconciliation_id,
                tenant_id: r.tenant_id,
                reconciliation_number: r.reconciliation_number,
                name: r.name,
                description: r.description,
                status: Self::string_to_reconciliation_status(&r.status)?,
                cycle_type: Self::string_to_cycle_type(&r.cycle_type)?,
                warehouse_id: r.warehouse_id,
                location_filter: r.location_filter,
                product_filter: r.product_filter,
                total_items: r.total_items,
                counted_items: r.counted_items,
                total_variance: r.total_variance,
                created_by: r.created_by,
                created_at: r.created_at,
                updated_at: r.updated_at,
                started_at: r.started_at,
                completed_at: r.completed_at,
                approved_by: r.approved_by,
                approved_at: r.approved_at,
                notes: r.notes,
            })
        })
        .transpose()
    }

    async fn update_status(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        status: ReconciliationStatus,
        updated_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_reconciliations
            SET status = $1, updated_at = NOW()
            WHERE tenant_id = $2 AND reconciliation_id = $3 AND deleted_at IS NULL
            "#,
            match status {
                ReconciliationStatus::Draft => "draft",
                ReconciliationStatus::InProgress => "in_progress",
                ReconciliationStatus::Completed => "completed",
                ReconciliationStatus::Cancelled => "cancelled",
            },
            tenant_id,
            reconciliation_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update reconciliation status: {}", e))
        })?;

        Ok(())
    }

    async fn finalize(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        completed_at: chrono::DateTime<chrono::Utc>,
        updated_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_reconciliations
            SET status = $1, completed_at = $2, updated_at = NOW()
            WHERE tenant_id = $3 AND reconciliation_id = $4 AND deleted_at IS NULL
            "#,
            "completed",
            completed_at,
            tenant_id,
            reconciliation_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to finalize reconciliation: {}", e))
        })?;

        Ok(())
    }

    async fn approve(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        approved_by: Uuid,
        approved_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_reconciliations
            SET approved_by = $1, approved_at = $2, updated_at = NOW()
            WHERE tenant_id = $3 AND reconciliation_id = $4 AND deleted_at IS NULL
            "#,
            approved_by,
            approved_at,
            tenant_id,
            reconciliation_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to approve reconciliation: {}", e)))?;

        Ok(())
    }

    async fn delete(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_reconciliations
            SET deleted_at = NOW(), deleted_by = $1, updated_at = NOW()
            WHERE tenant_id = $2 AND reconciliation_id = $3 AND deleted_at IS NULL
            "#,
            deleted_by,
            tenant_id,
            reconciliation_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete reconciliation: {}", e)))?;

        Ok(())
    }

    async fn list(
        &self,
        tenant_id: Uuid,
        warehouse_id: Option<Uuid>,
        status: Option<ReconciliationStatus>,
        cycle_type: Option<CycleType>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<StockReconciliation>, AppError> {
        let status_str = status.map(|s| match s {
            ReconciliationStatus::Draft => "draft",
            ReconciliationStatus::InProgress => "in_progress",
            ReconciliationStatus::Completed => "completed",
            ReconciliationStatus::Cancelled => "cancelled",
        });
        let cycle_type_str = cycle_type.map(|c| match c {
            CycleType::Full => "full",
            CycleType::AbcA => "abc_a",
            CycleType::AbcB => "abc_b",
            CycleType::AbcC => "abc_c",
            CycleType::Location => "location",
            CycleType::Random => "random",
        });

        let rows = sqlx::query!(
            r#"
            SELECT reconciliation_id, tenant_id, reconciliation_number, name, description, status,
                   cycle_type, warehouse_id, location_filter, product_filter, total_items,
                   counted_items, total_variance, created_by, created_at, updated_at,
                   started_at, completed_at, approved_by, approved_at, notes
            FROM stock_reconciliations
            WHERE tenant_id = $1 AND deleted_at IS NULL
            AND ($2::uuid IS NULL OR warehouse_id = $2)
            AND ($3::text IS NULL OR status = $3)
            AND ($4::text IS NULL OR cycle_type = $4)
            ORDER BY created_at DESC
            LIMIT $5 OFFSET $6
            "#,
            tenant_id,
            warehouse_id,
            status_str,
            cycle_type_str,
            limit.unwrap_or(50),
            offset.unwrap_or(0)
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to list reconciliations: {}", e)))?;

        let reconciliations = rows
            .into_iter()
            .map(|r| -> Result<StockReconciliation, AppError> {
                Ok(StockReconciliation {
                    reconciliation_id: r.reconciliation_id,
                    tenant_id: r.tenant_id,
                    reconciliation_number: r.reconciliation_number,
                    name: r.name,
                    description: r.description,
                    status: Self::string_to_reconciliation_status(&r.status)?,
                    cycle_type: Self::string_to_cycle_type(&r.cycle_type)?,
                    warehouse_id: r.warehouse_id,
                    location_filter: r.location_filter,
                    product_filter: r.product_filter,
                    total_items: r.total_items,
                    counted_items: r.counted_items,
                    total_variance: r.total_variance,
                    created_by: r.created_by,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    started_at: r.started_at,
                    completed_at: r.completed_at,
                    approved_by: r.approved_by,
                    approved_at: r.approved_at,
                    notes: r.notes,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(reconciliations)
    }

    async fn count(
        &self,
        tenant_id: Uuid,
        warehouse_id: Option<Uuid>,
        status: Option<ReconciliationStatus>,
        cycle_type: Option<CycleType>,
    ) -> Result<i64, AppError> {
        let status_str = status.map(|s| match s {
            ReconciliationStatus::Draft => "draft",
            ReconciliationStatus::InProgress => "in_progress",
            ReconciliationStatus::Completed => "completed",
            ReconciliationStatus::Cancelled => "cancelled",
        });
        let cycle_type_str = cycle_type.map(|c| match c {
            CycleType::Full => "full",
            CycleType::AbcA => "abc_a",
            CycleType::AbcB => "abc_b",
            CycleType::AbcC => "abc_c",
            CycleType::Location => "location",
            CycleType::Random => "random",
        });

        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM stock_reconciliations
            WHERE tenant_id = $1 AND deleted_at IS NULL
            AND ($2::uuid IS NULL OR warehouse_id = $2)
            AND ($3::text IS NULL OR status = $3)
            AND ($4::text IS NULL OR cycle_type = $4)
            "#,
            tenant_id,
            warehouse_id,
            status_str,
            cycle_type_str
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to count reconciliations: {}", e)))?;

        Ok(row.count.unwrap_or(0))
    }
}

/// PostgreSQL implementation of StockReconciliationItemRepository
pub struct PgStockReconciliationItemRepository {
    pool: Arc<PgPool>,
}

impl PgStockReconciliationItemRepository {
    /// Create a new repository instance
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StockReconciliationItemRepository for PgStockReconciliationItemRepository {
    async fn create_from_inventory(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        cycle_type: CycleType,
        warehouse_id: Option<Uuid>,
        location_filter: Option<serde_json::Value>,
        product_filter: Option<serde_json::Value>,
    ) -> Result<Vec<StockReconciliationItem>, AppError> {
        // This is a simplified implementation - in practice, you'd need to implement
        // the cycle counting logic based on ABC analysis, location filters, etc.
        // For now, we'll create items for all products in the warehouse
        let rows = sqlx::query!(
            r#"
            INSERT INTO stock_reconciliation_items (
                tenant_id, reconciliation_id, product_id, warehouse_id, location_id,
                expected_quantity, unit_cost
            )
            SELECT $1, $2, il.product_id, il.warehouse_id, NULL,
                   il.available_quantity::BIGINT, 0
            FROM inventory_levels il
            WHERE il.tenant_id = $1
            AND ($3::uuid IS NULL OR il.warehouse_id = $3)
            AND il.deleted_at IS NULL
            RETURNING tenant_id, reconciliation_id, product_id, warehouse_id, location_id,
                      expected_quantity, counted_quantity, variance, variance_percentage,
                      unit_cost, variance_value, notes, counted_by, counted_at,
                      created_at, updated_at
            "#,
            tenant_id,
            reconciliation_id,
            warehouse_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!(
                "Failed to create reconciliation items from inventory: {}",
                e
            ))
        })?;

        let items = rows
            .into_iter()
            .map(|r| StockReconciliationItem {
                tenant_id: r.tenant_id,
                reconciliation_id: r.reconciliation_id,
                product_id: r.product_id,
                warehouse_id: r.warehouse_id,
                location_id: r.location_id,
                expected_quantity: r.expected_quantity,
                counted_quantity: r.counted_quantity,
                variance: r.variance,
                variance_percentage: r
                    .variance_percentage
                    .map(|p| Decimal::new(p.mantissa(), p.scale() as u32)),
                unit_cost: Some(PgStockReconciliationRepository::cents_to_decimal(r.unit_cost)),
                variance_value: r
                    .variance_value
                    .map(PgStockReconciliationRepository::cents_to_decimal),
                notes: r.notes,
                counted_by: r.counted_by,
                counted_at: r.counted_at,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(items)
    }

    async fn find_by_reconciliation_id(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
    ) -> Result<Vec<StockReconciliationItem>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT tenant_id, reconciliation_id, product_id, warehouse_id, location_id,
                   expected_quantity, counted_quantity, variance, variance_percentage,
                   unit_cost, variance_value, notes, counted_by, counted_at,
                   created_at, updated_at
            FROM stock_reconciliation_items
            WHERE tenant_id = $1 AND reconciliation_id = $2 AND deleted_at IS NULL
            ORDER BY created_at
            "#,
            tenant_id,
            reconciliation_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to find reconciliation items: {}", e))
        })?;

        let items = rows
            .into_iter()
            .map(|r| StockReconciliationItem {
                tenant_id: r.tenant_id,
                reconciliation_id: r.reconciliation_id,
                product_id: r.product_id,
                warehouse_id: r.warehouse_id,
                location_id: r.location_id,
                expected_quantity: r.expected_quantity,
                counted_quantity: r.counted_quantity,
                variance: r.variance,
                variance_percentage: r
                    .variance_percentage
                    .map(|p| Decimal::new(p.mantissa(), p.scale() as u32)),
                unit_cost: Some(PgStockReconciliationRepository::cents_to_decimal(r.unit_cost)),
                variance_value: r
                    .variance_value
                    .map(PgStockReconciliationRepository::cents_to_decimal),
                notes: r.notes,
                counted_by: r.counted_by,
                counted_at: r.counted_at,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(items)
    }

    async fn find_by_key(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        product_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Option<StockReconciliationItem>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT tenant_id, reconciliation_id, product_id, warehouse_id, location_id,
                   expected_quantity, counted_quantity, variance, variance_percentage,
                   unit_cost, variance_value, notes, counted_by, counted_at,
                   created_at, updated_at
            FROM stock_reconciliation_items
            WHERE tenant_id = $1 AND reconciliation_id = $2 AND product_id = $3 AND warehouse_id = $4 AND deleted_at IS NULL
            "#,
            tenant_id,
            reconciliation_id,
            product_id,
            warehouse_id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find reconciliation item: {}", e)))?;

        Ok(row.map(|r| StockReconciliationItem {
            tenant_id: r.tenant_id,
            reconciliation_id: r.reconciliation_id,
            product_id: r.product_id,
            warehouse_id: r.warehouse_id,
            location_id: r.location_id,
            expected_quantity: r.expected_quantity,
            counted_quantity: r.counted_quantity,
            variance: r.variance,
            variance_percentage: r
                .variance_percentage
                .map(|p| Decimal::new(p.mantissa(), p.scale() as u32)),
            unit_cost: Some(PgStockReconciliationRepository::cents_to_decimal(r.unit_cost)),
            variance_value: r
                .variance_value
                .map(PgStockReconciliationRepository::cents_to_decimal),
            notes: r.notes,
            counted_by: r.counted_by,
            counted_at: r.counted_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn update_count(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        product_id: Uuid,
        warehouse_id: Uuid,
        counted_quantity: i64,
        unit_cost: Option<f64>,
        counted_by: Uuid,
        notes: Option<String>,
    ) -> Result<(), AppError> {
        let unit_cost_cents = unit_cost
            .map(|c| {
                PgStockReconciliationRepository::decimal_to_cents(
                    PgStockReconciliationRepository::f64_to_decimal(c),
                )
            })
            .unwrap_or(0);

        sqlx::query!(
            r#"
            UPDATE stock_reconciliation_items
            SET counted_quantity = $1, unit_cost = $2, counted_by = $3, counted_at = NOW(), notes = $4, updated_at = NOW()
            WHERE tenant_id = $5 AND reconciliation_id = $6 AND product_id = $7 AND warehouse_id = $8 AND deleted_at IS NULL
            "#,
            counted_quantity,
            unit_cost_cents,
            counted_by,
            notes,
            tenant_id,
            reconciliation_id,
            product_id,
            warehouse_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update reconciliation item count: {}", e)))?;

        Ok(())
    }

    async fn batch_update_counts(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        counts: &[ReconciliationItemCountUpdate],
    ) -> Result<(), AppError> {
        if counts.is_empty() {
            return Ok(());
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            UPDATE stock_reconciliation_items
            SET counted_quantity = data.counted_quantity,
                unit_cost = data.unit_cost,
                counted_by = data.counted_by,
                counted_at = NOW(),
                notes = data.notes,
                updated_at = NOW()
            FROM (VALUES
            "#,
        );

        let mut separated = query_builder.separated(", ");
        for count in counts {
            let unit_cost_cents = count
                .unit_cost
                .map(|c| {
                    PgStockReconciliationRepository::decimal_to_cents(
                        PgStockReconciliationRepository::f64_to_decimal(c),
                    )
                })
                .unwrap_or(0);
            separated.push("(");
            separated.push_bind_unseparated(count.product_id);
            separated.push_bind_unseparated(count.warehouse_id);
            separated.push_bind_unseparated(count.location_id);
            separated.push_bind_unseparated(count.counted_quantity);
            separated.push_bind_unseparated(unit_cost_cents);
            separated.push_bind_unseparated(count.counted_by);
            separated.push_bind_unseparated(&count.notes);
            separated.push_unseparated(")");
        }

        query_builder.push(
            r#"
            ) AS data(product_id, warehouse_id, location_id, counted_quantity, unit_cost, counted_by, notes)
            WHERE stock_reconciliation_items.product_id = data.product_id
            AND stock_reconciliation_items.warehouse_id = data.warehouse_id
            AND stock_reconciliation_items.tenant_id = "#,
        );
        query_builder.push_bind(tenant_id);
        query_builder.push(
            r#"
            AND stock_reconciliation_items.reconciliation_id = "#,
        );
        query_builder.push_bind(reconciliation_id);
        query_builder.push(
            r#"
            AND stock_reconciliation_items.deleted_at IS NULL
            "#,
        );

        let query = query_builder.build();
        query.execute(&*self.pool).await.map_err(|e| {
            AppError::DatabaseError(format!(
                "Failed to batch update reconciliation item counts: {}",
                e
            ))
        })?;

        Ok(())
    }

    async fn get_variance_analysis(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
    ) -> Result<VarianceAnalysisResult, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT tenant_id, reconciliation_id, product_id, warehouse_id, location_id,
                   expected_quantity, counted_quantity, variance, variance_percentage,
                   unit_cost, variance_value, notes, counted_by, counted_at,
                   created_at, updated_at
            FROM stock_reconciliation_items
            WHERE tenant_id = $1 AND reconciliation_id = $2 AND deleted_at IS NULL
            ORDER BY variance DESC
            "#,
            tenant_id,
            reconciliation_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get variance analysis: {}", e)))?;

        let items = rows
            .into_iter()
            .map(|r| StockReconciliationItem {
                tenant_id: r.tenant_id,
                reconciliation_id: r.reconciliation_id,
                product_id: r.product_id,
                warehouse_id: r.warehouse_id,
                location_id: r.location_id,
                expected_quantity: r.expected_quantity,
                counted_quantity: r.counted_quantity,
                variance: r.variance,
                variance_percentage: r
                    .variance_percentage
                    .map(|p| Decimal::new(p.mantissa(), p.scale() as u32)),
                unit_cost: Some(PgStockReconciliationRepository::cents_to_decimal(r.unit_cost)),
                variance_value: r
                    .variance_value
                    .map(PgStockReconciliationRepository::cents_to_decimal),
                notes: r.notes,
                counted_by: r.counted_by,
                counted_at: r.counted_at,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        // Calculate total items and counted items
        let total_items = items.len() as i64;
        let counted_items = items
            .iter()
            .filter(|i| i.counted_quantity.is_some())
            .count() as i64;

        Ok(VarianceAnalysisResult {
            items,
            total_items,
            counted_items,
        })
    }

    async fn delete(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        product_id: Uuid,
        warehouse_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_reconciliation_items
            SET deleted_at = NOW(), deleted_by = $1, updated_at = NOW()
            WHERE tenant_id = $2 AND reconciliation_id = $3 AND product_id = $4 AND warehouse_id = $5 AND deleted_at IS NULL
            "#,
            deleted_by,
            tenant_id,
            reconciliation_id,
            product_id,
            warehouse_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete reconciliation item: {}", e)))?;

        Ok(())
    }
}
