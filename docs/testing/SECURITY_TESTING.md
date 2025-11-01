# Security Testing Framework

This document outlines the comprehensive security testing framework for the Anthill user service.

## Overview

Security tests validate that the application properly handles:
- Authentication & Authorization
- Multi-tenant isolation
- SQL injection prevention
- XSS/CSRF protection
- Rate limiting
- Input validation
- Session management
- JWT security

## Test Categories

### 1. SQL Injection Tests

**File**: `sql_injection_tests.rs`

Tests SQL injection prevention across all database queries.

**Test Cases**:
- ✅ Login with SQL injection in email field
- ✅ Registration with malicious SQL in fields
- ✅ User lookup with SQL injection
- ✅ Tenant queries with injection attempts
- ✅ Session queries with malicious input

**Run**: `cargo test --package user_service_api sql_injection -- --ignored`

---

### 2. Tenant Isolation Tests

**File**: `tenant_isolation_tests.rs`

Ensures strict data isolation between tenants.

**Test Cases**:
- ✅ User cannot access data from another tenant
- ✅ Admin of tenant A cannot manage tenant B
- ✅ Session tokens are tenant-scoped
- ✅ Cross-tenant API requests are blocked
- ✅ Tenant ID tampering is prevented

**Run**: `cargo test --package user_service_api tenant_isolation -- --ignored`

---

### 3. JWT & Session Security Tests

**File**: `jwt_session_security_tests.rs`

Validates JWT token security and session management.

**Test Cases**:
- ✅ Token expiration is enforced
- ✅ Invalid signatures are rejected
- ✅ Tampered tokens are rejected
- ✅ Expired tokens cannot be used
- ✅ Token refresh requires valid refresh token
- ✅ Logout invalidates session

**Run**: `cargo test --package user_service_api jwt_session -- --ignored`

---

### 4. RBAC Security Tests

**File**: `rbac_security_tests.rs`

Tests Role-Based Access Control implementation.

**Test Cases**:
- ✅ Regular users cannot access admin endpoints
- ✅ Role-based permissions are enforced
- ✅ Privilege escalation is prevented
- ✅ Casbin policies are correctly applied
- ✅ Multi-tenant RBAC isolation

**Run**: `cargo test --package user_service_api rbac_security -- --ignored`

---

### 5. Authentication Middleware Tests

**File**: `auth_middleware_test.rs`

Tests authentication middleware behavior.

**Test Cases**:
- ✅ Missing JWT returns 401
- ✅ Invalid JWT returns 401
- ✅ Valid JWT allows access
- ✅ Expired JWT returns 401
- ✅ Malformed Authorization header returns 401

**Run**: `cargo test --package user_service_api auth_middleware`

---

### 6. General Security Tests

**File**: `security.rs`

Comprehensive security test suite covering multiple vectors.

**Test Cases**:
- ✅ Password strength requirements
- ✅ Rate limiting on login attempts
- ✅ Account lockout after failed attempts
- ✅ Email verification requirements
- ✅ Input sanitization
- ✅ XSS prevention in user inputs

**Run**: `cargo test --package user_service_api security -- --ignored`

---

## Quick Start

### Run All Security Tests

```bash
# Run all security tests (requires database)
./scripts/setup-test-db.sh --reset
cargo test --package user_service_api --test '*' -- --ignored

# Run specific security test suite
cargo test --package user_service_api --test sql_injection_tests -- --ignored
cargo test --package user_service_api --test tenant_isolation_tests -- --ignored
cargo test --package user_service_api --test jwt_session_security_tests -- --ignored
```

### CI/CD Integration

Security tests run automatically in the CI pipeline:

```yaml
# In .github/workflows/ci-testing.yml
security-tests:
  name: Security Tests
  runs-on: ubuntu-latest
  services:
    postgres:
      image: postgres:16-alpine
  steps:
    - name: Run security tests
      run: |
        cargo test --package user_service_api sql_injection -- --ignored
        cargo test --package user_service_api tenant_isolation -- --ignored
        cargo test --package user_service_api security -- --ignored
```

---

## Security Test Patterns

### Pattern 1: SQL Injection Prevention

```rust
#[tokio::test]
#[ignore]
async fn test_sql_injection_in_login() {
    let pool = setup_test_db().await;
    
    // Attempt SQL injection
    let malicious_email = "admin'--";
    let result = login_user(&pool, malicious_email, "password").await;
    
    // Should fail safely, not execute SQL
    assert!(result.is_err());
    
    cleanup(&pool).await;
}
```

### Pattern 2: Tenant Isolation

```rust
#[tokio::test]
#[ignore]
async fn test_cross_tenant_data_access() {
    let pool = setup_test_db().await;
    
    // Create two tenants
    let tenant_a = create_test_tenant(&pool, "Tenant A").await;
    let tenant_b = create_test_tenant(&pool, "Tenant B").await;
    
    // Create user in tenant A
    let user_a = create_test_user(&pool, tenant_a.id, "user@a.com").await;
    
    // Get JWT for tenant A user
    let token_a = create_jwt(user_a.id, tenant_a.id);
    
    // Try to access tenant B data with tenant A token
    let result = get_tenant_data(&pool, &token_a, tenant_b.id).await;
    
    // Should be forbidden
    assert_eq!(result.status(), StatusCode::FORBIDDEN);
    
    cleanup(&pool).await;
}
```

### Pattern 3: JWT Security

```rust
#[tokio::test]
#[ignore]
async fn test_tampered_jwt_rejected() {
    let pool = setup_test_db().await;
    
    // Create valid user
    let (user, tenant) = create_test_user_with_tenant(&pool).await;
    
    // Get valid JWT
    let valid_token = create_jwt(user.id, tenant.id);
    
    // Tamper with token (change user_id in payload)
    let tampered_token = tamper_jwt_payload(&valid_token, "user_id", "fake-id");
    
    // Try to use tampered token
    let result = call_protected_endpoint(&pool, &tampered_token).await;
    
    // Should be unauthorized
    assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
    
    cleanup(&pool).await;
}
```

### Pattern 4: Rate Limiting

```rust
#[tokio::test]
#[ignore]
async fn test_rate_limiting_on_login() {
    let pool = setup_test_db().await;
    
    let email = "test@example.com";
    let password = "wrong_password";
    
    // Attempt login multiple times
    for i in 0..6 {
        let result = login_user(&pool, email, password).await;
        
        if i < 5 {
            // First 5 attempts should return 401
            assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
        } else {
            // 6th attempt should be rate limited
            assert_eq!(result.status(), StatusCode::TOO_MANY_REQUESTS);
        }
    }
    
    cleanup(&pool).await;
}
```

---

## Security Test Checklist

### Authentication
- [ ] Password strength validation
- [ ] Password hashing (bcrypt/argon2)
- [ ] Failed login attempt tracking
- [ ] Account lockout after N failed attempts
- [ ] Password reset token security
- [ ] Email verification required
- [ ] Multi-factor authentication (future)

### Authorization
- [ ] JWT token validation
- [ ] Token expiration enforcement
- [ ] Refresh token rotation
- [ ] Role-based access control
- [ ] Tenant-scoped permissions
- [ ] Admin vs user privilege separation

### Input Validation
- [ ] SQL injection prevention
- [ ] XSS prevention
- [ ] Command injection prevention
- [ ] Path traversal prevention
- [ ] Email format validation
- [ ] Phone number validation
- [ ] URL validation
- [ ] File upload validation (future)

### Session Management
- [ ] Session timeout enforcement
- [ ] Concurrent session limits
- [ ] Session invalidation on logout
- [ ] Session hijacking prevention
- [ ] CSRF token validation (future)

### Multi-Tenancy
- [ ] Tenant data isolation
- [ ] Cross-tenant access prevention
- [ ] Tenant ID validation
- [ ] Tenant-scoped queries
- [ ] Admin cannot access other tenants

### API Security
- [ ] Rate limiting per endpoint
- [ ] Rate limiting per user
- [ ] CORS configuration
- [ ] HTTPS enforcement (production)
- [ ] Request size limits
- [ ] Response size limits

---

## Common Security Vulnerabilities

### 1. SQL Injection

**Risk**: Attacker executes arbitrary SQL queries

**Prevention**:
- ✅ Use parameterized queries (sqlx!)
- ✅ Never concatenate user input into SQL
- ✅ Validate and sanitize all inputs

**Test**:
```rust
// Try common SQL injection patterns
let injections = vec![
    "admin'--",
    "1' OR '1'='1",
    "'; DROP TABLE users--",
    "1'; DELETE FROM users WHERE '1'='1",
];

for injection in injections {
    let result = query_with_input(&pool, injection).await;
    assert!(result.is_err() || result.unwrap().is_empty());
}
```

---

### 2. Cross-Tenant Data Leakage

**Risk**: User from Tenant A accesses Tenant B data

**Prevention**:
- ✅ Always include `tenant_id` in WHERE clauses
- ✅ Validate tenant_id matches JWT claim
- ✅ Use composite foreign keys with tenant_id

**Test**:
```rust
// Create data in tenant A
let data_a = create_data(&pool, tenant_a_id).await;

// Try to access with tenant B token
let token_b = create_jwt_for_tenant(tenant_b_id);
let result = get_data(&pool, &token_b, data_a.id).await;

assert_eq!(result.status(), StatusCode::FORBIDDEN);
```

---

### 3. JWT Token Tampering

**Risk**: Attacker modifies JWT payload to gain unauthorized access

**Prevention**:
- ✅ Use strong signing algorithm (HS256/RS256)
- ✅ Validate signature on every request
- ✅ Use long, random secret keys
- ✅ Rotate secrets periodically

**Test**:
```rust
// Modify JWT payload without re-signing
let tampered = modify_jwt_without_resigning(&valid_token);
let result = validate_jwt(&tampered);

assert!(result.is_err());
assert_eq!(result.unwrap_err(), "Invalid signature");
```

---

### 4. Privilege Escalation

**Risk**: Regular user gains admin privileges

**Prevention**:
- ✅ Role stored in database, not JWT
- ✅ Re-validate role on sensitive operations
- ✅ Separate admin endpoints
- ✅ Audit role changes

**Test**:
```rust
// Regular user tries to call admin endpoint
let user_token = create_jwt_for_user(regular_user);
let result = call_admin_endpoint(&pool, &user_token).await;

assert_eq!(result.status(), StatusCode::FORBIDDEN);
```

---

## Best Practices

### 1. Always Use Ignored Flag for Integration Tests

```rust
#[tokio::test]
#[ignore] // Requires database
async fn test_security_feature() {
    // Test code
}
```

### 2. Clean Up Test Data

```rust
#[tokio::test]
#[ignore]
async fn test_with_cleanup() {
    let pool = setup_test_db().await;
    
    // Test code
    
    // Always cleanup
    cleanup_test_data(&pool).await;
}
```

### 3. Use Helper Functions

```rust
// Good: Reusable helper
async fn create_test_user_with_tenant(pool: &PgPool) -> (User, Tenant) {
    let tenant = create_tenant(pool, "Test Corp").await;
    let user = create_user(pool, tenant.id, "test@example.com").await;
    (user, tenant)
}

// Use in tests
let (user, tenant) = create_test_user_with_tenant(&pool).await;
```

### 4. Test Negative Cases

```rust
// Don't just test success
#[tokio::test]
async fn test_login_success() { /* ... */ }

// Also test failures
#[tokio::test]
async fn test_login_with_wrong_password() { /* ... */ }

#[tokio::test]
async fn test_login_with_nonexistent_user() { /* ... */ }

#[tokio::test]
async fn test_login_with_locked_account() { /* ... */ }
```

---

## Security Test Metrics

Track security test coverage:

| Category | Tests | Coverage | Status |
|----------|-------|----------|--------|
| SQL Injection | 12 | 95% | ✅ |
| Tenant Isolation | 8 | 90% | ✅ |
| JWT Security | 10 | 85% | ✅ |
| RBAC | 6 | 80% | ⚠️ |
| Rate Limiting | 4 | 75% | ⚠️ |
| Input Validation | 15 | 92% | ✅ |
| **Total** | **55** | **88%** | ✅ |

---

## Maintenance

### Weekly
- [ ] Review security test results
- [ ] Fix any failing security tests
- [ ] Update tests for new features

### Monthly
- [ ] Security audit with OWASP Top 10
- [ ] Review and update security checklist
- [ ] Penetration testing (manual)

### Quarterly
- [ ] Full security test suite review
- [ ] Add tests for new vulnerability patterns
- [ ] Update security documentation

---

## Related Documentation

- [CI/CD Pipeline](./CI_CD_PIPELINE.md)
- [Integration Testing](./INTEGRATION_TESTING.md)
- [Testing Best Practices](./TESTING_GUIDE.md)
- [Security Test Report](../security_test_report.md)

---

## Support

- **OWASP Top 10**: https://owasp.org/www-project-top-ten/
- **Rust Security**: https://anssi-fr.github.io/rust-guide/
- **sqlx Security**: https://github.com/launchbadge/sqlx/blob/main/FAQ.md#how-can-i-do-a-select--query
