//! Custom middleware for the inventory service

pub mod idempotency;

pub use idempotency::*;
pub use shared_auth::middleware::{casbin_middleware, AuthzState};
