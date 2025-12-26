// Error Handling and Edge Case Integration Tests
// Tests error scenarios, validation, boundary conditions, and resilience
// Run: docker-compose -f docker-compose.test.yml up -d && cargo test --test error_handling_tests -- --ignored

mod test_database;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use shared_auth::extractors::AuthUser;
use test_database::TestDatabaseConfig;
use tower::ServiceExt; // For proper router method resolution

mod test_helpers;
use test_helpers::{create_test_app as create_base_app, make_request};

/// Test helper to create app router with dummy profile route
async fn create_test_app(pool: &sqlx::PgPool) -> Router {
    let (base_router, _) = create_base_app(pool.clone()).await;

    // Extend the router with a dummy /api/v1/profile route for auth testing
    base_router.route(
        "/api/v1/profile",
        get(|_: AuthUser| async { Json(serde_json::json!({"message": "profile endpoint"})) }),
    )
}

// ============================================================================
// INPUT VALIDATION ERROR TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_registration_missing_required_fields() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Missing Fields Test", None).await;

    // Missing password
    let payload = json!({
        "tenant_id": tenant_id,
        "email": "test@example.com",
        "full_name": "Test User"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/auth/register", Some(payload), None, None).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].is_string());

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_registration_invalid_email_formats() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Invalid Email Test", None).await;

    let invalid_emails = vec![
        "not-an-email",
        "@example.com",
        "user@",
        "user @example.com",
        "user@.com",
        "",
        "user@example",
    ];

    for email in invalid_emails {
        let payload = json!({
            "tenant_id": tenant_id,
            "email": email,
            "password": "SecurePass123!",
            "full_name": "Test User"
        });

        let (status, _) =
            make_request(&app, "POST", "/api/v1/auth/register", Some(payload), None, None).await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "Expected validation error for email: {}",
            email
        );
    }

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_registration_weak_passwords() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Weak Password Test", None).await;

    let weak_passwords = vec![
        "123",      // Too short
        "password", // No numbers, no special chars
        "12345678", // Only numbers
        "abc",      // Too short
        "",         // Empty
        "Pass1",    // Too short
    ];

    for password in weak_passwords {
        let payload = json!({
            "tenant_id": tenant_id,
            "email": format!("user-{}@example.com", password.len()),
            "password": password,
            "full_name": "Test User"
        });

        let (status, _) =
            make_request(&app, "POST", "/api/v1/auth/register", Some(payload), None, None).await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "Expected validation error for password: {}",
            password
        );
    }

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_malformed_json_request() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    // Send invalid JSON
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .header("X-Tenant-ID", db.create_tenant("JSON Test", None).await.to_string())
        .body(Body::from("{invalid json"))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_extremely_long_input_values() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Long Input Test", None).await;

    // Extremely long email (over 255 chars)
    let long_email = format!("{}@example.com", "a".repeat(300));
    let payload = json!({
        "tenant_id": tenant_id,
        "email": long_email,
        "password": "SecurePass123!",
        "full_name": "Test User"
    });

    let (status, _) =
        make_request(&app, "POST", "/api/v1/auth/register", Some(payload), None, None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Extremely long full name
    let long_name = "a".repeat(1000);
    let payload = json!({
        "tenant_id": tenant_id,
        "email": "test@example.com",
        "password": "SecurePass123!",
        "full_name": long_name
    });

    let (status, _) =
        make_request(&app, "POST", "/api/v1/auth/register", Some(payload), None, None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    db.cleanup().await;
}

// ============================================================================
// AUTHENTICATION ERROR TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_login_nonexistent_user() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("NonExistent Test", None).await;
    let payload = json!({
        "email": "nonexistent@example.com",
        "password": "SomePassword123!"
    });

    let (status, response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(payload),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(response["error"].is_string());

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_login_wrong_password() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Wrong Password Test", None).await;

    let password_hash = bcrypt::hash("CorrectPassword123!", bcrypt::DEFAULT_COST).unwrap();
    db.create_user(tenant_id, "user@example.com", &password_hash, "user", Some("Test User"))
        .await;

    let payload = json!({
        "email": "user@example.com",
        "password": "WrongPassword123!"
    });

    let (status, _) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(payload),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_missing_authorization_header() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let (status, _) = make_request(&app, "GET", "/api/v1/profile", None, None, None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_malformed_authorization_header() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    // Test various malformed auth headers
    let malformed_tokens = vec![
        "NotBearer token",
        "Bearer",
        "Bearer ",
        "token-without-bearer",
        "",
    ];

    for token in malformed_tokens {
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/profile")
            .header("Authorization", token)
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Expected 401 for token: {}",
            token
        );
    }

    db.cleanup().await;
}

// ============================================================================
// RESOURCE NOT FOUND ERROR TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_get_nonexistent_user() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Not Found Test", None).await;

    let admin_password = bcrypt::hash("AdminPass123!", bcrypt::DEFAULT_COST).unwrap();
    let _admin_id = db
        .create_user(tenant_id, "admin@example.com", &admin_password, "admin", Some("Admin User"))
        .await;

    let admin_login = json!({
        "email": "admin@example.com",
        "password": "AdminPass123!"
    });

    let (status, login_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(admin_login),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let admin_token = login_response["access_token"].as_str().unwrap();

    // Try to get non-existent user
    let fake_uuid = uuid::Uuid::now_v7();
    let (status, _) = make_request(
        &app,
        "GET",
        &format!("/api/v1/admin/users/{}", fake_uuid),
        None,
        Some(admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_update_nonexistent_user() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Update Not Found Test", None).await;

    let admin_password = bcrypt::hash("AdminPass123!", bcrypt::DEFAULT_COST).unwrap();
    let _admin_id = db
        .create_user(tenant_id, "admin@example.com", &admin_password, "admin", Some("Admin User"))
        .await;

    let admin_login = json!({
        "email": "admin@example.com",
        "password": "AdminPass123!"
    });

    let (_status, login_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(admin_login),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    let admin_token = login_response["access_token"].as_str().unwrap();

    let fake_uuid = uuid::Uuid::now_v7();
    let payload = json!({
        "role": "manager"
    });

    let (status, _) = make_request(
        &app,
        "PUT",
        &format!("/api/v1/admin/users/{}/role", fake_uuid),
        Some(payload),
        Some(admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);

    db.cleanup().await;
}

// ============================================================================
// CONCURRENT OPERATION TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_concurrent_duplicate_registrations() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Concurrent Registration Test", None).await;

    // Try to register same email concurrently
    let mut handles = vec![];

    for _ in 0..5 {
        let app_clone = app.clone();
        let tenant_id_clone = tenant_id;

        let handle = tokio::spawn(async move {
            let payload = json!({
                "tenant_id": tenant_id_clone,
                "email": "concurrent@example.com",
                "password": "SecurePass123!",
                "full_name": "Concurrent User"
            });

            make_request(&app_clone, "POST", "/api/v1/auth/register", Some(payload), None, None)
                .await
        });

        handles.push(handle);
    }

    let mut success_count = 0;
    let mut conflict_count = 0;

    for handle in handles {
        let (status, _) = handle.await.unwrap();
        if status == StatusCode::CREATED {
            success_count += 1;
        } else if status == StatusCode::CONFLICT {
            conflict_count += 1;
        }
    }

    // Only one should succeed, others should get conflict
    assert_eq!(success_count, 1, "Expected exactly one successful registration");
    assert_eq!(conflict_count, 4, "Expected four conflict responses");

    db.cleanup().await;
}

// ============================================================================
// BOUNDARY CONDITION TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_empty_string_fields() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Empty String Test", None).await;

    let payload = json!({
        "tenant_id": tenant_id,
        "email": "",
        "password": "",
        "full_name": ""
    });

    let (status, _) =
        make_request(&app, "POST", "/api/v1/auth/register", Some(payload), None, None).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_null_values_in_request() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Null Values Test", None).await;

    let payload = json!({
        "tenant_id": tenant_id,
        "email": null,
        "password": "SecurePass123!",
        "full_name": "Test User"
    });

    let (status, _) =
        make_request(&app, "POST", "/api/v1/auth/register", Some(payload), None, None).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_unicode_and_special_characters() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Unicode Test", None).await;

    // Test with unicode characters in full name
    let payload = json!({
        "tenant_id": tenant_id,
        "email": "unicode@example.com",
        "password": "SecurePass123!",
        "full_name": "测试用户 Test Пользователь 👤"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/auth/register", Some(payload), None, None).await;

    // Should either accept unicode or reject with validation error
    assert!(
        status == StatusCode::CREATED || status == StatusCode::BAD_REQUEST,
        "Unexpected status: {}",
        status
    );

    if status == StatusCode::CREATED {
        assert_eq!(response["user"]["full_name"], "测试用户 Test Пользователь 👤");
    }

    db.cleanup().await;
}

// ============================================================================
// SQL INJECTION PREVENTION TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_sql_injection_attempts_in_email() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let sql_injection_attempts = vec![
        "admin'--",
        "admin' OR '1'='1",
        "'; DROP TABLE users; --",
        "admin'; DELETE FROM users WHERE '1'='1",
    ];

    let tenant_id = db.create_tenant("SQL Injection Test", None).await;
    for sql_attempt in sql_injection_attempts {
        let payload = json!({
            "email": sql_attempt,
            "password": "TestPassword123!"
        });

        let (status, _) = make_request(
            &app,
            "POST",
            "/api/v1/auth/login",
            Some(payload),
            None,
            Some(&tenant_id.to_string()),
        )
        .await;

        // Should either return validation error or unauthorized (not 500)
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::UNAUTHORIZED,
            "SQL injection attempt should not cause server error for: {}",
            sql_attempt
        );
    }

    db.cleanup().await;
}

// ============================================================================
// RATE LIMITING & ABUSE PREVENTION (if implemented)
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_multiple_failed_login_attempts() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Failed Login Test", None).await;

    let password_hash = bcrypt::hash("CorrectPassword123!", bcrypt::DEFAULT_COST).unwrap();
    db.create_user(
        tenant_id,
        "locktest@example.com",
        &password_hash,
        "user",
        Some("Lock Test User"),
    )
    .await;

    // Attempt multiple failed logins
    for i in 1..=10 {
        let payload = json!({
            "email": "locktest@example.com",
            "password": format!("WrongPassword{}", i)
        });

        let (status, _) = make_request(
            &app,
            "POST",
            "/api/v1/auth/login",
            Some(payload),
            None,
            Some(&tenant_id.to_string()),
        )
        .await;

        // All should be unauthorized
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    // After many failed attempts, account might be locked (depending on implementation)
    // Try correct password
    let payload = json!({
        "email": "locktest@example.com",
        "password": "CorrectPassword123!"
    });

    let (status, _) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(payload),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    // Might be OK or LOCKED depending on implementation
    assert!(
        status == StatusCode::OK
            || status == StatusCode::FORBIDDEN
            || status == StatusCode::UNAUTHORIZED,
        "Expected OK or account locked"
    );

    db.cleanup().await;
}
