// Tenant Bootstrap & Owner Role Assignment Integration Tests
// Tests for registration bootstrap behavior: owner role for new tenant, user role for existing tenant
// Run: docker-compose -f docker-compose.test.yml up -d && cargo test --test tenant_bootstrap_tests -- --ignored

mod test_database;

use axum::{http::StatusCode, Router};
use serde_json::{json, Value};
use sqlx::PgPool;
use test_database::TestDatabaseConfig;
use uuid::Uuid;

mod test_helpers;
use test_helpers::{create_test_app as create_base_app, make_request};

/// Test helper to create app router
async fn create_test_app(db_pool: PgPool) -> Router {
    let (router, _) = create_base_app(db_pool).await;
    router
}

/// Helper to register a user via API
async fn register_user(
    app: &Router,
    email: &str,
    password: &str,
    full_name: &str,
    tenant_name: &str,
) -> (StatusCode, Value) {
    let payload = json!({
        "email": email,
        "password": password,
        "full_name": full_name,
        "tenant_name": tenant_name
    });

    make_request(app, "POST", "/api/v1/auth/register", Some(payload), None, None).await
}

/// Helper to get tenant owner from database
async fn get_tenant_owner(pool: &PgPool, tenant_slug: &str) -> Option<(Uuid, Uuid)> {
    sqlx::query_as::<_, (Uuid, Option<Uuid>)>(
        r#"
        SELECT tenant_id, owner_user_id
        FROM tenants
        WHERE slug = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(tenant_slug)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .and_then(|(tid, owner)| owner.map(|o| (tid, o)))
}

/// Helper to get user role from database
async fn get_user_role(pool: &PgPool, user_id: Uuid) -> Option<String> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT role FROM users WHERE user_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

/// Helper to check Casbin grouping exists
async fn casbin_grouping_exists(pool: &PgPool, user_id: Uuid, role: &str, tenant_id: Uuid) -> bool {
    let user_str = user_id.to_string();
    let tenant_str = tenant_id.to_string();

    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM casbin_rule
        WHERE ptype = 'g'
          AND v0 = $1
          AND v1 = $2
          AND v2 = $3
        "#,
    )
    .bind(&user_str)
    .bind(role)
    .bind(&tenant_str)
    .fetch_one(pool)
    .await
    .unwrap_or(0)
        > 0
}

// ============================================================================
// TENANT BOOTSTRAP TESTS
// ============================================================================

/// Test: Register new tenant → user becomes owner
#[tokio::test]
#[ignore]
async fn test_register_new_tenant_assigns_owner_role() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    let unique_tenant = format!("New Company {}", Uuid::now_v7());
    let email = format!("owner-{}@test.com", Uuid::now_v7());

    let (status, response) =
        register_user(&app, &email, "SecurePass123!", "Test Owner", &unique_tenant).await;

    // Verify response
    assert_eq!(status, StatusCode::CREATED, "Registration should succeed");
    assert_eq!(
        response["user"]["role"].as_str(),
        Some("owner"),
        "New tenant creator should have 'owner' role in response"
    );

    // Verify database state
    let user_id = Uuid::parse_str(response["user"]["id"].as_str().unwrap()).unwrap();
    let tenant_id = Uuid::parse_str(response["user"]["tenant_id"].as_str().unwrap()).unwrap();

    // Check user role in DB
    let db_role = get_user_role(db.pool(), user_id).await;
    assert_eq!(db_role, Some("owner".to_string()), "User role in DB should be 'owner'");

    // Check tenant ownership in DB
    let tenant_slug = unique_tenant.to_lowercase().replace(' ', "-");
    let ownership = get_tenant_owner(db.pool(), &tenant_slug).await;
    assert!(ownership.is_some(), "Tenant should have an owner set");
    let (db_tenant_id, db_owner_id) = ownership.unwrap();
    assert_eq!(db_tenant_id, tenant_id, "Tenant ID should match");
    assert_eq!(db_owner_id, user_id, "Owner should be the registering user");

    // Check Casbin grouping
    let has_grouping = casbin_grouping_exists(db.pool(), user_id, "owner", tenant_id).await;
    assert!(
        has_grouping,
        "Casbin grouping for owner should exist: (user_id, 'owner', tenant_id)"
    );

    db.cleanup().await;
}

/// Test: Register into existing tenant → user gets 'user' role
#[tokio::test]
#[ignore]
async fn test_register_existing_tenant_assigns_user_role() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    let unique_tenant = format!("Existing Company {}", Uuid::now_v7());

    // First registration creates the tenant (owner)
    let owner_email = format!("owner-{}@test.com", Uuid::now_v7());
    let (owner_status, owner_response) =
        register_user(&app, &owner_email, "SecurePass123!", "Tenant Owner", &unique_tenant).await;

    assert_eq!(owner_status, StatusCode::CREATED);
    assert_eq!(owner_response["user"]["role"].as_str(), Some("owner"));

    let tenant_id = Uuid::parse_str(owner_response["user"]["tenant_id"].as_str().unwrap()).unwrap();

    // Second registration joins the existing tenant (user)
    let user_email = format!("employee-{}@test.com", Uuid::now_v7());
    let (user_status, user_response) =
        register_user(&app, &user_email, "SecurePass123!", "New Employee", &unique_tenant).await;

    // Verify response
    assert_eq!(user_status, StatusCode::CREATED, "Registration should succeed");
    assert_eq!(
        user_response["user"]["role"].as_str(),
        Some("user"),
        "User joining existing tenant should have 'user' role in response"
    );

    // Verify same tenant
    let user_tenant_id =
        Uuid::parse_str(user_response["user"]["tenant_id"].as_str().unwrap()).unwrap();
    assert_eq!(user_tenant_id, tenant_id, "User should join the same tenant as owner");

    // Verify database state
    let user_id = Uuid::parse_str(user_response["user"]["id"].as_str().unwrap()).unwrap();

    // Check user role in DB
    let db_role = get_user_role(db.pool(), user_id).await;
    assert_eq!(db_role, Some("user".to_string()), "User role in DB should be 'user'");

    // Check Casbin grouping
    let has_grouping = casbin_grouping_exists(db.pool(), user_id, "user", tenant_id).await;
    assert!(
        has_grouping,
        "Casbin grouping for user should exist: (user_id, 'user', tenant_id)"
    );

    // Verify tenant ownership unchanged
    let tenant_slug = unique_tenant.to_lowercase().replace(' ', "-");
    let ownership = get_tenant_owner(db.pool(), &tenant_slug).await;
    assert!(ownership.is_some(), "Tenant should still have an owner");
    let (_, db_owner_id) = ownership.unwrap();
    let owner_id = Uuid::parse_str(owner_response["user"]["id"].as_str().unwrap()).unwrap();
    assert_eq!(db_owner_id, owner_id, "Owner should still be the original registrant");

    db.cleanup().await;
}

/// Test: Tenant isolation - cannot set owner across tenants
#[tokio::test]
#[ignore]
async fn test_tenant_isolation_owner_cannot_cross_tenants() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    // Create two separate tenants
    let tenant_a_name = format!("Tenant A {}", Uuid::now_v7());
    let tenant_b_name = format!("Tenant B {}", Uuid::now_v7());

    // Register owner for Tenant A
    let owner_a_email = format!("owner-a-{}@test.com", Uuid::now_v7());
    let (status_a, response_a) =
        register_user(&app, &owner_a_email, "SecurePass123!", "Owner A", &tenant_a_name).await;

    assert_eq!(status_a, StatusCode::CREATED);
    let tenant_a_id = Uuid::parse_str(response_a["user"]["tenant_id"].as_str().unwrap()).unwrap();
    let owner_a_id = Uuid::parse_str(response_a["user"]["id"].as_str().unwrap()).unwrap();

    // Register owner for Tenant B
    let owner_b_email = format!("owner-b-{}@test.com", Uuid::now_v7());
    let (status_b, response_b) =
        register_user(&app, &owner_b_email, "SecurePass123!", "Owner B", &tenant_b_name).await;

    assert_eq!(status_b, StatusCode::CREATED);
    let tenant_b_id = Uuid::parse_str(response_b["user"]["tenant_id"].as_str().unwrap()).unwrap();
    let owner_b_id = Uuid::parse_str(response_b["user"]["id"].as_str().unwrap()).unwrap();

    // Verify tenants are different
    assert_ne!(tenant_a_id, tenant_b_id, "Tenants should be distinct entities");

    // Verify each tenant has correct owner
    let tenant_a_slug = tenant_a_name.to_lowercase().replace(' ', "-");
    let tenant_b_slug = tenant_b_name.to_lowercase().replace(' ', "-");

    let ownership_a = get_tenant_owner(db.pool(), &tenant_a_slug).await;
    let ownership_b = get_tenant_owner(db.pool(), &tenant_b_slug).await;

    assert!(ownership_a.is_some(), "Tenant A should have owner");
    assert!(ownership_b.is_some(), "Tenant B should have owner");

    let (_, a_owner) = ownership_a.unwrap();
    let (_, b_owner) = ownership_b.unwrap();

    assert_eq!(a_owner, owner_a_id, "Tenant A owner should be Owner A");
    assert_eq!(b_owner, owner_b_id, "Tenant B owner should be Owner B");
    assert_ne!(a_owner, b_owner, "Different tenants should have different owners");

    // Verify Casbin groupings are tenant-scoped
    assert!(
        casbin_grouping_exists(db.pool(), owner_a_id, "owner", tenant_a_id).await,
        "Owner A should have grouping in Tenant A"
    );
    assert!(
        casbin_grouping_exists(db.pool(), owner_b_id, "owner", tenant_b_id).await,
        "Owner B should have grouping in Tenant B"
    );

    // Verify no cross-tenant groupings
    assert!(
        !casbin_grouping_exists(db.pool(), owner_a_id, "owner", tenant_b_id).await,
        "Owner A should NOT have grouping in Tenant B"
    );
    assert!(
        !casbin_grouping_exists(db.pool(), owner_b_id, "owner", tenant_a_id).await,
        "Owner B should NOT have grouping in Tenant A"
    );

    db.cleanup().await;
}

/// Test: Multiple users joining same tenant all get 'user' role
#[tokio::test]
#[ignore]
async fn test_multiple_users_joining_tenant_get_user_role() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    let unique_tenant = format!("Multi User Company {}", Uuid::now_v7());

    // Create owner
    let owner_email = format!("owner-{}@test.com", Uuid::now_v7());
    let (_, owner_response) =
        register_user(&app, &owner_email, "SecurePass123!", "Owner", &unique_tenant).await;

    let tenant_id = Uuid::parse_str(owner_response["user"]["tenant_id"].as_str().unwrap()).unwrap();

    // Create 3 additional users
    let mut user_ids = vec![];
    for i in 1..=3 {
        let user_email = format!("user-{}-{}@test.com", i, Uuid::now_v7());
        let (status, response) = register_user(
            &app,
            &user_email,
            "SecurePass123!",
            &format!("User {}", i),
            &unique_tenant,
        )
        .await;

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(
            response["user"]["role"].as_str(),
            Some("user"),
            "User {} should have 'user' role",
            i
        );
        assert_eq!(
            Uuid::parse_str(response["user"]["tenant_id"].as_str().unwrap()).unwrap(),
            tenant_id,
            "User {} should be in same tenant",
            i
        );

        let user_id = Uuid::parse_str(response["user"]["id"].as_str().unwrap()).unwrap();
        user_ids.push(user_id);
    }

    // Verify all users have correct DB role and Casbin grouping
    for (i, user_id) in user_ids.iter().enumerate() {
        let db_role = get_user_role(db.pool(), *user_id).await;
        assert_eq!(
            db_role,
            Some("user".to_string()),
            "User {} should have 'user' role in DB",
            i + 1
        );

        let has_grouping = casbin_grouping_exists(db.pool(), *user_id, "user", tenant_id).await;
        assert!(has_grouping, "User {} should have Casbin grouping", i + 1);
    }

    // Verify tenant still has only one owner
    let tenant_slug = unique_tenant.to_lowercase().replace(' ', "-");
    let ownership = get_tenant_owner(db.pool(), &tenant_slug).await;
    let (_, owner_id) = ownership.unwrap();
    let expected_owner_id =
        Uuid::parse_str(owner_response["user"]["id"].as_str().unwrap()).unwrap();
    assert_eq!(owner_id, expected_owner_id, "Tenant should still have original owner");

    db.cleanup().await;
}

/// Test: Registration response includes JWT tokens
#[tokio::test]
#[ignore]
async fn test_registration_response_includes_jwt_tokens() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    let unique_tenant = format!("JWT Test Company {}", Uuid::now_v7());

    // Register as owner
    let owner_email = format!("jwt-owner-{}@test.com", Uuid::now_v7());
    let (status, response) =
        register_user(&app, &owner_email, "SecurePass123!", "JWT Owner", &unique_tenant).await;

    assert_eq!(status, StatusCode::CREATED);

    // Verify JWT fields exist
    assert!(
        response["access_token"].as_str().is_some(),
        "Response should include access_token"
    );
    assert!(
        response["refresh_token"].as_str().is_some(),
        "Response should include refresh_token"
    );
    assert_eq!(response["token_type"].as_str(), Some("Bearer"), "Token type should be Bearer");
    assert!(response["expires_in"].as_i64().is_some(), "Response should include expires_in");

    // The JWT claims can be verified by using the token to access protected endpoints
    // For this test, we just verify the response structure is correct
    let access_token = response["access_token"].as_str().unwrap();
    assert!(!access_token.is_empty(), "Access token should not be empty");

    db.cleanup().await;
}

/// Test: Owner can access protected endpoints after registration
#[tokio::test]
#[ignore]
async fn test_owner_can_access_profile_after_registration() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    let unique_tenant = format!("Profile Test Company {}", Uuid::now_v7());

    // Register as owner
    let owner_email = format!("profile-owner-{}@test.com", Uuid::now_v7());
    let (status, response) =
        register_user(&app, &owner_email, "SecurePass123!", "Profile Owner", &unique_tenant).await;

    assert_eq!(status, StatusCode::CREATED);
    let access_token = response["access_token"].as_str().unwrap();

    // Access profile endpoint with the token
    let (profile_status, profile_response) =
        make_request(&app, "GET", "/api/v1/profile", None, Some(access_token), None).await;

    assert_eq!(profile_status, StatusCode::OK, "Owner should be able to access profile");
    assert_eq!(
        profile_response["email"].as_str(),
        Some(owner_email.as_str()),
        "Profile email should match registration"
    );
    assert_eq!(
        profile_response["full_name"].as_str(),
        Some("Profile Owner"),
        "Profile name should match registration"
    );

    db.cleanup().await;
}
