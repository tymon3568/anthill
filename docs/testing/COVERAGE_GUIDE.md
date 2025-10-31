# Test Coverage Guide

This guide explains how to generate and view test coverage reports for the Anthill project.

## Overview

We use **two complementary tools** for code coverage:

1. **cargo-llvm-cov** (Primary, used in CI/CD)
   - Fast and accurate
   - Uses LLVM instrumentation
   - Integrated with GitHub Actions

2. **cargo-tarpaulin** (Alternative, for local development)
   - More detailed HTML reports
   - Good for interactive development
   - Configurable via `tarpaulin.toml`

Both tools generate coverage in **LCOV format** which is uploaded to **Codecov** for tracking and visualization.

---

## Quick Start

### Local Coverage Generation

#### Option 1: Using the Helper Script (Recommended)

```bash
# Generate coverage report (HTML + LCOV)
./scripts/coverage.sh

# Generate and open HTML report in browser
./scripts/coverage.sh --open

# Generate coverage for specific package only
./scripts/coverage.sh --package user_service_core

# Upload to Codecov (requires CODECOV_TOKEN)
export CODECOV_TOKEN=your_token_here
./scripts/coverage.sh --upload
```

#### Option 2: Using cargo-llvm-cov Directly

```bash
# Install cargo-llvm-cov (one-time)
cargo install cargo-llvm-cov

# Generate coverage for entire workspace
RUSTFLAGS="--cfg uuid_unstable" cargo llvm-cov --workspace --html

# Open HTML report
open target/llvm-cov/html/index.html  # macOS
xdg-open target/llvm-cov/html/index.html  # Linux
```

#### Option 3: Using cargo-tarpaulin

```bash
# Install cargo-tarpaulin (one-time)
cargo install cargo-tarpaulin

# Run with config file
RUSTFLAGS="--cfg uuid_unstable" cargo tarpaulin --config tarpaulin.toml

# Run for specific package
cargo tarpaulin --package user_service_core --out Html

# Open report
open target/coverage/index.html
```

---

## CI/CD Integration

### GitHub Actions

Coverage is automatically generated on:
- **Push to main/master/develop**
- **Pull requests** to protected branches

The workflow:
1. Runs all tests with coverage instrumentation
2. Generates LCOV report
3. Uploads to Codecov
4. Comments on PR with coverage link

**Workflow file:** `.github/workflows/test-coverage.yml`

### Codecov Configuration

Coverage thresholds are defined in `codecov.yml`:

```yaml
coverage:
  status:
    project:
      default:
        target: 80%  # Project-wide target
        threshold: 2%  # Allowed decrease

    patch:
      default:
        target: 75%  # New code target
        threshold: 5%
```

**Flags for service-specific coverage:**
- `unittests` - All tests
- `user-service` - User service only
- `inventory-service` - Inventory service only
- `order-service` - Order service only

---

## Configuration Files

### `tarpaulin.toml`

Main configuration for cargo-tarpaulin:

```toml
[config]
workspace = true
out = ["Lcov", "Html", "Json"]
output-dir = "target/coverage"
timeout = "5m"
engine = "llvm"
ignore-tests = true
```

**Key options:**
- `workspace = true` - Run on all crates
- `engine = "llvm"` - Use LLVM backend (more accurate)
- `ignore-tests = true` - Don't count test code in coverage
- `timeout = "5m"` - Max 5 minutes per test

### `codecov.yml`

Codecov configuration for reporting:

```yaml
coverage:
  precision: 2
  round: down
  range: "70...100"

  status:
    project:
      default:
        target: 80%
```

---

## Understanding Coverage Reports

### Coverage Metrics

- **Line Coverage**: % of executed lines
- **Branch Coverage**: % of executed branches (if/else, match)
- **Function Coverage**: % of called functions

**Target thresholds:**
- Core business logic: **≥ 80%**
- Repository layer: **≥ 75%**
- API handlers: **≥ 70%**

### Excluded from Coverage

The following are intentionally excluded:
- Test files (`**/tests/**`)
- Main entry points (`main.rs`)
- Build scripts (`build.rs`)
- Generated code (`target/`)

### Reading HTML Reports

**File-level view:**
- 🟢 Green lines: Covered (executed during tests)
- 🔴 Red lines: Not covered
- 🟡 Yellow lines: Partially covered (some branches)

**Summary view:**
- Shows coverage % per file/module
- Click on filename to see line-by-line details

---

## Best Practices

### Writing Testable Code

```rust
// ✅ Good: Easy to test, mockable dependencies
pub async fn get_user(
    repo: &impl UserRepository,
    user_id: Uuid
) -> Result<User, AppError> {
    repo.find_by_id(user_id).await
}

// ❌ Bad: Hard to test, concrete dependency
pub async fn get_user(user_id: Uuid) -> Result<User, AppError> {
    let pool = get_global_pool(); // Not mockable
    sqlx::query_as!(User, "SELECT * FROM users WHERE user_id = $1", user_id)
        .fetch_one(&pool)
        .await
}
```

### Test Organization

```
services/user_service/
├── core/
│   ├── src/
│   │   └── lib.rs
│   └── tests/
│       ├── mod.rs          # Test module exports
│       ├── test_utils.rs   # Builders, factories
│       ├── mocks.rs        # Mock implementations
│       └── db_mocks.rs     # Database mocks
└── api/
    └── tests/
        ├── unit_tests.rs      # Unit tests
        ├── integration_tests.rs  # Integration tests
        └── security_tests.rs     # Security tests
```

### Coverage Goals

**Priority 1 (Must have ≥80% coverage):**
- Business logic (core crate)
- Data validation
- Error handling paths

**Priority 2 (Should have ≥70% coverage):**
- API handlers
- Repository implementations
- Service layer

**Priority 3 (Nice to have ≥60% coverage):**
- Middleware
- Utilities
- Helper functions

---

## Troubleshooting

### Issue: "cargo-tarpaulin not found"

```bash
# Install it
cargo install cargo-tarpaulin

# Or use the script which auto-installs
./scripts/coverage.sh
```

### Issue: "RUSTFLAGS uuid_unstable required"

We use UUID v7 which requires unstable feature:

```bash
RUSTFLAGS="--cfg uuid_unstable" cargo tarpaulin
```

Or use the script which handles this automatically.

### Issue: "Coverage upload failed"

For Codecov uploads, you need a token:

```bash
# Get token from https://codecov.io/gh/tymon3568/anthill
export CODECOV_TOKEN=your_token

# Upload manually
codecov -t $CODECOV_TOKEN -f target/coverage/lcov.info
```

### Issue: "Tests timing out"

Increase timeout in `tarpaulin.toml`:

```toml
timeout = "10m"  # Default is 5m
```

### Issue: "Low coverage on integration tests"

Integration tests may require running database:

```bash
# Start test database
docker-compose -f infra/docker_compose/docker-compose.yml up -d postgres

# Run tests with coverage
./scripts/coverage.sh
```

---

## Continuous Improvement

### Viewing Trends

Visit **[Codecov Dashboard](https://codecov.io/gh/tymon3568/anthill)** to see:
- Coverage trends over time
- Per-service coverage breakdown
- Coverage diff on pull requests
- Sunburst visualization

### Coverage Badges

Add to README.md:

```markdown
[![codecov](https://codecov.io/gh/tymon3568/anthill/branch/main/graph/badge.svg)](https://codecov.io/gh/tymon3568/anthill)
```

### Pre-commit Hook (Optional)

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Fail commit if coverage drops below 70%

COVERAGE=$(cargo llvm-cov --workspace --summary-only | grep "TOTAL" | awk '{print $10}' | sed 's/%//')

if (( $(echo "$COVERAGE < 70" | bc -l) )); then
    echo "❌ Coverage is ${COVERAGE}% (below 70% threshold)"
    exit 1
fi

echo "✅ Coverage is ${COVERAGE}%"
```

---

## Resources

- [cargo-llvm-cov Documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [cargo-tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [Codecov Documentation](https://docs.codecov.com/)
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)

---

## Commands Cheat Sheet

```bash
# Quick local coverage
./scripts/coverage.sh --open

# Workspace coverage with llvm-cov
RUSTFLAGS="--cfg uuid_unstable" cargo llvm-cov --workspace --html

# Specific package with tarpaulin
cargo tarpaulin --package user_service_core

# Upload to Codecov
./scripts/coverage.sh --upload

# Check coverage in CI
gh workflow run test-coverage.yml

# View Codecov online
open https://codecov.io/gh/tymon3568/anthill
```
