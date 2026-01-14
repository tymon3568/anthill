//! Invitation system tests
//!
//! Unit tests for expiry validation and acceptance attempt logic,
//! plus integration tests for invitation creation, acceptance, expiry, resend, revoke flows.

mod helpers;

use chrono::{Duration, Utc};
use helpers::{
    cleanup_test_data, create_test_app, create_test_tenant, create_test_user, generate_jwt,
    get_test_config, setup_test_db,
};
use serde_json::json;
use shared_auth::enforcer::create_enforcer;
use sqlx::PgPool;
use std::sync::Arc;
use user_service_api::AppState;
use user_service_core::domains::auth::domain::model::{InvitationStatus, User};
use user_service_infra::auth::{
    AuthServiceImpl, InvitationServiceImpl, PgInvitationRepository, PgSessionRepository,
    PgTenantRepository, PgUserRepository,
};
use uuid::Uuid;

#[cfg(test)]
mod unit_tests {
    use super::*;
    use user_service_core::domains::auth::utils::invitation_utils::hash_token;

    /// Test that expiry is checked before incrementing acceptance attempts
    #[tokio::test]
    async fn test_expiry_checked_before_attempt_increment() {
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;

        let tenant = create_test_tenant(&pool, "Expiry Test Tenant").await;
        let admin = create_test_user(&pool, tenant.tenant_id, "admin@test.com", "Admin User", "admin").await;

        // Create invitation service
        let invitation_repo = PgInvitationRepository::new(pool.clone());
        let user_repo = PgUserRepository::new(pool.clone());
        let enforcer = create_enforcer(&get_test_config().database_url, Some("../../../shared/auth/model.conf")).await.unwrap();
        let invitation_service = InvitationServiceImpl::new(
            invitation_repo,
            user_repo,
            enforcer,
            48, // expiry hours
            5,  // max attempts
        );

        // Create an invitation
        let (invitation, token) = invitation_service
            .create_invitation(
                tenant.tenant_id,
                "test@example.com",
                "user",
                admin.user_id,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        // Manually expire the invitation in DB
        let expired_at = Utc::now() - Duration::hours(1);
        sqlx::query!(
            "UPDATE user_invitations SET expires_at = $1 WHERE invitation_id = $2",
            expired_at,
            invitation.invitation_id
        )
        .execute(&pool)
        .await
        .unwrap();

        // Attempt to accept - should fail with Gone (expired) without incrementing attempts
        let result = invitation_service
            .accept_invitation(&token, "ValidPass123!", Some("Test User"), None, None)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            shared_error::AppError::Gone(msg) => assert!(msg.contains("expired")),
            _ => panic!("Expected Gone error"),
        }

        // Check that attempts were NOT incremented (since expiry checked first)
        let attempts = sqlx::query_scalar!(
            "SELECT accept_attempts FROM user_invitations WHERE invitation_id = $1",
            invitation.invitation_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(attempts, Some(0), "Attempts should not be incremented for expired invitations");

        cleanup_test_data(&pool).await;
    }

    /// Test acceptance attempt limit enforcement
    #[tokio::test]
    async fn test_acceptance_attempt_limit() {
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;

        let tenant = create_test_tenant(&pool, "Attempt Limit Test Tenant").await;
        let admin = create_test_user(&pool, tenant.tenant_id, "admin@test.com", "Admin User", "admin").await;

        // Create invitation service with low attempt limit
        let invitation_repo = PgInvitationRepository::new(pool.clone());
        let user_repo = PgUserRepository::new(pool.clone());
        let enforcer = create_enforcer(&get_test_config().database_url, Some("../../../shared/auth/model.conf")).await.unwrap();
        let invitation_service = InvitationServiceImpl::new(
            invitation_repo,
            user_repo,
            enforcer,
            48, // expiry hours
            2,  // max attempts (low for testing)
        );

        // Create an invitation
        let (invitation, token) = invitation_service
            .create_invitation(
                tenant.tenant_id,
                "test@example.com",
                "user",
                admin.user_id,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        // Make attempts up to the limit using invalid password that fails validation
        for attempt in 1..=2 {
            let result = invitation_service
                .accept_invitation(&token, "short", Some("Test User"), None, None)
                .await;

            if attempt < 2 {
                // Should fail with validation error, but attempts should increment
                assert!(result.is_err());
                match result.unwrap_err() {
                    shared_error::AppError::ValidationError(_) => {}, // Expected validation error
                    _ => panic!("Expected validation error on attempt {}", attempt),
                }
            } else {
                // Last attempt should fail with TooManyRequests
                match result.unwrap_err() {
                    shared_error::AppError::TooManyRequests(msg) => assert!(msg.contains("attempts")),
                    _ => panic!("Expected TooManyRequests error on final attempt"),
                }
            }
        }

        // Verify attempts were incremented correctly
        let attempts = sqlx::query_scalar!(
            "SELECT accept_attempts FROM user_invitations WHERE invitation_id = $1",
            invitation.invitation_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(attempts, Some(2), "Attempts should be incremented up to limit");

        cleanup_test_data(&pool).await;
    }

    /// Test that invalid tokens return NotFound
    #[tokio::test]
    async fn test_invalid_token_returns_not_found() {
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;

        let tenant = create_test_tenant(&pool, "Invalid Token Test Tenant").await;
        let admin = create_test_user(&pool, tenant.tenant_id, "admin@test.com", "Admin User", "admin").await;

        // Create invitation service
        let invitation_repo = PgInvitationRepository::new(pool.clone());
        let user_repo = PgUserRepository::new(pool.clone());
        let enforcer = create_enforcer(&get_test_config().database_url, Some("../../../shared/auth/model.conf")).await.unwrap();
        let invitation_service = InvitationServiceImpl::new(
            invitation_repo,
            user_repo,
            enforcer,
            48, // expiry hours
            5,  // max attempts
        );

        // Try to accept with invalid token
        let result = invitation_service
            .accept_invitation("invalid-token", "ValidPass123!", Some("Test User"), None, None)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            shared_error::AppError::NotFound(msg) => assert!(msg.contains("Invalid or expired")),
            _ => panic!("Expected NotFound error"),
        }

        cleanup_test_data(&pool).await;
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    /// Test full invitation creation and acceptance flow
    #[tokio::test]
    async fn test_invitation_create_and_accept_flow() {
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;

        let tenant = create_test_tenant(&pool, "Full Flow Test Tenant").await;
        let admin = create_test_user(&pool, tenant.tenant_id, "admin@test.com", "Admin User", "admin").await;

        // Create app with invitation service
        let user_repo = PgUserRepository::new(pool.clone());
        let tenant_repo = PgTenantRepository::new(pool.clone());
        let session_repo = PgSessionRepository::new(pool.clone());
        let invitation_repo = PgInvitationRepository::new(pool.clone());

        let auth_service = AuthServiceImpl::new(
            user_repo.clone(),
            tenant_repo.clone(),
            session_repo,
            get_test_config().jwt_secret.clone(),
            900,
            604800,
        );

        let enforcer = create_enforcer(&get_test_config().database_url, Some("../../../shared/auth/model.conf")).await.unwrap();
        let invitation_service = InvitationServiceImpl::new(
            invitation_repo,
            user_repo,
            enforcer,
            48,
            5,
        );

        let state = AppState {
            auth_service: Arc::new(auth_service),
            enforcer: create_enforcer(&get_test_config().database_url, Some("../../../shared/auth/model.conf")).await.unwrap(),
            jwt_secret: get_test_config().jwt_secret,
            user_repo: Some(Arc::new(PgUserRepository::new(pool.clone()))),
            tenant_repo: Some(Arc::new(PgTenantRepository::new(pool.clone()))),
            invitation_service: Some(Arc::new(invitation_service)),
            config: get_test_config(),
            invitation_rate_limiter: Arc::new(crate::rate_limiter::InvitationRateLimiter::default()),
        };

        let app = user_service_api::create_router(&state);

        // Create invitation via API
        let jwt = generate_jwt(admin.user_id, tenant.tenant_id, "admin", &get_test_config());
        let create_response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/users/invite")
                    .header("Authorization", format!("Bearer {}", jwt))
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        json!({
                            "email": "invited@example.com",
                            "role": "user"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::CREATED);

        let create_body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(create_response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();

        let invitation_id = create_body["invitation_id"].as_str().unwrap();
        let invite_token = create_body["invite_token"].as_str().unwrap();

        // Accept invitation via API
        let accept_response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/accept-invite")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        json!({
                            "token": invite_token,
                            "password": "SecurePass123!",
                            "full_name": "Invited User"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(accept_response.status(), StatusCode::CREATED);

        // Verify user was created
        let user_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE email = $1 AND tenant_id = $2",
            "invited@example.com",
            tenant.tenant_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(user_count, Some(1));

        // Verify invitation was marked as accepted
        let invitation_status = sqlx::query_scalar!(
            "SELECT status FROM user_invitations WHERE invitation_id = $1",
            Uuid::parse_str(invitation_id).unwrap()
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(invitation_status, Some("accepted"));

        cleanup_test_data(&pool).await;
    }

    /// Test invitation expiry handling
    #[tokio::test]
    async fn test_invitation_expiry() {
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;

        let tenant = create_test_tenant(&pool, "Expiry Integration Test Tenant").await;
        let admin = create_test_user(&pool, tenant.tenant_id, "admin@test.com", "Admin User", "admin").await;

        // Create invitation service
        let invitation_repo = PgInvitationRepository::new(pool.clone());
        let user_repo = PgUserRepository::new(pool.clone());
        let enforcer = create_enforcer(&get_test_config().database_url, Some("../../../shared/auth/model.conf")).await.unwrap();
        let invitation_service = InvitationServiceImpl::new(
            invitation_repo,
            user_repo,
            enforcer,
            48,
            5,
        );

        // Create an invitation
        let (invitation, token) = invitation_service
            .create_invitation(
                tenant.tenant_id,
                "expire@example.com",
                "user",
                admin.user_id,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        // Manually expire the invitation
        let expired_at = Utc::now() - Duration::hours(1);
        sqlx::query!(
            "UPDATE user_invitations SET expires_at = $1 WHERE invitation_id = $2",
            expired_at,
            invitation.invitation_id
        )
        .execute(&pool)
        .await
        .unwrap();

        // Try to accept expired invitation
        let result = invitation_service
            .accept_invitation(&token, "ValidPass123!", Some("Test User"), None, None)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            shared_error::AppError::Gone(msg) => assert!(msg.contains("expired")),
            _ => panic!("Expected Gone error for expired invitation"),
        }

        // Verify invitation was marked as expired
        let status = sqlx::query_scalar!(
            "SELECT status FROM user_invitations WHERE invitation_id = $1",
            invitation.invitation_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(status, Some("expired"));

        cleanup_test_data(&pool).await;
    }

    /// Test invitation resend functionality
    #[tokio::test]
    async fn test_invitation_resend() {
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;

        let tenant = create_test_tenant(&pool, "Resend Test Tenant").await;
        let admin = create_test_user(&pool, tenant.tenant_id, "admin@test.com", "Admin User", "admin").await;

        // Create invitation service
        let invitation_repo = PgInvitationRepository::new(pool.clone());
        let user_repo = PgUserRepository::new(pool.clone());
        let enforcer = create_enforcer(&get_test_config().database_url, Some("../../../shared/auth/model.conf")).await.unwrap();
        let invitation_service = InvitationServiceImpl::new(
            invitation_repo,
            user_repo,
            enforcer,
            48,
            5,
        );

        // Create an invitation
        let (invitation, original_token) = invitation_service
            .create_invitation(
                tenant.tenant_id,
                "resend@example.com",
                "user",
                admin.user_id,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        // Resend the invitation
        let (updated_invitation, new_token) = invitation_service
            .resend_invitation(
                tenant.tenant_id,
                invitation.invitation_id,
                None,
                None,
            )
            .await
            .unwrap();

        // Verify new token is different
        assert_ne!(original_token, new_token);

        // Verify expiry was updated
        assert!(updated_invitation.expires_at > invitation.expires_at);

        // Verify old token no longer works
        let old_token_result = invitation_service
            .accept_invitation(&original_token, "ValidPass123!", Some("Test User"), None, None)
            .await;

        assert!(old_token_result.is_err());

        // Verify new token works
        let new_token_result = invitation_service
            .accept_invitation(&new_token, "ValidPass123!", Some("Test User"), None, None)
            .await;

        assert!(new_token_result.is_ok());

        cleanup_test_data(&pool).await;
    }

    /// Test invitation revocation
    #[tokio::test]
    async fn test_invitation_revoke() {
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;

        let tenant = create_test_tenant(&pool, "Revoke Test Tenant").await;
        let admin = create_test_user(&pool, tenant.tenant_id, "admin@test.com", "Admin User", "admin").await;

        // Create invitation service
        let invitation_repo = PgInvitationRepository::new(pool.clone());
        let user_repo = PgUserRepository::new(pool.clone());
        let enforcer = create_enforcer(&get_test_config().database_url, Some("../../../shared/auth/model.conf")).await.unwrap();
        let invitation_service = InvitationServiceImpl::new(
            invitation_repo,
            user_repo,
            enforcer,
            48,
            5,
        );

        // Create an invitation
        let (invitation, token) = invitation_service
            .create_invitation(
                tenant.tenant_id,
                "revoke@example.com",
                "user",
                admin.user_id,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        // Revoke the invitation
        invitation_service
            .revoke_invitation(tenant.tenant_id, invitation.invitation_id)
            .await
            .unwrap();

        // Verify status changed to revoked
        let status = sqlx::query_scalar!(
            "SELECT status FROM user_invitations WHERE invitation_id = $1",
            invitation.invitation_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(status, Some("revoked"));

        // Try to accept revoked invitation
        let result = invitation_service
            .accept_invitation(&token, "ValidPass123!", Some("Test User"), None, None)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            shared_error::AppError::Conflict(msg) => assert!(msg.contains("already used")),
            _ => panic!("Expected Conflict error for revoked invitation"),
        }

        cleanup_test_data(&pool).await;
    }
}
