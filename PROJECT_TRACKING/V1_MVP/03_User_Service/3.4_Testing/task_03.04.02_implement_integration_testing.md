# Task: Implement Integration Testing with Test Database

**Task ID:** V1_MVP/03_User_Service/3.4_Testing/task_03.04.02_implement_integration_testing.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.4_Testing
**Priority:** High  
**Status:** Todo  
**Assignee:**
**Created Date:** 2025-01-21
**Last Updated:** 2025-01-21

## Detailed Description:
Implement comprehensive integration testing that validates the entire user service stack including OAuth2 endpoints, Kanidm authentication flow, database operations, and tenant isolation. **Note**: This task has been updated for Kanidm OAuth2/OIDC authentication instead of custom JWT.

## Specific Sub-tasks:
- [ ] 1. Set up test database with Docker for integration tests
- [ ] 2. Set up mock Kanidm server or test instance for OAuth2 testing
- [ ] 3. Create test data seeding (tenants, kanidm_tenant_groups) and cleanup utilities
- [ ] 4. Implement OAuth2 flow integration tests (authorize, callback, refresh)
- [ ] 5. Test Kanidm JWT validation and token refresh
- [ ] 6. Test group-to-tenant mapping logic
- [ ] 7. Test authentication extractors (AuthUser, RequireAdmin) with Kanidm JWT
- [ ] 8. Test Casbin authorization with Kanidm JWT subject
- [ ] 9. Test tenant isolation with Kanidm groups
- [ ] 10. Test error handling (invalid tokens, missing groups, expired tokens)
- [ ] 11. Create test reporting and result analysis
- [ ] 12. Implement cross-service integration tests (Future: when other services are ready)

## Acceptance Criteria:
- [ ] Integration test suite operational with real database
- [ ] Mock Kanidm server or test instance configured
- [ ] OAuth2 flow tests covering authorize/callback/refresh endpoints
- [ ] Kanidm JWT validation tested with valid/invalid/expired tokens
- [ ] Group-tenant mapping tested (multiple groups, no groups, unmapped groups)
- [ ] Auth extractors work correctly with Kanidm JWT
- [ ] Casbin authorization tested with Kanidm subject format
- [ ] Tenant isolation validated (cross-tenant access prevented)
- [ ] Error scenarios properly handled in tests
- [ ] Test results reporting and analysis available
- [ ] Cross-service integration validated (Future: pending other services)

## Dependencies:
- V1_MVP/03_User_Service/3.4_Testing/task_03.04.01_create_unit_test_framework.md
- V1_MVP/03_User_Service/3.1_Kanidm_Integration/task_03.01.02_create_kanidm_client_library.md
- V1_MVP/03_User_Service/3.1_Kanidm_Integration/task_03.01.03_implement_oauth2_endpoints.md

## Related Documents:
- `services/user_service/api/tests/oauth2_flow_tests.rs` (to be created)
- `services/user_service/api/tests/kanidm_jwt_tests.rs` (to be created)
- `services/user_service/api/tests/tenant_mapping_tests.rs` (to be created)
- `services/user_service/api/tests/test_kanidm_mock.rs` (to be created)
- `docker-compose.test.yml` (update to include Kanidm test instance)

## Notes / Discussion:
---
* **IMPORTANT**: This task requires significant update for Kanidm OAuth2 flow
* Integration tests must use mock Kanidm server or dedicated test instance
* Test JWT generation must match Kanidm's token format and claims
* Group-tenant mapping is critical for multi-tenancy - test thoroughly
* Consider using `wiremock` crate for mocking Kanidm OAuth2 endpoints
* Kanidm test instance should be isolated from production/dev instances
* Test token validation caching to ensure performance
* Mock OAuth2 callback URLs must match configured redirect URIs

## Implementation Notes (Kanidm-Specific):
---

### 1. Mock Kanidm Server Strategy

**Option A: Wiremock** (Recommended for unit-like integration tests)
```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_oauth2_callback_success() {
    let mock_server = MockServer::start().await;
    
    // Mock token endpoint
    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "kanidm_test_token",
            "refresh_token": "kanidm_refresh_token",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;
    
    // Test OAuth2 callback
    let response = app
        .post("/api/v1/auth/oauth/callback")
        .json(&json!({"code": "test_code", "state": "test_state"}))
        .send()
        .await?;
    
    assert_eq!(response.status(), 200);
}
```

**Option B: Real Kanidm Test Instance** (For end-to-end validation)
```yaml
# docker-compose.test.yml
services:
  kanidm_test:
    image: kanidm/server:latest
    environment:
      KANIDM_DOMAIN: localhost
      KANIDM_ORIGIN: http://localhost:8300
    ports:
      - "8300:8300"
    volumes:
      - ./test-data/kanidm:/data
```

### 2. Test JWT Generation

Create helper to generate valid Kanidm-like JWTs:
```rust
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct KanidmClaims {
    sub: String,  // Kanidm user UUID
    email: String,
    preferred_username: String,
    groups: Vec<String>,  // ["tenant_acme_users", "tenant_acme_admins"]
    exp: usize,
    iat: usize,
}

fn create_test_kanidm_jwt(user_id: &str, groups: Vec<String>) -> String {
    let claims = KanidmClaims {
        sub: user_id.to_string(),
        email: "test@example.com".to_string(),
        preferred_username: "testuser".to_string(),
        groups,
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
        iat: chrono::Utc::now().timestamp() as usize,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test_secret")
    ).unwrap()
}

// Usage in tests
let jwt = create_test_kanidm_jwt(
    "550e8400-e29b-41d4-a716-446655440000",
    vec!["tenant_acme_users".to_string()]
);
```

### 3. Group-Tenant Mapping Tests

```rust
#[sqlx::test]
async fn test_group_tenant_mapping(pool: PgPool) {
    // Setup: Create tenant and mapping
    let tenant_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO tenants (tenant_id, tenant_name) VALUES ($1, 'Acme Corp')",
        tenant_id
    ).execute(&pool).await?;
    
    sqlx::query!(
        "INSERT INTO kanidm_tenant_groups (tenant_id, kanidm_group_uuid, kanidm_group_name)
         VALUES ($1, $2, 'tenant_acme_users')",
        tenant_id,
        Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000")?
    ).execute(&pool).await?;
    
    // Test: JWT with matching group
    let jwt = create_test_kanidm_jwt(
        "user-uuid",
        vec!["tenant_acme_users".to_string()]
    );
    
    let response = app
        .get("/api/v1/profile")
        .header("Authorization", format!("Bearer {}", jwt))
        .send()
        .await?;
    
    assert_eq!(response.status(), 200);
    
    // Test: JWT with non-existent group (should fail)
    let jwt_invalid = create_test_kanidm_jwt(
        "user-uuid",
        vec!["tenant_nonexistent_users".to_string()]
    );
    
    let response = app
        .get("/api/v1/profile")
        .header("Authorization", format!("Bearer {}", jwt_invalid))
        .send()
        .await?;
    
    assert_eq!(response.status(), 401); // Unauthorized
}
```

### 4. OAuth2 Flow Tests

```rust
#[tokio::test]
async fn test_complete_oauth2_flow() {
    let mock_kanidm = MockServer::start().await;
    
    // Step 1: Initiate authorization
    let response = app.get("/api/v1/auth/oauth/authorize").send().await?;
    assert_eq!(response.status(), 302); // Redirect to Kanidm
    let location = response.headers().get("Location").unwrap();
    assert!(location.to_str()?.contains("oauth2?client_id="));
    
    // Step 2: Mock Kanidm callback with authorization code
    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": create_test_kanidm_jwt("user-id", vec!["tenant_acme_users"]),
            "refresh_token": "refresh_token_here",
            "expires_in": 3600
        })))
        .mount(&mock_kanidm)
        .await;
    
    // Step 3: Exchange code for tokens
    let response = app
        .post("/api/v1/auth/oauth/callback")
        .json(&json!({"code": "auth_code", "state": "state"}))
        .send()
        .await?;
    
    assert_eq!(response.status(), 200);
    let body: AuthResp = response.json().await?;
    assert!(!body.access_token.is_empty());
    
    // Step 4: Use access token to call protected endpoint
    let response = app
        .get("/api/v1/profile")
        .header("Authorization", format!("Bearer {}", body.access_token))
        .send()
        .await?;
    
    assert_eq!(response.status(), 200);
}
```

### 5. Tenant Isolation Tests

```rust
#[sqlx::test]
async fn test_cross_tenant_access_prevention(pool: PgPool) {
    // Create two tenants
    let tenant_a = create_test_tenant(&pool, "Tenant A").await?;
    let tenant_b = create_test_tenant(&pool, "Tenant B").await?;
    
    // Create user in Tenant A
    let user_a_jwt = create_test_kanidm_jwt(
        "user-a-uuid",
        vec![format!("tenant_{}_users", tenant_a.tenant_id)]
    );
    
    // Try to access Tenant B's data with Tenant A's JWT
    let response = app
        .get(format!("/api/v1/tenants/{}/users", tenant_b.tenant_id))
        .header("Authorization", format!("Bearer {}", user_a_jwt))
        .send()
        .await?;
    
    assert_eq!(response.status(), 403); // Forbidden
}
```

### 6. Required Test Files Structure

```
services/user_service/api/tests/
├── oauth2_flow_tests.rs       # Complete OAuth2 authorize/callback/refresh flow
├── kanidm_jwt_tests.rs        # JWT validation, expiration, invalid tokens
├── tenant_mapping_tests.rs    # Group-to-tenant mapping logic
├── auth_extractor_tests.rs    # AuthUser, RequireAdmin with Kanidm JWT
├── casbin_kanidm_tests.rs     # Casbin with Kanidm subject format
├── test_kanidm_mock.rs        # Mock Kanidm server utilities
└── test_helpers.rs            # JWT generation, tenant creation helpers
```

## AI Agent Log:
---
### 2025-01-10 - Task Updated for Kanidm Migration

**Note:** Original implementation (custom JWT) is now DEPRECATED. Task needs complete rewrite for Kanidm OAuth2 flow.

**Changes Required:**
1. Replace JWT generation tests with OAuth2 flow tests
2. Add mock Kanidm server setup
3. Test group-tenant mapping logic
4. Update auth extractor tests for Kanidm JWT format
5. Test Casbin integration with Kanidm subject
6. Add tenant isolation tests with Kanidm groups

**Original Implementation (DEPRECATED):**
- Custom JWT generation and validation ❌
- Password-based login ❌
- Direct user registration ❌

**New Implementation (TODO):**
- OAuth2 authorization code flow with PKCE ⏳
- Kanidm JWT validation ⏳
- Group-to-tenant mapping ⏳
- External Identity Provider integration ⏳

**Next Steps:**
1. Complete Kanidm integration tasks (3.1.x)
2. Rewrite test suite for OAuth2 flow
3. Set up mock Kanidm server or test instance
4. Implement new integration tests following patterns above

---

### 2025-01-21 - Original Task Completed (NOW DEPRECATED)

⚠️ **WARNING**: The implementation below is for custom JWT auth and is NO LONGER VALID after Kanidm migration.

**Branch:** `feature/user-service-integration-tests` (DEPRECATED)

**Implemented:**

1. **Test Database Infrastructure** (`docker-compose.test.yml`)
   - Isolated PostgreSQL on port 5433
   - Redis test instance on port 6380
   - NATS test instance on port 4223
   - Automatic health checks and initialization

2. **Test Database Module** (`test_database.rs`)
   - `TestDatabaseConfig` with automatic resource tracking
   - Tenant, user, and session creation helpers
   - Automatic cleanup on drop
   - Transaction support for rollback testing
   - Database verification utilities

3. **API Endpoint Integration Tests** (`api_endpoint_tests.rs`)
   - Registration tests (success, duplicate email, validation)
   - Login tests (success, invalid credentials)
   - Profile management (get, update)
   - Admin functionality (list users, update roles)
   - Authorization tests (unauthorized access, forbidden)
   - Tenant isolation tests
   - Input validation tests
   - **Total: 15+ comprehensive endpoint tests**

4. **Authentication Flow Tests** (`auth_flow_tests.rs`)
   - Complete registration-to-request flow
   - Login with token refresh
   - Logout flow
   - RBAC flows (user to admin promotion, manager permissions)
   - JWT token validation and expiration
   - Invalid token handling
   - Cross-tenant access prevention
   - Password change flow
   - **Total: 10+ end-to-end flow tests**

5. **Error Handling Tests** (`error_handling_tests.rs`)
   - Missing required fields validation
   - Invalid email format detection
   - Weak password detection
   - Malformed JSON handling
   - Extremely long input handling
   - Authentication errors (wrong password, nonexistent user)
   - Malformed authorization headers
   - Resource not found errors
   - Concurrent duplicate registrations
   - SQL injection prevention
   - Rate limiting tests
   - **Total: 20+ error scenario tests**

6. **Test Automation Scripts**
   - `scripts/run-integration-tests.sh` - Comprehensive test runner
   - Automatic database setup and teardown
   - Verbose and filtered test execution
   - Test data cleanup
   - Colored output and reporting

7. **Documentation**
   - `INTEGRATION_TEST_GUIDE.md` - Complete testing guide
   - Quick start instructions
   - Test writing guidelines
   - Troubleshooting section
   - CI/CD integration examples

**Test Coverage:**
- ✅ Registration and login flows
- ✅ Profile management
- ✅ Admin operations
- ✅ Authorization and RBAC
- ✅ Multi-tenant isolation
- ✅ Input validation
- ✅ Error handling
- ✅ JWT token lifecycle
- ✅ Database transactions
- ✅ Concurrent operations
- ✅ SQL injection prevention

**Files Created:**
- `docker-compose.test.yml`
- `services/user_service/api/tests/test_database.rs`
- `services/user_service/api/tests/api_endpoint_tests.rs`
- `services/user_service/api/tests/auth_flow_tests.rs`
- `services/user_service/api/tests/error_handling_tests.rs`
- `scripts/run-integration-tests.sh`
- `services/user_service/api/tests/INTEGRATION_TEST_GUIDE.md`

**Running Tests:**
```bash
# Quick run
./scripts/run-integration-tests.sh

# With cleanup
./scripts/run-integration-tests.sh --teardown

# Verbose
./scripts/run-integration-tests.sh --verbose

# Specific test
./scripts/run-integration-tests.sh --filter test_user_registration_success
```

**Next Steps:**
1. Merge feature branch to main
2. Add integration tests to CI/CD pipeline
3. Monitor test execution time and optimize slow tests
4. Implement cross-service integration tests when other services are ready
5. Add load/stress testing for performance benchmarks
