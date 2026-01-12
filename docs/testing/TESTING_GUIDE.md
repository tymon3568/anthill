# Comprehensive Testing Guide

**Version:** 1.0  
**Last Updated:** 2025-11-01  
**Status:** Production Ready

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Testing Philosophy](#testing-philosophy)
4. [Test Types & When to Use](#test-types--when-to-use)
5. [Best Practices](#best-practices)
6. [Test Organization](#test-organization)
7. [Running Tests](#running-tests)
8. [Coverage Requirements](#coverage-requirements)
9. [CI/CD Integration](#cicd-integration)
10. [Troubleshooting](#troubleshooting)
11. [Related Documentation](#related-documentation)

---

## Overview

This guide provides comprehensive instructions for writing, running, and maintaining tests in the Anthill project. Our testing framework emphasizes:

- **Fast feedback loops** with unit tests
- **Reliable integration tests** with real database
- **Security-first approach** with dedicated security tests
- **Performance awareness** through benchmarking
- **High coverage** (80%+ target for core business logic)

### Testing Stack

| Tool | Purpose | Documentation |
|------|---------|---------------|
| **mockall** | Mock objects for unit tests | [Testing with Mocks](#testing-with-mocks) |
| **sqlx** | Database integration tests | [INTEGRATION_TESTING.md](./INTEGRATION_TESTING.md) |
| **criterion** | Performance benchmarking | [BENCHMARKING.md](./BENCHMARKING.md) |
| **cargo-llvm-cov** | Coverage reporting (CI) | [COVERAGE_GUIDE.md](./COVERAGE_GUIDE.md) |
| **cargo-tarpaulin** | Coverage reporting (local) | [COVERAGE_GUIDE.md](./COVERAGE_GUIDE.md) |
| **wiremock** | HTTP mocking | External API tests |
| **fake** | Test data generation | Fixture creation |
| **proptest** | Property-based testing | Complex validation logic |

---

## Quick Start

### 1. Run All Tests

```bash
# All tests (unit + integration + security)
./scripts/ci-helper.sh all

# Just unit tests (fast)
cargo test --workspace

# With coverage report
./scripts/coverage.sh --open
```

### 2. Run Specific Test Types

```bash
# Unit tests only
./scripts/ci-helper.sh unit

# Integration tests (requires database)
./scripts/ci-helper.sh integration

# Security tests
./scripts/ci-helper.sh security

# Benchmarks
cargo bench --workspace
```

### 3. Test Database Setup

```bash
# Set up test database
./scripts/setup-test-db.sh --reset --seed

# Clean test data
./scripts/setup-test-db.sh --clean
```

---

## Testing Philosophy

### The Testing Pyramid

```
        /\
       /  \      E2E Tests (few)
      /____\     - Full user workflows
     /      \    - End-to-end scenarios
    /________\   
   /          \  Integration Tests (moderate)
  /____________\ - Database operations
 /              \- Service interactions
/________________\
                  Unit Tests (many)
                  - Business logic
                  - Validation rules
                  - Data transformations
```

### Our Testing Strategy

1. **Unit Tests (60-70% of tests)**
   - Test business logic in isolation
   - Use mocks for external dependencies
   - Fast execution (< 1 second total)
   - High coverage of core logic

2. **Integration Tests (20-30% of tests)**
   - Test with real database
   - Verify multi-tenant isolation
   - Test concurrent operations
   - Validate constraints and transactions

3. **Security Tests (10% of tests)**
   - SQL injection prevention
   - Authentication & authorization
   - Tenant isolation verification
   - Session security

4. **Performance Tests (Benchmarks)**
   - Establish performance baselines
   - Catch performance regressions
   - Validate optimization efforts

---

## Test Types & When to Use

### Unit Tests

**When to use:**
- Testing pure functions (no side effects)
- Validation logic
- Data transformations
- Business rule enforcement
- Error handling

**Example:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("user@example.com"));
        assert!(!is_valid_email("invalid-email"));
    }

    #[test]
    fn test_password_strength() {
        let weak = "password";
        let strong = "C0mpl3x!P@ssw0rd";
        
        assert!(validate_password(weak).is_err());
        assert!(validate_password(strong).is_ok());
    }
}
```

### Testing with Mocks

**When to use:**
- Testing service layer without database
- Testing API handlers without HTTP
- Isolating component under test

**Example:**

```rust
use mockall::predicate::*;
use user_service_core::mocks::MockUserRepository;

#[tokio::test]
async fn test_user_creation_with_mock() {
    let mut mock_repo = MockUserRepository::new();
    
    // Set up expectations
    mock_repo
        .expect_create()
        .with(predicate::always())
        .times(1)
        .returning(|user| Ok(user.clone()));
    
    // Test your service with mock
    let service = UserService::new(Arc::new(mock_repo));
    let result = service.create_user(new_user).await;
    
    assert!(result.is_ok());
}
```

### Integration Tests

**When to use:**
- Testing database operations
- Verifying transaction behavior
- Testing multi-tenant isolation
- Complex queries with joins

**Example:**

```rust
#[tokio::test]
#[ignore] // Run explicitly with --ignored
async fn test_user_registration_flow() {
    let ctx = IntegrationTestContext::new().await;
    
    // Create user
    let user = ctx.create_test_user("test@example.com").await;
    assert!(user.user_id.is_some());
    
    // Verify in database
    let fetched = ctx.repo.find_by_email(ctx.tenant_id, "test@example.com").await?;
    assert_eq!(fetched.unwrap().email, "test@example.com");
    
    // Cleanup happens automatically via Drop
}
```

### Security Tests

**When to use:**
- Testing authentication flows
- Verifying authorization rules
- Testing tenant isolation
- SQL injection prevention
- Session security

**Example:**

```rust
#[tokio::test]
async fn test_tenant_isolation() {
    let tenant1_id = create_tenant("tenant1").await;
    let tenant2_id = create_tenant("tenant2").await;
    
    let user1 = create_user(tenant1_id, "user1@t1.com").await;
    
    // Attempt to access user1 from tenant2 context
    let result = repo.find_by_id(tenant2_id, user1.id).await;
    
    // Should fail - tenant isolation enforced
    assert!(result.is_err() || result.unwrap().is_none());
}
```

### Performance Benchmarks

**When to use:**
- Establishing performance baselines
- Testing optimization efforts
- Validating scalability improvements
- Catching performance regressions

**Example:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn password_validation_benchmark(c: &mut Criterion) {
    c.bench_function("password_validation_short", |b| {
        b.iter(|| validate_password(black_box("short123")))
    });
}

criterion_group!(benches, password_validation_benchmark);
criterion_main!(benches);
```

---

## Best Practices

### 1. Test Naming Conventions

```rust
// ✅ Good: Descriptive, tells what is tested and expected
#[test]
fn test_email_validation_rejects_invalid_format() { }

#[test]
fn test_create_user_fails_when_email_already_exists() { }

// ❌ Bad: Vague, unclear expectations
#[test]
fn test_email() { }

#[test]
fn test_user() { }
```

### 2. Arrange-Act-Assert Pattern

```rust
#[test]
fn test_user_creation() {
    // Arrange: Set up test data
    let email = "test@example.com";
    let tenant_id = Uuid::new_v4();
    
    // Act: Perform the action
    let result = create_user(email, tenant_id);
    
    // Assert: Verify expectations
    assert!(result.is_ok());
    assert_eq!(result.unwrap().email, email);
}
```

### 3. Test Isolation

```rust
// ✅ Good: Each test is independent
#[tokio::test]
async fn test_a() {
    let ctx = TestContext::new().await; // Fresh context
    // Test logic
} // Cleanup via Drop

#[tokio::test]
async fn test_b() {
    let ctx = TestContext::new().await; // New context
    // Test logic
}

// ❌ Bad: Tests share state
static mut SHARED_USER: Option<User> = None; // DON'T DO THIS
```

### 4. Use Test Builders

```rust
// ✅ Good: Readable, flexible
let user = UserBuilder::new()
    .email("test@example.com")
    .role(UserRole::Admin)
    .verified(true)
    .build();

// ❌ Bad: Hard to read, error-prone
let user = User {
    user_id: Some(Uuid::new_v4()),
    tenant_id: Uuid::new_v4(),
    email: "test@example.com".to_string(),
    password_hash: "hash".to_string(),
    role: UserRole::Admin,
    is_verified: true,
    is_locked: false,
    failed_login_attempts: 0,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    last_login_at: None,
};
```

### 5. Test Both Success and Failure Cases

```rust
#[test]
fn test_password_validation_success() {
    let strong = "C0mpl3x!P@ssw0rd";
    assert!(validate_password(strong).is_ok());
}

#[test]
fn test_password_validation_too_short() {
    let short = "weak";
    assert!(matches!(
        validate_password(short),
        Err(AppError::ValidationError(_))
    ));
}

#[test]
fn test_password_validation_no_uppercase() {
    let no_upper = "c0mpl3x!p@ssw0rd";
    assert!(validate_password(no_upper).is_err());
}
```

### 6. Use Fixtures for Common Scenarios

```rust
// Create reusable test environment
let env = TestEnvironment::new().await;
// env.tenant, env.admin_user, env.manager_user, env.regular_user ready to use

// Test with fixture
assert_user_has_role(&env.admin_user, UserRole::Admin);
assert_same_tenant(&env.admin_user, &env.manager_user);
```

### 7. Clean Up Test Data

```rust
// ✅ Good: Automatic cleanup with Drop
struct TestContext {
    tenant_id: Uuid,
    created_user_ids: Vec<Uuid>,
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Cleanup happens automatically
        cleanup_test_data(self.tenant_id);
    }
}

// Or use test-specific prefixes
let tenant = create_tenant("test-scenario-1"); // Cleanup script removes "test-*"
```

### 8. Mock External Dependencies

```rust
// ✅ Good: Test in isolation
let mut mock_email_service = MockEmailService::new();
mock_email_service
    .expect_send_verification()
    .returning(|_| Ok(()));

// ❌ Bad: Tests depend on external services
let email_service = RealEmailService::new(); // Might fail if SMTP down
```

### 9. Use Property-Based Testing for Complex Logic

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_slug_generation_is_lowercase(s in "\\w{1,50}") {
        let slug = generate_slug(&s);
        assert_eq!(slug, slug.to_lowercase());
    }
    
    #[test]
    fn test_slug_has_no_spaces(s in ".{1,50}") {
        let slug = generate_slug(&s);
        assert!(!slug.contains(' '));
    }
}
```

### 10. Document Complex Test Scenarios

```rust
/// Test the complete user registration flow:
/// 1. Create unverified user
/// 2. Send verification email
/// 3. Verify email with token
/// 4. Check user can log in
/// 5. Verify session created
#[tokio::test]
#[ignore]
async fn test_complete_registration_flow() {
    // Test implementation
}
```

---

## Test Organization

### Directory Structure

```
services/user_service/
├── api/
│   ├── src/
│   └── tests/
│       ├── mod.rs                          # Test module declaration
│       ├── helpers.rs                      # HTTP test helpers
│       ├── unit_tests.rs                   # API handler unit tests
│       ├── integration_tests.rs            # E2E integration tests
│       ├── comprehensive_security_tests.rs # Security test suite
│       └── benchmarks/
│           ├── password_benchmarks.rs
│           └── database_benchmarks.rs
├── core/
│   ├── src/
│   └── tests/
│       ├── mod.rs                          # Test exports
│       ├── test_utils.rs                   # Builders & factories
│       ├── mocks.rs                        # Mock repositories
│       ├── db_mocks.rs                     # Database mocks
│       ├── fixtures.rs                     # Test fixtures
│       ├── assertions.rs                   # Custom assertions
│       └── integration_utils.rs            # Integration test helpers
└── infra/
    ├── src/
    └── tests/
        └── repository_tests.rs             # Repository-specific tests
```

### Module Organization

```rust
// tests/mod.rs
pub mod test_utils;
pub mod mocks;
pub mod db_mocks;
pub mod fixtures;
pub mod assertions;
pub mod integration_utils;

// Re-export commonly used items
pub use test_utils::{UserBuilder, TenantBuilder, SessionBuilder};
pub use fixtures::TestEnvironment;
pub use assertions::*;
```

---

## Running Tests

### Local Development

```bash
# Quick test (unit tests only)
cargo test --workspace

# With output
cargo test --workspace -- --nocapture

# Specific package
cargo test --package user_service_core

# Specific test
cargo test test_email_validation

# Integration tests (requires database)
cargo test --package user_service_api --test integration_tests -- --ignored

# Security tests
cargo test --package user_service_api --test comprehensive_security_tests -- --ignored

# All tests including ignored
cargo test --workspace -- --include-ignored
```

### Using CI Helper Script

```bash
# Run all stages (like CI)
./scripts/ci-helper.sh all

# Individual stages
./scripts/ci-helper.sh lint
./scripts/ci-helper.sh unit
./scripts/ci-helper.sh integration
./scripts/ci-helper.sh security
./scripts/ci-helper.sh coverage --open

# With options
./scripts/ci-helper.sh unit --verbose --package user_service_core
./scripts/ci-helper.sh coverage --upload  # Upload to Codecov
```

### Running Benchmarks

```bash
# All benchmarks
cargo bench --workspace

# Specific benchmark
cargo bench --package user_service_api password_validation

# Save baseline
cargo bench --package user_service_api -- --save-baseline my-baseline

# Compare with baseline
cargo bench --package user_service_api -- --baseline my-baseline

# Open HTML report
open target/criterion/report/index.html
```

### Coverage Reports

```bash
# Quick coverage check
./scripts/coverage.sh

# With HTML report opened in browser
./scripts/coverage.sh --open

# Upload to Codecov
./scripts/coverage.sh --upload

# Specific package
./scripts/coverage.sh --package user_service_core

# Using cargo-llvm-cov directly
cargo llvm-cov --workspace --lcov --output-path lcov.info
cargo llvm-cov --workspace --html
open target/llvm-cov/html/index.html
```

---

## Coverage Requirements

### Project-Wide Targets

| Component | Minimum Coverage | Target Coverage |
|-----------|-----------------|-----------------|
| **Core Business Logic** | 80% | 90%+ |
| **API Handlers** | 70% | 80%+ |
| **Infrastructure** | 60% | 70%+ |
| **Overall Project** | 75% | 80%+ |

### What to Exclude from Coverage

```toml
# tarpaulin.toml
[coverage]
exclude = [
    "*/tests/*",           # Test code itself
    "*/benches/*",         # Benchmarks
    "*/examples/*",        # Example code
    "*/target/*",          # Build artifacts
    "*_tests.rs",          # Test files
    "**/main.rs",          # Entry points (tested via integration)
]
```

### Coverage Best Practices

1. **Focus on Core Logic**
   - Prioritize business rules and validation
   - Don't chase 100% coverage on boilerplate

2. **Test Edge Cases**
   - Boundary conditions
   - Error handling paths
   - Null/empty inputs

3. **Integration Over Unit**
   - Some code is better tested via integration
   - Don't write unit tests for simple pass-throughs

4. **Review Coverage Reports**
   - Weekly: Check coverage trends
   - PR reviews: Ensure new code is tested
   - Monthly: Review untested critical paths

---

## CI/CD Integration

### GitHub Actions Pipeline

Our CI pipeline runs automatically on:
- Every push to feature branches
- Pull requests to `main`
- Scheduled nightly builds

**Pipeline stages:**

1. **Lint & Format** (1-2 min)
   - `cargo fmt --check`
   - `cargo clippy --all-targets`

2. **Unit Tests** (2-3 min)
   - Fast mock-based tests
   - Parallel execution (4 threads)

3. **Integration Tests** (3-5 min)
   - Real PostgreSQL container
   - Database migrations
   - Multi-tenant scenarios

4. **Security Tests** (2-3 min)
   - SQL injection tests
   - Tenant isolation tests
   - Authentication security

5. **Coverage Report** (3-4 min)
   - Generate LCOV report
   - Upload to Codecov
   - Comment on PR with results

6. **Build Check** (2-3 min)
   - Matrix build for all services
   - Ensure no compilation errors

7. **Test Summary** (1 min)
   - Aggregate all results
   - Create status check

**Total pipeline time:** ~12-15 minutes

### Required Checks

PRs must pass:
- ✅ All unit tests
- ✅ All integration tests
- ✅ All security tests
- ✅ Coverage ≥ 80%
- ✅ No clippy warnings
- ✅ Code formatted

### Local Pre-Push Check

```bash
# Run this before pushing
./scripts/ci-helper.sh all --verbose

# Or create a git hook
cat > .git/hooks/pre-push << 'EOF'
#!/bin/bash
./scripts/ci-helper.sh all || exit 1
EOF
chmod +x .git/hooks/pre-push
```

---

## Troubleshooting

### Common Issues

#### 1. Integration Tests Fail: Database Connection

**Problem:**
```
Error: Failed to connect to PostgreSQL
```

**Solution:**
```bash
# Check database is running
docker ps | grep postgres

# Start database
cd infra/docker_compose
docker-compose up -d postgres

# Reset database
./scripts/setup-test-db.sh --reset
```

#### 2. Tests Pass Locally but Fail in CI

**Problem:** Environmental differences

**Solution:**
```bash
# Simulate CI environment locally
./scripts/ci-helper.sh all

# Check environment variables
cat .env.test

# Ensure migrations are up to date
sqlx migrate run --database-url $DATABASE_URL
```

#### 3. Coverage Report Shows Low Coverage

**Problem:** Coverage < 80%

**Solution:**
```bash
# Generate detailed HTML report
./scripts/coverage.sh --open

# Identify untested modules (red highlighting)
# Add tests for critical paths
# Re-run coverage
./scripts/coverage.sh
```

#### 4. Benchmarks Show Performance Regression

**Problem:** Benchmark slower than baseline

**Solution:**
```bash
# Compare with baseline
cargo bench -- --baseline previous

# Profile the code
cargo build --release
perf record --call-graph=dwarf target/release/user-service
perf report

# Fix performance issue
# Re-establish baseline
cargo bench -- --save-baseline current
```

#### 5. Mock Expectations Not Met

**Problem:**
```
thread 'test' panicked at 'Mock expectations not satisfied'
```

**Solution:**
```rust
// Ensure expectations match actual calls
mock_repo
    .expect_create()
    .times(1)           // Must be called exactly once
    .with(eq(user))     // With specific argument
    .returning(|u| Ok(u.clone()));

// Check test logic calls the mock
let result = service.create_user(user).await; // This must call mock
```

#### 6. Test Database Has Stale Data

**Problem:** Tests fail due to leftover data

**Solution:**
```bash
# Clean test data
./scripts/setup-test-db.sh --clean

# Or reset completely
./scripts/setup-test-db.sh --reset
```

---

## When to Un-ignore Stub Tests

Some tests are marked with `#[ignore]` because they require features not yet implemented. Here's when to remove the ignore attribute:

### Input Validation Tests

**Remove `#[ignore]` when:**
- ✅ Input validation functions implemented in `user_service_core`
- ✅ Email/username/URL validation added
- ✅ Sanitization functions created

**Task:** Create input validation implementation task

### Password Hashing Tests

**Remove `#[ignore]` when:**
- ✅ Argon2id implementation complete (task_03.01.03)
- ✅ Hash storage methods implemented
- ✅ Salt generation and verification added

**Task:** V1_MVP/03_User_Service/3.1_Core_Authentication/task_03.01.03_migrate_to_argon2id.md

### Session Security Tests

**Remove `#[ignore]` when:**
- ✅ Session management fully implemented
- ✅ Session timeout logic added
- ✅ Concurrent session limiting implemented

**Task:** V1_MVP/03_User_Service/3.2_Authorization/task_03.02.12_comprehensive_security_testing.md (Partially implemented)

### Rate Limiting Tests

**Remove `#[ignore]` when:**
- ✅ Rate limiting middleware implemented (task_03.01.01)
- ✅ tower_governor integrated
- ✅ Redis backing configured

**Task:** V1_MVP/03_User_Service/3.1_Core_Authentication/task_03.01.01_implement_rate_limiting.md

### Cryptography Tests

**Remove `#[ignore]` when:**
- ✅ JWT validation hardened
- ✅ Algorithm confusion prevention added
- ✅ Encryption at rest implemented

**Task:** V1_MVP/03_User_Service/3.2_Authorization/task_03.02.12_comprehensive_security_testing.md (Partially implemented)

### Authorization Tests

**Remove `#[ignore]` when:**
- ✅ Authorization middleware complete
- ✅ Casbin policies enforced
- ✅ Privilege escalation prevention verified

**Task:** V1_MVP/03_User_Service/3.2_Authorization/task_03.02.12_comprehensive_security_testing.md (Partially implemented)

**After un-ignoring, remember to:**
1. Run the specific test: `cargo test test_name -- --ignored`
2. Verify it passes
3. Remove `#[ignore]` attribute and reason comment
4. Update security test documentation
5. Commit with descriptive message

---

## Related Documentation

### Testing Documentation

- **[COVERAGE_GUIDE.md](./COVERAGE_GUIDE.md)** - Detailed coverage tool usage (llvm-cov, tarpaulin)
- **[INTEGRATION_TESTING.md](./INTEGRATION_TESTING.md)** - Integration test setup and patterns
- **[CI_CD_PIPELINE.md](./CI_CD_PIPELINE.md)** - GitHub Actions workflow details
- **[BENCHMARKING.md](./BENCHMARKING.md)** - Performance testing with Criterion
- **[SECURITY_TESTING.md](./SECURITY_TESTING.md)** - Security test catalog and patterns

### Project Documentation

- **[ARCHITECTURE.md](../../ARCHITECTURE.md)** - System architecture and multi-tenancy
- **[STRUCTURE.md](../../STRUCTURE.md)** - 3-crate service pattern
- **[migrations/README.md](../../migrations/README.md)** - Database schema conventions

### Code References

- **Test Utilities:** `services/user_service/core/tests/test_utils.rs`
- **Mock Repositories:** `services/user_service/core/tests/mocks.rs`
- **Test Fixtures:** `services/user_service/core/tests/fixtures.rs`
- **Integration Helpers:** `services/user_service/core/tests/integration_utils.rs`
- **Security Tests:** `services/user_service/api/tests/comprehensive_security_tests.rs`

---

## Appendix: Testing Checklist

Use this checklist when adding new features:

### For New Business Logic

- [ ] Unit tests for success cases
- [ ] Unit tests for validation failures
- [ ] Unit tests for edge cases (null, empty, boundary values)
- [ ] Property-based tests for complex validation
- [ ] Error handling tests
- [ ] Mock external dependencies

### For New API Endpoints

- [ ] Unit tests for handler logic
- [ ] Integration tests for full request/response cycle
- [ ] Security tests (authentication, authorization)
- [ ] Validation tests (input sanitization)
- [ ] Error response tests (4xx, 5xx)
- [ ] Multi-tenant isolation tests

### For New Repository Methods

- [ ] Integration tests with real database
- [ ] Transaction rollback tests
- [ ] Constraint violation tests
- [ ] Concurrent access tests
- [ ] Tenant isolation tests
- [ ] Pagination tests (if applicable)

### For Security Features

- [ ] Authentication bypass tests
- [ ] Authorization bypass tests
- [ ] SQL injection tests
- [ ] XSS prevention tests (if rendering HTML)
- [ ] CSRF protection tests (if applicable)
- [ ] Rate limiting tests

### For Performance-Critical Code

- [ ] Benchmark tests with Criterion
- [ ] Baseline performance established
- [ ] Regression detection in CI
- [ ] Scalability tests (100, 1000, 10000 items)

### Before Merging PR

- [ ] All tests passing locally
- [ ] Coverage ≥ 80% for new code
- [ ] CI pipeline passing
- [ ] No clippy warnings
- [ ] Code formatted with rustfmt
- [ ] Documentation updated
- [ ] Changelog updated (if applicable)

---

**Maintained by:** Anthill Development Team  
**Questions?** See [PROJECT_TRACKING/README.md](../../PROJECT_TRACKING/README.md) for task management process.
