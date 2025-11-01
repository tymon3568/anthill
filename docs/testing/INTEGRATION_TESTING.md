# Integration Testing Guide

This guide explains how to write and run integration tests for the Anthill project with real database connections.

## Overview

**Integration tests** verify that multiple components work together correctly with real infrastructure (database, etc.). Unlike unit tests which mock dependencies, integration tests use actual PostgreSQL connections.

### When to Use Integration Tests

✅ **Use integration tests for:**
- Multi-component workflows (e.g., user registration → session creation → JWT generation)
- Database constraints and triggers
- Transaction behavior and rollbacks
- Concurrent operations and race conditions
- Data migration verification
- Multi-tenant isolation

❌ **Don't use integration tests for:**
- Pure business logic (use unit tests)
- External API calls (use mocks)
- Simple CRUD operations (covered by unit tests)

---

## Quick Start

### 1. Setup Test Database

```bash
# One-time setup
./scripts/setup-test-db.sh --reset --seed

# Or just reset
./scripts/setup-test-db.sh --reset
```

### 2. Run Integration Tests

```bash
# Run all integration tests (requires database)
cargo test --test integration_tests -- --ignored

# Run specific test
cargo test --test integration_tests test_full_user_registration_flow -- --ignored

# Run with output
cargo test --test integration_tests -- --ignored --nocapture
```

### 3. Clean Up Test Data

```bash
# Clean all test data
./scripts/setup-test-db.sh --clean

# Or use SQL function
psql -d anthill_test -c "SELECT cleanup_test_data();"
```

---

## Test Database Setup

### Configuration

Test database uses these default settings:

```bash
DB_HOST=localhost
DB_PORT=5432
DB_USER=anthill
DB_PASSWORD=anthill
DB_NAME=anthill_test
```

Override with environment variables:

```bash
export DB_NAME=my_test_db
./scripts/setup-test-db.sh
```

### Database Schema

The test database includes all production migrations plus test-specific helpers:

**Test Helper Functions** (from `migrations/99999999999999_test_helpers.sql`):

```sql
-- Clean all test data
SELECT cleanup_test_data();

-- Check if tenant is a test tenant
SELECT is_test_tenant('tenant-uuid-here');

-- Get tenant data snapshot
SELECT * FROM snapshot_tenant_data('tenant-uuid-here');

-- Generate bulk test users
SELECT generate_test_users('tenant-uuid-here', 50);
```

---

## Writing Integration Tests

### Basic Structure

```rust
use integration_utils::IntegrationTestContext;

#[tokio::test]
#[ignore] // Must be run with --ignored flag
async fn test_my_feature() {
    // 1. Setup context
    let ctx = IntegrationTestContext::new().await;

    // 2. Create test data
    let tenant_id = ctx.db.create_test_tenant("My Test").await;
    let user_id = ctx.db.create_test_user(tenant_id, "test@example.com", "user").await;

    // 3. Run your test logic
    let token = ctx.create_jwt(user_id, tenant_id, "user");
    assert!(!token.is_empty());

    // 4. Verify results
    let snapshot = ctx.db.snapshot_tenant(tenant_id).await;
    assert_eq!(snapshot.users_count, 1);

    // 5. Cleanup (IMPORTANT!)
    ctx.cleanup().await;
}
```

### Using IntegrationTestContext

The `IntegrationTestContext` provides:

```rust
// Database access
ctx.db.pool()                    // Get PgPool
ctx.db.create_test_tenant(name)  // Create tracked tenant
ctx.db.create_test_user(...)     // Create tracked user
ctx.db.snapshot_tenant(id)       // Get tenant statistics
ctx.db.verify_clean()            // Check no test data remains

// JWT creation
ctx.create_jwt(user_id, tenant_id, role)  // Generate test JWT

// Cleanup
ctx.cleanup()  // Remove all tracked test data
```

### Test Data Isolation

All test tenants MUST use slug prefix `test-`:

```rust
// ✅ Good - Auto-tracked for cleanup
let tenant_id = ctx.db.create_test_tenant("My Company").await;
// Creates tenant with slug: "test-my-company"

// ❌ Bad - Manual tenant creation (won't be cleaned up)
sqlx::query!("INSERT INTO tenants (slug, ...) VALUES ('production-company', ...)")
```

### Verification with Snapshots

Use snapshots to verify state:

```rust
let snapshot = ctx.db.snapshot_tenant(tenant_id).await;

assert_eq!(snapshot.users_count, 5);
assert_eq!(snapshot.sessions_count, 2);
assert_eq!(snapshot.profiles_count, 3);
assert_eq!(snapshot.tenant_status, "active");
```

---

## Common Patterns

### Pattern 1: Full Workflow Test

```rust
#[tokio::test]
#[ignore]
async fn test_user_registration_to_login_flow() {
    let ctx = IntegrationTestContext::new().await;

    // Setup
    let tenant_id = ctx.db.create_test_tenant("Workflow Corp").await;

    // Step 1: Register user
    let user_id = ctx.db.create_test_user(tenant_id, "user@example.com", "user").await;

    // Step 2: Create session (simulate login)
    let session_id = sqlx::query_scalar!(
        r#"
        INSERT INTO sessions (
            session_id, user_id, tenant_id,
            access_token_hash, refresh_token_hash,
            access_token_expires_at, refresh_token_expires_at,
            created_at, last_used_at
        )
        VALUES ($1, $2, $3, 'hash1', 'hash2', NOW() + INTERVAL '15 minutes',
                NOW() + INTERVAL '7 days', NOW(), NOW())
        RETURNING session_id
        "#,
        Uuid::now_v7(),
        user_id,
        tenant_id
    )
    .fetch_one(ctx.db.pool())
    .await
    .unwrap();

    // Step 3: Verify JWT generation
    let token = ctx.create_jwt(user_id, tenant_id, "user");
    assert!(!token.is_empty());

    // Step 4: Verify session exists
    let snapshot = ctx.db.snapshot_tenant(tenant_id).await;
    assert_eq!(snapshot.sessions_count, 1);

    ctx.cleanup().await;
}
```

### Pattern 2: Multi-Tenant Isolation

```rust
#[tokio::test]
#[ignore]
async fn test_tenant_data_isolation() {
    let ctx = IntegrationTestContext::new().await;

    // Create two tenants
    let tenant_a = ctx.db.create_test_tenant("Tenant A").await;
    let tenant_b = ctx.db.create_test_tenant("Tenant B").await;

    // Add data to each
    ctx.db.create_test_user(tenant_a, "user@a.com", "user").await;
    ctx.db.create_test_user(tenant_a, "admin@a.com", "admin").await;
    ctx.db.create_test_user(tenant_b, "user@b.com", "user").await;

    // Verify isolation
    let snapshot_a = ctx.db.snapshot_tenant(tenant_a).await;
    let snapshot_b = ctx.db.snapshot_tenant(tenant_b).await;

    assert_eq!(snapshot_a.users_count, 2);
    assert_eq!(snapshot_b.users_count, 1);

    // Verify query isolation
    let tenant_a_users = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE tenant_id = $1",
        tenant_a
    )
    .fetch_one(ctx.db.pool())
    .await
    .unwrap();

    assert_eq!(tenant_a_users, Some(2));

    ctx.cleanup().await;
}
```

### Pattern 3: Concurrent Operations

```rust
#[tokio::test]
#[ignore]
async fn test_concurrent_user_creation() {
    use tokio::task::JoinSet;
    use std::sync::Arc;

    let ctx = IntegrationTestContext::new().await;
    let tenant_id = ctx.db.create_test_tenant("Concurrent Test").await;

    let mut tasks = JoinSet::new();
    
    // Wrap TestDatabase in Arc to share across tasks
    let db = Arc::new(ctx.db);

    // Spawn 100 concurrent tasks
    for i in 0..100 {
        let db_clone = Arc::clone(&db);
        let tid = tenant_id;

        tasks.spawn(async move {
            db_clone.create_test_user(tid, &format!("user{}@test.com", i), "user").await
        });
    }

    // Wait for all
    while let Some(_) = tasks.join_next().await {}

    // Verify all created
    let snapshot = db.snapshot_tenant(tenant_id).await;
    assert_eq!(snapshot.users_count, 100);

    db.cleanup().await;
}
```

### Pattern 4: Transaction Rollback

```rust
#[tokio::test]
#[ignore]
async fn test_transaction_atomicity() {
    let ctx = IntegrationTestContext::new().await;
    let tenant_id = ctx.db.create_test_tenant("Transaction Test").await;

    // Start transaction
    let mut tx = ctx.db.pool().begin().await.unwrap();

    // Create user in transaction
    let user_id = Uuid::now_v7();
    sqlx::query!(
        "INSERT INTO users (...) VALUES (...)"
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    // Rollback
    tx.rollback().await.unwrap();

    // Verify user doesn't exist
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_one(ctx.db.pool())
    .await
    .unwrap();

    assert_eq!(count, Some(0));

    ctx.cleanup().await;
}
```

---

## CI/CD Integration

Integration tests run in GitHub Actions:

```yaml
# .github/workflows/test-coverage.yml
- name: Setup PostgreSQL for Integration Tests
  run: |
    sudo apt-get update
    sudo apt-get install -y postgresql
    sudo systemctl start postgresql
    sudo -u postgres createdb anthill_test
    sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'postgres';"

- name: Run Integration Tests
  run: cargo test --test integration_tests -- --ignored
  env:
    DATABASE_URL: postgres://postgres:postgres@localhost/anthill_test
```

---

## Troubleshooting

### Issue: Connection Refused

```
Error: error communicating with database: Connection refused (os error 111)
```

**Solution:**

```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Start if needed
sudo systemctl start postgresql

# Setup test database
./scripts/setup-test-db.sh --reset
```

### Issue: Permission Denied

```
Error: permission denied for table users
```

**Solution:**

```bash
# Grant permissions
psql -d anthill_test -c "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO anthill;"
psql -d anthill_test -c "GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO anthill;"
```

### Issue: Test Data Not Cleaned Up

```bash
# Manual cleanup
psql -d anthill_test -c "SELECT cleanup_test_data();"

# Or reset entire database
./scripts/setup-test-db.sh --reset
```

### Issue: Migration Errors

```bash
# Rollback migrations
sqlx migrate revert --source migrations

# Re-run migrations
sqlx migrate run --source migrations
```

---

## Best Practices

### ✅ DO

- **Always call `ctx.cleanup()` at end of test**
- Use `#[ignore]` attribute for integration tests
- Use `test-` prefix for tenant slugs
- Verify state with snapshots
- Test edge cases (constraints, race conditions)
- Run tests with `--test-threads=1` if testing serialization

### ❌ DON'T

- Don't create production-like data in tests
- Don't rely on data from other tests
- Don't skip cleanup (causes flaky tests)
- Don't hardcode database URLs (use env vars)
- Don't test business logic here (use unit tests)

---

## Performance Tips

### Parallel Execution

Integration tests can run in parallel:

```bash
# Parallel (faster, but may have conflicts)
cargo test --test integration_tests -- --ignored

# Serial (slower, but safer)
cargo test --test integration_tests -- --ignored --test-threads=1
```

### Connection Pooling

Reuse pools where possible:

```rust
// ✅ Good - One pool per test
let ctx = IntegrationTestContext::new().await;
let pool = ctx.db.pool();

// Do all operations with same pool

ctx.cleanup().await;

// ❌ Bad - New pool per operation
for i in 0..100 {
    let ctx = IntegrationTestContext::new().await; // Expensive!
}
```

### Bulk Operations

Use bulk inserts for large datasets:

```sql
-- ✅ Good
INSERT INTO users (...) VALUES (...), (...), (...);

-- ❌ Bad
INSERT INTO users (...) VALUES (...);
INSERT INTO users (...) VALUES (...);
INSERT INTO users (...) VALUES (...);
```

---

## Example: Complete Integration Test

```rust
#[tokio::test]
#[ignore]
async fn test_complete_user_lifecycle() {
    let ctx = IntegrationTestContext::new().await;

    // Setup tenant
    let tenant_id = ctx.db.create_test_tenant("Lifecycle Corp").await;

    // 1. User registration
    let user_id = ctx.db.create_test_user(tenant_id, "user@example.com", "user").await;

    let snapshot = ctx.db.snapshot_tenant(tenant_id).await;
    assert_eq!(snapshot.users_count, 1);

    // 2. User login (create session)
    let session_id = Uuid::now_v7();
    sqlx::query!(
        r#"
        INSERT INTO sessions (
            session_id, user_id, tenant_id,
            access_token_hash, refresh_token_hash,
            access_token_expires_at, refresh_token_expires_at,
            created_at, last_used_at
        )
        VALUES ($1, $2, $3, 'hash1', 'hash2',
                NOW() + INTERVAL '15 minutes',
                NOW() + INTERVAL '7 days',
                NOW(), NOW())
        "#,
        session_id,
        user_id,
        tenant_id
    )
    .execute(ctx.db.pool())
    .await
    .unwrap();

    let snapshot = ctx.db.snapshot_tenant(tenant_id).await;
    assert_eq!(snapshot.sessions_count, 1);

    // 3. User profile update
    sqlx::query!(
        r#"
        INSERT INTO user_profiles (
            profile_id, user_id, bio, timezone, language, theme, created_at, updated_at
        )
        VALUES ($1, $2, 'Test bio', 'UTC', 'en', 'light', NOW(), NOW())
        "#,
        Uuid::now_v7(),
        user_id
    )
    .execute(ctx.db.pool())
    .await
    .unwrap();

    let snapshot = ctx.db.snapshot_tenant(tenant_id).await;
    assert_eq!(snapshot.profiles_count, 1);

    // 4. User logout (revoke session)
    sqlx::query!(
        "UPDATE sessions SET revoked = true, revoked_at = NOW() WHERE session_id = $1",
        session_id
    )
    .execute(ctx.db.pool())
    .await
    .unwrap();

    let revoked = sqlx::query_scalar!(
        "SELECT revoked FROM sessions WHERE session_id = $1",
        session_id
    )
    .fetch_one(ctx.db.pool())
    .await
    .unwrap();

    assert_eq!(revoked, Some(true));

    // Cleanup
    ctx.cleanup().await;

    // Verify all deleted
    let users_left = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_one(ctx.db.pool())
    .await
    .unwrap();

    assert_eq!(users_left, Some(0));
}
```

---

## Resources

- **Integration Test Utils:** `services/user_service/api/tests/integration_utils.rs`
- **Test Setup Script:** `scripts/setup-test-db.sh`
- **Test Helpers Migration:** `migrations/99999999999999_test_helpers.sql`
- **Example Tests:** `services/user_service/api/tests/integration_tests.rs`

---

## Summary

Integration tests verify that components work together with real infrastructure. They're slower than unit tests but catch issues that mocks can't.

**Key Points:**
- ✅ Always use `IntegrationTestContext` for setup
- ✅ Always call `cleanup()` at the end
- ✅ Use `#[ignore]` attribute
- ✅ Test with real database, not mocks
- ✅ Focus on workflows and interactions
