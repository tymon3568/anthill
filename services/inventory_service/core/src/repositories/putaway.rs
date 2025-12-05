use async_trait::async_trait;
use shared_error::AppError;
use uuid::Uuid;

use crate::models::{PutawayRequest, PutawayRule, PutawaySuggestion, StorageLocation};

#[async_trait]
pub trait TransactionalPutawayRepository: Send + Sync {
    /// Begin a database transaction
    async fn begin_transaction(
        &self,
    ) -> Result<sqlx::Transaction<'static, sqlx::Postgres>, AppError>;

    /// Update current stock for a storage location within a transaction
    async fn update_location_stock_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        tenant_id: &Uuid,
        location_id: &Uuid,
        new_stock: i64,
    ) -> Result<(), AppError>;
}

#[async_trait]
pub trait PutawayRepository: Send + Sync {
    /// Get all active putaway rules for a tenant, ordered by sequence
    async fn get_active_rules(&self, tenant_id: &Uuid) -> Result<Vec<PutawayRule>, AppError>;

    /// Get storage locations for a warehouse with capacity checks
    async fn get_available_locations(
        &self,
        tenant_id: &Uuid,
        warehouse_id: &Uuid,
        location_type: Option<&str>,
    ) -> Result<Vec<StorageLocation>, AppError>;

    /// Get a specific storage location by ID
    async fn get_location_by_id(
        &self,
        tenant_id: &Uuid,
        location_id: &Uuid,
    ) -> Result<Option<StorageLocation>, AppError>;

    /// Update current stock for a storage location
    async fn update_location_stock(
        &self,
        tenant_id: &Uuid,
        location_id: &Uuid,
        new_stock: i64,
    ) -> Result<(), AppError>;

    /// Create putaway rules
    async fn create_rule(&self, rule: &PutawayRule) -> Result<PutawayRule, AppError>;

    /// Update putaway rule
    async fn update_rule(&self, rule: &PutawayRule) -> Result<(), AppError>;

    /// Delete putaway rule (soft delete)
    async fn delete_rule(&self, tenant_id: &Uuid, rule_id: &Uuid) -> Result<(), AppError>;

    /// Get putaway rules with pagination
    async fn get_rules_paginated(
        &self,
        tenant_id: &Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PutawayRule>, AppError>;

    /// Create storage location
    async fn create_location(
        &self,
        location: &StorageLocation,
    ) -> Result<StorageLocation, AppError>;

    /// Update storage location
    async fn update_location(&self, location: &StorageLocation) -> Result<(), AppError>;

    /// Delete storage location (soft delete)
    async fn delete_location(&self, tenant_id: &Uuid, location_id: &Uuid) -> Result<(), AppError>;

    /// Get storage locations with pagination
    async fn get_locations_paginated(
        &self,
        tenant_id: &Uuid,
        warehouse_id: Option<&Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StorageLocation>, AppError>;
}

#[async_trait]
pub trait PutawayService: Send + Sync {
    /// Evaluate putaway rules and suggest optimal locations
    async fn suggest_putaway_locations(
        &self,
        tenant_id: &Uuid,
        request: &PutawayRequest,
    ) -> Result<Vec<PutawaySuggestion>, AppError>;

    /// Confirm putaway and create stock moves
    async fn confirm_putaway(
        &self,
        tenant_id: &Uuid,
        request: &crate::models::ConfirmPutawayRequest,
        user_id: &Uuid,
    ) -> Result<crate::models::ConfirmPutawayResponse, AppError>;

    /// Validate location capacity for putaway
    async fn validate_location_capacity(
        &self,
        tenant_id: &Uuid,
        location_id: &Uuid,
        quantity: i64,
    ) -> Result<bool, AppError>;
}
