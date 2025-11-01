# Integration Testing Guide

This guide covers running integration tests for the user service with a real database.

## Quick Start

```bash
# Start test database and run all integration tests
./scripts/run-integration-tests.sh

# Run tests without setting up database (if already running)
./scripts/run-integration-tests.sh --no-setup

# Run with verbose output
./scripts/run-integration-tests.sh --verbose

# Run specific test
./scripts/run-integration-tests.sh --filter test_user_registration_success

# Cleanup and remove test containers after tests
./scripts/run-integration-tests.sh --teardown
```

## Test Environment Setup

### 1. Docker Compose Test Environment

The `docker-compose.test.yml` file provides isolated test infrastructure:

- **PostgreSQL**: Port 5433 (to avoid conflicts with dev DB on 5432)
- **Redis**: Port 6380
- **NATS**: Port 4223

Start the test environment:

```bash
docker-compose -f docker-compose.test.yml up -d
```

Stop the test environment:

```bash
docker-compose -f docker-compose.test.yml down
```

Remove test data volumes:

```bash
docker-compose -f docker-compose.test.yml down -v
```

### 2. Database Setup

Initialize test database with migrations:

```bash
./scripts/setup-test-db.sh

# With options
./scripts/setup-test-db.sh --reset --seed
```

## Test Structure

### Test Files

- **`test_database.rs`**: Database utilities, test context, cleanup helpers
- **`api_endpoint_tests.rs`**: API endpoint integration tests
- **`auth_flow_tests.rs`**: Authentication & authorization flow tests
- **`error_handling_tests.rs`**: Error handling and edge case tests
- **`integration_tests.rs`**: Legacy integration tests
- **`helpers.rs`**: Shared test helpers

### Test Categories

#### 1. API Endpoint Tests (`api_endpoint_tests.rs`)

Tests all API endpoints with real database:

- User registration and validation
- Login and authentication
- Profile management (get, update)
- Admin user management
- Authorization (access control)
- Tenant isolation
- Input validation

**Run:**
```bash
cargo test --package user_service_api --test api_endpoint_tests -- --ignored
```

#### 2. Authentication Flow Tests (`auth_flow_tests.rs`)

End-to-end authentication flows:

- Complete registration to authenticated request flow
- Login with token refresh
- Logout flow
- RBAC (role-based access control) flows
- User to admin promotion
- Manager permissions
- JWT token validation and expiration
- Cross-tenant access prevention
- Password change flow

**Run:**
```bash
cargo test --package user_service_api --test auth_flow_tests -- --ignored
```

#### 3. Error Handling Tests (`error_handling_tests.rs`)

Error scenarios and edge cases:

- Input validation errors
- Invalid email formats
- Weak password detection
- Malformed JSON requests
- Extremely long input values
- Authentication errors
- Missing authorization headers
- Resource not found errors
- Concurrent duplicate registrations
- SQL injection prevention
- Rate limiting and abuse prevention

**Run:**
```bash
cargo test --package user_service_api --test error_handling_tests -- --ignored
```

## Writing Integration Tests

### Basic Test Structure

```rust
#[tokio::test]
#[ignore]  // Use --ignored flag to run
async fn test_your_feature() {
    // 1. Setup test database
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    // 2. Create test data
    let tenant_id = db.create_tenant("Test Corp", None).await;
    let user_id = db.create_user(
        tenant_id,
        "test@example.com",
        "$argon2id$v=19$m=19456,t=2,p=1$test$test",
        "user",
        Some("Test User"),
    ).await;

    // 3. Perform test actions
    let token = create_jwt(user_id, tenant_id, "user");
    let (status, response) = make_request(
        &app,
        "GET",
        "/api/v1/profile",
        None,
        Some(&token),
    ).await;

    // 4. Assert expectations
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["email"], "test@example.com");

    // 5. Cleanup (automatic with TestDatabaseConfig)
    db.cleanup().await;
}
```

### Test Database Utilities

The `TestDatabaseConfig` provides automatic resource tracking and cleanup:

```rust
// Create configuration
let db = TestDatabaseConfig::new().await;

// Create tenant (automatically tracked)
let tenant_id = db.create_tenant("Company Name", None).await;

// Create user (automatically tracked)
let user_id = db.create_user(
    tenant_id,
    "user@example.com",
    "$password_hash",
    "user",
    Some("Full Name"),
).await;

// Create session (automatically tracked)
let session_id = db.create_session(
    user_id,
    tenant_id,
    "refresh_token",
    expires_at,
).await;

// Cleanup all tracked resources
db.cleanup().await;

// Or cleanup all test data (including untracked)
db.cleanup_all_test_data().await;
```

### Making HTTP Requests

```rust
// Unauthenticated request
let (status, response) = make_request(
    &app,
    "POST",
    "/api/v1/auth/login",
    Some(json!({
        "email": "user@example.com",
        "password": "Password123!"
    })),
    None,  // No auth token
).await;

// Authenticated request
let token = create_jwt(user_id, tenant_id, "user");
let (status, response) = make_request(
    &app,
    "GET",
    "/api/v1/profile",
    None,  // No body
    Some(&token),
).await;
```

### Testing Transactions

```rust
let db = TestDatabaseConfig::new().await;

// Begin transaction
let mut tx = db.begin_transaction().await.unwrap();

// Perform operations in transaction
sqlx::query!("INSERT INTO users (...) VALUES (...)")
    .execute(&mut *tx)
    .await
    .unwrap();

// Rollback or commit
tx.rollback().await.unwrap();
// or
tx.commit().await.unwrap();
```

## Best Practices

### 1. Test Isolation

- Each test should be independent
- Use `TestDatabaseConfig` for automatic cleanup
- Don't rely on data from other tests
- Create fresh test data for each test

### 2. Naming Conventions

```rust
#[tokio::test]
#[ignore]
async fn test_<feature>_<scenario>() {
    // Example: test_user_registration_success
    // Example: test_login_invalid_credentials
}
```

### 3. Test Organization

- Group related tests in the same file
- Use descriptive names
- Add comments for complex scenarios
- Test both success and failure paths

### 4. Assertions

```rust
// Status codes
assert_eq!(status, StatusCode::OK);
assert_eq!(status, StatusCode::CREATED);
assert_eq!(status, StatusCode::UNAUTHORIZED);

// Response fields
assert_eq!(response["email"], "expected@example.com");
assert!(response["access_token"].is_string());

// Database state
let user = db.get_user(user_id).await.unwrap();
assert_eq!(user.email, "expected@example.com");
```

### 5. Cleanup

Always cleanup test data:

```rust
// Automatic cleanup (recommended)
let db = TestDatabaseConfig::new().await;
// ... test code ...
db.cleanup().await;  // Cleans tracked resources

// Manual cleanup if needed
db.cleanup_all_test_data().await;  // Cleans ALL test data
```

## Environment Variables

```bash
# Test database URL
export TEST_DATABASE_URL="postgres://anthill:anthill@localhost:5433/anthill_test"
export DATABASE_URL="postgres://anthill:anthill@localhost:5433/anthill_test"

# JWT secret for testing
export JWT_SECRET="test-secret-key-at-least-32-characters-long"

# Logging level
export RUST_LOG="info"  # or debug, warn, error

# UUID v7 support (unstable feature)
export RUSTFLAGS="--cfg uuid_unstable"
```

## Troubleshooting

### Database Connection Errors

```bash
# Check if PostgreSQL is running
docker-compose -f docker-compose.test.yml ps

# Check PostgreSQL logs
docker-compose -f docker-compose.test.yml logs postgres-test

# Restart PostgreSQL
docker-compose -f docker-compose.test.yml restart postgres-test

# Reset database
./scripts/setup-test-db.sh --reset
```

### Test Failures

```bash
# Run with verbose output
./scripts/run-integration-tests.sh --verbose

# Run specific test
cargo test --package user_service_api --test api_endpoint_tests test_user_registration_success -- --ignored --nocapture

# Check for leftover test data
docker-compose -f docker-compose.test.yml exec postgres-test psql -U anthill -d anthill_test -c "SELECT * FROM tenants WHERE slug LIKE 'test%';"

# Clean up test data
docker-compose -f docker-compose.test.yml exec postgres-test psql -U anthill -d anthill_test -c "SELECT cleanup_test_data();"
```

### Port Conflicts

If test ports are already in use:

```bash
# Check what's using the port
lsof -i :5433

# Kill the process or stop the service
# Then restart test environment
docker-compose -f docker-compose.test.yml down
docker-compose -f docker-compose.test.yml up -d
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Start test environment
        run: docker-compose -f docker-compose.test.yml up -d
      
      - name: Wait for PostgreSQL
        run: |
          timeout 30 bash -c 'until docker-compose -f docker-compose.test.yml exec -T postgres-test pg_isready -U anthill -d anthill_test; do sleep 1; done'
      
      - name: Run migrations
        run: |
          cargo install sqlx-cli --no-default-features --features postgres
          export DATABASE_URL="postgres://anthill:anthill@localhost:5433/anthill_test"
          sqlx migrate run
      
      - name: Run integration tests
        run: ./scripts/run-integration-tests.sh --no-setup --teardown
        env:
          DATABASE_URL: postgres://anthill:anthill@localhost:5433/anthill_test
          JWT_SECRET: test-secret-key-at-least-32-characters-long
          RUSTFLAGS: --cfg uuid_unstable
```

## Performance Considerations

- Integration tests are slower than unit tests
- Use `--test-threads=1` if tests have race conditions
- Consider using test containers for better isolation
- Monitor test execution time and optimize slow tests

## Related Documentation

- [Testing Guide](../../../docs/testing/TESTING_GUIDE.md)
- [Integration Testing](../../../docs/testing/INTEGRATION_TESTING.md)
- [Security Testing](../../../docs/testing/SECURITY_TESTING.md)
- [CI/CD Pipeline](../../../docs/testing/CI_CD_PIPELINE.md)
