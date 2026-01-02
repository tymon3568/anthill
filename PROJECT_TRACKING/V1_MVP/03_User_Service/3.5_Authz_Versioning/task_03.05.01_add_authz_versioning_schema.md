# Task: Add AuthZ Versioning Schema (Hybrid: Tenant + User)

**Task ID:** `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.01_add_authz_versioning_schema.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.5_Authz_Versioning  
**Priority:** High  
**Status:** NeedsReview  
**Assignee:** GPT-5.2_Backend  
**Created Date:** 2026-01-02  
**Last Updated:** 2026-01-02

## Detailed Description
Introduce **hybrid authorization versioning** to support **immediate effect** permission changes while using Redis as the fast-path cache for version checks.

This task adds the **database schema** components required for:
- `tenant_authz_version` (tenant-wide invalidation when role/policy definitions change)
- `user_authz_version` (per-user invalidation when role assignment or account security state changes)

These versions will be used by a **global auth middleware** (implemented in later tasks) to reject stale tokens immediately.

## Scope
- ✅ Included:
  - SQL migration(s) to store authz versions for tenants and users
  - Default values and indexing
  - Backfill strategy for existing rows
- ❌ Excluded:
  - Redis caching layer
  - Middleware enforcement
  - Bump/version update logic in services/handlers

## Decisions / Standards
- **Hybrid model**:
  - Tenant-level version bumps for role/policy changes
  - User-level version bumps for role assignment, suspend/reset-password, etc.
- **Immediate effect requirement**: tokens minted before version bump must be rejected (via version gate).
- **Multi-tenancy**: ensure tenant-scoped schema and indexes follow `(tenant_id, ...)` conventions where applicable.
- **Soft-delete**: preserve existing soft-delete patterns; versioning should not break them.

## Proposed Schema (Recommended)
### Option A (recommended): columns on existing tables
1) Add `authz_version BIGINT NOT NULL DEFAULT 1` to `tenants`
2) Add `authz_version BIGINT NOT NULL DEFAULT 1` to `users`

Notes:
- `BIGINT` chosen to avoid wrap risk and allow monotonically increasing version.
- Use DB-side increments in update queries where possible.

### Indexes
- `tenants(tenant_id, authz_version)` (optional; tenant_id is already unique/PK, version read is by tenant_id)
- `users(user_id, tenant_id, authz_version)` or `users(tenant_id, user_id, authz_version)` depending on existing PK/index layout

The middleware will typically look up:
- tenant version by `tenant_id`
- user version by `(tenant_id, user_id)` or `user_id` (depending on existing constraints)

## Specific Sub-tasks
- [x] 1. Inspect current `tenants` and `users` table schema (PKs, indexes, existing columns)
- [x] 2. Create a migration:
  - [x] 2.1 Add `authz_version` to `tenants` (default `1`)
  - [x] 2.2 Add `authz_version` to `users` (default `1`)
  - [x] 2.3 Ensure existing rows are backfilled (handled by default + NOT NULL)
- [x] 3. Add any required indexes consistent with repo standards:
  - [x] 3.1 Prefer composite indexes `(tenant_id, user_id)` if not already present
  - [x] 3.2 Add partial indexes only if justified (not expected here)
- [x] 4. Add minimal documentation in migration header/comments describing purpose (authz version gate)

## Acceptance Criteria
- [x] Migration applies cleanly on existing DB
- [x] `tenants.authz_version` and `users.authz_version` exist and are `NOT NULL` with default `1`
- [x] Indexing is present and aligns with multi-tenant performance rules
- [x] No breaking change to existing queries (existing services compile and run)

## Dependencies
- None (schema-only addition)

## Related / Follow-up Tasks (planned)
- `task_03.05.02_implement_authz_version_store_with_redis.md` (Redis cache + DB fallback)
- `task_03.05.03_add_global_authz_version_middleware.md` (global middleware gate)
- `task_03.05.04_bump_versions_on_role_policy_and_user_changes.md` (hook into admin/user operations)

## Notes / Discussion
- Tokens should include `tenant_authz_version` and `user_authz_version` at mint time (future task).
- For immediate effect, middleware must compare JWT versions to current versions (Redis fast path, DB fallback).
- On Redis miss, middleware should warm cache from DB (future task).
- On Redis outage, middleware should degrade gracefully to DB lookup with timeouts/circuit breaker (future task).

## AI Agent Log
---
* 2026-01-02: Task created to introduce DB schema for hybrid authz versioning (tenant + user).
* 2026-01-02: Task claimed by GPT-5.2_Backend.
  - Verified dependencies: none (schema-only task).
  - Plan: add `authz_version BIGINT NOT NULL DEFAULT 1` to `tenants` and `users`, plus any required indexes following `(tenant_id, ...)` standards.
  - Next: proceed via GitHub flow (feature branch), then run local quality gates before marking `NeedsReview`.
* 2026-01-02: Implemented schema via migration `migrations/20260102000001_add_authz_versioning.sql`.
  - Added `tenants.authz_version BIGINT NOT NULL DEFAULT 1`
  - Added `users.authz_version BIGINT NOT NULL DEFAULT 1`
  - Added index `idx_users_authz_version_active` on `users(tenant_id, user_id, authz_version)` WHERE `deleted_at IS NULL`
  - Validation: `sqlx migrate run` applied successfully on dev DB; `cargo check --workspace` OK; `cargo fmt` OK; `cargo clippy --workspace -- -D warnings` OK.
  - Note: `cargo test --workspace` did not fully pass due to unrelated compilation/test errors in `inventory_service_api` (`AppState` initializer missing `scrap_service`), not caused by this migration.
