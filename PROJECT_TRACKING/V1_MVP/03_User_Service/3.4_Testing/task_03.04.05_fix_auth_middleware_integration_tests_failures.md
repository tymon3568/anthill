# Task: Fix `user_service_api` Auth Middleware Integration Test Failures (Follow-up from PR #118)

**Task ID:** V1_MVP/03_User_Service/3.4_Testing/task_03.04.05_fix_auth_middleware_integration_tests_failures.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.4_Testing
**Priority:** High
**Status:** NeedsReview
**Assignee:** Claude
**Created Date:** 2025-12-26
**Last Updated:** 2026-01-16

## Context / Background

Follow-up task created after PR #118 work (tracking PR-review fixes for Casbin auth layer and SQLx test macro standardization). During local verification, `user_service_api` integration tests in `services/user_service/api/tests/auth_middleware_test.rs` were observed failing. These failures are not caused by SQLx macro conversion itself, but rather by test seeding / policy expectations / response shape mismatches.

This task ensures the integration tests are deterministic, correctly seeded, and aligned with current auth behavior (CasbinAuthLayer, extractors, routes, and policy model).

### Observed Failures (as of PR #118 local run)
- `test_admin_can_access_admin_route`: expected `200 OK`, got `403 Forbidden`
- `test_list_users_tenant_isolation`: JSON parsing error (`invalid type: map, expected a sequence`) suggesting the response body is an error JSON object rather than a list

> Note: Exact failure details should be re-validated when starting this task.

## Goal

Make `services/user_service/api/tests/auth_middleware_test.rs` pass reliably by addressing:
- Policy seeding correctness (role/grouping rules, policy rules, tenant IDs, resource/action patterns)
- Test database cleanup idempotency and isolation (avoid cross-run and cross-test interference)
- Correct route expectations and response shape assumptions

## Scope

### In Scope
- Update integration tests and their supporting helpers:
  - `services/user_service/api/tests/auth_middleware_test.rs`
  - `services/user_service/api/tests/helpers.rs`
- Fix test seeding/cleanup so that:
  - Casbin rules are inserted consistently for the correct tenant(s)
  - Grouping policies (`g`) and policies (`p`) match the enforcer model and enforcement inputs
  - Tests do not depend on pre-existing DB state from other suites
- Adjust test expectations if the service API behavior has changed legitimately (e.g., response status codes or error shapes)

### Out of Scope
- Major redesign of auth architecture
- Large changes to Kanidm integration / tenant mapping rules beyond what is required for tests
- Fixing unrelated integration tests in other services (e.g., `inventory_service_api` 401 failures)

## Why This Matters

- CI stability: flaky or failing integration tests block merges and reduce confidence.
- Security correctness: authorization tests validate core RBAC guarantees and tenant isolation.
- Developer productivity: deterministic tests reduce local environment friction.

## Acceptance Criteria

- [x] `cargo test -p user_service_api --test auth_middleware_test` passes consistently
- [x] Test DB cleanup and seeding is repeatable across multiple runs without manual intervention
- [x] Tests continue using SQLx compile-time macros (`sqlx::query!`, `sqlx::query_as!`) per enterprise standard
- [x] No production logic changes unless strictly required to align runtime behavior with intended contract (if needed, must be explicitly documented in this task)
- [x] Task file includes clear "Root Cause" and "Fix Summary" sections when work is completed
- [x] Quality gates documented (see below)

## Root Cause Analysis

### Issue 1: Casbin model file not found
**Symptom:** `IoError: No such file or directory` when initializing Casbin enforcer
**Cause:** `setup_test_app()` used `Config::default()` which has `casbin_model_path: "shared/auth/model.conf"` - a relative path that doesn't resolve correctly when tests run from crate directory.
**Fix:** Calculate absolute path from `CARGO_MANIFEST_DIR` environment variable.

### Issue 2: Test data not seeded / duplicate key errors
**Symptom:** `RowNotFound` for test users, then `duplicate key value violates unique constraint`
**Cause:** 
- `setup_test_app()` didn't call `seed_test_data()`
- `cleanup_test_data()` used DELETE in wrong order and missed tables with FK constraints
**Fix:** 
- Added cleanup and seed calls to `setup_test_app()`
- Changed cleanup to use `TRUNCATE TABLE tenants CASCADE` for reliable FK handling

### Issue 3: JWT role format mismatch
**Symptom:** `test_admin_can_access_admin_route` - Casbin passes (Response: true) but handler returns 403
**Cause:** Tests used `"role:admin"` in JWT claims, but `RequireAdmin` extractor checks `user.role == "admin"` (without prefix)
**Fix:** Changed JWT role in tests from `"role:admin"` to `"admin"`, `"role:manager"` to `"manager"`, etc.

### Issue 4: Missing Casbin g rules for users
**Symptom:** `test_user_can_access_read_only_route` - Casbin denies (Response: false)
**Cause:** `seed_test_data()` only created g rules for admin and manager, not for regular users
**Fix:** Added g rules for user and user_b in `seed_test_data()`

### Issue 5: Wrong request body format for add_policy
**Symptom:** Handler returns 422 Unprocessable Entity
**Cause:** Tests sent `{ptype, v0, v1, v2, v3}` but handler expects `{role, resource, action}`
**Fix:** Updated request body to use correct `CreatePolicyReq` format

### Issue 6: Wrong response parsing for list_users
**Symptom:** `invalid type: map, expected a sequence`
**Cause:** `list_users` handler returns `UserListResp { users: [...], total, page, page_size }`, not a direct array
**Fix:** Changed test to parse response as object and extract `users` array

## Fix Summary

**Files Modified:**
1. `services/user_service/api/tests/auth_middleware_test.rs`:
   - Fixed `setup_test_app()` to calculate absolute Casbin model path from `CARGO_MANIFEST_DIR`
   - Added cleanup and seed calls to `setup_test_app()`
   - Changed JWT roles from `"role:admin"` to `"admin"`, etc.
   - Fixed request body format for `/api/v1/admin/policies` POST
   - Fixed response parsing for list users (object with `users` array, not direct array)

2. `services/user_service/api/tests/helpers.rs`:
   - Simplified `cleanup_test_data()` to use `TRUNCATE TABLE tenants CASCADE`
   - Added g rules for user and user_b in `seed_test_data()`

## Quality Gates

- [x] `cargo fmt` - passed
- [x] `cargo check -p user_service_api` - passed
- [x] `cargo test -p user_service_api --test auth_middleware_test` - 6/6 tests pass

## Dependencies

- V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.13_pr_118_review_fixes.md (context)
- A reachable Postgres DB with migrated schema for running integration tests

## AI Agent Log
---
* 2025-12-26 00:00: [Task Created] by GPT-5.2_Backend
  - Created follow-up task to address failing `user_service_api` auth middleware integration tests observed during PR #118 verification.
  - Recorded failing tests and initial suspected root causes (policy seeding mismatch, response shape mismatch).
  - Status: Todo

* 2026-01-16 11:06: Task claimed by Claude
  - Verified dependencies
  - Created feature branch: feature/03.04.05-fix-auth-middleware-tests
  - Beginning investigation

* 2026-01-16 11:07: Initial investigation by Claude
  - Reproduced failures: all 6 tests fail with "No such file or directory" for Casbin model
  - Root cause: relative path in Config doesn't resolve from test crate directory

* 2026-01-16 11:08: Fix 1 applied by Claude
  - Added absolute path calculation using CARGO_MANIFEST_DIR
  - Tests now fail with "RowNotFound" - data not seeded

* 2026-01-16 11:10: Fix 2 applied by Claude
  - Added cleanup_test_data() and seed_test_data() calls
  - Tests now fail with duplicate key - cleanup incomplete

* 2026-01-16 11:12: Fix 3 applied by Claude
  - Changed cleanup to TRUNCATE CASCADE
  - Tests now: 3 pass, 3 fail (role format and response shape issues)

* 2026-01-16 11:14: Fixes 4-6 applied by Claude
  - Fixed JWT role format (removed "role:" prefix)
  - Added g rules for users in seed_test_data()
  - Fixed request body and response parsing
  - All 6 tests pass

* 2026-01-16 11:18: Quality gates completed by Claude
  - cargo fmt: passed
  - cargo check: passed
  - cargo test: 6/6 passed
  - Ready for review
