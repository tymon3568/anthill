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
pub struct ValuationRepositoryImpl {
    pool: PgPool,
}

impl ValuationRepositoryImpl {
    /// Create new repository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ValuationRepository for ValuationRepositoryImpl {
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

        Ok(row.map(|r| Valuation {
            valuation_id: r.valuation_id,
            tenant_id: r.tenant_id,
            product_id: r.product_id,
            valuation_method: match r.valuation_method.as_str() {
                "fifo" => ValuationMethod::Fifo,
                "avco" => ValuationMethod::Avco,
                "standard" => ValuationMethod::Standard,
                _ => ValuationMethod::Fifo, // Default fallback
            },
            current_unit_cost: r.current_unit_cost,
            total_quantity: r.total_quantity.unwrap_or(0),
            total_value: r.total_value.unwrap_or(0),
            standard_cost: r.standard_cost,
            last_updated: r.last_updated,
            updated_by: r.updated_by,
        }))
    }

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

        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method: match row.valuation_method.as_str() {
                "fifo" => ValuationMethod::Fifo,
                "avco" => ValuationMethod::Avco,
                "standard" => ValuationMethod::Standard,
                _ => ValuationMethod::Fifo,
            },
            current_unit_cost: row.current_unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            last_updated: row.last_updated,
            updated_by: row.updated_by,
        })
    }

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

        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method: match row.valuation_method.as_str() {
                "fifo" => ValuationMethod::Fifo,
                "avco" => ValuationMethod::Avco,
                "standard" => ValuationMethod::Standard,
                _ => ValuationMethod::Fifo,
            },
            current_unit_cost: row.current_unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            last_updated: row.last_updated,
            updated_by: row.updated_by,
        })
    }

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

        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method: match row.valuation_method.as_str() {
                "fifo" => ValuationMethod::Fifo,
                "avco" => ValuationMethod::Avco,
                "standard" => ValuationMethod::Standard,
                _ => ValuationMethod::Fifo,
            },
            current_unit_cost: row.current_unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            last_updated: row.last_updated,
            updated_by: row.updated_by,
        })
    }

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

        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method: match row.valuation_method.as_str() {
                "fifo" => ValuationMethod::Fifo,
                "avco" => ValuationMethod::Avco,
                "standard" => ValuationMethod::Standard,
                _ => ValuationMethod::Fifo,
            },
            current_unit_cost: row.current_unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            last_updated: row.last_updated,
            updated_by: row.updated_by,
        })
    }

    async fn update_from_stock_move(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity_change: i64,
        unit_cost: Option<i64>,
        updated_by: Option<Uuid>,
    ) -> Result<Valuation> {
        // Get current valuation
        let current = self
            .find_by_product_id(tenant_id, product_id)
            .await?
            .ok_or_else(|| shared_error::AppError::NotFound("Valuation not found".to_string()))?;

        let (new_quantity, new_value, new_unit_cost) = match current.valuation_method {
            ValuationMethod::Fifo => {
                // For FIFO, we handle layers separately - just update totals
                let new_quantity = current.total_quantity + quantity_change;
                let new_value = if quantity_change > 0 {
                    // Receipt: add to value
                    current.total_value + (unit_cost.unwrap_or(0) * quantity_change)
                } else {
                    // Delivery: subtract from value (layers handle the cost)
                    current.total_value
                };
                (new_quantity, new_value, current.current_unit_cost)
            },
            ValuationMethod::Avco => {
                let new_quantity = current.total_quantity + quantity_change;
                if new_quantity == 0 {
                    (0, 0, None)
                } else if quantity_change > 0 {
                    // Receipt: recalculate average
                    let receipt_value = unit_cost.unwrap_or(0) * quantity_change;
                    let new_value = current.total_value + receipt_value;
                    let new_unit_cost = Some(new_value / new_quantity);
                    (new_quantity, new_value, new_unit_cost)
                } else {
                    // Delivery: use current average
                    let delivery_value =
                        current.current_unit_cost.unwrap_or(0) * quantity_change.abs();
                    let new_value = current.total_value + delivery_value; // quantity_change is negative
                    (new_quantity, new_value, current.current_unit_cost)
                }
            },
            ValuationMethod::Standard => {
                let new_quantity = current.total_quantity + quantity_change;
                let new_value = if new_quantity == 0 {
                    0
                } else {
                    current.standard_cost.unwrap_or(0) * new_quantity
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
        .fetch_one(&self.pool)
        .await?;

        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method: match row.valuation_method.as_str() {
                "fifo" => ValuationMethod::Fifo,
                "avco" => ValuationMethod::Avco,
                "standard" => ValuationMethod::Standard,
                _ => ValuationMethod::Fifo,
            },
            current_unit_cost: row.current_unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            last_updated: row.last_updated,
            updated_by: row.updated_by,
        })
    }

    async fn adjust_cost(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        adjustment_amount: i64,
        reason: &str,
        updated_by: Option<Uuid>,
    ) -> Result<Valuation> {
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

        // Insert history record
        sqlx::query!(
            r#"
            INSERT INTO inventory_valuation_history (
                valuation_id, tenant_id, product_id, valuation_method,
                unit_cost, total_quantity, total_value, standard_cost,
                changed_by, change_reason
            )
            SELECT valuation_id, tenant_id, product_id, valuation_method,
                   current_unit_cost, total_quantity, total_value, standard_cost,
                   $6, $7
            FROM inventory_valuations
            WHERE tenant_id = $1 AND product_id = $2
            "#,
            tenant_id,
            product_id,
            updated_by,
            reason
        )
        .execute(&self.pool)
        .await?;

        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method: match row.valuation_method.as_str() {
                "fifo" => ValuationMethod::Fifo,
                "avco" => ValuationMethod::Avco,
                "standard" => ValuationMethod::Standard,
                _ => ValuationMethod::Fifo,
            },
            current_unit_cost: row.current_unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            last_updated: row.last_updated,
            updated_by: row.updated_by,
        })
    }

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

        let new_value = current.total_quantity * new_unit_cost;

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

        // Insert history record
        sqlx::query!(
            r#"
            INSERT INTO inventory_valuation_history (
                valuation_id, tenant_id, product_id, valuation_method,
                unit_cost, total_quantity, total_value, standard_cost,
                changed_by, change_reason
            )
            SELECT valuation_id, tenant_id, product_id, valuation_method,
                   current_unit_cost, total_quantity, total_value, standard_cost,
                   $6, $7
            FROM inventory_valuations
            WHERE tenant_id = $1 AND product_id = $2
            "#,
            tenant_id,
            product_id,
            updated_by,
            reason
        )
        .execute(&self.pool)
        .await?;

        Ok(Valuation {
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method: match row.valuation_method.as_str() {
                "fifo" => ValuationMethod::Fifo,
                "avco" => ValuationMethod::Avco,
                "standard" => ValuationMethod::Standard,
                _ => ValuationMethod::Fifo,
            },
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

    async fn consume_layers(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity_to_consume: i64,
    ) -> Result<i64> {
        let mut remaining_to_consume = quantity_to_consume;
        let mut total_cost = 0i64;

        // Get layers ordered by creation time (FIFO)
        let layers = self
            .find_active_by_product_id(tenant_id, product_id)
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
                SET quantity = quantity - $3, total_value = quantity * unit_cost
                WHERE layer_id = $1 AND quantity >= $3
                "#,
                layer.layer_id,
                tenant_id,
                consume_from_this_layer
            )
            .execute(&self.pool)
            .await?;

            total_cost += consume_from_this_layer * layer.unit_cost;
            remaining_to_consume -= consume_from_this_layer;
        }

        // Clean up empty layers
        self.cleanup_empty_layers(tenant_id, product_id).await?;

        Ok(total_cost)
    }

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
            .map(|r| ValuationHistory {
                history_id: r.history_id,
                valuation_id: r.valuation_id,
                tenant_id: r.tenant_id,
                product_id: r.product_id,
                valuation_method: match r.valuation_method.as_str() {
                    "fifo" => ValuationMethod::Fifo,
                    "avco" => ValuationMethod::Avco,
                    "standard" => ValuationMethod::Standard,
                    _ => ValuationMethod::Fifo,
                },
                unit_cost: r.unit_cost,
                total_quantity: r.total_quantity.unwrap_or(0),
                total_value: r.total_value.unwrap_or(0),
                standard_cost: r.standard_cost,
                changed_at: r.changed_at,
                changed_by: r.changed_by,
                change_reason: r.change_reason,
            })
            .collect())
    }

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

        Ok(ValuationHistory {
            history_id: row.history_id,
            valuation_id: row.valuation_id,
            tenant_id: row.tenant_id,
            product_id: row.product_id,
            valuation_method: match row.valuation_method.as_str() {
                "fifo" => ValuationMethod::Fifo,
                "avco" => ValuationMethod::Avco,
                "standard" => ValuationMethod::Standard,
                _ => ValuationMethod::Fifo,
            },
            unit_cost: row.unit_cost,
            total_quantity: row.total_quantity.unwrap_or(0),
            total_value: row.total_value.unwrap_or(0),
            standard_cost: row.standard_cost,
            changed_at: row.changed_at,
            changed_by: row.changed_by,
            change_reason: row.change_reason,
        })
    }

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
