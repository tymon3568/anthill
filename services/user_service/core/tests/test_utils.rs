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
    password_hash: Option<String>, // Now Option<String>
    email_verified: bool,
    full_name: Option<String>,
    role: String,
    status: String,
    auth_method: String,          // NEW
    kanidm_user_id: Option<Uuid>, // NEW
}

impl UserBuilder {
    /// Create a new UserBuilder with random default values
    pub fn new() -> Self {
        Self {
            user_id: Uuid::now_v7(),
            tenant_id: Uuid::now_v7(),
            email: SafeEmail().fake(),
            password_hash: Some(format!("$2b$12${}", Password(8..16).fake::<String>())), // Some()
            email_verified: true,
            full_name: Some(Name().fake()),
            role: "user".to_string(),
            status: "active".to_string(),
            auth_method: "password".to_string(), // NEW: Default to password auth
            kanidm_user_id: None,                // NEW
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
        self.password_hash = Some(password_hash.into()); // Wrap in Some()
        self
    }

    /// Set password hash to None (Kanidm-only user)
    pub fn without_password(mut self) -> Self {
        self.password_hash = None;
        self
    }

    /// Set auth method
    pub fn with_auth_method(mut self, auth_method: impl Into<String>) -> Self {
        self.auth_method = auth_method.into();
        self
    }

    /// Set Kanidm user ID
    pub fn with_kanidm_user_id(mut self, kanidm_user_id: Uuid) -> Self {
        self.kanidm_user_id = Some(kanidm_user_id);
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
            password_hash: self.password_hash, // Now Option<String>
            email_verified: self.email_verified,
            email_verified_at: if self.email_verified { Some(now) } else { None },
            full_name: self.full_name,
            avatar_url: None,
            phone: None,
            role: self.role,
            status: self.status,
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            password_changed_at: Some(now),
            kanidm_user_id: self.kanidm_user_id, // NEW
            kanidm_synced_at: if self.kanidm_user_id.is_some() {
                Some(now)
            } else {
                None
            }, // NEW
            auth_method: self.auth_method,       // NEW
            migration_invited_at: None,          // NEW
            migration_completed_at: if self.kanidm_user_id.is_some() {
                Some(now)
            } else {
                None
            }, // NEW
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

/// Builder for creating test Session instances
#[derive(Debug, Clone)]
pub struct SessionBuilder {
    session_id: Uuid,
    user_id: Uuid,
    tenant_id: Uuid,
    access_token_hash: String,
    refresh_token_hash: String,
    ip_address: Option<String>,
    user_agent: Option<String>,
    revoked: bool,
}

impl SessionBuilder {
    /// Create a new SessionBuilder with default values
    pub fn new() -> Self {
        Self {
            session_id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            tenant_id: Uuid::now_v7(),
            access_token_hash: format!("access_hash_{}", Uuid::now_v7()),
            refresh_token_hash: format!("refresh_hash_{}", Uuid::now_v7()),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Mozilla/5.0 Test Agent".to_string()),
            revoked: false,
        }
    }

    pub fn with_session_id(mut self, session_id: Uuid) -> Self {
        self.session_id = session_id;
        self
    }

    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn with_tenant_id(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = tenant_id;
        self
    }

    pub fn with_access_token_hash(mut self, hash: impl Into<String>) -> Self {
        self.access_token_hash = hash.into();
        self
    }

    pub fn with_refresh_token_hash(mut self, hash: impl Into<String>) -> Self {
        self.refresh_token_hash = hash.into();
        self
    }

    pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    pub fn with_revoked(mut self, revoked: bool) -> Self {
        self.revoked = revoked;
        self
    }

    pub fn build(self) -> user_service_core::domains::auth::domain::model::Session {
        use user_service_core::domains::auth::domain::model::Session;
        let now = Utc::now();

        Session {
            session_id: self.session_id,
            user_id: self.user_id,
            tenant_id: self.tenant_id,
            access_token_hash: Some(self.access_token_hash), // Now Option<String>
            refresh_token_hash: Some(self.refresh_token_hash), // Now Option<String>
            ip_address: self.ip_address,
            user_agent: self.user_agent,
            device_info: None,
            access_token_expires_at: now + chrono::Duration::minutes(15),
            refresh_token_expires_at: now + chrono::Duration::days(7),
            revoked: self.revoked,
            kanidm_session_id: None,        // NEW: Default to None
            auth_method: "jwt".to_string(), // NEW: Default to JWT auth
            revoked_at: if self.revoked { Some(now) } else { None },
            revoked_reason: if self.revoked {
                Some("test_revoke".to_string())
            } else {
                None
            },
            created_at: now,
            last_used_at: now,
        }
    }
}

impl Default for SessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Test fixtures - Pre-defined test scenarios
pub mod fixtures {
    use super::*;

    /// Complete test environment with tenant, users, and sessions
    pub struct TestEnvironment {
        pub tenant: Tenant,
        pub admin: User,
        pub manager: User,
        pub user: User,
        pub admin_session: user_service_core::domains::auth::domain::model::Session,
    }

    impl TestEnvironment {
        pub fn new() -> Self {
            let tenant = TenantBuilder::new()
                .with_name("Test Corporation")
                .with_slug("test-corp")
                .build();

            let (admin, manager, user) = create_role_based_users(tenant.tenant_id);

            let admin_session = SessionBuilder::new()
                .with_user_id(admin.user_id)
                .with_tenant_id(tenant.tenant_id)
                .build();

            Self {
                tenant,
                admin,
                manager,
                user,
                admin_session,
            }
        }
    }

    impl Default for TestEnvironment {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Locked user scenario for testing account lockout
    pub fn create_locked_user(tenant_id: Uuid) -> User {
        UserBuilder::new()
            .with_tenant_id(tenant_id)
            .with_status("locked")
            .with_email("locked@test.com")
            .build()
    }

    /// Unverified user scenario for email verification testing
    pub fn create_unverified_user(tenant_id: Uuid) -> User {
        UserBuilder::new()
            .with_tenant_id(tenant_id)
            .with_email_verified(false)
            .with_email("unverified@test.com")
            .build()
    }

    /// Expired tenant scenario for subscription testing
    pub fn create_expired_tenant() -> Tenant {
        TenantBuilder::new()
            .with_name("Expired Company")
            .with_plan("trial")
            .with_status("suspended")
            .build()
    }

    /// Multi-tenant test scenario
    pub fn create_multi_tenant_scenario() -> (Tenant, Tenant, Vec<User>, Vec<User>) {
        let tenant_a = TenantBuilder::new()
            .with_name("Company A")
            .with_slug("company-a")
            .build();

        let tenant_b = TenantBuilder::new()
            .with_name("Company B")
            .with_slug("company-b")
            .build();

        let users_a = create_test_users(tenant_a.tenant_id, 3);
        let users_b = create_test_users(tenant_b.tenant_id, 3);

        (tenant_a, tenant_b, users_a, users_b)
    }
}

/// Assertion helpers for testing
pub mod assertions {
    use super::*;

    /// Assert that two users belong to the same tenant
    pub fn assert_same_tenant(user1: &User, user2: &User) {
        assert_eq!(user1.tenant_id, user2.tenant_id, "Users should belong to the same tenant");
    }

    /// Assert that a user has a specific role
    pub fn assert_user_has_role(user: &User, expected_role: &str) {
        assert_eq!(user.role, expected_role, "User should have role '{}'", expected_role);
    }

    /// Assert that a user is active
    pub fn assert_user_is_active(user: &User) {
        assert_eq!(user.status, "active", "User should be active");
        assert!(user.deleted_at.is_none(), "User should not be deleted");
    }

    /// Assert that a user email is verified
    pub fn assert_email_verified(user: &User) {
        assert!(user.email_verified, "User email should be verified");
        assert!(user.email_verified_at.is_some(), "Email verified timestamp should be set");
    }

    /// Assert that a session is valid (not revoked, not expired)
    pub fn assert_session_valid(
        session: &user_service_core::domains::auth::domain::model::Session,
    ) {
        assert!(!session.revoked, "Session should not be revoked");
        assert!(session.revoked_at.is_none(), "Revoked timestamp should be None");
        assert!(
            session.access_token_expires_at > Utc::now(),
            "Access token should not be expired"
        );
    }

    /// Assert that a tenant is active
    pub fn assert_tenant_active(tenant: &Tenant) {
        assert_eq!(tenant.status, "active", "Tenant should be active");
        assert!(tenant.deleted_at.is_none(), "Tenant should not be deleted");
    }
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
        let tenant = TenantBuilder::new().with_name("Test Company Name").build();
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

    #[test]
    fn test_session_builder_defaults() {
        let session = SessionBuilder::new().build();

        assert!(!session.revoked);
        assert!(session.revoked_at.is_none());
        assert_eq!(session.ip_address, Some("127.0.0.1".to_string()));
        assert!(session.user_agent.is_some());
        assert!(session.access_token_expires_at > Utc::now());
        assert!(session.refresh_token_expires_at > Utc::now());
    }

    #[test]
    fn test_session_builder_customization() {
        let user_id = Uuid::now_v7();
        let tenant_id = Uuid::now_v7();

        let session = SessionBuilder::new()
            .with_user_id(user_id)
            .with_tenant_id(tenant_id)
            .with_ip_address("192.168.1.100")
            .with_revoked(true)
            .build();

        assert_eq!(session.user_id, user_id);
        assert_eq!(session.tenant_id, tenant_id);
        assert_eq!(session.ip_address, Some("192.168.1.100".to_string()));
        assert!(session.revoked);
        assert!(session.revoked_at.is_some());
    }

    #[test]
    fn test_fixtures_test_environment() {
        let env = fixtures::TestEnvironment::new();

        assert_eq!(env.admin.tenant_id, env.tenant.tenant_id);
        assert_eq!(env.manager.tenant_id, env.tenant.tenant_id);
        assert_eq!(env.user.tenant_id, env.tenant.tenant_id);

        assert_eq!(env.admin.role, "admin");
        assert_eq!(env.manager.role, "manager");
        assert_eq!(env.user.role, "user");

        assert_eq!(env.admin_session.user_id, env.admin.user_id);
        assert_eq!(env.admin_session.tenant_id, env.tenant.tenant_id);
    }

    #[test]
    fn test_fixtures_locked_user() {
        let tenant_id = Uuid::now_v7();
        let locked_user = fixtures::create_locked_user(tenant_id);

        assert_eq!(locked_user.status, "locked");
        assert_eq!(locked_user.tenant_id, tenant_id);
    }

    #[test]
    fn test_fixtures_unverified_user() {
        let tenant_id = Uuid::now_v7();
        let user = fixtures::create_unverified_user(tenant_id);

        assert!(!user.email_verified);
        assert_eq!(user.tenant_id, tenant_id);
    }

    #[test]
    fn test_fixtures_expired_tenant() {
        let tenant = fixtures::create_expired_tenant();

        assert_eq!(tenant.status, "suspended");
        assert_eq!(tenant.plan, "trial");
    }

    #[test]
    fn test_fixtures_multi_tenant_scenario() {
        let (tenant_a, tenant_b, users_a, users_b) = fixtures::create_multi_tenant_scenario();

        assert_ne!(tenant_a.tenant_id, tenant_b.tenant_id);
        assert_eq!(users_a.len(), 3);
        assert_eq!(users_b.len(), 3);

        for user in &users_a {
            assert_eq!(user.tenant_id, tenant_a.tenant_id);
        }
        for user in &users_b {
            assert_eq!(user.tenant_id, tenant_b.tenant_id);
        }
    }

    #[test]
    fn test_assertions_same_tenant() {
        let tenant_id = Uuid::now_v7();
        let user1 = UserBuilder::new().with_tenant_id(tenant_id).build();
        let user2 = UserBuilder::new().with_tenant_id(tenant_id).build();

        assertions::assert_same_tenant(&user1, &user2);
    }

    #[test]
    fn test_assertions_user_has_role() {
        let user = UserBuilder::new().with_role("admin").build();
        assertions::assert_user_has_role(&user, "admin");
    }

    #[test]
    fn test_assertions_user_is_active() {
        let user = UserBuilder::new().with_status("active").build();
        assertions::assert_user_is_active(&user);
    }

    #[test]
    fn test_assertions_email_verified() {
        let user = UserBuilder::new().with_email_verified(true).build();
        assertions::assert_email_verified(&user);
    }

    #[test]
    fn test_assertions_session_valid() {
        let session = SessionBuilder::new().build();
        assertions::assert_session_valid(&session);
    }

    #[test]
    fn test_assertions_tenant_active() {
        let tenant = TenantBuilder::new().with_status("active").build();
        assertions::assert_tenant_active(&tenant);
    }
}
