# Task: Add Global AuthZ Version Middleware (Redis, Immediate Effect)

**Task ID:** `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.03_add_global_authz_version_middleware.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.5_Authz_Versioning  
**Priority:** High  
**Status:** Todo  
**Assignee:**  

## Goal
Implement a **global middleware** for all protected routes in `user_service` that enforces **immediate authorization changes** using **Hybrid AuthZ Versioning** (tenant-level + user-level) backed by **Redis**, with safe fallback behavior.

This middleware must run **before** Casbin authorization checks to fail fast.

## Background / Context
We use Casbin for RBAC, but JWT tokens are snapshots. To make role/permission changes effective immediately, we introduce:
- `tenant_authz_version` (bumped when role/policy changes)
- `user_authz_version` (bumped when a user’s role/status/security changes)

JWTs include both versions at issuance time. Middleware validates versions against current values in Redis (fallback to DB) and rejects stale tokens immediately.

## Requirements (Non-negotiables)
- Must be applied as a **global middleware** to all protected routes in `services/user_service/api`.
- Must use **Redis** as the primary lookup store.
- Must implement **degrade gracefully** behavior:
  - Redis **cache miss** → fallback to DB → warm Redis.
  - Redis **unavailable/timeout** → fallback to DB with bounded timeout (do not hang request).
- Must enforce **tenant isolation** consistently (versions keyed by `tenant_id` / `user_id`; no cross-tenant leakage).
- Must not add new cross-crate dependency violations (respect 3-crate pattern: `api → infra → core → shared/*`).
- Must use `AppError` (no `unwrap()` or `expect()` in production code).

## Proposed Design
### Inputs
- `Authorization: Bearer <jwt>` header
- JWT claims must include:
  - `sub` (user_id)
  - `tenant_id`
  - `tenant_v` (tenant_authz_version at issue time)
  - `user_v` (user_authz_version at issue time)
  - `exp`
  - `token_type = access`

### Lookups
- Redis keys:
  - `authz:tenant:{tenant_id}:v` → integer
  - `authz:user:{user_id}:v` → integer
- Prefer `MGET`/pipelining to reduce RTT.

### Failure Modes
- Version mismatch:
  - Return `401 Unauthorized` with standardized error code (via `AppError`) indicating token is stale.
- Missing claim fields:
  - Return `401 Unauthorized`.
- Timeout / Redis error:
  - Attempt DB fallback.
  - If DB fallback fails, return `503 Service Unavailable` (or mapped `AppError::InternalError` if policy requires).

### Placement in Router
- Apply middleware to the “protected routes” router group (before Casbin layer).
- Must also cover admin + profile endpoints (anything that requires auth).

## Specific Sub-tasks
- [ ] 1. Define middleware interface & state wiring
  - [ ] 1.1. Add a new middleware module in `user_service_api` (or `shared/auth` if designed reusable across services—confirm by architect).
  - [ ] 1.2. Ensure middleware gets access to Redis client / AuthzVersionStore via `Extension` state.
- [ ] 2. Implement JWT parsing & claim validation
  - [ ] 2.1. Validate `token_type == access`
  - [ ] 2.2. Extract `sub`, `tenant_id`, `tenant_v`, `user_v`
- [ ] 3. Implement Redis lookup + DB fallback + cache warm
  - [ ] 3.1. Redis MGET both keys
  - [ ] 3.2. If missing, fetch from DB and set Redis with sane TTL
  - [ ] 3.3. If Redis errors, fallback DB with timeout
- [ ] 4. Enforce mismatch behavior
  - [ ] 4.1. If `tenant_v` mismatch → reject
  - [ ] 4.2. If `user_v` mismatch → reject
- [ ] 5. Integrate middleware globally into `user_service/api/src/main.rs`
  - [ ] 5.1. Ensure ordering: JWT validation → authz version gate → Casbin layer/extractors
- [ ] 6. Observability
  - [ ] 6.1. Add structured logs for mismatch and fallback events (do not log secrets)
  - [ ] 6.2. Add metrics hooks if project has a metrics crate (optional)
- [ ] 7. Documentation
  - [ ] 7.1. Document behavior in `services/user_service/ADMIN_ROLE_API.md` and/or auth docs
- [ ] 8. Tests (integration)
  - [ ] 8.1. Add tests that show immediate rejection after bumping versions (see testing task dependency)

## Acceptance Criteria
- [ ] Protected endpoints reject stale tokens immediately after:
  - [ ] tenant-level bump (role/policy change)
  - [ ] user-level bump (role assignment/status change)
- [ ] Redis miss warms cache and request proceeds correctly (when versions match).
- [ ] Redis outage triggers DB fallback (bounded) and does not hang requests.
- [ ] Local quality gates pass for affected crates:
  - [ ] `cargo fmt`
  - [ ] `cargo check --workspace`
  - [ ] `cargo clippy --workspace -- -D warnings`
  - [ ] `cargo test --workspace` (or at least user_service packages + relevant shared crates)

## Dependencies
- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.01_add_authz_versioning_schema.md` (must exist; DB schema for versions)
- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.02_implement_authz_version_store_with_redis.md` (must exist; Redis/DB store abstraction)
- `V1_MVP/03_User_Service/3.4_Testing/task_03.04.xx_integration_tests_authz_versioning_immediate_effect.md` (to be created/linked)

## Notes / Decisions
- Versioning mode: **Hybrid** (tenant-level + user-level).
- Redis is primary store; DB is source of truth.
- Middleware must be applied broadly (“middleware chung”), not endpoint-by-endpoint.

## AI Agent Log
---
* (To be filled when claimed and executed. Must include dependency verification and GitHub flow branch name.)
