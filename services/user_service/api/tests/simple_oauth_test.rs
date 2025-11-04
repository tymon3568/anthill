// Simple OAuth2 Integration Test for Kanidm
// This test validates the OAuth2 flow with a real Kanidm server

use serde_json::json;
use user_service_api::get_app;
use tower::util::ServiceExt;

#[tokio::test]
#[ignore] // Requires running Kanidm server
async fn test_oauth_authorize_url_generation() {
    // Setup test environment with test config
    let config = shared_config::Config {
        database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        jwt_secret: "test-jwt-secret-for-testing-only".to_string(),
        jwt_expiration: 900,
        jwt_refresh_expiration: 604800,
        host: "127.0.0.1".to_string(),
        port: 3000,
        kanidm_url: Some("https://idm.example.com".to_string()),
        kanidm_client_id: Some("anthill".to_string()),
        kanidm_client_secret: Some("test-secret".to_string()),
        kanidm_redirect_url: Some("http://localhost:3000/api/v1/auth/oauth/callback".to_string()),
    };

    let db_pool = shared_db::init_pool(&config.database_url, 5)
        .await
        .expect("Failed to connect to database");
    let app = get_app(db_pool, &config).await;

    // Test OAuth authorize endpoint
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

    // Should return 200 with authorization URL
    assert_eq!(response.status(), 200);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Validate response structure
    assert!(json_response["authorization_url"].as_str().is_some());
    assert_eq!(json_response["state"].as_str().unwrap(), "test-state-123");
    assert!(json_response["code_verifier"].as_str().is_some());

    let auth_url = json_response["authorization_url"].as_str().unwrap();
    println!("✅ Authorization URL generated: {}", auth_url);

    // URL should contain expected parameters
    assert!(auth_url.contains("client_id=anthill"));
    assert!(auth_url.contains("response_type=code"));
    assert!(auth_url.contains("scope="));
    assert!(auth_url.contains("state=test-state-123"));
    assert!(auth_url.contains("code_challenge="));
    assert!(auth_url.contains("code_challenge_method=S256"));
}
