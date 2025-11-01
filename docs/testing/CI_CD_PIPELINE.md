# CI/CD Testing Pipeline Documentation

## Overview

The Anthill project uses GitHub Actions for continuous integration and continuous deployment (CI/CD). This document describes the automated testing pipeline that runs on every push and pull request.

## Pipeline Architecture

### Workflow Files

1. **`ci-testing.yml`** - Main testing pipeline (comprehensive)
2. **`test-coverage.yml`** - Coverage-focused pipeline (legacy, enhanced)
3. **`openapi-export.yml`** - API documentation generation

## Main CI Testing Pipeline

**File**: `.github/workflows/ci-testing.yml`

### Trigger Conditions

```yaml
# Runs on:
- Push to: master, main, develop, feature/** branches
- Pull requests to any branch
- Changes in: services/, shared/, migrations/
```

### Pipeline Stages

```mermaid
graph TD
    A[Lint & Format] --> B[Unit Tests]
    A --> C[Integration Tests]
    A --> D[Security Tests]
    A --> E[Build Check]
    B --> F[Coverage Report]
    C --> F
    D --> G[Test Summary]
    E --> G
    F --> G
```

### Job Breakdown

#### 1. Lint & Format Check (~2 minutes)

**Purpose**: Ensure code quality and consistency

```bash
# Checks
- cargo fmt --all -- --check    # Format validation
- cargo clippy --workspace      # Linter warnings/errors
```

**Fails if**:
- Code is not formatted with `rustfmt`
- Clippy warnings exist (`-D warnings`)

**Fix locally**:
```bash
cargo fmt --all
cargo clippy --workspace --all-targets --fix
```

---

#### 2. Unit Tests (~5 minutes)

**Purpose**: Fast tests without external dependencies

**Runs**:
```bash
cargo test --workspace --lib --bins \
  --exclude integration_tests \
  -- --nocapture --test-threads=4
```

**Characteristics**:
- ✅ Uses mocks (MockUserRepo, MockDbPool, etc.)
- ✅ No database required
- ✅ Parallel execution (4 threads)
- ✅ Tests in `services/*/core/tests/` and `services/*/api/tests/unit_tests.rs`

**Artifacts**:
- Test result XML files (7-day retention)

---

#### 3. Integration Tests (~8 minutes)

**Purpose**: Test with real PostgreSQL database

**Infrastructure**:
```yaml
services:
  postgres:
    image: postgres:16-alpine
    env:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: anthill_test
    ports:
      - 5432:5432
```

**Runs**:
```bash
# 1. Setup database
sqlx migrate run --database-url $DATABASE_URL

# 2. Run tests
cargo test --workspace --test '*' -- --ignored --nocapture --test-threads=1

# 3. Cleanup
SELECT cleanup_test_data();
```

**Characteristics**:
- ✅ Real PostgreSQL connection
- ✅ Runs migrations automatically
- ✅ Serial execution (`--test-threads=1`) to avoid conflicts
- ✅ Tests in `services/*/api/tests/integration_tests.rs`
- ✅ Auto-cleanup via SQL function

**Environment Variables**:
```bash
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/anthill_test
JWT_SECRET=test-secret-key-min-32-chars-long-for-testing
```

---

#### 4. Security Tests (~6 minutes)

**Purpose**: Validate security controls

**Tests**:
```bash
# SQL Injection protection
cargo test sql_injection -- --ignored --nocapture

# Multi-tenant isolation
cargo test tenant_isolation -- --ignored --nocapture

# Authentication/JWT security
cargo test security -- --ignored --nocapture
```

**Test Files**:
- `services/user_service/api/tests/sql_injection_tests.rs`
- `services/user_service/api/tests/tenant_isolation_tests.rs`
- Security tests in `integration_tests.rs`

**Artifacts**:
- Security test results (30-day retention)
- `docs/security_test_report.md` (if generated)

---

#### 5. Coverage Report (~10 minutes)

**Purpose**: Measure test coverage

**Tools**:
- Primary: `cargo-llvm-cov` (fast, accurate)
- Format: LCOV (for Codecov)

**Runs**:
```bash
cargo llvm-cov --workspace \
  --all-features \
  --lcov \
  --output-path lcov.info \
  -- --include-ignored
```

**Outputs**:
1. **Codecov Upload**: Automatic PR comments with coverage delta
2. **HTML Report**: Interactive coverage browser (artifact)
3. **Threshold Check**: Warns if coverage < 70%, fails if < 60%

**Codecov Integration**:
```yaml
# In codecov.yml
coverage:
  status:
    project:
      default:
        target: 80%
    patch:
      default:
        target: 75%
```

**View Reports**:
- Online: `https://codecov.io/gh/tymon3568/anthill`
- Local HTML: Download artifact → open `index.html`

---

#### 6. Build Check (~15 minutes)

**Purpose**: Ensure all services compile

**Strategy**: Matrix build (parallel)
```yaml
matrix:
  service:
    - user_service
    - inventory_service
    - order_service
    - payment_service
    - integration_service
```

**Per-service**:
```bash
cd services/$SERVICE
cargo check --all-features
cargo build --release
```

**Artifacts**:
- Release binaries (1-day retention)

---

#### 7. Test Summary (Final)

**Purpose**: Aggregate results and report

**Actions**:
1. Downloads all artifacts
2. Generates GitHub Step Summary
3. Comments on PR with results table

**PR Comment Example**:
```markdown
## 🧪 Test Pipeline Results

| Test Suite | Status |
|------------|--------|
| Unit Tests | ✅ |
| Integration Tests | ✅ |
| Security Tests | ✅ |
| Coverage Report | ✅ |

### 📊 Coverage Report
View the detailed coverage report on Codecov

### 📦 Test Artifacts
- Coverage HTML Report
- Unit Test Results
- Integration Test Results  
- Security Test Results
```

---

## Running Tests Locally

### Unit Tests Only
```bash
# All unit tests
cargo test --workspace --lib

# Specific package
cargo test --package user_service_core

# With output
cargo test --lib -- --nocapture
```

### Integration Tests
```bash
# 1. Setup database
./scripts/setup-test-db.sh --reset

# 2. Run tests
cargo test --test integration_tests -- --ignored

# 3. Cleanup
psql $DATABASE_URL -c "SELECT cleanup_test_data();"
```

### Security Tests
```bash
# Setup database first
./scripts/setup-test-db.sh --reset

# Run all security tests
cargo test --package user_service_api sql_injection -- --ignored
cargo test --package user_service_api tenant_isolation -- --ignored
cargo test --package user_service_api security -- --ignored
```

### Coverage Report
```bash
# Quick coverage (LLVM)
./scripts/coverage.sh

# With Codecov upload
./scripts/coverage.sh --upload

# HTML report only
./scripts/coverage.sh --open
```

---

## Troubleshooting

### ❌ Lint Failures

**Error**: `cargo fmt` or `clippy` warnings

**Fix**:
```bash
# Auto-format
cargo fmt --all

# Fix clippy issues
cargo clippy --workspace --all-targets --fix --allow-dirty

# Check again
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

---

### ❌ Integration Test Failures

**Error**: `Connection refused (os error 111)`

**Cause**: No PostgreSQL running

**Fix**:
```bash
# Start PostgreSQL via Docker
docker run -d \
  --name anthill-test-db \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=anthill_test \
  -p 5432:5432 \
  postgres:16-alpine

# Setup database
./scripts/setup-test-db.sh --reset

# Run tests
cargo test --test integration_tests -- --ignored
```

---

### ❌ Coverage Too Low

**Warning**: Coverage below 70%

**Fix**:
1. Identify untested code:
   ```bash
   ./scripts/coverage.sh --open
   # Open coverage-html/index.html
   ```

2. Add tests for red/orange lines

3. Focus on:
   - Core business logic (`services/*/core/`)
   - API handlers (`services/*/api/`)
   - Error paths

---

### ❌ Database Migration Failures

**Error**: Migration failed in CI

**Fix**:
```bash
# Test migrations locally
./scripts/setup-test-db.sh --reset

# Or manually
dropdb anthill_test
createdb anthill_test
sqlx migrate run --database-url postgresql://localhost/anthill_test
```

**Common Issues**:
- SQL syntax error → fix in `migrations/*.sql`
- Duplicate migration → check timestamp order
- Missing dependency → ensure migrations run sequentially

---

### ⚠️ Flaky Tests

**Symptom**: Tests pass locally, fail in CI (or vice versa)

**Common Causes**:
1. **Race conditions**: Use `--test-threads=1` for integration tests
2. **Unclean state**: Tests not cleaning up → use `TestDatabase::cleanup()`
3. **Timing issues**: Add `tokio::time::sleep()` if needed
4. **Database state**: Use transactions or cleanup functions

**Fix**:
```rust
#[tokio::test]
#[ignore]
async fn test_with_cleanup() {
    let db = TestDatabase::new().await;
    
    // Test logic
    // ...
    
    // Always cleanup
    db.cleanup().await.unwrap();
}
```

---

## Environment Variables

### Required in CI

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection | `postgresql://postgres:postgres@localhost:5432/anthill_test` |
| `JWT_SECRET` | JWT signing key (test) | `test-secret-key-min-32-chars-long-for-testing` |
| `CODECOV_TOKEN` | Codecov upload token | `${{ secrets.CODECOV_TOKEN }}` |

### Optional

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_BACKTRACE` | Stack traces on panic | `1` |
| `CARGO_TERM_COLOR` | Colored output | `always` |
| `RUSTFLAGS` | Compiler flags | `--cfg uuid_unstable` |

---

## Secrets Configuration

### GitHub Secrets

Add in: **Settings → Secrets and variables → Actions**

1. **`CODECOV_TOKEN`**
   - Get from: https://codecov.io/gh/tymon3568/anthill/settings
   - Required for coverage upload

2. **`TEST_DATABASE_URL`** (optional)
   - Override default test DB
   - Format: `postgresql://user:pass@host:port/db`

3. **`TEST_JWT_SECRET`** (optional)
   - Override default test JWT secret
   - Min 32 characters

---

## Performance Optimization

### Caching Strategy

```yaml
# Cargo registry (dependencies)
~/.cargo/bin/
~/.cargo/registry/index/
~/.cargo/registry/cache/
~/.cargo/git/db/

# Build artifacts (per-workspace)
uses: Swatinem/rust-cache@v2
with:
  workspaces: "services/user_service"
```

### Typical Run Times

| Job | Duration | Can Parallelize? |
|-----|----------|------------------|
| Lint | 2 min | ❌ (must pass first) |
| Unit Tests | 5 min | ✅ |
| Integration Tests | 8 min | ✅ |
| Security Tests | 6 min | ✅ |
| Coverage | 10 min | ⚠️ (after tests) |
| Build Check | 15 min | ✅ (matrix) |
| **Total** | ~12 min | (parallel execution) |

**Cache Hit**: ~5-8 minutes
**Cache Miss**: ~15-20 minutes

---

## Best Practices

### 1. Test Naming Conventions

```rust
// Unit tests (fast, mocked)
#[tokio::test]
async fn test_create_user_success() { ... }

// Integration tests (slow, real DB)
#[tokio::test]
#[ignore]
async fn test_full_user_registration_flow() { ... }

// Security tests
#[tokio::test]
#[ignore]
async fn test_sql_injection_protection() { ... }
```

### 2. Use `#[ignore]` for Slow Tests

```rust
// Runs in CI (--include-ignored), skipped locally
#[ignore]
#[tokio::test]
async fn test_bulk_user_creation() {
    // Creates 1000 users...
}
```

### 3. Cleanup After Tests

```rust
#[tokio::test]
#[ignore]
async fn test_multi_tenant() {
    let db = TestDatabase::new().await;
    let tenant_id = db.create_test_tenant("test-tenant").await.unwrap();
    
    // Test logic...
    
    // Cleanup (auto-tracked)
    db.cleanup().await.unwrap();
}
```

### 4. Parallel-Safe Tests

```rust
// ❌ BAD: Shared state
static GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);

// ✅ GOOD: Isolated state
async fn test_isolated() {
    let tenant_id = Uuid::now_v7(); // Unique per test
    // ...
}
```

### 5. Deterministic Test Data

```rust
// ❌ BAD: Random data
let user = UserBuilder::default().build()?;

// ✅ GOOD: Fixed data
let user = UserBuilder::default()
    .email("test@example.com")
    .name("Test User")
    .build()?;
```

---

## Monitoring & Alerts

### GitHub Actions

- **Status Badges**: Add to README.md
  ```markdown
  ![CI](https://github.com/tymon3568/anthill/workflows/CI%20Testing%20Pipeline/badge.svg)
  ```

- **Email Notifications**: Settings → Notifications → Actions

### Codecov

- **Coverage Alerts**: Auto-comments on PRs
- **Status Checks**: Required for merge
- **Coverage Trends**: View in dashboard

---

## Adding New Services

When adding a new service to the workspace:

1. **Add to Build Check**:
   ```yaml
   # In ci-testing.yml
   matrix:
     service:
       - user_service
       - inventory_service
       - your_new_service  # Add here
   ```

2. **Create Tests**:
   ```
   services/your_service/
   ├── core/tests/
   │   ├── mod.rs
   │   ├── test_utils.rs
   │   └── mocks.rs
   └── api/tests/
       ├── unit_tests.rs
       └── integration_tests.rs
   ```

3. **Update Coverage Config**:
   ```yaml
   # In codecov.yml
   flags:
     your-service:
       paths:
         - services/your_service/**
   ```

4. **Test Locally**:
   ```bash
   cd services/your_service
   cargo test --lib
   cargo test --test integration_tests -- --ignored
   ```

---

## Maintenance

### Weekly Tasks

- [ ] Review failed test trends
- [ ] Update dependencies (`cargo update`)
- [ ] Check coverage trends (aim for 80%+)
- [ ] Review and fix flaky tests

### Monthly Tasks

- [ ] Update GitHub Actions versions
- [ ] Review and optimize caching strategy
- [ ] Update Rust toolchain
- [ ] Security audit: `cargo audit`

### Quarterly Tasks

- [ ] Review and refactor slow tests
- [ ] Update testing documentation
- [ ] Benchmark test suite performance
- [ ] Review and update CI/CD pipeline

---

## Related Documentation

- **[Coverage Guide](./COVERAGE_GUIDE.md)** - Detailed coverage tooling
- **[Integration Testing](./INTEGRATION_TESTING.md)** - Integration test patterns
- **[Security Testing](../security_test_report.md)** - Security test results
- **[GitHub Actions](.github/workflows/)** - Workflow files

---

## Support

- **Issues**: https://github.com/tymon3568/anthill/issues
- **Discussions**: https://github.com/tymon3568/anthill/discussions
- **CI Logs**: https://github.com/tymon3568/anthill/actions
