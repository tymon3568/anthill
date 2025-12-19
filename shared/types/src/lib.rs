// Common types used across services
pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;

// Money as cents (i64 to avoid floating point issues)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct Money(pub i64);

// Tenant context for multi-tenancy
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: Uuid,
}
