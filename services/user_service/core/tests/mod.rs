//! Test module for user_service_core
//!
//! Provides test utilities, mocks, and fixtures for unit testing

pub mod test_utils;
pub mod mocks;

// Re-export commonly used items
pub use test_utils::{UserBuilder, TenantBuilder, create_test_users, create_role_based_users};
pub use mocks::{MockUserRepo, MockTenantRepo, MockSessionRepo};
