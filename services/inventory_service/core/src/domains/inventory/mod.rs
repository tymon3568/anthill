pub mod dto;
pub mod picking_method;
pub mod product;
pub mod reconciliation;
pub mod removal_strategy;
pub mod stock_take;
pub mod transfer;
pub mod valuation;
pub mod warehouse;
pub mod warehouse_location;
pub mod warehouse_zone;

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Common trait for entities with soft delete and audit fields
pub trait BaseEntity {
    fn id(&self) -> Uuid;
    fn tenant_id(&self) -> Uuid;
    fn code(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn is_active(&self) -> bool;
    fn is_deleted(&self) -> bool;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
    fn deleted_at(&self) -> Option<DateTime<Utc>>;

    /// Check if entity is active (active flag and not deleted)
    fn is_active_status(&self) -> bool {
        self.is_active() && !self.is_deleted()
    }

    /// Get display name (code + name)
    fn display_name(&self) -> String {
        format!("{} ({})", self.name(), self.code())
    }

    /// Mark as deleted (soft delete)
    fn mark_deleted(&mut self);

    /// Update timestamps
    fn touch(&mut self);
}
