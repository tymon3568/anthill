// Integration Tests - Real Database Tests
// These tests run against a real PostgreSQL database
// Run: ./scripts/setup-test-db.sh && cargo test --test integration_tests

mod integration_utils;

use integration_utils::IntegrationTestContext;
use uuid::Uuid;

/// Setup: Run before tests
/// ```bash
/// ./scripts/setup-test-db.sh --reset --seed
/// ```

#[tokio::test]
#[ignore] // Run with: cargo test --test integration_tests -- --ignored
async fn test_full_user_registration_flow() {
    let ctx = IntegrationTestContext::new().await;

    // Create test tenant
    let tenant_id = ctx.db.create_test_tenant("Integration Test Corp").await;

    // Verify tenant was created
    let snapshot = ctx.db.snapshot_tenant(tenant_id).await;
    assert_eq!(snapshot.tenant_status, "active");
    assert_eq!(snapshot.users_count, 0);

    // Create user
    let user_id = ctx.db.create_test_user(tenant_id, "test@example.com", "user").await;

    // Verify user was created
    let snapshot = ctx.db.snapshot_tenant(tenant_id).await;
    assert_eq!(snapshot.users_count, 1);

    // Create JWT for user
    let token = ctx.create_jwt(user_id, tenant_id, "user");
    assert!(!token.is_empty());

    // Cleanup
    ctx.cleanup().await;

    // Verify cleanup
    let snapshot_result = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tenants WHERE tenant_id = $1",
        tenant_id
    )
    .fetch_one(ctx.db.pool())
    .await
    .unwrap();

    assert_eq!(snapshot_result, Some(0));
}

#[tokio::test]
#[ignore]
async fn test_multi_tenant_isolation() {
    let ctx = IntegrationTestContext::new().await;

    // Create two separate tenants
    let tenant_a_id = ctx.db.create_test_tenant("Tenant A").await;
    let tenant_b_id = ctx.db.create_test_tenant("Tenant B").await;

    // Create users in each tenant
    let user_a1 = ctx.db.create_test_user(tenant_a_id, "user1@tenant-a.com", "user").await;
    let user_a2 = ctx.db.create_test_user(tenant_a_id, "user2@tenant-a.com", "user").await;
    let user_b1 = ctx.db.create_test_user(tenant_b_id, "user1@tenant-b.com", "user").await;

    // Verify isolation
    let snapshot_a = ctx.db.snapshot_tenant(tenant_a_id).await;
    let snapshot_b = ctx.db.snapshot_tenant(tenant_b_id).await;

    assert_eq!(snapshot_a.users_count, 2);
    assert_eq!(snapshot_b.users_count, 1);

    // Verify users belong to correct tenant
    let user_a1_tenant = sqlx::query_scalar!(
        "SELECT tenant_id FROM users WHERE user_id = $1",
        user_a1
    )
    .fetch_one(ctx.db.pool())
    .await
    .unwrap();

    assert_eq!(user_a1_tenant, tenant_a_id);

    let user_b1_tenant = sqlx::query_scalar!(
        "SELECT tenant_id FROM users WHERE user_id = $1",
        user_b1
    )
    .fetch_one(ctx.db.pool())
    .await
    .unwrap();

    assert_eq!(user_b1_tenant, tenant_b_id);

    // Cleanup
    ctx.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_bulk_user_creation() {
    let ctx = IntegrationTestContext::new().await;

    let tenant_id = ctx.db.create_test_tenant("Bulk Test Tenant").await;

    // Create multiple users
    let mut user_ids = Vec::new();
    for i in 1..=50 {
        let user_id = ctx.db.create_test_user(
            tenant_id,
            &format!("user{}@bulk.com", i),
            "user"
        ).await;
        user_ids.push(user_id);
    }

    // Verify all created
    let snapshot = ctx.db.snapshot_tenant(tenant_id).await;
    assert_eq!(snapshot.users_count, 50);

    // Cleanup
    ctx.cleanup().await;

    // Verify all deleted
    let remaining = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE user_id = ANY($1)",
        &user_ids
    )
    .fetch_one(ctx.db.pool())
    .await
    .unwrap();

    assert_eq!(remaining, Some(0));
}

#[tokio::test]
#[ignore]
async fn test_concurrent_operations() {
    use tokio::task::JoinSet;

    let ctx = IntegrationTestContext::new().await;
    let tenant_id = ctx.db.create_test_tenant("Concurrent Test").await;

    // Spawn concurrent user creation tasks
    let mut tasks = JoinSet::new();
    let pool = ctx.db.pool().clone();

    for i in 1..=10 {
        let pool_clone = pool.clone();
        let tenant_id_clone = tenant_id;

        tasks.spawn(async move {
            let user_id = Uuid::now_v7();
            sqlx::query!(
                r#"
                INSERT INTO users (
                    user_id, tenant_id, email, password_hash, role, status,
                    email_verified, email_verified_at, full_name, created_at, updated_at
                )
                VALUES (
                    $1, $2, $3, $4, 'user', 'active',
                    true, NOW(), $5, NOW(), NOW()
                )
                "#,
                user_id,
                tenant_id_clone,
                format!("concurrent{}@test.com", i),
                "$argon2id$v=19$m=19456,t=2,p=1$test$test",
                format!("Concurrent User {}", i)
            )
            .execute(&pool_clone)
            .await
            .expect("Failed to create user concurrently");

            user_id
        });
    }

    // Wait for all tasks
    let mut created_count = 0;
    while let Some(result) = tasks.join_next().await {
        if result.is_ok() {
            created_count += 1;
        }
    }

    assert_eq!(created_count, 10);

    // Verify count
    let snapshot = ctx.db.snapshot_tenant(tenant_id).await;
    assert_eq!(snapshot.users_count, 10);

    // Cleanup
    ctx.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_database_constraint_violations() {
    let ctx = IntegrationTestContext::new().await;

    let tenant_id = ctx.db.create_test_tenant("Constraint Test").await;

    // Create user
    let email = "duplicate@test.com";
    ctx.db.create_test_user(tenant_id, email, "user").await;

    // Try to create duplicate email (should fail)
    let result = sqlx::query!(
        r#"
        INSERT INTO users (
            user_id, tenant_id, email, password_hash, role, status,
            email_verified, email_verified_at, full_name, created_at, updated_at
        )
        VALUES (
            $1, $2, $3, $4, 'user', 'active',
            true, NOW(), 'Duplicate User', NOW(), NOW()
        )
        "#,
        Uuid::now_v7(),
        tenant_id,
        email,
        "$argon2id$v=19$m=19456,t=2,p=1$test$test"
    )
    .execute(ctx.db.pool())
    .await;

    // Should fail due to unique constraint on (tenant_id, email)
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("duplicate") || error_msg.contains("unique"),
        "Expected duplicate key error, got: {}",
        error_msg
    );

    // Cleanup
    ctx.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_transaction_rollback() {
    let ctx = IntegrationTestContext::new().await;

    let tenant_id = ctx.db.create_test_tenant("Transaction Test").await;

    // Start transaction
    let mut tx = ctx.db.pool().begin().await.unwrap();

    // Create user in transaction
    let user_id = Uuid::now_v7();
    sqlx::query!(
        r#"
        INSERT INTO users (
            user_id, tenant_id, email, password_hash, role, status,
            email_verified, email_verified_at, full_name, created_at, updated_at
        )
        VALUES (
            $1, $2, 'rollback@test.com', $3, 'user', 'active',
            true, NOW(), 'Rollback User', NOW(), NOW()
        )
        "#,
        user_id,
        tenant_id,
        "$argon2id$v=19$m=19456,t=2,p=1$test$test"
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    // Rollback transaction
    tx.rollback().await.unwrap();

    // Verify user was not created
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_one(ctx.db.pool())
    .await
    .unwrap();

    assert_eq!(count, Some(0));

    // Cleanup
    ctx.cleanup().await;
}

/// Helper: Run this test to clean up any leftover test data
#[tokio::test]
#[ignore]
async fn cleanup_all_test_data() {
    let ctx = IntegrationTestContext::new().await;

    // Clean up using SQL function
    sqlx::query!("SELECT cleanup_test_data()")
        .execute(ctx.db.pool())
        .await
        .expect("Failed to cleanup test data");

    // Verify clean
    assert!(ctx.db.verify_clean().await);

    println!("✓ All test data cleaned up successfully");
}
