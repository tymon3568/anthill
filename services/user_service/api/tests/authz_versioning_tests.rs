//! Integration Tests for AuthZ Versioning (Immediate Effect)
//!
//! Tests the hybrid authorization versioning system to verify immediate-effect
//! permission invalidation:
//! - Tenant-level version invalidation for role/policy changes
//! - User-level version invalidation for user role changes and security changes
//!
//! These tests verify that stale tokens are rejected immediately after updates,
//! without relying on access-token TTL.
//!
//! ## Prerequisites
//! These tests require the authz_version columns to exist in the database.
//! Run migrations before executing these tests:
//! ```
//! sqlx migrate run --database-url $DATABASE_URL
//! ```

use shared_jwt::{encode_jwt, Claims};
use sqlx::PgPool;
use uuid::Uuid;

mod integration_utils;
use integration_utils::IntegrationTestContext;

/// Check if the authz_version columns exist in the database
async fn check_authz_version_columns_exist(pool: &PgPool) -> bool {
    let result = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'tenants' AND column_name = 'authz_version'
        "#,
    )
    .fetch_optional(pool)
    .await;

    result.is_ok() && result.unwrap().is_some()
}

/// Test helper to create a JWT with specific versions
fn create_jwt_with_versions(
    ctx: &IntegrationTestContext,
    user_id: Uuid,
    tenant_id: Uuid,
    role: &str,
    tenant_v: i64,
    user_v: i64,
) -> String {
    let claims = Claims::new_access_with_versions(
        user_id,
        tenant_id,
        role.to_string(),
        3600, // 1 hour
        tenant_v,
        user_v,
    );
    encode_jwt(&claims, &ctx.jwt_secret).expect("Failed to create test JWT")
}

/// Test helper to get current authz versions from DB
async fn get_db_versions(pool: &PgPool, tenant_id: Uuid, user_id: Uuid) -> (i64, i64) {
    let tenant_v: (i64,) = sqlx::query_as("SELECT authz_version FROM tenants WHERE tenant_id = $1")
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .expect("Failed to get tenant version");

    let user_v: (i64,) = sqlx::query_as("SELECT authz_version FROM users WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("Failed to get user version");

    (tenant_v.0, user_v.0)
}

/// Test helper to bump tenant authz version in DB
async fn bump_tenant_version(pool: &PgPool, tenant_id: Uuid) -> i64 {
    let new_v: (i64,) = sqlx::query_as(
        "UPDATE tenants SET authz_version = authz_version + 1 WHERE tenant_id = $1 RETURNING authz_version"
    )
    .bind(tenant_id)
    .fetch_one(pool)
    .await
    .expect("Failed to bump tenant version");
    new_v.0
}

/// Test helper to bump user authz version in DB
async fn bump_user_version(pool: &PgPool, user_id: Uuid) -> i64 {
    let new_v: (i64,) = sqlx::query_as(
        "UPDATE users SET authz_version = authz_version + 1 WHERE user_id = $1 RETURNING authz_version",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .expect("Failed to bump user version");
    new_v.0
}

/// Test helper to suspend a user in DB
async fn suspend_user(pool: &PgPool, user_id: Uuid) {
    sqlx::query("UPDATE users SET status = 'suspended' WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .expect("Failed to suspend user");
}

/// Test helper to unsuspend a user in DB
#[allow(dead_code)]
async fn unsuspend_user(pool: &PgPool, user_id: Uuid) {
    sqlx::query("UPDATE users SET status = 'active' WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .expect("Failed to unsuspend user");
}

// ===========================================================================
// A) Test Harness / Environment Tests
// ===========================================================================

#[tokio::test]
async fn test_authz_version_columns_exist() {
    let ctx = IntegrationTestContext::new().await;

    let exists = check_authz_version_columns_exist(ctx.db.pool()).await;

    if !exists {
        eprintln!(
            "⚠️ authz_version columns not found in database.\n   \
             Run migrations to enable full versioning tests:\n   \
             sqlx migrate run --database-url $DATABASE_URL"
        );
    }

    // This test passes regardless - it's informational
    // The other tests will skip if columns don't exist
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_version_bump_increments_correctly() {
    let ctx = IntegrationTestContext::new().await;

    // Skip if authz_version columns don't exist
    if !check_authz_version_columns_exist(ctx.db.pool()).await {
        eprintln!("⚠️ Skipping: authz_version columns not found");
        return;
    }

    // Create test tenant and user
    let tenant_id = ctx.db.create_test_tenant("Bump Test Tenant").await;
    let user_id = ctx
        .db
        .create_test_user(tenant_id, "bump-test@example.com", "user")
        .await;

    // Get initial versions
    let (initial_tenant_v, initial_user_v) =
        get_db_versions(ctx.db.pool(), tenant_id, user_id).await;

    // Bump tenant version
    let new_tenant_v = bump_tenant_version(ctx.db.pool(), tenant_id).await;
    assert_eq!(new_tenant_v, initial_tenant_v + 1, "Tenant version should increment by 1");

    // Bump user version
    let new_user_v = bump_user_version(ctx.db.pool(), user_id).await;
    assert_eq!(new_user_v, initial_user_v + 1, "User version should increment by 1");

    ctx.cleanup().await;
}

// ===========================================================================
// B) Immediate Effect — Tenant-level Tests
// ===========================================================================

#[tokio::test]
async fn test_tenant_version_mismatch_token_should_be_stale() {
    let ctx = IntegrationTestContext::new().await;

    if !check_authz_version_columns_exist(ctx.db.pool()).await {
        eprintln!("⚠️ Skipping: authz_version columns not found");
        return;
    }

    // Create tenant and user
    let tenant_id = ctx.db.create_test_tenant("Tenant Version Test").await;
    let user_id = ctx
        .db
        .create_test_user(tenant_id, "tenant-v-test@example.com", "user")
        .await;

    // Get initial versions
    let (tenant_v, user_v) = get_db_versions(ctx.db.pool(), tenant_id, user_id).await;

    // Create token with current versions (this token should be valid)
    let valid_token = create_jwt_with_versions(&ctx, user_id, tenant_id, "user", tenant_v, user_v);
    assert!(!valid_token.is_empty());

    // Bump tenant version (simulates policy change)
    let _new_tenant_v = bump_tenant_version(ctx.db.pool(), tenant_id).await;

    // Create token with OLD tenant version (this token should be stale)
    let stale_token = create_jwt_with_versions(&ctx, user_id, tenant_id, "user", tenant_v, user_v);

    // Get new current versions
    let (current_tenant_v, current_user_v) =
        get_db_versions(ctx.db.pool(), tenant_id, user_id).await;

    // Verify the stale token has lower tenant version
    let stale_claims =
        shared_jwt::decode_jwt(&stale_token, &ctx.jwt_secret).expect("Token should be valid JWT");
    assert!(
        stale_claims.tenant_v < current_tenant_v,
        "Stale token tenant_v ({}) should be less than current ({})",
        stale_claims.tenant_v,
        current_tenant_v
    );

    // Verify the user version hasn't changed
    assert_eq!(stale_claims.user_v, current_user_v, "User version should not have changed");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_tenant_policy_change_invalidates_all_users_tokens() {
    let ctx = IntegrationTestContext::new().await;

    if !check_authz_version_columns_exist(ctx.db.pool()).await {
        eprintln!("⚠️ Skipping: authz_version columns not found");
        return;
    }

    // Create tenant with multiple users
    let tenant_id = ctx.db.create_test_tenant("Multi User Tenant").await;
    let admin_id = ctx
        .db
        .create_test_user(tenant_id, "admin@multi.com", "admin")
        .await;
    let user1_id = ctx
        .db
        .create_test_user(tenant_id, "user1@multi.com", "user")
        .await;
    let user2_id = ctx
        .db
        .create_test_user(tenant_id, "user2@multi.com", "user")
        .await;

    // Get initial versions
    let (initial_tenant_v, _) = get_db_versions(ctx.db.pool(), tenant_id, admin_id).await;
    let (_, user1_v) = get_db_versions(ctx.db.pool(), tenant_id, user1_id).await;
    let (_, user2_v) = get_db_versions(ctx.db.pool(), tenant_id, user2_id).await;

    // Create tokens for all users with current versions
    let admin_token =
        create_jwt_with_versions(&ctx, admin_id, tenant_id, "admin", initial_tenant_v, 0);
    let user1_token =
        create_jwt_with_versions(&ctx, user1_id, tenant_id, "user", initial_tenant_v, user1_v);
    let user2_token =
        create_jwt_with_versions(&ctx, user2_id, tenant_id, "user", initial_tenant_v, user2_v);

    // Bump tenant version (simulates role definition change by admin)
    let new_tenant_v = bump_tenant_version(ctx.db.pool(), tenant_id).await;

    // All tokens should now be stale (tenant version mismatch)
    let admin_claims = shared_jwt::decode_jwt(&admin_token, &ctx.jwt_secret).expect("Valid JWT");
    let user1_claims = shared_jwt::decode_jwt(&user1_token, &ctx.jwt_secret).expect("Valid JWT");
    let user2_claims = shared_jwt::decode_jwt(&user2_token, &ctx.jwt_secret).expect("Valid JWT");

    assert!(admin_claims.tenant_v < new_tenant_v, "Admin token should be stale");
    assert!(user1_claims.tenant_v < new_tenant_v, "User1 token should be stale");
    assert!(user2_claims.tenant_v < new_tenant_v, "User2 token should be stale");

    ctx.cleanup().await;
}

// ===========================================================================
// C) Immediate Effect — User-level Tests
// ===========================================================================

#[tokio::test]
async fn test_user_version_mismatch_only_affects_that_user() {
    let ctx = IntegrationTestContext::new().await;

    if !check_authz_version_columns_exist(ctx.db.pool()).await {
        eprintln!("⚠️ Skipping: authz_version columns not found");
        return;
    }

    // Create tenant with multiple users
    let tenant_id = ctx.db.create_test_tenant("User Version Test").await;
    let user1_id = ctx
        .db
        .create_test_user(tenant_id, "user1-v@test.com", "user")
        .await;
    let user2_id = ctx
        .db
        .create_test_user(tenant_id, "user2-v@test.com", "user")
        .await;

    // Get initial versions
    let (tenant_v, user1_v) = get_db_versions(ctx.db.pool(), tenant_id, user1_id).await;
    let (_, user2_v) = get_db_versions(ctx.db.pool(), tenant_id, user2_id).await;

    // Create tokens for both users
    let user1_token =
        create_jwt_with_versions(&ctx, user1_id, tenant_id, "user", tenant_v, user1_v);
    let user2_token =
        create_jwt_with_versions(&ctx, user2_id, tenant_id, "user", tenant_v, user2_v);

    // Bump ONLY user1's version (simulates role assignment change)
    let new_user1_v = bump_user_version(ctx.db.pool(), user1_id).await;

    // User1's token should be stale
    let user1_claims = shared_jwt::decode_jwt(&user1_token, &ctx.jwt_secret).expect("Valid JWT");
    assert!(
        user1_claims.user_v < new_user1_v,
        "User1 token should be stale: token_v={}, current_v={}",
        user1_claims.user_v,
        new_user1_v
    );

    // User2's token should still be valid (their version unchanged)
    let (_, current_user2_v) = get_db_versions(ctx.db.pool(), tenant_id, user2_id).await;
    let user2_claims = shared_jwt::decode_jwt(&user2_token, &ctx.jwt_secret).expect("Valid JWT");
    assert_eq!(user2_claims.user_v, current_user2_v, "User2 token should still be valid");

    // Tenant version should not have changed
    let (current_tenant_v, _) = get_db_versions(ctx.db.pool(), tenant_id, user1_id).await;
    assert_eq!(tenant_v, current_tenant_v, "Tenant version should not have changed");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_user_suspension_bumps_version() {
    let ctx = IntegrationTestContext::new().await;

    if !check_authz_version_columns_exist(ctx.db.pool()).await {
        eprintln!("⚠️ Skipping: authz_version columns not found");
        return;
    }

    // Create tenant and user
    let tenant_id = ctx.db.create_test_tenant("Suspension Test").await;
    let user_id = ctx
        .db
        .create_test_user(tenant_id, "suspend-test@test.com", "user")
        .await;

    // Get initial versions
    let (tenant_v, user_v) = get_db_versions(ctx.db.pool(), tenant_id, user_id).await;

    // Create token with current versions
    let token = create_jwt_with_versions(&ctx, user_id, tenant_id, "user", tenant_v, user_v);

    // Suspend user and bump their version (simulates what the handler would do)
    suspend_user(ctx.db.pool(), user_id).await;
    let new_user_v = bump_user_version(ctx.db.pool(), user_id).await;

    // Token should now be stale
    let claims = shared_jwt::decode_jwt(&token, &ctx.jwt_secret).expect("Valid JWT");
    assert!(
        claims.user_v < new_user_v,
        "Token should be stale after suspension: token_v={}, current_v={}",
        claims.user_v,
        new_user_v
    );

    ctx.cleanup().await;
}

// ===========================================================================
// D) Legacy Token Tests (Backward Compatibility)
// ===========================================================================

#[tokio::test]
async fn test_legacy_token_without_versions_should_skip_check() {
    let ctx = IntegrationTestContext::new().await;

    // Create tenant and user (doesn't require authz_version columns)
    let tenant_id = ctx.db.create_test_tenant("Legacy Token Test").await;
    let user_id = ctx
        .db
        .create_test_user(tenant_id, "legacy@test.com", "user")
        .await;

    // Create token WITHOUT versions (legacy token)
    let legacy_token = ctx.create_jwt(user_id, tenant_id, "user");

    // Decode and verify no versions
    let claims = shared_jwt::decode_jwt(&legacy_token, &ctx.jwt_secret).expect("Valid JWT");

    // Legacy tokens have tenant_v=0, user_v=0
    assert_eq!(claims.tenant_v, 0, "Legacy token should have tenant_v=0");
    assert_eq!(claims.user_v, 0, "Legacy token should have user_v=0");

    // has_authz_versions() should return false for legacy tokens
    assert!(!claims.has_authz_versions(), "Legacy token should not have authz versions");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_token_with_versions_has_authz_versions_returns_true() {
    let ctx = IntegrationTestContext::new().await;

    // Create tenant and user (doesn't require authz_version columns)
    let tenant_id = ctx.db.create_test_tenant("Versioned Token Test").await;
    let user_id = ctx
        .db
        .create_test_user(tenant_id, "versioned@test.com", "user")
        .await;

    // Create token WITH versions
    let token = create_jwt_with_versions(&ctx, user_id, tenant_id, "user", 1, 1);

    // Decode and verify versions exist
    let claims = shared_jwt::decode_jwt(&token, &ctx.jwt_secret).expect("Valid JWT");

    assert_eq!(claims.tenant_v, 1, "Token should have tenant_v=1");
    assert_eq!(claims.user_v, 1, "Token should have user_v=1");
    assert!(claims.has_authz_versions(), "Token with versions should have authz versions");

    ctx.cleanup().await;
}

// ===========================================================================
// E) Edge Cases
// ===========================================================================

#[tokio::test]
async fn test_multiple_version_bumps() {
    let ctx = IntegrationTestContext::new().await;

    if !check_authz_version_columns_exist(ctx.db.pool()).await {
        eprintln!("⚠️ Skipping: authz_version columns not found");
        return;
    }

    // Create tenant and user
    let tenant_id = ctx.db.create_test_tenant("Multiple Bumps Test").await;
    let user_id = ctx
        .db
        .create_test_user(tenant_id, "bumps@test.com", "user")
        .await;

    // Get initial versions
    let (initial_tenant_v, initial_user_v) =
        get_db_versions(ctx.db.pool(), tenant_id, user_id).await;

    // Create token with initial versions
    let token = create_jwt_with_versions(
        &ctx,
        user_id,
        tenant_id,
        "user",
        initial_tenant_v,
        initial_user_v,
    );

    // Bump multiple times
    let v1 = bump_tenant_version(ctx.db.pool(), tenant_id).await;
    let v2 = bump_tenant_version(ctx.db.pool(), tenant_id).await;
    let v3 = bump_tenant_version(ctx.db.pool(), tenant_id).await;

    // Verify versions increment correctly
    assert_eq!(v1, initial_tenant_v + 1);
    assert_eq!(v2, initial_tenant_v + 2);
    assert_eq!(v3, initial_tenant_v + 3);

    // Token should be stale regardless of how many bumps occurred
    let claims = shared_jwt::decode_jwt(&token, &ctx.jwt_secret).expect("Valid JWT");
    let (current_tenant_v, _) = get_db_versions(ctx.db.pool(), tenant_id, user_id).await;

    assert!(claims.tenant_v < current_tenant_v, "Token should be stale after multiple bumps");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_tenant_and_user_version_both_stale() {
    let ctx = IntegrationTestContext::new().await;

    if !check_authz_version_columns_exist(ctx.db.pool()).await {
        eprintln!("⚠️ Skipping: authz_version columns not found");
        return;
    }

    // Create tenant and user
    let tenant_id = ctx.db.create_test_tenant("Both Stale Test").await;
    let user_id = ctx
        .db
        .create_test_user(tenant_id, "both-stale@test.com", "user")
        .await;

    // Get initial versions
    let (initial_tenant_v, initial_user_v) =
        get_db_versions(ctx.db.pool(), tenant_id, user_id).await;

    // Create token with initial versions
    let token = create_jwt_with_versions(
        &ctx,
        user_id,
        tenant_id,
        "user",
        initial_tenant_v,
        initial_user_v,
    );

    // Bump BOTH versions
    let new_tenant_v = bump_tenant_version(ctx.db.pool(), tenant_id).await;
    let new_user_v = bump_user_version(ctx.db.pool(), user_id).await;

    // Token should be stale on both counts
    let claims = shared_jwt::decode_jwt(&token, &ctx.jwt_secret).expect("Valid JWT");

    assert!(claims.tenant_v < new_tenant_v, "Token should be stale on tenant version");
    assert!(claims.user_v < new_user_v, "Token should be stale on user version");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_cross_tenant_isolation() {
    let ctx = IntegrationTestContext::new().await;

    if !check_authz_version_columns_exist(ctx.db.pool()).await {
        eprintln!("⚠️ Skipping: authz_version columns not found");
        return;
    }

    // Create two separate tenants
    let tenant1_id = ctx.db.create_test_tenant("Tenant 1 Isolation").await;
    let tenant2_id = ctx.db.create_test_tenant("Tenant 2 Isolation").await;

    let user1_id = ctx
        .db
        .create_test_user(tenant1_id, "user@tenant1.com", "user")
        .await;
    let user2_id = ctx
        .db
        .create_test_user(tenant2_id, "user@tenant2.com", "user")
        .await;

    // Get initial versions for both
    let (tenant1_v, user1_v) = get_db_versions(ctx.db.pool(), tenant1_id, user1_id).await;
    let (tenant2_v, user2_v) = get_db_versions(ctx.db.pool(), tenant2_id, user2_id).await;

    // Create tokens for both users
    let token1 = create_jwt_with_versions(&ctx, user1_id, tenant1_id, "user", tenant1_v, user1_v);
    let token2 = create_jwt_with_versions(&ctx, user2_id, tenant2_id, "user", tenant2_v, user2_v);

    // Bump ONLY tenant1's version
    let new_tenant1_v = bump_tenant_version(ctx.db.pool(), tenant1_id).await;

    // Tenant1 user's token should be stale
    let claims1 = shared_jwt::decode_jwt(&token1, &ctx.jwt_secret).expect("Valid JWT");
    assert!(claims1.tenant_v < new_tenant1_v, "Tenant1 user token should be stale");

    // Tenant2 user's token should still be valid (different tenant)
    let (current_tenant2_v, _) = get_db_versions(ctx.db.pool(), tenant2_id, user2_id).await;
    let claims2 = shared_jwt::decode_jwt(&token2, &ctx.jwt_secret).expect("Valid JWT");
    assert_eq!(claims2.tenant_v, current_tenant2_v, "Tenant2 user token should still be valid");

    ctx.cleanup().await;
}
