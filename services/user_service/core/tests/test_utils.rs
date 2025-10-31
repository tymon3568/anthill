//! Test utilities and helper functions for user_service_core
//!
//! This module provides common utilities for unit testing:
//! - Mock data builders
//! - Test fixtures
//! - Assertion helpers

use chrono::Utc;
use fake::{
    faker::{
        internet::en::{Password, SafeEmail},
        name::en::Name,
    },
    Fake,
};
use user_service_core::domains::auth::domain::model::{Tenant, User};
use uuid::Uuid;

/// Builder for creating test User instances
#[derive(Debug, Clone)]
pub struct UserBuilder {
    user_id: Uuid,
    tenant_id: Uuid,
    email: String,
    password_hash: String,
    email_verified: bool,
    full_name: Option<String>,
    role: String,
    status: String,
}

impl UserBuilder {
    /// Create a new UserBuilder with random default values
    pub fn new() -> Self {
        Self {
            user_id: Uuid::now_v7(),
            tenant_id: Uuid::now_v7(),
            email: SafeEmail().fake(),
            password_hash: format!("$2b$12${}", Password(8..16).fake::<String>()),
            email_verified: true,
            full_name: Some(Name().fake()),
            role: "user".to_string(),
            status: "active".to_string(),
        }
    }

    /// Set a specific user ID
    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = user_id;
        self
    }

    /// Set a specific tenant ID
    pub fn with_tenant_id(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = tenant_id;
        self
    }

    /// Set a specific email
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = email.into();
        self
    }

    /// Set a specific password hash
    pub fn with_password_hash(mut self, password_hash: impl Into<String>) -> Self {
        self.password_hash = password_hash.into();
        self
    }

    /// Set email verification status
    pub fn with_email_verified(mut self, verified: bool) -> Self {
        self.email_verified = verified;
        self
    }

    /// Set full name
    pub fn with_full_name(mut self, name: impl Into<String>) -> Self {
        self.full_name = Some(name.into());
        self
    }

    /// Set role
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = role.into();
        self
    }

    /// Set status
    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    /// Build the User instance
    pub fn build(self) -> User {
        let now = Utc::now();
        User {
            user_id: self.user_id,
            tenant_id: self.tenant_id,
            email: self.email,
            password_hash: self.password_hash,
            email_verified: self.email_verified,
            email_verified_at: if self.email_verified {
                Some(now)
            } else {
                None
            },
            full_name: self.full_name,
            avatar_url: None,
            phone: None,
            role: self.role,
            status: self.status,
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            password_changed_at: Some(now),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

impl Default for UserBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating test Tenant instances
#[derive(Debug, Clone)]
pub struct TenantBuilder {
    tenant_id: Uuid,
    name: String,
    slug: String,
    plan: String,
    status: String,
}

impl TenantBuilder {
    /// Create a new TenantBuilder with random default values
    pub fn new() -> Self {
        let name: String = Name().fake();
        let slug = name.to_lowercase().replace(" ", "-");
        Self {
            tenant_id: Uuid::now_v7(),
            name,
            slug,
            plan: "free".to_string(),
            status: "active".to_string(),
        }
    }

    /// Set a specific tenant ID
    pub fn with_tenant_id(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = tenant_id;
        self
    }

    /// Set tenant name (also updates slug)
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self.slug = self.name.to_lowercase().replace(" ", "-");
        self
    }

    /// Set tenant slug explicitly
    pub fn with_slug(mut self, slug: impl Into<String>) -> Self {
        self.slug = slug.into();
        self
    }

    /// Set plan
    pub fn with_plan(mut self, plan: impl Into<String>) -> Self {
        self.plan = plan.into();
        self
    }

    /// Set status
    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    /// Build the Tenant instance
    pub fn build(self) -> Tenant {
        let now = Utc::now();
        Tenant {
            tenant_id: self.tenant_id,
            name: self.name,
            slug: self.slug,
            plan: self.plan,
            plan_expires_at: None,
            settings: sqlx::types::Json(serde_json::json!({})),
            status: self.status,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

impl Default for TenantBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a batch of test users for a given tenant
pub fn create_test_users(tenant_id: Uuid, count: usize) -> Vec<User> {
    (0..count)
        .map(|_| UserBuilder::new().with_tenant_id(tenant_id).build())
        .collect()
}

/// Create admin, manager, and regular users for testing RBAC
pub fn create_role_based_users(tenant_id: Uuid) -> (User, User, User) {
    let admin = UserBuilder::new()
        .with_tenant_id(tenant_id)
        .with_role("admin")
        .with_email("admin@test.com")
        .build();

    let manager = UserBuilder::new()
        .with_tenant_id(tenant_id)
        .with_role("manager")
        .with_email("manager@test.com")
        .build();

    let user = UserBuilder::new()
        .with_tenant_id(tenant_id)
        .with_role("user")
        .with_email("user@test.com")
        .build();

    (admin, manager, user)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_builder_defaults() {
        let user = UserBuilder::new().build();
        assert_eq!(user.role, "user");
        assert_eq!(user.status, "active");
        assert!(user.email_verified);
        assert!(user.full_name.is_some());
    }

    #[test]
    fn test_user_builder_customization() {
        let tenant_id = Uuid::now_v7();
        let user = UserBuilder::new()
            .with_tenant_id(tenant_id)
            .with_email("custom@test.com")
            .with_role("admin")
            .with_email_verified(false)
            .build();

        assert_eq!(user.tenant_id, tenant_id);
        assert_eq!(user.email, "custom@test.com");
        assert_eq!(user.role, "admin");
        assert!(!user.email_verified);
    }

    #[test]
    fn test_tenant_builder_defaults() {
        let tenant = TenantBuilder::new().build();
        assert_eq!(tenant.plan, "free");
        assert_eq!(tenant.status, "active");
        assert!(!tenant.name.is_empty());
        assert!(!tenant.slug.is_empty());
    }

    #[test]
    fn test_tenant_builder_slug_generation() {
        let tenant = TenantBuilder::new()
            .with_name("Test Company Name")
            .build();
        assert_eq!(tenant.name, "Test Company Name");
        assert_eq!(tenant.slug, "test-company-name");
    }

    #[test]
    fn test_create_test_users() {
        let tenant_id = Uuid::now_v7();
        let users = create_test_users(tenant_id, 5);
        assert_eq!(users.len(), 5);
        assert!(users.iter().all(|u| u.tenant_id == tenant_id));
    }

    #[test]
    fn test_create_role_based_users() {
        let tenant_id = Uuid::now_v7();
        let (admin, manager, user) = create_role_based_users(tenant_id);

        assert_eq!(admin.role, "admin");
        assert_eq!(manager.role, "manager");
        assert_eq!(user.role, "user");

        assert_eq!(admin.tenant_id, tenant_id);
        assert_eq!(manager.tenant_id, tenant_id);
        assert_eq!(user.tenant_id, tenant_id);
    }
}
