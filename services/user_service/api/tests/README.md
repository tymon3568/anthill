# User Service API Tests

## Overview

This directory contains integration and security tests for the User Service API:
- User registration with session creation
- Token refresh with session rotation
- Logout with session revocation
- Tenant isolation (CRITICAL SECURITY)
- RBAC enforcement
- SQL injection prevention
- Unit tests with mocks

## Documentation

- **[Testing Guide](../../../../docs/testing/TESTING_GUIDE.md)** - Complete testing best practices
- **[Coverage Guide](../../../../docs/testing/COVERAGE_GUIDE.md)** - Coverage reporting with Tarpaulin & Codecov

## Prerequisites

1. **PostgreSQL Database**: A test database must be running
2. **Environment Variables**: `DATABASE_URL` must be set
3. **Migrations**: Database migrations must be applied

## Setup

### 1. Start Test Database

```bash
# Using docker-compose
cd infra/docker-compose
docker-compose up -d postgres

# Or start a standalone container
docker run -d \
  --name test-postgres \
  -e POSTGRES_USER=anthill \
  -e POSTGRES_PASSWORD=anthill \
  -e POSTGRES_DB=anthill_test \
  -p 5432:5432 \
  postgres:16
```

### 2. Set Environment Variables

```bash
export DATABASE_URL="postgres://anthill:anthill@localhost:5432/anthill_test"
export JWT_SECRET="test-secret-key-at-least-32-characters-long"
export JWT_EXPIRATION=900  # 15 minutes
export JWT_REFRESH_EXPIRATION=604800  # 7 days
export PORT=3000
```

Or use `.env` file:

```bash
cp .env.example .env
# Edit .env with test database credentials
```

**Note**: In CI/CD environments, these should be set via repository secrets:
- `DATABASE_URL`: PostgreSQL connection string for test database
- `JWT_SECRET`: Secret key for JWT token signing (minimum 32 characters)

### 3. Run Migrations

```bash
sqlx migrate run --database-url $DATABASE_URL
```

## Test Structure

```
tests/
├── helpers.rs                      # Shared test utilities (DB setup, fixtures)
├── unit_tests.rs                   # Pure unit tests (with mocks, no DB)
├── security.rs                     # Main security test suite
├── tenant_isolation_tests.rs       # Multi-tenant isolation (7 tests)
├── rbac_security_tests.rs          # RBAC enforcement (11 tests)
├── jwt_session_security_tests.rs   # JWT and sessions (10 tests)
├── sql_injection_tests.rs          # SQL injection prevention (12 tests)
└── auth_middleware_test.rs         # Middleware tests
```

## Running Tests

### Run All Tests

```bash
# All API tests (unit + integration)
cargo test --package user_service_api

# Security tests only (with database)
cargo test --package user_service_api --test security -- --ignored --test-threads=1

# Unit tests only (no database needed)
cargo test --package user_service_api --lib
```

**Note**: `--test-threads=1` is required because tests share the same database and may interfere with each other if run in parallel.

### Run Specific Test

```bash
cargo test --package user_service_api --test integration_test test_register_creates_user_and_session -- --ignored
```

### Run with Output

```bash
cargo test --package user_service_api --test integration_test -- --ignored --test-threads=1 --nocapture
```

## Test Coverage Summary

| Test Suite | Tests | Description |
|-----------|-------|-------------|
| Tenant Isolation | 7 | Multi-tenant data isolation (CRITICAL) |
| RBAC Security | 11 | Role-based access control |
| JWT & Sessions | 10 | Token security and session management |
| SQL Injection | 12 | Input validation and injection prevention |
| Unit Tests | 8+ | Business logic with mocks |

**Total: 40+ security tests**

See `docs/security_test_report.md` for comprehensive test documentation.

## Test Verification

Before running tests, validate your setup using the verification script:

```bash
# From repository root
./services/user_service/api/test_verification.sh

# Or from services/user_service/api directory
./test_verification.sh
```

**The script will:**
- ✅ **Securely validate** environment variables (no credential exposure)
- ✅ **Check JWT_SECRET length** (minimum 32 characters required)
- ✅ **Verify test file structure** (3+ tests, proper integration markers)
- ✅ **Validate environment usage** in tests (std::env::var usage)
- ✅ **Fail CI fast** if setup is incorrect (non-zero exit code)
- ✅ **Work from any directory** (robust path resolution)

**Exit codes:**
- `0`: All validations passed ✅
- `1`: Validation errors found ❌ (will fail CI)

### Test Fails with "Connection Refused"

```bash
# Check if PostgreSQL is running
docker ps | grep postgres

# Check DATABASE_URL is correct
echo $DATABASE_URL
```

### Test Fails with "Table Not Found"

```bash
# Run migrations
sqlx migrate run --database-url $DATABASE_URL
```

### Test Fails with "JWT Secret" Error

```bash
# Ensure JWT_SECRET is set
export JWT_SECRET="test-secret-key-at-least-32-characters-long"
```

### Clean Test Database

```bash
# Reset database
sqlx database drop --database-url $DATABASE_URL
sqlx database create --database-url $DATABASE_URL
sqlx migrate run --database-url $DATABASE_URL
```

## Documentation

- **Comprehensive Security Report**: `docs/security_test_report.md`
- **Test Helpers**: See `helpers.rs` for available fixtures and utilities
- **Mock Usage**: See `user_service_core/tests/` for mock examples

## Notes

- Security tests use `#[ignore]` attribute (require database)
- Unit tests run fast without external dependencies
- Must run security tests sequentially (`--test-threads=1`)
- See security_test_report.md for detailed test scenarios and expected results
