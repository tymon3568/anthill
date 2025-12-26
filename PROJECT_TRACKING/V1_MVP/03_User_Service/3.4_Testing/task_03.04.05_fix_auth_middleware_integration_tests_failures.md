# Task: Fix `user_service_api` Auth Middleware Integration Test Failures (Follow-up from PR #118)

**Task ID:** V1_MVP/03_User_Service/3.4_Testing/task_03.04.05_fix_auth_middleware_integration_tests_failures.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.4_Testing
**Priority:** High
**Status:** Todo
**Assignee:**
**Created Date:** 2025-12-26
**Last Updated:** 2025-12-26

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

- [ ] `cargo test -p user_service_api --test auth_middleware_test` passes consistently
- [ ] Test DB cleanup and seeding is repeatable across multiple runs without manual intervention
- [ ] Tests continue using SQLx compile-time macros (`sqlx::query!`, `sqlx::query_as!`) per enterprise standard
- [ ] No production logic changes unless strictly required to align runtime behavior with intended contract (if needed, must be explicitly documented in this task)
- [ ] Task file includes clear “Root Cause” and “Fix Summary” sections when work is completed
- [ ] Quality gates documented (see below)

## Suspected Root Causes / Investigation Checklist

- [ ] Verify Casbin model expectations (subject/tenant/resource/action) and how the middleware/enforcer is called
- [ ] Verify test policy seeding inserts correct columns:
  - `ptype='g'` grouping rules and which columns contain tenant_id vs role vs subject
  - `ptype='p'` policy rules and which column stores tenant identifier (string UUID) and resource/action values
- [ ] Confirm that `admin@test.com` user is assigned `role:admin` for the correct tenant used in JWT
- [ ] Confirm that `/api/v1/admin/policies` route requires which permission and how it is enforced (middleware vs extractor)
- [ ] For list users:
  - Verify `/api/v1/users` handler returns a JSON array for success
  - On error (401/403/500), it likely returns an error map/object; tests should print body on failure for debugging
- [ ] Ensure cleanup removes relevant `casbin_rule` rows for the seeded tenant(s) to avoid duplicates and drift
- [ ] Ensure tests do not rely on a shared DB schema state that changes across suites (consider per-test tenant slugs/ids or transactions)

## Implementation Plan

1. Reproduce failures
   - Run: `DATABASE_URL=... cargo test -p user_service_api --test auth_middleware_test -- --nocapture`

2. Improve diagnostics (test-only)
   - Log response body on non-2xx for easier root cause identification (ensure not too noisy)

3. Fix seeding/cleanup
   - Make seeding idempotent:
     - Use `ON CONFLICT DO NOTHING` where appropriate
     - Delete previous seeded data deterministically
   - Ensure tenant IDs used in JWT match tenant IDs used in Casbin policies

4. Align test expectations with current service behavior
   - Update expected status codes if the security policy is correct but tests are outdated
   - Update JSON parsing expectations to only parse arrays on success responses

5. Validate
   - `cargo fmt`
   - `SQLX_OFFLINE=true cargo check --workspace`
   - `SQLX_OFFLINE=true cargo clippy --workspace -- -D warnings`
   - `DATABASE_URL=... cargo test -p user_service_api --test auth_middleware_test`

## Dependencies

- V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.13_pr_118_review_fixes.md (context)
- A reachable Postgres DB with migrated schema for running integration tests
- SQLx offline metadata may need refresh (`cargo sqlx prepare --workspace`) if queries change

## Related References

- PR #118: https://github.com/tymon3568/anthill/pull/118
- Tests:
  - `services/user_service/api/tests/auth_middleware_test.rs`
  - `services/user_service/api/tests/helpers.rs`
- Shared auth:
  - `shared/auth/src/layer.rs`
  - `shared/auth/src/extractors.rs`

## Quality Gates (Before NeedsReview)

- [ ] `cargo fmt`
- [ ] `SQLX_OFFLINE=true cargo check --workspace`
- [ ] `SQLX_OFFLINE=true cargo clippy --workspace -- -D warnings`
- [ ] `DATABASE_URL=... cargo test -p user_service_api --test auth_middleware_test`

## AI Agent Log
---
* 2025-12-26 00:00: [Task Created] by GPT-5.2_Backend
  - Created follow-up task to address failing `user_service_api` auth middleware integration tests observed during PR #118 verification.
  - Recorded failing tests and initial suspected root causes (policy seeding mismatch, response shape mismatch).
  - Status: Todo
  - Files modified: (task file only)
