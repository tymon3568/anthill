//! Putaway repository implementation
//!
//! PostgreSQL implementation of the PutawayRepository trait.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use inventory_service_core::models::{PutawayRule, StorageLocation};
use inventory_service_core::repositories::putaway::PutawayRepository;
use shared_error::AppError;

/// PostgreSQL implementation of PutawayRepository
pub struct PgPutawayRepository {
    pool: PgPool,
}

impl PgPutawayRepository {
    /// Create new repository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PutawayRepository for PgPutawayRepository {
    async fn get_active_rules(&self, tenant_id: &Uuid) -> Result<Vec<PutawayRule>, AppError> {
        let rules = sqlx::query_as::<_, PutawayRule>(
            r#"
            SELECT
                rule_id,
                tenant_id,
                name,
                description,
                sequence,
                product_id,
                product_category_id,
                warehouse_id,
                preferred_location_type,
                preferred_zone,
                preferred_aisle,
                conditions,
                rule_type,
                match_mode,
                max_quantity,
                min_quantity,
                priority_score,
                is_active,
                created_by,
                updated_by,
                created_at,
                updated_at,
                deleted_at
            FROM putaway_rules
            WHERE tenant_id = $1 AND is_active = true AND deleted_at IS NULL
            ORDER BY sequence ASC, priority_score DESC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch putaway rules: {}", e)))?;

        Ok(rules)
    }

    async fn get_available_locations(
        &self,
        tenant_id: &Uuid,
        warehouse_id: &Uuid,
        location_type: Option<&str>,
    ) -> Result<Vec<StorageLocation>, AppError> {
        let mut query = sqlx::query_as::<_, StorageLocation>(
            r#"
            SELECT
                location_id,
                tenant_id,
                warehouse_id,
                location_code,
                location_type,
                zone,
                aisle,
                rack,
                level,
                position,
                capacity,
                current_stock,
                is_active,
                is_quarantine,
                is_picking_location,
                length_cm,
                width_cm,
                height_cm,
                weight_limit_kg,
                created_by,
                updated_by,
                created_at,
                updated_at,
                deleted_at
            FROM storage_locations
            WHERE tenant_id = $1 AND warehouse_id = $2 AND is_active = true AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(warehouse_id);

        if let Some(loc_type) = location_type {
            query = sqlx::query_as::<_, StorageLocation>(
                r#"
                SELECT
                    location_id,
                    tenant_id,
                    warehouse_id,
                    location_code,
                    location_type,
                    zone,
                    aisle,
                    rack,
                    level,
                    position,
                    capacity,
                    current_stock,
                    is_active,
                    is_quarantine,
                    is_picking_location,
                    length_cm,
                    width_cm,
                    height_cm,
                    weight_limit_kg,
                    created_by,
                    updated_by,
                    created_at,
                    updated_at,
                    deleted_at
                FROM storage_locations
                WHERE tenant_id = $1 AND warehouse_id = $2 AND location_type = $3
                  AND is_active = true AND deleted_at IS NULL
                "#,
            )
            .bind(tenant_id)
            .bind(warehouse_id)
            .bind(loc_type);
        }

        let locations = query.fetch_all(&self.pool).await.map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch storage locations: {}", e))
        })?;

        Ok(locations)
    }

    async fn get_location_by_id(
        &self,
        tenant_id: &Uuid,
        location_id: &Uuid,
    ) -> Result<Option<StorageLocation>, AppError> {
        let location = sqlx::query_as::<_, StorageLocation>(
            r#"
            SELECT
                location_id,
                tenant_id,
                warehouse_id,
                location_code,
                location_type,
                zone,
                aisle,
                rack,
                level,
                position,
                capacity,
                current_stock,
                is_active,
                is_quarantine,
                is_picking_location,
                length_cm,
                width_cm,
                height_cm,
                weight_limit_kg,
                created_by,
                updated_by,
                created_at,
                updated_at,
                deleted_at
            FROM storage_locations
            WHERE tenant_id = $1 AND location_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(location_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch storage location: {}", e)))?;

        Ok(location)
    }

    async fn update_location_stock(
        &self,
        tenant_id: &Uuid,
        location_id: &Uuid,
        new_stock: i64,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE storage_locations
            SET current_stock = $3, updated_at = NOW()
            WHERE tenant_id = $1 AND location_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            location_id,
            new_stock
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update location stock: {}", e)))?;

        Ok(())
    }

    async fn create_rule(&self, rule: &PutawayRule) -> Result<PutawayRule, AppError> {
        let created_rule = sqlx::query_as::<_, PutawayRule>(
            r#"
            INSERT INTO putaway_rules (
                rule_id, tenant_id, name, description, sequence, product_id,
                product_category_id, warehouse_id, preferred_location_type,
                preferred_zone, preferred_aisle, conditions, rule_type,
                match_mode, max_quantity, min_quantity, priority_score,
                is_active, created_by, updated_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            RETURNING
                rule_id, tenant_id, name, description, sequence, product_id,
                product_category_id, warehouse_id, preferred_location_type,
                preferred_zone, preferred_aisle, conditions, rule_type,
                match_mode, max_quantity, min_quantity,
                priority_score, is_active, created_by, updated_by,
                created_at, updated_at, deleted_at
            "#
        )
        .bind(rule.rule_id)
        .bind(rule.tenant_id)
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(rule.sequence)
        .bind(rule.product_id)
        .bind(rule.product_category_id)
        .bind(rule.warehouse_id)
        .bind(&rule.preferred_location_type)
        .bind(&rule.preferred_zone)
        .bind(&rule.preferred_aisle)
        .bind(&rule.conditions)
        .bind(&rule.rule_type)
        .bind(&rule.match_mode)
        .bind(rule.max_quantity)
        .bind(rule.min_quantity)
        .bind(rule.priority_score)
        .bind(rule.is_active)
        .bind(rule.created_by)
        .bind(rule.updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create putaway rule: {}", e)))?;

        Ok(created_rule)
    }

    async fn update_rule(&self, rule: &PutawayRule) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE putaway_rules
            SET
                name = $1,
                description = $2,
                sequence = $3,
                product_id = $4,
                product_category_id = $5,
                warehouse_id = $6,
                preferred_location_type = $7,
                preferred_zone = $8,
                preferred_aisle = $9,
                conditions = $10,
                rule_type = $11,
                match_mode = $12,
                max_quantity = $13,
                min_quantity = $14,
                priority_score = $15,
                is_active = $16,
                updated_by = $17,
                updated_at = NOW()
            WHERE tenant_id = $18 AND rule_id = $19 AND deleted_at IS NULL
            "#,
        )
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(rule.sequence)
        .bind(rule.product_id)
        .bind(rule.product_category_id)
        .bind(rule.warehouse_id)
        .bind(&rule.preferred_location_type)
        .bind(&rule.preferred_zone)
        .bind(&rule.preferred_aisle)
        .bind(&rule.conditions)
        .bind(&rule.rule_type)
        .bind(&rule.match_mode)
        .bind(rule.max_quantity)
        .bind(rule.min_quantity)
        .bind(rule.priority_score)
        .bind(rule.is_active)
        .bind(rule.updated_by)
        .bind(rule.tenant_id)
        .bind(rule.rule_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update putaway rule: {}", e)))?;

        Ok(())
    }

    async fn delete_rule(&self, tenant_id: &Uuid, rule_id: &Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE putaway_rules
            SET deleted_at = NOW()
            WHERE tenant_id = $1 AND rule_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            rule_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete putaway rule: {}", e)))?;

        Ok(())
    }

    async fn get_rules_paginated(
        &self,
        tenant_id: &Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PutawayRule>, AppError> {
        let rules = sqlx::query_as::<_, PutawayRule>(
            r#"
            SELECT
                rule_id,
                tenant_id,
                name,
                description,
                sequence,
                product_id,
                product_category_id,
                warehouse_id,
                preferred_location_type,
                preferred_zone,
                preferred_aisle,
                conditions,
                rule_type,
                match_mode,
                max_quantity,
                min_quantity,
                priority_score,
                is_active,
                created_by,
                updated_by,
                created_at,
                updated_at,
                deleted_at
            FROM putaway_rules
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY sequence ASC, priority_score DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch putaway rules: {}", e)))?;

        Ok(rules)
    }

    async fn create_location(
        &self,
        location: &StorageLocation,
    ) -> Result<StorageLocation, AppError> {
        let created_location = sqlx::query_as::<_, StorageLocation>(
            r#"
            INSERT INTO storage_locations (
                location_id, tenant_id, warehouse_id, location_code, location_type,
                zone, aisle, rack, level, position, capacity, current_stock,
                is_active, is_quarantine, is_picking_location, length_cm,
                width_cm, height_cm, weight_limit_kg, created_by, updated_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            RETURNING
                location_id, tenant_id, warehouse_id, location_code, location_type,
                zone, aisle, rack, level, position, capacity, current_stock,
                is_active, is_quarantine, is_picking_location, length_cm,
                width_cm, height_cm, weight_limit_kg, created_by, updated_by,
                created_at, updated_at, deleted_at
            "#
        )
        .bind(location.location_id)
        .bind(location.tenant_id)
        .bind(location.warehouse_id)
        .bind(&location.location_code)
        .bind(&location.location_type)
        .bind(&location.zone)
        .bind(&location.aisle)
        .bind(&location.rack)
        .bind(location.level)
        .bind(location.position)
        .bind(location.capacity)
        .bind(location.current_stock)
        .bind(location.is_active)
        .bind(location.is_quarantine)
        .bind(location.is_picking_location)
        .bind(location.length_cm)
        .bind(location.width_cm)
        .bind(location.height_cm)
        .bind(location.weight_limit_kg)
        .bind(location.created_by)
        .bind(location.updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create storage location: {}", e)))?;

        Ok(created_location)
    }

    async fn update_location(&self, location: &StorageLocation) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE storage_locations
            SET
                location_code = $3,
                location_type = $4,
                zone = $5,
                aisle = $6,
                rack = $7,
                level = $8,
                position = $9,
                capacity = $10,
                current_stock = $11,
                is_active = $12,
                is_quarantine = $13,
                is_picking_location = $14,
                length_cm = $15,
                width_cm = $16,
                height_cm = $17,
                weight_limit_kg = $18,
                updated_by = $19,
                updated_at = NOW()
            WHERE tenant_id = $1 AND location_id = $2 AND deleted_at IS NULL
            "#,
            location.tenant_id,
            location.location_id,
            location.location_code,
            location.location_type,
            location.zone,
            location.aisle,
            location.rack,
            location.level,
            location.position,
            location.capacity,
            location.current_stock,
            location.is_active,
            location.is_quarantine,
            location.is_picking_location,
            location.length_cm,
            location.width_cm,
            location.height_cm,
            location.weight_limit_kg,
            location.updated_by
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update storage location: {}", e))
        })?;

        Ok(())
    }

    async fn delete_location(&self, tenant_id: &Uuid, location_id: &Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE storage_locations
            SET deleted_at = NOW()
            WHERE tenant_id = $1 AND location_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            location_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to delete storage location: {}", e))
        })?;

        Ok(())
    }

    async fn get_locations_paginated(
        &self,
        tenant_id: &Uuid,
        warehouse_id: Option<&Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StorageLocation>, AppError> {
        let locations = if let Some(wh_id) = warehouse_id {
            sqlx::query_as::<_, StorageLocation>(
                r#"
                SELECT
                    location_id,
                    tenant_id,
                    warehouse_id,
                    location_code,
                    location_type,
                    zone,
                    aisle,
                    rack,
                    level,
                    position,
                    capacity,
                    current_stock,
                    is_active,
                    is_quarantine,
                    is_picking_location,
                    length_cm,
                    width_cm,
                    height_cm,
                    weight_limit_kg,
                    created_by,
                    updated_by,
                    created_at,
                    updated_at,
                    deleted_at
                FROM storage_locations
                WHERE tenant_id = $1 AND warehouse_id = $2 AND deleted_at IS NULL
                ORDER BY location_code ASC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(tenant_id)
            .bind(wh_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, StorageLocation>(
                r#"
                SELECT
                    location_id,
                    tenant_id,
                    warehouse_id,
                    location_code,
                    location_type,
                    zone,
                    aisle,
                    rack,
                    level,
                    position,
                    capacity,
                    current_stock,
                    is_active,
                    is_quarantine,
                    is_picking_location,
                    length_cm,
                    width_cm,
                    height_cm,
                    weight_limit_kg,
                    created_by,
                    updated_by,
                    created_at,
                    updated_at,
                    deleted_at
                FROM storage_locations
                WHERE tenant_id = $1 AND deleted_at IS NULL
                ORDER BY warehouse_id ASC, location_code ASC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(tenant_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch storage locations: {}", e))
        })?;

        Ok(locations)
    }
}
