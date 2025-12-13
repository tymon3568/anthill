// OAuth2 Integration Tests
//
// These tests verify the Kanidm OAuth2 authentication flow.
// They are marked with #[ignore] by default because they require:
// 1. Running Kanidm server
// 2. Configured OAuth2 client
// 3. Test user in Kanidm
//
// Run with: cargo test --test oauth2_flow_tests -- --ignored --test-threads=1

use serde_json::json;
use tower::util::ServiceExt;
use user_service_api::get_app;

#[tokio::test]
#[ignore] // Requires Kanidm server
async fn test_oauth_authorize_generates_url() {
    // Setup test environment with test config
    let config = shared_config::Config {
        database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        jwt_secret: "test-jwt-secret-for-testing-only".to_string(),
        jwt_expiration: 900,
        jwt_refresh_expiration: 604800,
        host: "127.0.0.1".to_string(),
        port: 3000,
        cors_origins: None,
        kanidm_url: Some("https://localhost:8300".to_string()),
        kanidm_client_id: Some("anthill".to_string()),
        kanidm_client_secret: Some("test-secret".to_string()),
        kanidm_redirect_url: Some("http://localhost:8000/api/v1/auth/oauth/callback".to_string()),
        nats_url: None,
        redis_url: None,
        casbin_model_path: "./shared/auth/model.conf".to_string(),
        max_connections: None,
    };
    let db_pool = shared_db::init_pool(&config.database_url, 5)
        .await
        .expect("Failed to connect to database");
    let app = get_app(db_pool, &config).await;

    // Test
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/auth/oauth/authorize")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "state": "test-state-123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), 200);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["authorization_url"]
        .as_str()
        .unwrap()
        .starts_with("http"));
    assert_eq!(json["state"].as_str().unwrap(), "test-state-123");
    assert!(json["code_verifier"].as_str().is_some());

    println!("✅ OAuth authorize endpoint working");
    println!("   Authorization URL: {}", json["authorization_url"]);
    println!("   Code Verifier: {}", json["code_verifier"]);
    println!("   State: {}", json["state"]);
}

#[tokio::test]
#[ignore] // Requires Kanidm server and valid authorization code
async fn test_oauth_callback_maps_tenant() {
    // This test requires manual setup:
    // 1. Run test_oauth_authorize_generates_url()
    // 2. Open authorization_url in browser
    // 3. Login and get authorization code
    // 4. Update this test with actual code and verifier

    let auth_code = std::env::var("TEST_AUTH_CODE")
        .expect("Set TEST_AUTH_CODE environment variable from OAuth redirect");
    let code_verifier = std::env::var("TEST_CODE_VERIFIER")
        .expect("Set TEST_CODE_VERIFIER from authorize response");

    // Setup
    let config = shared_config::Config::from_env().expect("Failed to load config");
    let db_pool = shared_db::init_pool(&config.database_url, 5)
        .await
        .expect("Failed to connect to database");
    let app = get_app(db_pool, &config).await;

    // Test
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/auth/oauth/callback")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "code": auth_code,
                        "state": "test-state-123",
                        "code_verifier": code_verifier
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), 200);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json["access_token"].as_str().is_some());
    assert!(json["user"]["kanidm_user_id"].as_str().is_some());
    assert!(json["user"]["email"].as_str().is_some());

    // Verify tenant mapping worked
    let tenant = &json["tenant"];
    assert!(tenant.is_object(), "Tenant should be mapped");
    assert_eq!(tenant["name"].as_str().unwrap(), "ACME Corporation", "Tenant name should match");
    assert!(tenant["role"].as_str().is_some(), "Role should be assigned");

    println!("✅ OAuth callback endpoint working");
    println!("   User: {}", json["user"]["email"]);
    println!("   Tenant: {}", tenant["name"]);
    println!("   Role: {}", tenant["role"]);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_user_created_after_oauth() {
    // Setup test environment with test config
    let config = shared_config::Config {
        database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        jwt_secret: "test-jwt-secret-for-testing-only".to_string(),
        jwt_expiration: 900,
        jwt_refresh_expiration: 604800,
        host: "127.0.0.1".to_string(),
        port: 3000,
        cors_origins: None,
        kanidm_url: Some("https://localhost:8300".to_string()),
        kanidm_client_id: Some("anthill".to_string()),
        kanidm_client_secret: Some("test-secret".to_string()),
        kanidm_redirect_url: Some("http://localhost:8000/api/v1/auth/oauth/callback".to_string()),
        nats_url: None,
        redis_url: None,
        casbin_model_path: "./shared/auth/model.conf".to_string(),
        max_connections: None,
    };
    let db_pool = shared_db::init_pool(&config.database_url, 5)
        .await
        .expect("Failed to connect to database");

    // Query for users with kanidm_user_id
    let result = sqlx::query!(
        r#"
        SELECT user_id, email, kanidm_user_id, kanidm_synced_at, tenant_id
        FROM users
        WHERE kanidm_user_id IS NOT NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(&db_pool)
    .await
    .unwrap();

    assert!(result.is_some(), "Should have at least one Kanidm user");

    let user = result.unwrap();
    assert!(user.kanidm_user_id.is_some());
    assert!(user.kanidm_synced_at.is_some());
    assert_eq!(
        user.tenant_id.to_string(),
        "018c3f1e-1234-7890-abcd-000000000001",
        "Should be mapped to ACME tenant"
    );

    println!("✅ User created in database");
    println!("   Email: {}", user.email);
    println!("   Kanidm ID: {}", user.kanidm_user_id.unwrap());
    println!("   Synced: {}", user.kanidm_synced_at.unwrap());
}
