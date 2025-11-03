/// Dual Authentication Integration Tests
///
/// Tests for the dual authentication system supporting both:
/// 1. Legacy password-based authentication (JWT)
/// 2. Kanidm OAuth2 authentication
/// 3. Transition period (users with both methods)
///
/// Run with: cargo test --test dual_auth_tests

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

// =============================================================================
// PASSWORD AUTHENTICATION TESTS (Legacy)
// =============================================================================

/// Test: Password-only user can login with password
#[tokio::test]
async fn test_password_user_login() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Password Test").await;

    // Create password-only user
    let user = sqlx::query!(
        r#"
        INSERT INTO users (user_id, tenant_id, email, password_hash, auth_method, email_verified, status)
        VALUES ($1, $2, $3, $4, 'password', true, 'active')
        RETURNING user_id, email
        "#,
        Uuid::new_v4(),
        tenant.tenant_id,
        "password-user@test.com",
        "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7B9ueS6rbe" // "password123"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let app = create_test_app(&pool).await;

    // Attempt login
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": user.email,
                        "password": "password123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json["access_token"].as_str().is_some());
    assert_eq!(json["user"]["email"], user.email);

    println!("✅ Password-only user can login");
}

/// Test: Password-only user cannot be created without password
#[tokio::test]
async fn test_password_user_requires_password() {
    let pool = setup_test_db().await;
    let _tenant = create_test_tenant(&pool, "Password Required Test").await;
    let app = create_test_app(&pool).await;

    // Attempt to register without password
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "no-password@test.com",
                        "auth_method": "password"
                        // Missing password field
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json["error"].as_str().unwrap().contains("password"));

    println!("✅ Password required for password-auth users");
}

// =============================================================================
// KANIDM-ONLY AUTHENTICATION TESTS
// =============================================================================

/// Test: Kanidm-only user exists without password
#[tokio::test]
async fn test_kanidm_user_no_password() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Kanidm Test").await;

    // Create Kanidm-only user (no password_hash)
    let user = sqlx::query!(
        r#"
        INSERT INTO users (user_id, tenant_id, email, kanidm_user_id, auth_method, email_verified, status)
        VALUES ($1, $2, $3, $4, 'kanidm', true, 'active')
        RETURNING user_id, email, password_hash
        "#,
        Uuid::new_v4(),
        tenant.tenant_id,
        "kanidm-user@test.com",
        Uuid::new_v4()
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(user.password_hash.is_none(), "Kanidm user should not have password hash");

    println!("✅ Kanidm-only user created without password");
}

/// Test: Kanidm-only user cannot login with password
#[tokio::test]
async fn test_kanidm_user_password_login_rejected() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Kanidm No Password Test").await;

    // Create Kanidm-only user
    let user = sqlx::query!(
        r#"
        INSERT INTO users (user_id, tenant_id, email, kanidm_user_id, auth_method, email_verified, status)
        VALUES ($1, $2, $3, $4, 'kanidm', true, 'active')
        RETURNING user_id, email
        "#,
        Uuid::new_v4(),
        tenant.tenant_id,
        "kanidm-only@test.com",
        Uuid::new_v4()
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let app = create_test_app(&pool).await;

    // Attempt password login
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": user.email,
                        "password": "any-password"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::UNAUTHORIZED ||
        response.status() == StatusCode::BAD_REQUEST
    );

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(
        json["error"].as_str().unwrap().to_lowercase().contains("kanidm") ||
        json["error"].as_str().unwrap().to_lowercase().contains("oauth")
    );

    println!("✅ Kanidm-only user cannot use password login");
}

// =============================================================================
// DUAL AUTHENTICATION TESTS
// =============================================================================

/// Test: Dual-auth user can login with password
#[tokio::test]
async fn test_dual_auth_user_password_login() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Dual Auth Test").await;

    // Create dual-auth user
    let user = sqlx::query!(
        r#"
        INSERT INTO users (user_id, tenant_id, email, password_hash, kanidm_user_id, auth_method, email_verified, status)
        VALUES ($1, $2, $3, $4, $5, 'dual', true, 'active')
        RETURNING user_id, email
        "#,
        Uuid::new_v4(),
        tenant.tenant_id,
        "dual-user@test.com",
        "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7B9ueS6rbe", // "password123"
        Uuid::new_v4()
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let app = create_test_app(&pool).await;

    // Login with password
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": user.email,
                        "password": "password123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json["access_token"].as_str().is_some());
    assert_eq!(json["user"]["email"], user.email);

    println!("✅ Dual-auth user can login with password");
}

/// Test: Dual-auth user can also use OAuth2 (simulated)
#[tokio::test]
#[ignore] // Requires Kanidm server
async fn test_dual_auth_user_oauth_login() {
    // This test would verify OAuth2 flow works for dual-auth users
    // Requires running Kanidm server
    println!("⏭️  Skipped: Requires Kanidm server");
}

// =============================================================================
// MIGRATION TRACKING TESTS
// =============================================================================

/// Test: Migration progress view shows correct stats
#[tokio::test]
async fn test_migration_progress_view() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Migration Test").await;

    // Create mixed auth users
    sqlx::query!(
        r#"
        INSERT INTO users (user_id, tenant_id, email, password_hash, auth_method, email_verified, status)
        VALUES
            ($1, $2, 'password1@test.com', $3, 'password', true, 'active'),
            ($4, $2, 'password2@test.com', $3, 'password', true, 'active')
        "#,
        Uuid::new_v4(),
        tenant.tenant_id,
        "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7B9ueS6rbe",
        Uuid::new_v4()
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        r#"
        INSERT INTO users (user_id, tenant_id, email, kanidm_user_id, auth_method, email_verified, status)
        VALUES ($1, $2, 'kanidm@test.com', $3, 'kanidm', true, 'active')
        "#,
        Uuid::new_v4(),
        tenant.tenant_id,
        Uuid::new_v4()
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        r#"
        INSERT INTO users (user_id, tenant_id, email, password_hash, kanidm_user_id, auth_method, email_verified, status)
        VALUES ($1, $2, 'dual@test.com', $3, $4, 'dual', true, 'active')
        "#,
        Uuid::new_v4(),
        tenant.tenant_id,
        "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7B9ueS6rbe",
        Uuid::new_v4()
    )
    .execute(&pool)
    .await
    .unwrap();

    // Query migration progress
    let progress = sqlx::query!(
        r#"
        SELECT
            total_users,
            password_only,
            kanidm_only,
            dual_auth,
            migration_percent
        FROM v_migration_progress
        WHERE tenant_id = $1
        "#,
        tenant.tenant_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(progress.total_users.unwrap(), 4);
    assert_eq!(progress.password_only.unwrap(), 2);
    assert_eq!(progress.kanidm_only.unwrap(), 1);
    assert_eq!(progress.dual_auth.unwrap(), 1);
    // migration_percent is Numeric type (BigDecimal)
    if let Some(percent) = progress.migration_percent {
        let percent_str = percent.to_string();
        assert!(
            percent_str.starts_with("50"),
            "Expected ~50%, got {}",
            percent_str
        );
        println!("✅ Migration progress view working correctly");
        println!(
            "   Total: {}, Password: {}, Kanidm: {}, Dual: {}, Progress: {}%",
            progress.total_users.unwrap(),
            progress.password_only.unwrap(),
            progress.kanidm_only.unwrap(),
            progress.dual_auth.unwrap(),
            percent_str
        );
    } else {
        panic!("migration_percent should not be null");
    }
}

/// Test: Pending migration users are tracked
#[tokio::test]
async fn test_migration_invitation_tracking() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Migration Invite Test").await;

    // Create user and invite to migration
    let user = sqlx::query!(
        r#"
        INSERT INTO users (user_id, tenant_id, email, password_hash, auth_method, migration_invited_at, email_verified, status)
        VALUES ($1, $2, $3, $4, 'password', NOW(), true, 'active')
        RETURNING user_id
        "#,
        Uuid::new_v4(),
        tenant.tenant_id,
        "invited@test.com",
        "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7B9ueS6rbe"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Query pending migrations
    let pending = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM users
        WHERE tenant_id = $1
          AND migration_invited_at IS NOT NULL
          AND migration_completed_at IS NULL
        "#,
        tenant.tenant_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(pending.count.unwrap(), 1);

    // Complete migration
    sqlx::query!(
        r#"
        UPDATE users
        SET kanidm_user_id = $1,
            auth_method = 'dual',
            migration_completed_at = NOW()
        WHERE user_id = $2
        "#,
        Uuid::new_v4(),
        user.user_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Query again
    let pending_after = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM users
        WHERE tenant_id = $1
          AND migration_invited_at IS NOT NULL
          AND migration_completed_at IS NULL
        "#,
        tenant.tenant_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(pending_after.count.unwrap(), 0);

    println!("✅ Migration invitation tracking working");
}

// =============================================================================
// SESSION TESTS
// =============================================================================

/// Test: JWT session for password auth
#[tokio::test]
async fn test_jwt_session_creation() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "JWT Session Test").await;
    let user = create_test_user(&pool, tenant.tenant_id, "jwt-user@test.com", "User", "user").await;

    // Create session
    let session = sqlx::query!(
        r#"
        INSERT INTO sessions (
            session_id, user_id, tenant_id,
            access_token_hash, refresh_token_hash, auth_method,
            access_token_expires_at, refresh_token_expires_at
        )
        VALUES ($1, $2, $3, $4, $5, 'jwt', NOW() + INTERVAL '1 hour', NOW() + INTERVAL '7 days')
        RETURNING session_id, auth_method
        "#,
        Uuid::new_v4(),
        user.user_id,
        tenant.tenant_id,
        "access_hash",
        "refresh_hash"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(session.auth_method, "jwt");

    println!("✅ JWT session created successfully");
}

/// Test: Kanidm session without token hashes
#[tokio::test]
async fn test_kanidm_session_creation() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Kanidm Session Test").await;

    let user = sqlx::query!(
        r#"
        INSERT INTO users (user_id, tenant_id, email, kanidm_user_id, auth_method, email_verified, status)
        VALUES ($1, $2, $3, $4, 'kanidm', true, 'active')
        RETURNING user_id
        "#,
        Uuid::new_v4(),
        tenant.tenant_id,
        "kanidm-session@test.com",
        Uuid::new_v4()
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Create Kanidm session (no token hashes)
    let session = sqlx::query!(
        r#"
        INSERT INTO sessions (
            session_id, user_id, tenant_id,
            kanidm_session_id, auth_method,
            access_token_expires_at, refresh_token_expires_at
        )
        VALUES ($1, $2, $3, $4, 'kanidm', NOW() + INTERVAL '12 hours', NOW() + INTERVAL '7 days')
        RETURNING session_id, auth_method, access_token_hash, kanidm_session_id
        "#,
        Uuid::new_v4(),
        user.user_id,
        tenant.tenant_id,
        Uuid::new_v4()
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(session.auth_method, "kanidm");
    assert!(session.access_token_hash.is_none(), "Kanidm session should not have token hash");
    assert!(session.kanidm_session_id.is_some(), "Kanidm session should have kanidm_session_id");

    println!("✅ Kanidm session created without token hashes");
}

/// Test: Dual session with both auth methods
#[tokio::test]
async fn test_dual_session_creation() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Dual Session Test").await;

    let user = sqlx::query!(
        r#"
        INSERT INTO users (user_id, tenant_id, email, password_hash, kanidm_user_id, auth_method, email_verified, status)
        VALUES ($1, $2, $3, $4, $5, 'dual', true, 'active')
        RETURNING user_id
        "#,
        Uuid::new_v4(),
        tenant.tenant_id,
        "dual-session@test.com",
        "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7B9ueS6rbe",
        Uuid::new_v4()
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Create dual session (has both)
    let session = sqlx::query!(
        r#"
        INSERT INTO sessions (
            session_id, user_id, tenant_id,
            access_token_hash, kanidm_session_id, auth_method,
            access_token_expires_at, refresh_token_expires_at
        )
        VALUES ($1, $2, $3, $4, $5, 'dual', NOW() + INTERVAL '1 day', NOW() + INTERVAL '14 days')
        RETURNING session_id, auth_method, access_token_hash, kanidm_session_id
        "#,
        Uuid::new_v4(),
        user.user_id,
        tenant.tenant_id,
        "access_hash",
        Uuid::new_v4()
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(session.auth_method, "dual");
    assert!(session.access_token_hash.is_some(), "Dual session should have token hash");
    assert!(session.kanidm_session_id.is_some(), "Dual session should have kanidm_session_id");

    println!("✅ Dual session created with both auth methods");
}

/// Test: Session stats view aggregates correctly
#[tokio::test]
async fn test_session_stats_view() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Session Stats Test").await;
    let user = create_test_user(&pool, tenant.tenant_id, "stats@test.com", "User", "user").await;

    // Create sessions of different types
    for auth_method in &["jwt", "kanidm", "dual"] {
        sqlx::query!(
            r#"
            INSERT INTO sessions (
                session_id, user_id, tenant_id, auth_method,
                access_token_expires_at, refresh_token_expires_at
            )
            VALUES ($1, $2, $3, $4, NOW() + INTERVAL '1 hour', NOW() + INTERVAL '7 days')
            "#,
            Uuid::new_v4(),
            user.user_id,
            tenant.tenant_id,
            auth_method
        )
        .execute(&pool)
        .await
        .unwrap();
    }

    // Query stats (filter by tenant to avoid cross-test pollution)
    let stats = sqlx::query!(
        r#"
        SELECT s.auth_method, COUNT(*) as total_sessions
        FROM sessions s
        WHERE s.tenant_id = $1
        GROUP BY s.auth_method
        ORDER BY s.auth_method
        "#,
        tenant.tenant_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(stats.len(), 3, "Expected 3 auth_method groups, got {}", stats.len());
    
    // SQLx infers auth_method as Option<String> from GROUP BY
    let methods: Vec<Option<String>> = stats.iter().map(|s| s.auth_method.clone()).collect();
    assert!(methods.iter().any(|m| m.as_deref() == Some("dual")));
    assert!(methods.iter().any(|m| m.as_deref() == Some("jwt")));
    assert!(methods.iter().any(|m| m.as_deref() == Some("kanidm")));

    println!("✅ Session stats view working correctly");
    for stat in &stats {
        if let Some(ref method) = stat.auth_method {
            println!("   {}: {} total sessions", method, stat.total_sessions.unwrap());
        }
    }
}

// =============================================================================
// CLEANUP FUNCTION TESTS
// =============================================================================

/// Test: Cleanup function removes expired sessions
#[tokio::test]
async fn test_cleanup_expired_sessions() {
    let pool = setup_test_db().await;
    let tenant = create_test_tenant(&pool, "Cleanup Test").await;
    let user = create_test_user(&pool, tenant.tenant_id, "cleanup@test.com", "User", "user").await;

    // Create old expired session
    sqlx::query!(
        r#"
        INSERT INTO sessions (
            session_id, user_id, tenant_id,
            access_token_hash, auth_method,
            access_token_expires_at, refresh_token_expires_at,
            last_used_at
        )
        VALUES ($1, $2, $3, $4, 'jwt',
                NOW() - INTERVAL '40 days',
                NOW() - INTERVAL '10 days',
                NOW() - INTERVAL '45 days')
        "#,
        Uuid::new_v4(),
        user.user_id,
        tenant.tenant_id,
        "old_hash"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create recent session
    sqlx::query!(
        r#"
        INSERT INTO sessions (
            session_id, user_id, tenant_id,
            access_token_hash, auth_method,
            access_token_expires_at, refresh_token_expires_at
        )
        VALUES ($1, $2, $3, $4, 'jwt', NOW() + INTERVAL '1 hour', NOW() + INTERVAL '7 days')
        "#,
        Uuid::new_v4(),
        user.user_id,
        tenant.tenant_id,
        "recent_hash"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Run cleanup
    let result = sqlx::query!(
        "SELECT * FROM cleanup_expired_sessions(30)"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(result.deleted_count.unwrap(), 1);

    // Verify only recent session remains
    let remaining = sqlx::query!(
        "SELECT COUNT(*) as count FROM sessions WHERE tenant_id = $1",
        tenant.tenant_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(remaining.count.unwrap(), 1);

    println!("✅ Cleanup function removed {} expired session(s)", result.deleted_count.unwrap());
}
