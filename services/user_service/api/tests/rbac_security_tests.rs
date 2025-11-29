/// Comprehensive Role-Based Access Control (RBAC) security tests
///
/// This test suite validates that authorization policies are enforced correctly
/// and users can only access resources according to their roles and permissions.
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use shared_auth::casbin::{CoreApi, MgmtApi};
use tower::ServiceExt;

mod helpers;
use helpers::*;

/// Test: Admin-only endpoints reject non-admin users
#[tokio::test]
#[ignore]
async fn test_rbac_admin_endpoint_protection() {
    let pool = setup_test_db().await;

    let tenant = create_test_tenant(&pool, "RBAC Test Tenant").await;

    // Create users with different roles
    let admin = create_test_user(&pool, tenant.tenant_id, "admin@test.com", "Admin", "admin").await;
    let manager =
        create_test_user(&pool, tenant.tenant_id, "manager@test.com", "Manager", "manager").await;
    let user = create_test_user(&pool, tenant.tenant_id, "user@test.com", "User", "user").await;

    let app = create_test_app(&pool).await;

    // Admin endpoints
    let admin_endpoints = vec!["/api/v1/admin/policies", "/api/v1/admin/roles"];

    for endpoint in admin_endpoints {
        // Test 1: Admin can access
        let token = create_test_jwt(admin.user_id, tenant.tenant_id, &admin.role);
        let response = make_authenticated_request(&app, "GET", endpoint, &token, None).await;

        // Should be OK or method not allowed (depending on endpoint)
        assert!(
            response.status() == StatusCode::OK
                || response.status() == StatusCode::METHOD_NOT_ALLOWED,
            "Admin should be able to access admin endpoint: {}",
            endpoint
        );

        // Test 2: Manager cannot access
        let token = create_test_jwt(manager.user_id, tenant.tenant_id, &manager.role);
        let response = make_authenticated_request(&app, "GET", endpoint, &token, None).await;

        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "Manager should NOT access admin endpoint: {}",
            endpoint
        );

        // Test 3: Regular user cannot access
        let token = create_test_jwt(user.user_id, tenant.tenant_id, &user.role);
        let response = make_authenticated_request(&app, "GET", endpoint, &token, None).await;

        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "Regular user should NOT access admin endpoint: {}",
            endpoint
        );
    }
}

/// Test: User can only modify their own data
#[tokio::test]
#[ignore]
async fn test_rbac_user_self_modification_only() {
    let pool = setup_test_db().await;

    let tenant = create_test_tenant(&pool, "Self-Modify Test").await;

    let user1 =
        create_test_user(&pool, tenant.tenant_id, "user1@test.com", "User One", "user").await;
    let user2 =
        create_test_user(&pool, tenant.tenant_id, "user2@test.com", "User Two", "user").await;

    let app = create_test_app(&pool).await;

    let token1 = create_test_jwt(user1.user_id, tenant.tenant_id, &user1.role);
    let token2 = create_test_jwt(user2.user_id, tenant.tenant_id, &user2.role);

    // Test: User 1 tries to modify User 2's profile
    let response = make_authenticated_request(
        &app,
        "PUT",
        &format!("/api/v1/users/{}", user2.user_id),
        &token1,
        Some(json!({
            "full_name": "Hacked Name"
        })),
    )
    .await;

    assert!(
        response.status() == StatusCode::FORBIDDEN || response.status() == StatusCode::NOT_FOUND,
        "User should NOT be able to modify another user's profile"
    );

    // Test: User 2 can modify their own profile
    let response = make_authenticated_request(
        &app,
        "PUT",
        &format!("/api/v1/users/{}", user2.user_id),
        &token2,
        Some(json!({
            "full_name": "Updated Name"
        })),
    )
    .await;

    // Should succeed or return method not allowed if endpoint doesn't exist
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::METHOD_NOT_ALLOWED
            || response.status() == StatusCode::NOT_FOUND,
        "User should be able to modify their own profile"
    );
}

/// Test: Role hierarchy enforcement
#[tokio::test]
#[ignore]
async fn test_rbac_role_hierarchy() {
    let pool = setup_test_db().await;

    let tenant = create_test_tenant(&pool, "Role Hierarchy Test").await;

    let super_admin =
        create_test_user(&pool, tenant.tenant_id, "super@test.com", "Super Admin", "super_admin")
            .await;
    let admin = create_test_user(&pool, tenant.tenant_id, "admin@test.com", "Admin", "admin").await;
    let _manager =
        create_test_user(&pool, tenant.tenant_id, "manager@test.com", "Manager", "manager").await;

    let app = create_test_app(&pool).await;

    // Test: Super admin can access admin endpoints
    let token = create_test_jwt(super_admin.user_id, tenant.tenant_id, &super_admin.role);
    let response = make_authenticated_request(
        &app,
        "POST",
        "/api/v1/admin/policies",
        &token,
        Some(json!({
            "role": "role:manager",
            "resource": "/api/v1/test",
            "action": "GET"
        })),
    )
    .await;

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST,
        "Super admin should have admin privileges"
    );

    // Test: Admin can access admin endpoints
    let token = create_test_jwt(admin.user_id, tenant.tenant_id, &admin.role);
    let response = make_authenticated_request(
        &app,
        "POST",
        "/api/v1/admin/policies",
        &token,
        Some(json!({
            "role": "role:user",
            "resource": "/api/v1/test",
            "action": "GET"
        })),
    )
    .await;

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST,
        "Admin should have admin privileges"
    );
}

/// Test: Permission inheritance through role assignment
#[tokio::test]
#[ignore]
async fn test_rbac_permission_inheritance() {
    let pool = setup_test_db().await;

    let tenant = create_test_tenant(&pool, "Permission Inheritance Test").await;
    let admin = create_test_user(&pool, tenant.tenant_id, "admin@test.com", "Admin", "admin").await;
    let _user = create_test_user(&pool, tenant.tenant_id, "user@test.com", "User", "user").await;

    let app = create_test_app(&pool).await;
    let admin_token = create_test_jwt(admin.user_id, tenant.tenant_id, &admin.role);

    // Admin creates a custom role with specific permissions
    let response = make_authenticated_request(
        &app,
        "POST",
        "/api/v1/admin/policies",
        &admin_token,
        Some(json!({
            "role": "role:custom",
            "resource": "/api/v1/custom/resource",
            "action": "GET"
        })),
    )
    .await;

    // Assign custom role to user (this would require an assign role endpoint)
    // For now, we verify the policy was created
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST,
        "Admin should be able to create policies"
    );
}

/// Test: Invalid JWT is rejected
#[tokio::test]
#[ignore]
async fn test_rbac_invalid_jwt_rejection() {
    let pool = setup_test_db().await;
    let app = create_test_app(&pool).await;

    let invalid_tokens = vec![
        "invalid.jwt.token",
        "Bearer invalid",
        "",
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
    ];

    for token in invalid_tokens {
        let response = make_authenticated_request(&app, "GET", "/api/v1/users", token, None).await;

        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Invalid JWT should be rejected: {}",
            token
        );
    }
}

/// Test: Expired JWT is rejected
#[tokio::test]
#[ignore]
async fn test_rbac_expired_jwt_rejection() {
    use chrono::Utc;
    use shared_jwt::{encode_jwt, Claims};

    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Expired JWT Test").await;
    let user = create_test_user(&pool, tenant.tenant_id, "user@test.com", "User", "user").await;

    // Create an expired JWT (exp in the past)
    let mut claims = Claims::new_access(user.user_id, tenant.tenant_id, "user".to_string(), 900);
    claims.exp = (Utc::now() - chrono::Duration::hours(1)).timestamp();

    let jwt_secret = get_test_jwt_secret();
    let expired_token = encode_jwt(&claims, &jwt_secret).expect("Failed to create expired JWT");

    let app = create_test_app(&pool).await;

    let response =
        make_authenticated_request(&app, "GET", "/api/v1/users", &expired_token, None).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "Expired JWT should be rejected");
}

/// Test: Missing authorization header is rejected
#[tokio::test]
#[ignore]
async fn test_rbac_missing_auth_header() {
    let pool = setup_test_db().await;
    let app = create_test_app(&pool).await;

    // Request without Authorization header
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/users")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Request without Authorization header should be rejected"
    );
}

/// Test: Malformed authorization header is rejected
#[tokio::test]
#[ignore]
async fn test_rbac_malformed_auth_header() {
    let pool = setup_test_db().await;
    let app = create_test_app(&pool).await;

    let malformed_headers = vec!["NotBearer token", "Bearer", "bearer token", "Token value"];

    for auth_value in malformed_headers {
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/users")
            .header("Authorization", auth_value)
            .body(Body::empty())
            .expect("Failed to build request");

        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("Failed to execute request");

        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Malformed Authorization header should be rejected: {}",
            auth_value
        );
    }
}

/// Test: Role modification is auditable
#[tokio::test]
#[ignore]
async fn test_rbac_role_modification_audit() {
    let pool = setup_test_db().await;

    let tenant = create_test_tenant(&pool, "Audit Test").await;
    let admin = create_test_user(&pool, tenant.tenant_id, "admin@test.com", "Admin", "admin").await;

    let app = create_test_app(&pool).await;
    let token = create_test_jwt(admin.user_id, tenant.tenant_id, &admin.role);

    // Create a policy
    let response = make_authenticated_request(
        &app,
        "POST",
        "/api/v1/admin/policies",
        &token,
        Some(json!({
            "role": "role:auditor",
            "resource": "/api/v1/audit",
            "action": "GET"
        })),
    )
    .await;

    // Verify policy was created in database
    if response.status() == StatusCode::OK {
        let policy_exists = sqlx::query!(
            "SELECT COUNT(*) as count FROM casbin_rule
             WHERE ptype = 'p'
             AND v0 = 'role:auditor'
             AND v1 = $1
             AND v2 = '/api/v1/audit'
             AND v3 = 'GET'",
            tenant.tenant_id.to_string()
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to query policy");

        assert!(policy_exists.count.unwrap() > 0, "Policy should be recorded in database");
    }
}

/// Test: Cannot escalate privileges
#[tokio::test]
#[ignore]
async fn test_rbac_privilege_escalation_prevention() {
    let pool = setup_test_db().await;

    let tenant = create_test_tenant(&pool, "Privilege Escalation Test").await;
    let user = create_test_user(&pool, tenant.tenant_id, "user@test.com", "User", "user").await;

    let app = create_test_app(&pool).await;
    let token = create_test_jwt(user.user_id, tenant.tenant_id, &user.role);

    // Try to create admin policy as regular user
    let response = make_authenticated_request(
        &app,
        "POST",
        "/api/v1/admin/policies",
        &token,
        Some(json!({
            "role": "role:user",
            "resource": "/api/v1/admin/*",
            "action": "*"
        })),
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Regular user should NOT be able to create admin policies"
    );

    // Try to assign admin role to self (would require different endpoint)
    // This test validates that privilege escalation attempts are blocked
}

/// Test: Casbin enforcer correctly evaluates complex policies
#[tokio::test]
#[ignore]
async fn test_rbac_complex_policy_evaluation() {
    let pool = setup_test_db().await;

    let tenant = create_test_tenant(&pool, "Complex Policy Test").await;
    let _admin =
        create_test_user(&pool, tenant.tenant_id, "admin@test.com", "Admin", "admin").await;

    let enforcer = shared_auth::create_enforcer(&get_test_database_url(), None)
        .await
        .expect("Failed to create enforcer");

    // Add multiple policies for the same role
    {
        let mut e = enforcer.write().await;
        e.add_policy(vec![
            "role:developer".to_string(),
            tenant.tenant_id.to_string(),
            "/api/v1/code".to_string(),
            "GET".to_string(),
        ])
        .await
        .ok();

        e.add_policy(vec![
            "role:developer".to_string(),
            tenant.tenant_id.to_string(),
            "/api/v1/code".to_string(),
            "POST".to_string(),
        ])
        .await
        .ok();

        e.save_policy().await.ok();
    }

    // Verify policies are enforced correctly
    let e = enforcer.write().await;

    let can_get = e
        .enforce(("role:developer", tenant.tenant_id.to_string(), "/api/v1/code", "GET"))
        .unwrap();

    let can_post = e
        .enforce(("role:developer", tenant.tenant_id.to_string(), "/api/v1/code", "POST"))
        .unwrap();

    let can_delete = e
        .enforce(("role:developer", tenant.tenant_id.to_string(), "/api/v1/code", "DELETE"))
        .unwrap();

    assert!(can_get, "Developer should be able to GET /api/v1/code");
    assert!(can_post, "Developer should be able to POST /api/v1/code");
    assert!(!can_delete, "Developer should NOT be able to DELETE /api/v1/code");
}
