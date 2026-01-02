# Task: Implement Redis-backed AuthZ Version Store (Hybrid) with DB Fallback

**Task ID:** `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.02_implement_authz_version_store_with_redis.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.5_Authz_Versioning  
**Priority:** High  
**Status:** Todo  
**Assignee:**  
**Created Date:** 2026-01-02  
**Last Updated:** 2026-01-02  

## Context / Goal

Implement a **Redis-backed, hybrid AuthZ version store** to enable **immediate effect** for role/permission changes while keeping **high performance**.

Hybrid means:
- **Tenant-level version**: invalidates all users in a tenant when role permissions/policies are changed.
- **User-level version**: invalidates only a specific user when their role/status changes.

Request-time checks must be fast (Redis), but reliable:
- **Redis miss** → fallback to Postgres (source of truth) and **warm** Redis.
- **Redis down / timeout** → fallback to Postgres with timeouts and conservative behavior to preserve availability.

This task covers only the **version store** (core traits + infra Redis/DB implementation) and does **not** add middleware or bumping logic (handled by separate tasks).

## Requirements

### Functional
- Provide APIs to:
  - Read current tenant/user authz versions
  - Bump tenant/user authz versions atomically
  - Warm Redis from DB when missing
- Must support both:
  - `tenant_authz_version`
  - `user_authz_version`

### Non-functional
- High performance: Redis on hot path
- Immediate effect: version mismatches reject requests (handled by middleware task)
- Resilience:
  - Redis miss: DB fallback + set Redis
  - Redis error: DB fallback (with timeout), log degradation
- Multi-tenancy correctness:
  - Tenant version scoped by `tenant_id`
  - User version scoped by `user_id` (and belongs to tenant logically; enforcement via middleware/service invariants)

### Architecture (3-crate pattern)
- `core/`: traits & DTOs only (no Redis crate deps)
- `infra/`: Redis client + SQLx implementation
- `api/`: wiring only (added in separate task)

### Error handling
- No `unwrap()` / `expect()`
- Use `AppError` from `shared/error`

## Proposed Data Model / Keys

### Redis keys
- `authz:tenant:{tenant_id}:v` → integer version
- `authz:user:{user_id}:v` → integer version

### Storage in DB
- Depends on migration task `task_03.05.01_add_authz_versioning_schema.md` (dependency below).
- Expected columns:
  - `tenants.authz_version BIGINT NOT NULL DEFAULT 1`
  - `users.authz_version BIGINT NOT NULL DEFAULT 1`

## Public Interface (Core Traits)

Add a core trait (example naming; final naming should match repo conventions):

- `AuthzVersionRepository` or `AuthzVersionStore`
  - `get_tenant_version(tenant_id) -> i64`
  - `get_user_version(user_id) -> i64`
  - `bump_tenant_version(tenant_id) -> i64`
  - `bump_user_version(user_id) -> i64`

Also provide a higher-level `AuthzVersionService` if needed, but prefer keeping minimal.

## Infra Implementation

Implement `RedisAuthzVersionStore` (or `HybridAuthzVersionStore`) that composes:
- Redis client (e.g., `redis::Client` / pooled connection manager)
- SQLx PgPool repositories for tenant/user version

Rules:
- Reads:
  1) Try Redis GET
  2) If missing → query DB → SET Redis with the DB value → return
  3) If Redis errors/timeouts → query DB (with timeout) → return
- Bumps:
  - Preferred: bump in DB (transaction), get new value, then set Redis to the new value.
  - Ensure monotonic increments.

## Logging / Observability

Add structured logs at least for:
- Redis cache miss + warm-up
- Redis error + DB fallback (degraded mode)
- Version bumps (tenant/user)

No sensitive data logs.

## Specific Sub-tasks

- [ ] 1. Add core trait(s) for authz version store under `services/user_service/core/`
  - [ ] 1.1. Include methods for get/bump tenant/user versions
  - [ ] 1.2. Return types use `Result<_, AppError>`
- [ ] 2. Add infra implementations under `services/user_service/infra/`
  - [ ] 2.1. SQLx queries to read/bump versions in Postgres
  - [ ] 2.2. Redis client integration and key conventions
  - [ ] 2.3. Implement fallback behavior (miss vs error)
- [ ] 3. Add minimal unit tests (core) and infra-focused tests where feasible
  - [ ] 3.1. Validate key formatting
  - [ ] 3.2. Validate bump is monotonic
  - [ ] 3.3. Validate miss triggers warm (can be via mocked store or integration test later)
- [ ] 4. Wire config expectations (documented, not full wiring here)
  - [ ] 4.1. Document required env vars: `REDIS_URL` (and any timeouts)
  - [ ] 4.2. Provide sane defaults for local dev if policy allows

## Acceptance Criteria

- [ ] Core trait exists and is used by infra implementation
- [ ] Redis keys follow the specified format and are consistent
- [ ] Read path:
  - [ ] Redis hit returns without DB access
  - [ ] Redis miss falls back to DB and warms cache
  - [ ] Redis error falls back to DB and logs degradation
- [ ] Bump path:
  - [ ] DB increments and returns new version
  - [ ] Redis updated to new version
- [ ] No `unwrap()`/`expect()` added
- [ ] Code respects 3-crate dependency rules (`api → infra → core → shared/*`)
- [ ] Documented how version store will be used by middleware (link to task 03.05.03)

## Dependencies

- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.01_add_authz_versioning_schema.md`
- (Later integration) `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.03_add_global_authz_version_middleware.md`

## Notes / Decisions

- Selected strategy: **Hybrid versioning** (tenant-level + user-level)
- Redis chosen for performance; Postgres is source of truth.
- Fallback strategy: **Degrade gracefully** (DB fallback on Redis miss/error with timeouts).
- Access token TTL recommendation lives in design notes (middleware task); this task is storage-only.

## AI Agent Log

---
* 2026-01-02: Task created to implement Redis-backed AuthZ version store with DB fallback and cache warm-up logic.
"""
