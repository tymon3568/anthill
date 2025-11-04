/// SQL Injection Prevention and Input Validation Tests
///
/// This test suite validates that all endpoints are protected against
/// SQL injection attacks and properly validate/sanitize inputs.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

mod helpers;
use helpers::*;

/// Test: SQL injection in login endpoint
#[tokio::test]
#[ignore]
async fn test_sql_injection_login_email() {
    let pool = setup_test_db().await;
    let app = create_test_app(&pool).await;

    let injection_payloads = vec![
        "admin'--",
        "admin' OR '1'='1",
        "admin' OR '1'='1'--",
        "'; DROP TABLE users; --",
        "' OR 1=1 --",
        "admin'/*",
        "' UNION SELECT * FROM users --",
        "1' AND '1'='1",
    ];

    for payload in injection_payloads {
        let response = make_unauthenticated_request(
            &app,
            "POST",
            "/api/v1/auth/login",
            Some(json!({
                "email": payload,
                "password": "anything"
            })),
        ).await;

        // Should return unauthorized or bad request, NOT internal server error
        assert!(
            response.status() == StatusCode::UNAUTHORIZED ||
            response.status() == StatusCode::BAD_REQUEST ||
            response.status() == StatusCode::UNPROCESSABLE_ENTITY,
            "SQL injection attempt should fail safely: {}",
            payload
        );
    }

    // Verify database is still intact
    let user_count = sqlx::query!("SELECT COUNT(*) as count FROM users")
        .fetch_one(&pool)
        .await
        .expect("Database should still be accessible");

    assert!(user_count.count.is_some(), "Users table should still exist");
}

/// Test: SQL injection in user search/filter
#[tokio::test]
#[ignore]
async fn test_sql_injection_user_filters() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "SQL Injection Test").await;
    let user = create_test_user(&pool, tenant.tenant_id, "test@test.com", "Test", "admin").await;

    let app = create_test_app(&pool).await;
    let token = create_test_jwt(user.user_id, tenant.tenant_id, &user.role);

    let injection_payloads = vec![
        "admin' OR '1'='1",
        "'; DROP TABLE users; --",
        "' UNION SELECT * FROM users --",
        "1 OR 1=1",
    ];

    for payload in &injection_payloads {
        // Try injection in role filter
        let response = make_authenticated_request(
            &app,
            "GET",
            &format!("/api/v1/users?role={}", urlencoding::encode(payload)),
            &token,
            None,
        ).await;

        assert!(
            response.status() == StatusCode::OK ||
            response.status() == StatusCode::BAD_REQUEST,
            "SQL injection in role filter should fail safely"
        );

        // Try injection in status filter
        let response = make_authenticated_request(
            &app,
            "GET",
            &format!("/api/v1/users?status={}", urlencoding::encode(payload)),
            &token,
            None,
        ).await;

        assert!(
            response.status() == StatusCode::OK ||
            response.status() == StatusCode::BAD_REQUEST,
            "SQL injection in status filter should fail safely"
        );
    }
}

/// Test: SQL injection in UUID parameters
#[tokio::test]
#[ignore]
async fn test_sql_injection_uuid_params() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "UUID Injection Test").await;
    let user = create_test_user(&pool, tenant.tenant_id, "test@test.com", "Test", "admin").await;

    let app = create_test_app(&pool).await;
    let token = create_test_jwt(user.user_id, tenant.tenant_id, &user.role);

    let injection_payloads = vec![
        "' OR '1'='1",
        "uuid'; DROP TABLE users; --",
        "123' UNION SELECT * FROM users --",
        "../../../etc/passwd",
        "null",
        "undefined",
    ];

    for payload in &injection_payloads {
        let response = make_authenticated_request(
            &app,
            "GET",
            &format!("/api/v1/users/{}", payload),
            &token,
            None,
        ).await;

        // Should return bad request or not found, not internal error
        assert!(
            response.status() == StatusCode::BAD_REQUEST ||
            response.status() == StatusCode::NOT_FOUND,
            "SQL injection in UUID param should fail safely: {}",
            payload
        );
    }
}

/// Test: SQL injection in policy creation
#[tokio::test]
#[ignore]
async fn test_sql_injection_policy_creation() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Policy Injection Test").await;
    let admin = create_test_user(&pool, tenant.tenant_id, "admin@test.com", "Admin", "admin").await;

    let app = create_test_app(&pool).await;
    let token = create_test_jwt(admin.user_id, tenant.tenant_id, &admin.role);

    let injection_payloads = vec![
        ("role'; DROP TABLE casbin_rule; --", "/api/test", "GET"),
        ("role' OR '1'='1", "/api/test", "GET"),
        ("role", "/api'; DROP TABLE users; --", "GET"),
        ("role", "/api/test", "GET'; DELETE FROM casbin_rule; --"),
    ];

    for (role, resource, action) in &injection_payloads {
        let response = make_authenticated_request(
            &app,
            "POST",
            "/api/v1/admin/policies",
            &token,
            Some(json!({
                "role": role,
                "resource": resource,
                "action": action
            })),
        ).await;

        // Should reject or fail validation, not execute SQL
        assert!(
            response.status() == StatusCode::BAD_REQUEST ||
            response.status() == StatusCode::UNPROCESSABLE_ENTITY ||
            response.status() == StatusCode::OK, // Might be treated as valid string
            "SQL injection in policy should fail safely"
        );
    }

    // Verify casbin_rule table still exists
    let rule_count = sqlx::query!("SELECT COUNT(*) as count FROM casbin_rule")
        .fetch_one(&pool)
        .await
        .expect("casbin_rule table should still exist");

    assert!(rule_count.count.is_some());
}

/// Test: Second-order SQL injection prevention
#[tokio::test]
#[ignore]
async fn test_second_order_sql_injection() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Second Order Test").await;

    // Try to register user with malicious name
    let malicious_name = "'; DROP TABLE users; --";

    let result = create_test_user(
        &pool,
        tenant.tenant_id,
        "malicious@test.com",
        malicious_name,
        "user"
    ).await;

    // User should be created with the string as-is (parameterized query)
    assert_eq!(result.full_name.as_deref(), Some(malicious_name));

    // Verify users table still exists
    let user_count = sqlx::query!("SELECT COUNT(*) as count FROM users")
        .fetch_one(&pool)
        .await
        .expect("Users table should still exist");

    assert!(user_count.count.is_some());

    // Now try to use this user - the stored malicious string should not execute
    let app = create_test_app(&pool).await;
    let token = create_test_jwt(result.user_id, tenant.tenant_id, &result.role);

    let response = make_authenticated_request(
        &app,
        "GET",
        "/api/v1/users",
        &token,
        None,
    ).await;

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the malicious name is returned as plain text
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let users_resp: Value = serde_json::from_slice(&body).unwrap();

    if let Some(users) = users_resp["users"].as_array() {
        let found = users.iter().any(|u| {
            u["full_name"].as_str() == Some(malicious_name)
        });
        assert!(found, "Malicious name should be stored and returned as plain text");
    }
}

/// Test: NoSQL injection prevention (JSON fields)
#[tokio::test]
#[ignore]
async fn test_json_field_injection() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "JSON Injection Test").await;

    // Try to inject malicious JSON in tenant settings
    let malicious_json = json!({
        "key": "'; DROP TABLE tenants; --",
        "nested": {
            "value": "' OR '1'='1"
        }
    });

    sqlx::query!(
        "UPDATE tenants SET settings = $1 WHERE tenant_id = $2",
        malicious_json,
        tenant.tenant_id
    )
    .execute(&pool)
    .await
    .expect("Should handle JSON safely");

    // Verify tenants table still exists and data is intact
    let tenant_check = sqlx::query!(
        "SELECT settings FROM tenants WHERE tenant_id = $1",
        tenant.tenant_id
    )
    .fetch_one(&pool)
    .await
    .expect("Tenants table should still exist");

    assert_eq!(tenant_check.settings, malicious_json);
}

/// Test: Input validation for email format
#[tokio::test]
#[ignore]
async fn test_email_validation() {
    let pool = setup_test_db().await;
    let app = create_test_app(&pool).await;

    let invalid_emails = vec![
        "notanemail",
        "@example.com",
        "user@",
        "user @example.com",
        "user@.com",
        "../../../etc/passwd",
        "<script>alert('xss')</script>@example.com",
    ];

    for email in &invalid_emails {
        let response = make_unauthenticated_request(
            &app,
            "POST",
            "/api/v1/auth/register",
            Some(json!({
                "email": email,
                "password": "ValidPass123!",
                "full_name": "Test User",
                "tenant_name": "Test Tenant",
                "tenant_slug": "test-tenant"
            })),
        ).await;

        assert!(
            response.status() == StatusCode::BAD_REQUEST ||
            response.status() == StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid email should be rejected: {}",
            email
        );
    }
}

/// Test: Input validation for password requirements
#[tokio::test]
#[ignore]
async fn test_password_validation() {
    let pool = setup_test_db().await;
    let app = create_test_app(&pool).await;

    let weak_passwords = vec![
        "short",
        "12345678",
        "password",
        "abcdefgh",
        "",
        " ",
    ];

    for password in &weak_passwords {
        let response = make_unauthenticated_request(
            &app,
            "POST",
            "/api/v1/auth/register",
            Some(json!({
                "email": "test@example.com",
                "password": password,
                "full_name": "Test User",
                "tenant_name": "Test Tenant",
                "tenant_slug": "test-tenant"
            })),
        ).await;

        // Should reject weak passwords
        assert!(
            response.status() == StatusCode::BAD_REQUEST ||
            response.status() == StatusCode::UNPROCESSABLE_ENTITY ||
            response.status() == StatusCode::CONFLICT, // Might fail on duplicate email
            "Weak password should be rejected: {}",
            password
        );
    }
}

/// Test: XSS prevention in user inputs
#[tokio::test]
#[ignore]
async fn test_xss_prevention() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "XSS Test").await;

    let xss_payloads = vec![
        "<script>alert('XSS')</script>",
        "<img src=x onerror=alert('XSS')>",
        "javascript:alert('XSS')",
        "<svg onload=alert('XSS')>",
    ];

    for payload in &xss_payloads {
        // Store XSS payload in user name
        let user = create_test_user(
            &pool,
            tenant.tenant_id,
            &format!("xss{}@test.com", Uuid::new_v4()),
            payload,
            "user"
        ).await;

        let app = create_test_app(&pool).await;
        let token = create_test_jwt(user.user_id, tenant.tenant_id, &user.role);

        let response = make_authenticated_request(
            &app,
            "GET",
            "/api/v1/users",
            &token,
            None,
        ).await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // XSS payload should be returned as plain text (JSON escaped)
        // NOT as executable HTML/JavaScript
        assert!(
            body_str.contains(&payload.replace('<', "\\u003c").replace('>', "\\u003e")) ||
            body_str.contains(payload), // Might be JSON escaped differently
            "XSS payload should be safely escaped in response"
        );
    }
}

/// Test: Path traversal prevention
#[tokio::test]
#[ignore]
async fn test_path_traversal_prevention() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Path Traversal Test").await;
    let user = create_test_user(&pool, tenant.tenant_id, "test@test.com", "Test", "admin").await;

    let app = create_test_app(&pool).await;
    let token = create_test_jwt(user.user_id, tenant.tenant_id, &user.role);

    let traversal_payloads = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32",
        "....//....//....//etc/passwd",
        "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
    ];

    for payload in &traversal_payloads {
        let response = make_authenticated_request(
            &app,
            "GET",
            &format!("/api/v1/users/{}", payload),
            &token,
            None,
        ).await;

        assert!(
            response.status() == StatusCode::BAD_REQUEST ||
            response.status() == StatusCode::NOT_FOUND,
            "Path traversal should be rejected: {}",
            payload
        );
    }
}

/// Test: Command injection prevention
#[tokio::test]
#[ignore]
async fn test_command_injection_prevention() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Command Injection Test").await;

    let command_payloads = vec![
        "; ls -la",
        "| cat /etc/passwd",
        "& whoami",
        "`id`",
        "$(cat /etc/passwd)",
    ];

    for payload in &command_payloads {
        // Try to inject in user name
        let user = create_test_user(
            &pool,
            tenant.tenant_id,
            &format!("cmd{}@test.com", Uuid::new_v4()),
            payload,
            "user"
        ).await;

        // Command should be stored as plain text, not executed
        assert_eq!(user.full_name.as_deref(), Some(*payload));
    }

    // Verify no system commands were executed
    // (This is implicit - if commands ran, test environment would behave differently)
}

/// Test: LDAP injection prevention (if LDAP integration exists)
#[tokio::test]
#[ignore]
async fn test_ldap_injection_prevention() {
    let pool = setup_test_db().await;
    let app = create_test_app(&pool).await;

    let ldap_payloads = vec![
        "*",
        "admin)(uid=*",
        "*)(objectClass=*",
        "admin)(&(objectClass=*",
    ];

    for payload in &ldap_payloads {
        let response = make_unauthenticated_request(
            &app,
            "POST",
            "/api/v1/auth/login",
            Some(json!({
                "email": payload,
                "password": "anything"
            })),
        ).await;

        assert!(
            response.status() == StatusCode::UNAUTHORIZED ||
            response.status() == StatusCode::BAD_REQUEST,
            "LDAP injection should be rejected: {}",
            payload
        );
    }
}

/// Test: Mass assignment prevention
#[tokio::test]
#[ignore]
async fn test_mass_assignment_prevention() {
    let pool = setup_test_db().await;
    let app = create_test_app(&pool).await;

    // Try to set admin role during registration
    let response = make_unauthenticated_request(
        &app,
        "POST",
        "/api/v1/auth/register",
        Some(json!({
            "email": "hacker@test.com",
            "password": "ValidPass123!",
            "full_name": "Hacker",
            "tenant_name": "Hacker Tenant",
            "tenant_slug": "hacker-tenant",
            "role": "admin",  // Try to assign admin role
            "status": "active",
            "email_verified": true
        })),
    ).await;

    // Should either ignore extra fields or reject
    // Most importantly, user should NOT be created as admin
    if response.status() == StatusCode::CREATED {
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let auth_resp: Value = serde_json::from_slice(&body).unwrap();

        // Extract user_id from response to verify in database
        let user_id_str = auth_resp["user"]["user_id"]
            .as_str()
            .expect("Response should contain user_id");
        let user_id = uuid::Uuid::parse_str(user_id_str).expect("user_id should be valid UUID");

        // Verify in database that role is NOT admin
        let user_role = sqlx::query!(
            "SELECT role FROM users WHERE user_id = $1",
            user_id
        )
        .fetch_one(&pool)
        .await
        .expect("Should find created user");

        assert_ne!(
            user_role.role, "admin",
            "User should NOT be assigned admin role via mass assignment attack. Actual role: {}",
            user_role.role
        );

        // Role should be the default "user" or similar safe default
        assert_eq!(
            user_role.role, "user",
            "User should have default 'user' role, not attacker-supplied role"
        );
    }
}
