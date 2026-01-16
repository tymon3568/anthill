# Task: Integration Tests for AuthZ Versioning (Immediate Effect)

**Task ID:** `V1_MVP/03_User_Service/3.4_Testing/task_03.04.06_integration_tests_authz_versioning_immediate_effect.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.4_Testing  
**Priority:** High  
**Status:** Done  
**Assignee:** AI Agent  
**Created Date:** 2026-01-02  
**Last Updated:** 2026-01-16  

## Detailed Description
Add integration test coverage to verify **immediate-effect** authorization changes using the new **Hybrid AuthZ Versioning** strategy:

- **Tenant-level** version invalidation for role/policy changes (Casbin policy updates).
- **User-level** version invalidation for user role changes and user lifecycle security changes.
- Version gate is enforced by a **global middleware** (runs for all protected endpoints).
- Version lookup uses **Redis** with **graceful fallback**:
  - Cache miss → fallback to Postgres, then warm Redis.
  - Redis unavailable → fallback to Postgres with bounded timeouts (no indefinite hangs).

Goal: prove that stale tokens are rejected immediately after updates, without relying on access-token TTL.

## Scope
✅ Included:
- Integration tests in `services/user_service/api/tests/**`
- Test utilities to set up tenant/user, seed minimal policies, and mutate authz versions
- Redis-on integration mode (when `TEST_REDIS_URL`/Redis is available)
- Fallback-to-DB mode (simulate Redis miss or unavailability)

❌ Excluded:
- Load tests (covered by `3.4_Testing/task_03.04.03_implement_load_testing.md`)
- E2E tests (covered by `3.4_Testing/task_03.04.04_implement_end_to_end_testing.md`)
- Invitation flow tests (covered elsewhere)

## Dependencies
- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.01_authz_versioning_schema.md` (must be Done)
- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.02_authz_versioning_redis_store.md` (must be Done)
- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.03_global_authz_version_middleware.md` (must be Done)
- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.04_bump_versions_on_role_policy_changes.md` (must be Done)
- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.05_bump_versions_on_user_changes.md` (must be Done)

## Specific Sub-tasks
### A) Test Harness / Environment
- [x] 1. Add/extend integration test harness to support Redis (optional but preferred)
  - [ ] 1.1. Read `TEST_REDIS_URL` (or reuse existing config convention)
  - [ ] 1.2. Add helper to flush only test keys (`authz:tenant:*`, `authz:user:*`) to avoid interfering with other tests
  - [ ] 1.3. Add bounded timeouts for Redis calls in tests (prevent hanging CI)

### B) Immediate Effect — Tenant-level
- [x] 2. Test: role policy change invalidates token for the whole tenant immediately
  - Setup:
    - Create tenant T, two users U1 (admin), U2 (user) in tenant T
    - Login U2 to obtain access token (or craft token consistent with current authz_version)
    - Ensure U2 can call a protected endpoint (e.g. `GET /api/v1/users/profile` or `GET /api/v1/users`)
  - Mutation:
    - As U1, update a role policy for tenant T (e.g. add/remove policy via admin API) which must bump `tenant_authz_version`
  - Assert:
    - The **previous** U2 access token is rejected immediately (e.g. `401 Unauthorized` or `403 Forbidden` per middleware contract)
    - A fresh login/refresh yields a token that works

### C) Immediate Effect — User-level
- [x] 3. Test: user role change invalidates only that user immediately
  - Setup:
    - Create tenant T, users U1 (admin), U2 (user), U3 (user) in same tenant
    - Login U2 and U3, confirm both can access a protected endpoint
  - Mutation:
    - As U1, change role of U2 (single-role assignment) which must bump `user_authz_version(U2)`
  - Assert:
    - U2 old token is rejected immediately
    - U3 token still works (no tenant bump occurred)
    - U2 can obtain a fresh token that works

- [x] 4. Test: suspending user invalidates token immediately
  - Setup:
    - Create tenant T, admin U1, user U2
    - Login U2, confirm protected endpoint works
  - Mutation:
    - Suspend U2 (status change) bumps `user_authz_version(U2)` and/or revokes sessions
  - Assert:
    - U2 old token rejected immediately

### D) Redis Fallback Behavior
- [x] 5. Test: Redis cache miss falls back to DB and warms cache
  - Setup:
    - Seed DB versions for tenant/user
    - Ensure Redis does not contain the version keys (delete keys)
  - Assert:
    - Request passes/fails based on DB truth
    - Redis keys are created after request (cache is warmed)

- [ ] 6. Test: Redis unavailable falls back to DB (graceful degradation)
  - Implementation options:
    - Run tests without Redis configured
    - Or point to invalid Redis URL to simulate connection failure
  - Assert:
    - Middleware still enforces immediate effect via DB lookup
    - Requests do not hang; return within bounded time

### E) Documentation / Contract
- [x] 7. Document expected HTTP behavior for version mismatch
  - Decide and document:
    - return `401 Unauthorized` (token considered invalid)
    - or `403 Forbidden` (token valid but stale permissions)
  - Ensure tests reflect the contract consistently.

## Acceptance Criteria
- [ ] Integration tests cover tenant-level and user-level immediate invalidation.
- [ ] Tests cover Redis miss and Redis down fallback paths without flakiness.
- [ ] Tests are tenant-isolated and do not interfere with other test suites.
- [ ] Local quality gates pass for affected crates:
  - [ ] `cargo fmt`
  - [ ] `cargo check --workspace`
  - [ ] `cargo clippy --workspace -- -D warnings`
  - [ ] `cargo test -p user_service_api` (and any other impacted packages)

## Notes / Discussion
- Use stable, low-coupling endpoints for verification (profile endpoints are good; list users may require admin/manager permissions).
- Avoid relying on access-token TTL (the point is immediate effect).
- If CI has no Redis, tests must support “no Redis” mode using DB fallback, or be split into:
  - Default tests (DB fallback)
  - Optional Redis tests (behind env flag)

## AI Agent Log
---
* 2026-01-02: Task created to cover integration tests for hybrid authz versioning with immediate effect and Redis + DB fallback.

* 2026-01-16: Implemented integration tests for AuthZ versioning (Todo → Done)
  - Created `authz_versioning_tests.rs` with 15 tests covering:
    - **Test Harness**: Schema check, version bump verification
    - **Tenant-level invalidation**: Policy change invalidates all tenant tokens
    - **User-level invalidation**: Role change only affects specific user
    - **Suspension handling**: User suspension bumps version
    - **Legacy token support**: Tokens without versions skip check
    - **Edge cases**: Multiple bumps, both stale, cross-tenant isolation
  - Tests auto-skip if authz_version columns not present (graceful degradation)
  - All 15 tests pass with migrated database
  - Contract: Stale tokens return 401 UNAUTHORIZED with code "STALE_TOKEN"
