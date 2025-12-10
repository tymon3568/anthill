//! PostgreSQL implementation of RemovalStrategyRepository
//!
//! This module provides the concrete implementation of the RemovalStrategyRepository trait
//! using PostgreSQL as the data store. It handles all database operations for removal strategies.

use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use inventory_service_core::domains::inventory::removal_strategy::RemovalStrategy;
use inventory_service_core::dto::removal_strategy::{
    RemovalStrategyCreateRequest, RemovalStrategyListQuery, RemovalStrategyUpdateRequest,
    StockLocationInfo, StockSuggestion, SuggestRemovalRequest, SuggestRemovalResponse,
};
use inventory_service_core::repositories::removal_strategy::RemovalStrategyRepository;
use inventory_service_core::Result;

/// PostgreSQL implementation of RemovalStrategyRepository
///
/// Provides concrete implementations of all removal strategy repository operations
/// using SQLx for database interactions with PostgreSQL.
pub struct RemovalStrategyRepositoryImpl {
    pool: PgPool,
}

impl RemovalStrategyRepositoryImpl {
    /// Create a new RemovalStrategyRepositoryImpl with the given database connection pool
    ///
    /// # Arguments
    /// * `pool` - PostgreSQL connection pool
    ///
    /// # Returns
    /// New RemovalStrategyRepositoryImpl instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RemovalStrategyRepository for RemovalStrategyRepositoryImpl {
    async fn create(
        &self,
        tenant_id: uuid::Uuid,
        request: RemovalStrategyCreateRequest,
        created_by: uuid::Uuid,
    ) -> Result<RemovalStrategy> {
        let strategy = sqlx::query_as!(
            RemovalStrategy,
            r#"
            INSERT INTO removal_strategies (
                tenant_id, name, strategy_type, warehouse_id, product_id, active, config,
                created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                strategy_id, tenant_id, name, strategy_type, warehouse_id, product_id,
                active, config, created_at, updated_at, deleted_at, created_by, updated_by
            "#,
            tenant_id,
            request.name,
            request.strategy_type,
            request.warehouse_id,
            request.product_id,
            true,
            request.config,
            created_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(strategy)
    }

    async fn find_by_id(
        &self,
        tenant_id: uuid::Uuid,
        strategy_id: uuid::Uuid,
    ) -> Result<Option<RemovalStrategy>> {
        let strategy = sqlx::query_as!(
            RemovalStrategy,
            r#"
            SELECT
                strategy_id, tenant_id, name, strategy_type, warehouse_id, product_id,
                active, config, created_at, updated_at, deleted_at, created_by, updated_by
            FROM removal_strategies
            WHERE tenant_id = $1 AND strategy_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            strategy_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(strategy)
    }

    async fn find_by_name(
        &self,
        tenant_id: uuid::Uuid,
        name: &str,
    ) -> Result<Option<RemovalStrategy>> {
        let strategy = sqlx::query_as!(
            RemovalStrategy,
            r#"
            SELECT
                strategy_id, tenant_id, name, strategy_type, warehouse_id, product_id,
                active, config, created_at, updated_at, deleted_at, created_by, updated_by
            FROM removal_strategies
            WHERE tenant_id = $1 AND name = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(strategy)
    }

    async fn list(
        &self,
        tenant_id: uuid::Uuid,
        query: RemovalStrategyListQuery,
    ) -> Result<(Vec<RemovalStrategy>, u64)> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).max(1);
        let offset = ((page - 1) * page_size) as i64;
        let limit = page_size as i64;

        let mut conditions = vec![
            "rs.tenant_id = $1".to_string(),
            "rs.deleted_at IS NULL".to_string(),
        ];
        let mut param_count = 1;

        if let Some(_warehouse_id) = query.warehouse_id {
            param_count += 1;
            conditions.push(format!("rs.warehouse_id = ${}", param_count));
        }
        if let Some(_product_id) = query.product_id {
            param_count += 1;
            conditions.push(format!("rs.product_id = ${}", param_count));
        }
        if let Some(_strategy_type) = &query.strategy_type {
            param_count += 1;
            conditions.push(format!("rs.strategy_type = ${}", param_count));
        }
        if let Some(_active) = query.active {
            param_count += 1;
            conditions.push(format!("rs.active = ${}", param_count));
        }
        if let Some(_search) = &query.search {
            param_count += 1;
            conditions.push(format!("rs.name ILIKE ${}", param_count));
        }

        let where_clause = conditions.join(" AND ");

        let count_sql =
            format!("SELECT COUNT(*) as count FROM removal_strategies rs WHERE {}", where_clause);
        let data_sql = format!(
            r#"
            SELECT
                rs.strategy_id, rs.tenant_id, rs.name, rs.strategy_type, rs.warehouse_id, rs.product_id,
                rs.active, rs.config, rs.created_at, rs.updated_at, rs.deleted_at, rs.created_by, rs.updated_by
            FROM removal_strategies rs
            WHERE {}
            ORDER BY rs.created_at DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_count + 1,
            param_count + 2
        );

        // Build count query
        let mut count_query = sqlx::query(&count_sql).bind(tenant_id);
        if let Some(warehouse_id) = query.warehouse_id {
            count_query = count_query.bind(warehouse_id);
        }
        if let Some(product_id) = query.product_id {
            count_query = count_query.bind(product_id);
        }
        if let Some(strategy_type) = &query.strategy_type {
            count_query = count_query.bind(strategy_type);
        }
        if let Some(active) = query.active {
            count_query = count_query.bind(active);
        }
        if let Some(search) = &query.search {
            count_query = count_query.bind(format!("%{}%", search));
        }

        let count_row = count_query.fetch_one(&self.pool).await?;
        let count: i64 = count_row.get("count");

        // Build data query
        let mut data_query = sqlx::query(&data_sql).bind(tenant_id);
        if let Some(warehouse_id) = query.warehouse_id {
            data_query = data_query.bind(warehouse_id);
        }
        if let Some(product_id) = query.product_id {
            data_query = data_query.bind(product_id);
        }
        if let Some(strategy_type) = &query.strategy_type {
            data_query = data_query.bind(strategy_type);
        }
        if let Some(active) = query.active {
            data_query = data_query.bind(active);
        }
        if let Some(search) = &query.search {
            data_query = data_query.bind(format!("%{}%", search));
        }
        data_query = data_query.bind(limit).bind(offset);

        let strategies = data_query
            .map(|row: PgRow| RemovalStrategy {
                strategy_id: row.get("strategy_id"),
                tenant_id: row.get("tenant_id"),
                name: row.get("name"),
                strategy_type: row.get("strategy_type"),
                warehouse_id: row.get("warehouse_id"),
                product_id: row.get("product_id"),
                active: row.get("active"),
                config: row.get("config"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                deleted_at: row.get("deleted_at"),
                created_by: row.get("created_by"),
                updated_by: row.get("updated_by"),
            })
            .fetch_all(&self.pool)
            .await?;

        Ok((strategies, count as u64))
    }

    async fn find_active_for_scope(
        &self,
        tenant_id: uuid::Uuid,
        warehouse_id: uuid::Uuid,
        product_id: uuid::Uuid,
    ) -> Result<Vec<RemovalStrategy>> {
        let strategies = sqlx::query_as!(
            RemovalStrategy,
            r#"
            SELECT
                strategy_id, tenant_id, name, strategy_type, warehouse_id, product_id,
                active, config, created_at, updated_at, deleted_at, created_by, updated_by
            FROM removal_strategies
            WHERE tenant_id = $1 AND active = true AND deleted_at IS NULL
              AND (warehouse_id IS NULL OR warehouse_id = $2)
              AND (product_id IS NULL OR product_id = $3)
            ORDER BY
                CASE
                    WHEN warehouse_id = $2 AND product_id = $3 THEN 1
                    WHEN warehouse_id = $2 THEN 2
                    WHEN product_id = $3 THEN 3
                    ELSE 4
                END,
                created_at DESC
            "#,
            tenant_id,
            warehouse_id,
            product_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(strategies)
    }

    async fn update(
        &self,
        tenant_id: uuid::Uuid,
        strategy_id: uuid::Uuid,
        request: RemovalStrategyUpdateRequest,
        updated_by: uuid::Uuid,
    ) -> Result<RemovalStrategy> {
        let strategy = sqlx::query_as!(
            RemovalStrategy,
            r#"
            UPDATE removal_strategies
            SET
                name = COALESCE($3, name),
                strategy_type = COALESCE($4, strategy_type),
                warehouse_id = COALESCE($5, warehouse_id),
                product_id = COALESCE($6, product_id),
                active = COALESCE($7, active),
                config = COALESCE($8, config),
                updated_at = NOW(),
                updated_by = $9
            WHERE tenant_id = $1 AND strategy_id = $2 AND deleted_at IS NULL
            RETURNING
                strategy_id, tenant_id, name, strategy_type, warehouse_id, product_id,
                active, config, created_at, updated_at, deleted_at, created_by, updated_by
            "#,
            tenant_id,
            strategy_id,
            request.name,
            request.strategy_type,
            request.warehouse_id,
            request.product_id,
            request.active,
            request.config,
            updated_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(strategy)
    }

    async fn delete(
        &self,
        tenant_id: uuid::Uuid,
        strategy_id: uuid::Uuid,
        _deleted_by: uuid::Uuid,
    ) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE removal_strategies
            SET deleted_at = NOW(), updated_at = NOW(), updated_by = $3
            WHERE tenant_id = $1 AND strategy_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            strategy_id,
            _deleted_by
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn toggle_active(
        &self,
        tenant_id: uuid::Uuid,
        strategy_id: uuid::Uuid,
        active: bool,
        _updated_by: uuid::Uuid,
    ) -> Result<RemovalStrategy> {
        let strategy = sqlx::query_as!(
            RemovalStrategy,
            r#"
            UPDATE removal_strategies
            SET active = $3, updated_at = NOW(), updated_by = $4
            WHERE tenant_id = $1 AND strategy_id = $2 AND deleted_at IS NULL
            RETURNING
                strategy_id, tenant_id, name, strategy_type, warehouse_id, product_id,
                active, config, created_at, updated_at, deleted_at, created_by, updated_by
            "#,
            tenant_id,
            strategy_id,
            active,
            _updated_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(strategy)
    }

    async fn suggest_removal(
        &self,
        tenant_id: uuid::Uuid,
        request: SuggestRemovalRequest,
    ) -> Result<SuggestRemovalResponse> {
        // Get available stock locations for the product in the warehouse
        let stock_locations = self
            .get_available_stock_locations(tenant_id, request.warehouse_id, request.product_id)
            .await?;

        if stock_locations.is_empty() {
            return Ok(SuggestRemovalResponse {
                suggestions: vec![],
                total_suggested: 0,
                strategy_applied: "none".to_string(),
                can_fulfill: false,
            });
        }

        // Get applicable strategies
        let strategies = self
            .find_active_for_scope(tenant_id, request.warehouse_id, request.product_id)
            .await?;

        // Select best strategy or use forced strategy
        let selected_strategy = if let Some(strategy_id) = request.force_strategy_id {
            strategies
                .into_iter()
                .find(|s| s.strategy_id == strategy_id)
                .ok_or_else(|| {
                    inventory_service_core::AppError::NotFound("Strategy not found".to_string())
                })?
        } else if let Some(strategy) = strategies.first() {
            strategy.clone()
        } else {
            // Default to FIFO if no strategies configured
            return self.apply_fifo_strategy(stock_locations, request.quantity);
        };

        // Apply the selected strategy
        match selected_strategy.strategy_type.as_str() {
            "fifo" => self.apply_fifo_strategy(stock_locations, request.quantity),
            "lifo" => self.apply_lifo_strategy(stock_locations, request.quantity),
            "fefo" => {
                self.apply_fefo_strategy(stock_locations, request.quantity, &selected_strategy)
            },
            "closest_location" => self.apply_closest_location_strategy(
                stock_locations,
                request.quantity,
                &selected_strategy,
            ),
            "least_packages" => {
                self.apply_least_packages_strategy(stock_locations, request.quantity)
            },
            _ => self.apply_fifo_strategy(stock_locations, request.quantity),
        }
    }

    async fn get_available_stock_locations(
        &self,
        tenant_id: uuid::Uuid,
        warehouse_id: uuid::Uuid,
        product_id: uuid::Uuid,
    ) -> Result<Vec<StockLocationInfo>> {
        let locations = sqlx::query!(
            r#"
        SELECT
            wl.location_id,
            wl.location_code,
            COALESCE(il.available_quantity, 0) as available_quantity,
            NULL::UUID as lot_serial_id,
            NULL::TIMESTAMPTZ as expiry_date,
            (SELECT move_date FROM stock_moves
             WHERE tenant_id = wl.tenant_id
               AND destination_location_id = wl.location_id
               AND product_id = $3
               AND move_type = 'in'
             ORDER BY move_date DESC
             LIMIT 1) as last_receipt_date
        FROM warehouse_locations wl
        LEFT JOIN inventory_levels il ON il.warehouse_id = wl.warehouse_id
            AND il.tenant_id = wl.tenant_id
            AND il.product_id = $3
            AND il.deleted_at IS NULL
        WHERE wl.tenant_id = $1
          AND wl.warehouse_id = $2
          AND wl.deleted_at IS NULL
          AND COALESCE(il.available_quantity, 0) > 0
        ORDER BY wl.location_code
    "#,
            tenant_id,
            warehouse_id,
            product_id
        )
        .map(|row| StockLocationInfo {
            location_id: row.location_id,
            location_code: row.location_code,
            available_quantity: row.available_quantity.unwrap_or(0),
            lot_serial_id: row.lot_serial_id,
            expiry_date: row.expiry_date,
            last_receipt_date: row.last_receipt_date,
        })
        .fetch_all(&self.pool)
        .await?;

        Ok(locations)
    }

    async fn record_strategy_usage(
        &self,
        _tenant_id: uuid::Uuid,
        _strategy_id: uuid::Uuid,
        _product_id: uuid::Uuid,
        _quantity: i64,
        _pick_time_seconds: Option<f64>,
    ) -> Result<bool> {
        // For now, just return success. In a real implementation, you'd insert into a usage table
        // This would be used for analytics
        Ok(true)
    }
}

impl RemovalStrategyRepositoryImpl {
    fn apply_fifo_strategy(
        &self,
        locations: Vec<StockLocationInfo>,
        required_quantity: i64,
    ) -> Result<SuggestRemovalResponse> {
        let mut suggestions = Vec::new();
        let mut remaining = required_quantity;
        let mut total_suggested = 0;

        // Sort by last receipt date (oldest first)
        let mut sorted_locations = locations;
        sorted_locations.sort_by(|a, b| {
            let a_date = a
                .last_receipt_date
                .unwrap_or(chrono::DateTime::<chrono::Utc>::MAX_UTC);
            let b_date = b
                .last_receipt_date
                .unwrap_or(chrono::DateTime::<chrono::Utc>::MAX_UTC);
            a_date.cmp(&b_date)
        });

        for location in sorted_locations {
            if remaining <= 0 {
                break;
            }

            let suggest_qty = remaining.min(location.available_quantity);
            if suggest_qty > 0 {
                suggestions.push(StockSuggestion {
                    location_id: location.location_id,
                    location_code: location.location_code.clone(),
                    available_quantity: location.available_quantity,
                    suggested_quantity: suggest_qty,
                    lot_serial_id: location.lot_serial_id,
                    expiry_date: location.expiry_date,
                    strategy_used: "fifo".to_string(),
                    strategy_reason: "Oldest stock first".to_string(),
                });
                remaining -= suggest_qty;
                total_suggested += suggest_qty;
            }
        }

        Ok(SuggestRemovalResponse {
            suggestions,
            total_suggested,
            strategy_applied: "fifo".to_string(),
            can_fulfill: remaining <= 0,
        })
    }

    fn apply_fefo_strategy(
        &self,
        locations: Vec<StockLocationInfo>,
        required_quantity: i64,
        strategy: &RemovalStrategy,
    ) -> Result<SuggestRemovalResponse> {
        let mut suggestions = Vec::new();
        let mut remaining = required_quantity;
        let mut total_suggested = 0;

        // Sort by expiry date (soonest first), considering buffer days
        let buffer_days = strategy.fefo_buffer_days() as i64;
        let now = chrono::Utc::now();
        let buffer_date = now + chrono::Duration::days(buffer_days);

        let mut sorted_locations = locations;
        sorted_locations.sort_by(|a, b| {
            let a_date = a
                .expiry_date
                .unwrap_or(chrono::DateTime::<chrono::Utc>::MAX_UTC);
            let b_date = b
                .expiry_date
                .unwrap_or(chrono::DateTime::<chrono::Utc>::MAX_UTC);
            a_date.cmp(&b_date)
        });

        for location in sorted_locations {
            if remaining <= 0 {
                break;
            }

            // Skip items that expire too soon (within buffer period)
            if let Some(expiry) = location.expiry_date {
                if expiry < buffer_date {
                    continue;
                }
            }

            let suggest_qty = remaining.min(location.available_quantity);
            if suggest_qty > 0 {
                suggestions.push(StockSuggestion {
                    location_id: location.location_id,
                    location_code: location.location_code.clone(),
                    available_quantity: location.available_quantity,
                    suggested_quantity: suggest_qty,
                    lot_serial_id: location.lot_serial_id,
                    expiry_date: location.expiry_date,
                    strategy_used: "fefo".to_string(),
                    strategy_reason: format!("Expires after buffer period ({} days)", buffer_days),
                });
                remaining -= suggest_qty;
                total_suggested += suggest_qty;
            }
        }

        Ok(SuggestRemovalResponse {
            suggestions,
            total_suggested,
            strategy_applied: "fefo".to_string(),
            can_fulfill: remaining <= 0,
        })
    }

    fn apply_closest_location_strategy(
        &self,
        locations: Vec<StockLocationInfo>,
        required_quantity: i64,
        strategy: &RemovalStrategy,
    ) -> Result<SuggestRemovalResponse> {
        let mut suggestions = Vec::new();
        let mut remaining = required_quantity;
        let mut total_suggested = 0;

        // Get location priorities from config
        let priorities = strategy.location_priorities();

        // Sort locations by priority
        let mut sorted_locations = locations;
        sorted_locations.sort_by(|a, b| {
            let a_priority = priorities
                .iter()
                .position(|p| p == &a.location_code)
                .unwrap_or(usize::MAX);
            let b_priority = priorities
                .iter()
                .position(|p| p == &b.location_code)
                .unwrap_or(usize::MAX);
            a_priority.cmp(&b_priority)
        });

        for location in sorted_locations {
            if remaining <= 0 {
                break;
            }

            let suggest_qty = remaining.min(location.available_quantity);
            if suggest_qty > 0 {
                suggestions.push(StockSuggestion {
                    location_id: location.location_id,
                    location_code: location.location_code.clone(),
                    available_quantity: location.available_quantity,
                    suggested_quantity: suggest_qty,
                    lot_serial_id: location.lot_serial_id,
                    expiry_date: location.expiry_date,
                    strategy_used: "closest_location".to_string(),
                    strategy_reason: format!("Location priority: {}", location.location_code),
                });
                remaining -= suggest_qty;
                total_suggested += suggest_qty;
            }
        }

        Ok(SuggestRemovalResponse {
            suggestions,
            total_suggested,
            strategy_applied: "closest_location".to_string(),
            can_fulfill: remaining <= 0,
        })
    }

    fn apply_least_packages_strategy(
        &self,
        locations: Vec<StockLocationInfo>,
        required_quantity: i64,
    ) -> Result<SuggestRemovalResponse> {
        let mut suggestions = Vec::new();
        let mut remaining = required_quantity;
        let mut total_suggested = 0;

        // Sort by available quantity (largest first to minimize packages)
        let mut sorted_locations = locations;
        sorted_locations.sort_by(|a, b| b.available_quantity.cmp(&a.available_quantity));

        for location in sorted_locations {
            if remaining <= 0 {
                break;
            }

            let suggest_qty = remaining.min(location.available_quantity);
            if suggest_qty > 0 {
                suggestions.push(StockSuggestion {
                    location_id: location.location_id,
                    location_code: location.location_code.clone(),
                    available_quantity: location.available_quantity,
                    suggested_quantity: suggest_qty,
                    lot_serial_id: location.lot_serial_id,
                    expiry_date: location.expiry_date,
                    strategy_used: "least_packages".to_string(),
                    strategy_reason: "Minimize number of locations accessed".to_string(),
                });
                remaining -= suggest_qty;
                total_suggested += suggest_qty;
            }
        }

        Ok(SuggestRemovalResponse {
            suggestions,
            total_suggested,
            strategy_applied: "least_packages".to_string(),
            can_fulfill: remaining <= 0,
        })
    }

    fn apply_lifo_strategy(
        &self,
        locations: Vec<StockLocationInfo>,
        required_quantity: i64,
    ) -> Result<SuggestRemovalResponse> {
        let mut suggestions = Vec::new();
        let mut remaining = required_quantity;
        let mut total_suggested = 0;

        // Sort by last receipt date (newest first)
        let mut sorted_locations = locations;
        sorted_locations.sort_by(|a, b| {
            let a_date = a
                .last_receipt_date
                .unwrap_or(chrono::DateTime::<chrono::Utc>::MIN_UTC);
            let b_date = b
                .last_receipt_date
                .unwrap_or(chrono::DateTime::<chrono::Utc>::MIN_UTC);
            b_date.cmp(&a_date)
        });

        for location in sorted_locations {
            if remaining <= 0 {
                break;
            }

            let suggest_qty = remaining.min(location.available_quantity);
            if suggest_qty > 0 {
                suggestions.push(StockSuggestion {
                    location_id: location.location_id,
                    location_code: location.location_code.clone(),
                    available_quantity: location.available_quantity,
                    suggested_quantity: suggest_qty,
                    lot_serial_id: location.lot_serial_id,
                    expiry_date: location.expiry_date,
                    strategy_used: "lifo".to_string(),
                    strategy_reason: "Newest stock first".to_string(),
                });
                remaining -= suggest_qty;
                total_suggested += suggest_qty;
            }
        }

        Ok(SuggestRemovalResponse {
            suggestions,
            total_suggested,
            strategy_applied: "lifo".to_string(),
            can_fulfill: remaining <= 0,
        })
    }
}
