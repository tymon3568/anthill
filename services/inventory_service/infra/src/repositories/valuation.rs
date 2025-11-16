//! Valuation repository implementations
//!
//! PostgreSQL implementations of the ValuationRepository, ValuationLayerRepository,
//! and ValuationHistoryRepository traits.

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use inventory_service_core::domains::inventory::valuation::{
    Valuation, ValuationHistory, ValuationLayer, ValuationMethod,
};
use inventory_service_core::repositories::valuation::{
    ValuationHistoryRepository, ValuationLayerRepository, ValuationRepository,
};
use inventory_service_core::Result;

/// PostgreSQL implementation of valuation repositories
///
/// This struct provides concrete implementations for:
/// - ValuationRepository: Core valuation operations
/// - ValuationLayerRepository: FIFO cost layer management
/// - ValuationHistoryRepository: Audit trail and historical tracking
pub struct ValuationRepositoryImpl {
    pool: PgPool,
}

impl ValuationRepositoryImpl {
    /// Create new repository instance
    ///
    /// # Arguments
    /// * `pool` - PostgreSQL connection pool
    ///
    /// # Returns
    /// New ValuationRepositoryImpl instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Convert database string to ValuationMethod enum
    ///
    /// # Arguments
    /// * `s` - String representation from database
    ///
    /// # Returns
    /// Corresponding ValuationMethod enum value or error for unknown values
    fn string_to_valuation_method(s: &str) -> Result<ValuationMethod, shared_error::AppError> {
        match s {
            "fifo" => Ok(ValuationMethod::Fifo),
            "avco" => Ok(ValuationMethod::Avco),
            "standard" => Ok(ValuationMethod::Standard),
            unknown => Err(shared_error::AppError::DataCorruption(format!(
                "Unknown valuation method in database: {}",
                unknown
            ))),
        }
    }
}

#[async_trait]
impl ValuationRepository for ValuationRepositoryImpl {
    /// Find valuation record by product ID
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for multi-tenancy
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Option containing the valuation if found
    async fn find_by_product_id(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<Option<Valuation>> {
        let row = sqlx::query!(
            r#"
            SELECT
                valuation_id, tenant_id, product_id, valuation_method,
                current_unit_cost, total_quantity, total_value, standard_cost,
                last_updated, updated_by
            FROM inventory_valuations
            WHERE tenant_id = $1 AND product_id = $2
            "#,
            tenant_id,
            product_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row
            .map(|r| -> Result<Valuation> {
                Ok(Valuation {
                    valuation_id: r.valuation_id,
                    tenant_id: r.tenant_id,
                    product_id: r.product_id,
                    valuation_method: Self::string_to_valuation_method(
                        r.valuation_method.as_str(),
                    )?,
                    current_unit_cost: r.current_unit_cost,
                    total_quantity: r.total_quantity.unwrap_or(0),
                    total_value: r.total_value.unwrap_or(0),
                    standard_cost: r.standard_cost,
                    last_updated: r.last_updated,
                    updated_by: r.updated_by,
                })
            })
            .transpose()?)
    }

    /// Create a new valuation record
    ///
    /// # Arguments
    /// * `valuation` - Valuation data to insert
    ///
    /// # Returns
    /// Created valuation with generated fields
    async fn create(&self, valuation: &Valuation) -> Result<Valuation> {
        let method_str = match valuation.valuation_method {
            ValuationMethod::Fifo => "fifo",
            ValuationMethod::Avco => "avco",
            ValuationMethod::Standard => "standard",
        };

        let row = sqlx::query!(
            r#"
            INSERT INTO inventory_valuations (
                valuation_id, tenant_id, product_id, valuation_method,
                current_unit_cost, total_quantity, total_value, standard_cost,
                updated_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING valuation_id, tenant_id, product_id, valuation_method,
                      current_unit_cost, total_quantity, total_value, standard_cost,
                      last_updated, updated_by
            "#,
            valuation.valuation_id,
            valuation.tenant_id,
            valuation.product_id,
            method_str,
            valuation.current_unit_cost,
            valuation.total_quantity,
            valuation.total_value,
            valuation.standard_cost,
            valuation.updated_by
        )
        .fetch_one(&self.pool)
        .await?;

        let valuation_method = Self::string_to_valuation_method(row.valuation_method.as_str())?;
        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method,
            current_unit_cost: row.current_unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            last_updated: row.last_updated,
            updated_by: row.updated_by,
        })
    }

    /// Update an existing valuation record
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `valuation` - Updated valuation data
    ///
    /// # Returns
    /// Updated valuation record
    async fn update(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        valuation: &Valuation,
    ) -> Result<Valuation> {
        let method_str = match valuation.valuation_method {
            ValuationMethod::Fifo => "fifo",
            ValuationMethod::Avco => "avco",
            ValuationMethod::Standard => "standard",
        };

        let row = sqlx::query!(
            r#"
            UPDATE inventory_valuations
            SET valuation_method = $3, current_unit_cost = $4, total_quantity = $5,
                total_value = $6, standard_cost = $7, updated_by = $8
            WHERE tenant_id = $1 AND product_id = $2
            RETURNING valuation_id, tenant_id, product_id, valuation_method,
                      current_unit_cost, total_quantity, total_value, standard_cost,
                      last_updated, updated_by
            "#,
            tenant_id,
            product_id,
            method_str,
            valuation.current_unit_cost,
            valuation.total_quantity,
            valuation.total_value,
            valuation.standard_cost,
            valuation.updated_by
        )
        .fetch_one(&self.pool)
        .await?;

        let valuation_method = Self::string_to_valuation_method(row.valuation_method.as_str())?;
        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method,
            current_unit_cost: row.current_unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            last_updated: row.last_updated,
            updated_by: row.updated_by,
        })
    }

    /// Change the valuation method for a product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `method` - New valuation method
    /// * `updated_by` - User who made the change
    ///
    /// # Returns
    /// Updated valuation record
    async fn set_valuation_method(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        method: ValuationMethod,
        updated_by: Option<Uuid>,
    ) -> Result<Valuation> {
        let method_str = match method {
            ValuationMethod::Fifo => "fifo",
            ValuationMethod::Avco => "avco",
            ValuationMethod::Standard => "standard",
        };

        let row = sqlx::query!(
            r#"
            UPDATE inventory_valuations
            SET valuation_method = $3, updated_by = $4
            WHERE tenant_id = $1 AND product_id = $2
            RETURNING valuation_id, tenant_id, product_id, valuation_method,
                      current_unit_cost, total_quantity, total_value, standard_cost,
                      last_updated, updated_by
            "#,
            tenant_id,
            product_id,
            method_str,
            updated_by
        )
        .fetch_one(&self.pool)
        .await?;

        let valuation_method = Self::string_to_valuation_method(row.valuation_method.as_str())?;
        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method,
            current_unit_cost: row.current_unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            last_updated: row.last_updated,
            updated_by: row.updated_by,
        })
    }

    /// Set the standard cost for a product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `standard_cost` - New standard cost in cents
    /// * `updated_by` - User who made the change
    ///
    /// # Returns
    /// Updated valuation record
    async fn set_standard_cost(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        standard_cost: i64,
        updated_by: Option<Uuid>,
    ) -> Result<Valuation> {
        let row = sqlx::query!(
            r#"
            UPDATE inventory_valuations
            SET standard_cost = $3, updated_by = $4
            WHERE tenant_id = $1 AND product_id = $2
            RETURNING valuation_id, tenant_id, product_id, valuation_method,
                      current_unit_cost, total_quantity, total_value, standard_cost,
                      last_updated, updated_by
            "#,
            tenant_id,
            product_id,
            standard_cost,
            updated_by
        )
        .fetch_one(&self.pool)
        .await?;

        let valuation_method = Self::string_to_valuation_method(row.valuation_method.as_str())?;
        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method,
            current_unit_cost: row.current_unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            last_updated: row.last_updated,
            updated_by: row.updated_by,
        })
    }

    /// Update valuation based on stock movement
    ///
    /// Handles receipts and deliveries for all valuation methods (FIFO, AVCO, Standard)
    /// with proper cost layer management and transaction safety.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `quantity_change` - Positive for receipts, negative for deliveries
    /// * `unit_cost` - Cost per unit for receipts
    /// * `updated_by` - User who initiated the movement
    ///
    /// # Returns
    /// Updated valuation record
    async fn update_from_stock_move(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity_change: i64,
        unit_cost: Option<i64>,
        updated_by: Option<Uuid>,
    ) -> Result<Valuation> {
        let mut tx = self.pool.begin().await?;

        // Lock the row for update
        let current = sqlx::query_as!(
            Valuation,
            r#"
            SELECT
                valuation_id, tenant_id, product_id, valuation_method,
                current_unit_cost, total_quantity, total_value, standard_cost,
                last_updated, updated_by
            FROM inventory_valuations
            WHERE tenant_id = $1 AND product_id = $2
            FOR UPDATE
            "#,
            tenant_id,
            product_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| shared_error::AppError::NotFound("Valuation not found".to_string()))?;

        let (new_quantity, new_value, new_unit_cost) = match current.valuation_method {
            ValuationMethod::Fifo => {
                let new_quantity = current.total_quantity + quantity_change;
                let new_value = if quantity_change > 0 {
                    // Receipt: create new cost layer
                    let unit_cost = unit_cost.ok_or_else(|| {
                        shared_error::AppError::ValidationError(
                            "Unit cost required for receipt".to_string(),
                        )
                    })?;
                    let layer_id = Uuid::now_v7();

                    let layer_value = unit_cost.checked_mul(quantity_change).ok_or_else(|| {
                        shared_error::AppError::ValidationError(
                            "Inventory value calculation overflow".to_string(),
                        )
                    })?;
                    let new_total =
                        current
                            .total_value
                            .checked_add(layer_value)
                            .ok_or_else(|| {
                                shared_error::AppError::ValidationError(
                                    "Inventory value calculation overflow".to_string(),
                                )
                            })?;

                    sqlx::query!(
                        r#"
                        INSERT INTO inventory_valuation_layers (
                            layer_id, tenant_id, product_id, quantity, unit_cost, total_value
                        )
                        VALUES ($1, $2, $3, $4, $5, $6)
                        "#,
                        layer_id,
                        tenant_id,
                        product_id,
                        quantity_change,
                        unit_cost,
                        layer_value
                    )
                    .execute(&mut *tx)
                    .await?;
                    new_total
                } else {
                    // Delivery: consume layers within this transaction
                    let mut remaining_to_consume = quantity_change.abs();
                    let mut total_cost = 0i64;

                    // Get layers ordered by creation time (FIFO)
                    let layers = sqlx::query_as!(
                        ValuationLayer,
                        r#"
                        SELECT layer_id, tenant_id, product_id, quantity, unit_cost, total_value,
                               created_at, updated_at
                        FROM inventory_valuation_layers
                        WHERE tenant_id = $1 AND product_id = $2 AND quantity > 0
                        ORDER BY created_at ASC
                        "#,
                        tenant_id,
                        product_id
                    )
                    .fetch_all(&mut *tx)
                    .await?;

                    for layer in layers {
                        if remaining_to_consume <= 0 {
                            break;
                        }

                        let consume_from_this_layer = remaining_to_consume.min(layer.quantity);

                        // Update layer quantity
                        sqlx::query!(
                            r#"
                            UPDATE inventory_valuation_layers
                            SET quantity = quantity - $3, total_value = (quantity - $3) * unit_cost
                            WHERE layer_id = $1 AND tenant_id = $2 AND quantity >= $3
                            "#,
                            layer.layer_id,
                            tenant_id,
                            consume_from_this_layer
                        )
                        .execute(&mut *tx)
                        .await?;

                        let cost_increment = consume_from_this_layer
                            .checked_mul(layer.unit_cost)
                            .ok_or_else(|| {
                            shared_error::AppError::ValidationError(
                                "Inventory value calculation overflow".to_string(),
                            )
                        })?;
                        total_cost = total_cost.checked_add(cost_increment).ok_or_else(|| {
                            shared_error::AppError::ValidationError(
                                "Inventory value calculation overflow".to_string(),
                            )
                        })?;
                        remaining_to_consume -= consume_from_this_layer;
                    }

                    if remaining_to_consume > 0 {
                        return Err(shared_error::AppError::BusinessError(
                            format!(
                                "Insufficient FIFO layers: {} units still needed after consuming all layers",
                                remaining_to_consume
                            )
                        ));
                    }

                    // Clean up empty layers
                    sqlx::query!(
                        r#"
                        DELETE FROM inventory_valuation_layers
                        WHERE tenant_id = $1 AND product_id = $2 AND quantity = 0
                        "#,
                        tenant_id,
                        product_id
                    )
                    .execute(&mut *tx)
                    .await?;

                    current.total_value.checked_sub(total_cost).ok_or_else(|| {
                        shared_error::AppError::ValidationError(
                            "Inventory value calculation overflow".to_string(),
                        )
                    })?
                };
                (new_quantity, new_value, current.current_unit_cost)
            },
            ValuationMethod::Avco => {
                let new_quantity = current.total_quantity + quantity_change;
                if new_quantity == 0 {
                    (0, 0, None)
                } else if quantity_change > 0 {
                    // Receipt: recalculate average
                    let receipt_value = unit_cost
                        .unwrap_or(0)
                        .checked_mul(quantity_change)
                        .ok_or_else(|| {
                            shared_error::AppError::ValidationError(
                                "Inventory value calculation overflow".to_string(),
                            )
                        })?;
                    let new_value =
                        current
                            .total_value
                            .checked_add(receipt_value)
                            .ok_or_else(|| {
                                shared_error::AppError::ValidationError(
                                    "Inventory value calculation overflow".to_string(),
                                )
                            })?;
                    let new_unit_cost = Some(new_value / new_quantity);
                    (new_quantity, new_value, new_unit_cost)
                } else {
                    // Delivery: use current average
                    let delivery_value = current
                        .current_unit_cost
                        .unwrap_or(0)
                        .checked_mul(quantity_change.abs())
                        .ok_or_else(|| {
                            shared_error::AppError::ValidationError(
                                "Inventory value calculation overflow".to_string(),
                            )
                        })?;
                    let new_value = current.total_value.checked_sub(delivery_value).ok_or_else(|| {
                        shared_error::AppError::BusinessError(
                            format!(
                                "Insufficient inventory value: attempting to deliver {} but only {} available",
                                delivery_value, current.total_value
                            )
                        )
                    })?;
                    (new_quantity, new_value, current.current_unit_cost)
                }
            },
            ValuationMethod::Standard => {
                let new_quantity = current.total_quantity + quantity_change;
                let new_value = if new_quantity == 0 {
                    0
                } else {
                    current
                        .standard_cost
                        .unwrap_or(0)
                        .checked_mul(new_quantity)
                        .ok_or_else(|| {
                            shared_error::AppError::ValidationError(
                                "Inventory value calculation overflow".to_string(),
                            )
                        })?
                };
                (new_quantity, new_value, current.current_unit_cost)
            },
        };

        let row = sqlx::query!(
            r#"
            UPDATE inventory_valuations
            SET total_quantity = $3, total_value = $4, current_unit_cost = $5, updated_by = $6
            WHERE tenant_id = $1 AND product_id = $2
            RETURNING valuation_id, tenant_id, product_id, valuation_method,
                      current_unit_cost, total_quantity, total_value, standard_cost,
                      last_updated, updated_by
            "#,
            tenant_id,
            product_id,
            new_quantity,
            new_value,
            new_unit_cost,
            updated_by
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        let valuation_method = Self::string_to_valuation_method(row.valuation_method.as_str())?;
        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method,
            current_unit_cost: row.current_unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            last_updated: row.last_updated,
            updated_by: row.updated_by,
        })
    }

    /// Adjust inventory cost without changing quantity
    ///
    /// Used for cost adjustments, revaluations, or corrections.
    /// Creates audit trail entry.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `adjustment_amount` - Amount to add/subtract from total value
    /// * `reason` - Reason for the adjustment
    /// * `updated_by` - User who made the adjustment
    ///
    /// # Returns
    /// Updated valuation record
    async fn adjust_cost(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        adjustment_amount: i64,
        reason: &str,
        updated_by: Option<Uuid>,
    ) -> Result<Valuation> {
        // Capture pre-change state
        let before = self
            .find_by_product_id(tenant_id, product_id)
            .await?
            .ok_or_else(|| shared_error::AppError::NotFound("Valuation not found".to_string()))?;

        let row = sqlx::query!(
            r#"
            UPDATE inventory_valuations
            SET total_value = total_value + $3, updated_by = $4
            WHERE tenant_id = $1 AND product_id = $2
            RETURNING valuation_id, tenant_id, product_id, valuation_method,
                      current_unit_cost, total_quantity, total_value, standard_cost,
                      last_updated, updated_by
            "#,
            tenant_id,
            product_id,
            adjustment_amount,
            updated_by
        )
        .fetch_one(&self.pool)
        .await?;

        // Insert history record with pre-change state
        let method_str = match before.valuation_method {
            ValuationMethod::Fifo => "fifo",
            ValuationMethod::Avco => "avco",
            ValuationMethod::Standard => "standard",
        };
        sqlx::query!(
            r#"
            INSERT INTO inventory_valuation_history (
                valuation_id, tenant_id, product_id, valuation_method,
                unit_cost, total_quantity, total_value, standard_cost,
                changed_by, change_reason
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            before.valuation_id,
            before.tenant_id,
            before.product_id,
            method_str,
            before.current_unit_cost,
            before.total_quantity,
            before.total_value,
            before.standard_cost,
            updated_by,
            reason
        )
        .execute(&self.pool)
        .await?;

        let valuation_method = Self::string_to_valuation_method(row.valuation_method.as_str())?;
        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method,
            current_unit_cost: row.current_unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            last_updated: row.last_updated,
            updated_by: row.updated_by,
        })
    }

    /// Revalue entire inventory at new unit cost
    ///
    /// Recalculates total value based on current quantity and new unit cost.
    /// Creates audit trail entry.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `new_unit_cost` - New cost per unit
    /// * `reason` - Reason for revaluation
    /// * `updated_by` - User who performed revaluation
    ///
    /// # Returns
    /// Updated valuation record
    async fn revalue_inventory(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        new_unit_cost: i64,
        reason: &str,
        updated_by: Option<Uuid>,
    ) -> Result<Valuation> {
        // Get current valuation
        let current = self
            .find_by_product_id(tenant_id, product_id)
            .await?
            .ok_or_else(|| shared_error::AppError::NotFound("Valuation not found".to_string()))?;

        let new_value = current
            .total_quantity
            .checked_mul(new_unit_cost)
            .ok_or_else(|| {
                shared_error::AppError::ValidationError(
                    "Inventory value calculation overflow".to_string(),
                )
            })?;

        let row = sqlx::query!(
            r#"
            UPDATE inventory_valuations
            SET current_unit_cost = $3, total_value = $4, updated_by = $5
            WHERE tenant_id = $1 AND product_id = $2
            RETURNING valuation_id, tenant_id, product_id, valuation_method,
                      current_unit_cost, total_quantity, total_value, standard_cost,
                      last_updated, updated_by
            "#,
            tenant_id,
            product_id,
            new_unit_cost,
            new_value,
            updated_by
        )
        .fetch_one(&self.pool)
        .await?;

        // Insert history record with pre-change state
        let method_str = match current.valuation_method {
            ValuationMethod::Fifo => "fifo",
            ValuationMethod::Avco => "avco",
            ValuationMethod::Standard => "standard",
        };
        sqlx::query!(
            r#"
            INSERT INTO inventory_valuation_history (
                valuation_id, tenant_id, product_id, valuation_method,
                unit_cost, total_quantity, total_value, standard_cost,
                changed_by, change_reason
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            current.valuation_id,
            current.tenant_id,
            current.product_id,
            method_str,
            current.current_unit_cost,
            current.total_quantity,
            current.total_value,
            current.standard_cost,
            updated_by,
            reason
        )
        .execute(&self.pool)
        .await?;

        let valuation_method = Self::string_to_valuation_method(row.valuation_method.as_str())?;
        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method,
            current_unit_cost: row.current_unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            last_updated: row.last_updated,
            updated_by: row.updated_by,
        })
    }
}

#[async_trait]
impl ValuationLayerRepository for ValuationRepositoryImpl {
    /// Find all active cost layers for a product
    ///
    /// Active layers have quantity > 0, ordered by creation time (FIFO).
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Vector of active valuation layers
    async fn find_active_by_product_id(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<Vec<ValuationLayer>> {
        let rows = sqlx::query!(
            r#"
            SELECT layer_id, tenant_id, product_id, quantity, unit_cost, total_value,
                   created_at, updated_at
            FROM inventory_valuation_layers
            WHERE tenant_id = $1 AND product_id = $2 AND quantity > 0
            ORDER BY created_at ASC
            "#,
            tenant_id,
            product_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ValuationLayer {
                layer_id: r.layer_id,
                tenant_id: r.tenant_id,
                product_id: r.product_id,
                quantity: r.quantity.unwrap_or(0),
                unit_cost: r.unit_cost.unwrap_or(0),
                total_value: r.total_value.unwrap_or(0),
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    /// Create a new cost layer
    ///
    /// Used for FIFO receipts to track cost layers separately.
    ///
    /// # Arguments
    /// * `layer` - Layer data to insert
    ///
    /// # Returns
    /// Created layer with generated fields
    async fn create(&self, layer: &ValuationLayer) -> Result<ValuationLayer> {
        let row = sqlx::query!(
            r#"
            INSERT INTO inventory_valuation_layers (
                layer_id, tenant_id, product_id, quantity, unit_cost, total_value
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING layer_id, tenant_id, product_id, quantity, unit_cost, total_value,
                      created_at, updated_at
            "#,
            layer.layer_id,
            layer.tenant_id,
            layer.product_id,
            layer.quantity,
            layer.unit_cost,
            layer.total_value
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ValuationLayer {
            layer_id: row.layer_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            quantity: row.quantity.unwrap_or(0),
            unit_cost: row.unit_cost.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// Consume cost layers for delivery (FIFO)
    ///
    /// Reduces layer quantities starting from oldest layers.
    /// Returns total cost of consumed quantity.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `quantity_to_consume` - Quantity to consume from layers
    ///
    /// # Returns
    /// Total cost of consumed layers
    async fn consume_layers(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity_to_consume: i64,
    ) -> Result<i64> {
        let mut tx = self.pool.begin().await?;
        let mut remaining_to_consume = quantity_to_consume;
        let mut total_cost = 0i64;

        // Get layers ordered by creation time (FIFO)
        let layers = sqlx::query_as!(
            ValuationLayer,
            r#"
            SELECT layer_id, tenant_id, product_id, quantity, unit_cost, total_value,
                   created_at, updated_at
            FROM inventory_valuation_layers
            WHERE tenant_id = $1 AND product_id = $2 AND quantity > 0
            ORDER BY created_at ASC
            "#,
            tenant_id,
            product_id
        )
        .fetch_all(&mut *tx)
        .await?;

        for layer in layers {
            if remaining_to_consume <= 0 {
                break;
            }

            let consume_from_this_layer = remaining_to_consume.min(layer.quantity);

            // Update layer quantity
            sqlx::query!(
                r#"
                UPDATE inventory_valuation_layers
                SET quantity = quantity - $3, total_value = (quantity - $3) * unit_cost
                WHERE layer_id = $1 AND tenant_id = $2 AND quantity >= $3
                "#,
                layer.layer_id,
                tenant_id,
                consume_from_this_layer
            )
            .execute(&mut *tx)
            .await?;

            total_cost += consume_from_this_layer * layer.unit_cost;
            remaining_to_consume -= consume_from_this_layer;
        }

        if remaining_to_consume > 0 {
            return Err(shared_error::AppError::BusinessError(format!(
                "Insufficient FIFO layers: {} units still needed after consuming all layers",
                remaining_to_consume
            )));
        }

        // Clean up empty layers
        sqlx::query!(
            r#"
            DELETE FROM inventory_valuation_layers
            WHERE tenant_id = $1 AND product_id = $2 AND quantity = 0
            "#,
            tenant_id,
            product_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(total_cost)
    }

    /// Get total quantity across all active layers
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Sum of quantities in all active layers
    async fn get_total_quantity(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64> {
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(quantity), 0) as total_quantity
            FROM inventory_valuation_layers
            WHERE tenant_id = $1 AND product_id = $2 AND quantity > 0
            "#,
            tenant_id,
            product_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.total_quantity.unwrap_or(0))
    }

    /// Remove layers with zero quantity
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Number of layers removed
    async fn cleanup_empty_layers(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM inventory_valuation_layers
            WHERE tenant_id = $1 AND product_id = $2 AND quantity = 0
            "#,
            tenant_id,
            product_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}

#[async_trait]
impl ValuationHistoryRepository for ValuationRepositoryImpl {
    /// Find valuation history records for a product
    ///
    /// Returns historical snapshots ordered by change time (newest first).
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `limit` - Maximum records to return (default 50, max 100)
    /// * `offset` - Number of records to skip
    ///
    /// # Returns
    /// Vector of historical valuation records
    async fn find_by_product_id(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ValuationHistory>> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT h.history_id, h.valuation_id, h.tenant_id, h.product_id,
                   h.valuation_method, h.unit_cost, h.total_quantity, h.total_value,
                   h.standard_cost, h.changed_at, h.changed_by, h.change_reason
            FROM inventory_valuation_history h
            WHERE h.tenant_id = $1 AND h.product_id = $2
            ORDER BY h.changed_at DESC
            LIMIT $3 OFFSET $4
            "#,
            tenant_id,
            product_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| -> Result<ValuationHistory> {
                Ok(ValuationHistory {
                    history_id: r.history_id,
                    valuation_id: r.valuation_id,
                    tenant_id: r.tenant_id,
                    product_id: r.product_id,
                    valuation_method: Self::string_to_valuation_method(
                        r.valuation_method.as_str(),
                    )?,
                    unit_cost: r.unit_cost,
                    total_quantity: r.total_quantity.unwrap_or(0),
                    total_value: r.total_value.unwrap_or(0),
                    standard_cost: r.standard_cost,
                    changed_at: r.changed_at,
                    changed_by: r.changed_by,
                    change_reason: r.change_reason,
                })
            })
            .collect::<Result<Vec<_>>>()?)
    }

    /// Create a new history record
    ///
    /// # Arguments
    /// * `history` - History data to insert
    ///
    /// # Returns
    /// Created history record with generated fields
    async fn create(&self, history: &ValuationHistory) -> Result<ValuationHistory> {
        let method_str = match history.valuation_method {
            ValuationMethod::Fifo => "fifo",
            ValuationMethod::Avco => "avco",
            ValuationMethod::Standard => "standard",
        };

        let row = sqlx::query!(
            r#"
            INSERT INTO inventory_valuation_history (
                history_id, valuation_id, tenant_id, product_id, valuation_method,
                unit_cost, total_quantity, total_value, standard_cost,
                changed_by, change_reason
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING history_id, valuation_id, tenant_id, product_id, valuation_method,
                      unit_cost, total_quantity, total_value, standard_cost,
                      changed_at, changed_by, change_reason
            "#,
            history.history_id,
            history.valuation_id,
            history.tenant_id,
            history.product_id,
            method_str,
            history.unit_cost,
            history.total_quantity,
            history.total_value,
            history.standard_cost,
            history.changed_by,
            history.change_reason
        )
        .fetch_one(&self.pool)
        .await?;

        let valuation_method = Self::string_to_valuation_method(row.valuation_method.as_str())?;
        Ok(ValuationHistory {
            history_id: row.history_id,
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method,
            unit_cost: row.unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            changed_at: row.changed_at,
            changed_by: row.changed_by,
            change_reason: row.change_reason,
        })
    }

    /// Count total history records for a product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Total number of history records
    async fn count_by_product_id(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM inventory_valuation_history
            WHERE tenant_id = $1 AND product_id = $2
            "#,
            tenant_id,
            product_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count.unwrap_or(0))
    }
}
