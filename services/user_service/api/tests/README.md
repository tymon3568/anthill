# Integration Tests for User Service

## Overview

These integration tests verify the complete auth flow including:
- User registration with session creation
- Token refresh with session rotation
- Logout with session revocation
- Tenant isolation (CRITICAL SECURITY)
- Input validation

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

## Running Tests

### Run All Integration Tests

```bash
# From project root
cargo test --package user_service_api --test integration_test -- --ignored --test-threads=1

# Or from api directory
cd services/user_service/api
cargo test --test integration_test -- --ignored --test-threads=1
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

## Test Cases

| Test Name | Description |
|-----------|-------------|
| `test_register_creates_user_and_session` | Verifies user registration creates user + session with metadata |
| `test_register_duplicate_email_fails` | Ensures duplicate emails are rejected |
| `test_refresh_token_rotates_session` | Validates session rotation on token refresh |
| `test_logout_revokes_session` | Confirms logout revokes session |
| `test_tenant_isolation_users_cannot_see_other_tenants` | **CRITICAL SECURITY TEST** - Verifies tenant isolation |
| `test_cross_tenant_access_admin_cannot_access_other_tenant_users` | **CRITICAL SECURITY TEST** - Admin cross-tenant access prevention |
| `test_tenant_isolation_with_multiple_users_per_tenant` | **CRITICAL SECURITY TEST** - Multi-user tenant isolation |
| `test_invalid_email_format_fails` | Tests email validation |
| `test_weak_password_fails` | Tests password length validation |

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

## CI/CD Integration

For GitHub Actions:

```yaml
- name: Setup PostgreSQL for Integration Tests
  run: |
    sudo apt-get update
    sudo apt-get install -y postgresql postgresql-contrib
    sudo systemctl start postgresql
    
    # Create test database if it doesn't exist (idempotent)
    sudo -u postgres psql -c "SELECT 1 FROM pg_database WHERE datname = 'test_db';" | grep -q 1 || \
    sudo -u postgres createdb test_db
    
    # Set password for postgres user if not already set (idempotent)
    sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'postgres';" 2>/dev/null || true
    
    # Grant privileges (idempotent)
    sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE test_db TO postgres;"

- name: Run Integration Tests
  env:
    DATABASE_URL: ${{ secrets.TEST_DATABASE_URL }}
    JWT_SECRET: ${{ secrets.TEST_JWT_SECRET }}
  run: |
    cargo test --test security -- --ignored --test-threads=1 --nocapture
```

## Notes

- Tests use `#[ignore]` attribute to prevent running with unit tests
- Must be run sequentially (`--test-threads=1`) due to shared database state
- Each test should ideally clean up after itself, but database can be reset between runs
- Consider using `sqlx::test` macro in the future for automatic database transactions/rollback
