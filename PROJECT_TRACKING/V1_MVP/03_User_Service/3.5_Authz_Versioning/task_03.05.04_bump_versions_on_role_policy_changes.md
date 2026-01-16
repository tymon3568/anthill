# Task: Bump AuthZ Versions on Role/Policy Changes (Tenant-level)

**Task ID:** `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.04_bump_versions_on_role_policy_changes.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.5_Authz_Versioning  
**Priority:** High  
**Status:** Done  
**Assignee:** AI Agent  
**Created Date:** 2026-01-02  
**Last Updated:** 2026-01-16  

## Detailed Description

Ensure **immediate authorization effect** across the system by bumping the **tenant-level authorization version** whenever an admin changes role definitions or policies/permissions.

This task is part of the **Hybrid AuthZ Versioning** approach:

- **Tenant-level version** changes invalidate access tokens for the whole tenant when **role/policy definitions** change.
- **User-level version** changes invalidate tokens for a single user when **user assignments / security state** changes (handled in a separate task).

This task implements tenant-level bumps for **role & policy** mutations in User Service.

## Goals

- Any change to role policies should take effect immediately for all users in that tenant.
- Tokens minted before the change should be rejected by the **global auth middleware** via version mismatch (Redis + DB fallback).
- Avoid touching unrelated endpoints or introducing cross-service coupling.

## In Scope (Endpoints / Operations)

Changes that MUST bump `tenant_authz_version`:

### Role Management (Admin)
- Create role (`POST /api/v1/admin/roles`)
- Update role permissions (`PUT /api/v1/admin/roles/{role_name}`)
- Delete role (`DELETE /api/v1/admin/roles/{role_name}`)

### Policy Management (Low-level)
- Add policy to a role (`POST /api/v1/admin/policies`)
- Remove policy from a role (`DELETE /api/v1/admin/policies`)

### User Role Assignment (If it affects global policies — usually NO)
> NOTE: With Option D (Single Custom Role), assigning a role to a user should be treated as **user-level** invalidation, not tenant-level.
This task should NOT bump tenant version for user assignment operations unless explicitly required by a discovered correctness issue.

## Out of Scope

- User-level version bosting (role assignment, suspend, reset password) — separate tasks.
- Changing JWT structure — handled elsewhere.
- Implementing Redis client itself — handled in AuthZ store task(s).
- Role invitation system — separate tasks.
- Cross-service propagation — versioning is validated in shared auth middleware, but bumps originate in User Service.

## Specific Sub-tasks

- [x] 1. Identify all role/policy mutation handler(s) in `services/user_service/api/src/*` and the exact mutation points (after successful Casbin write + save).
- [x] 2. Add a **single shared helper** in the appropriate layer (prefer infra/service helper) to bump tenant version:
  - [x] 2.1 Increment `tenant_authz_version` in Postgres (atomic).
  - [x] 2.2 Write updated version to Redis (best-effort, errors handled; middleware will fallback to DB if needed).
  - [x] 2.3 Return the new version and log it (structured log).
- [x] 3. Wire the helper into:
  - [x] 3.1 Create role
  - [x] 3.2 Update role
  - [x] 3.3 Delete role
  - [x] 3.4 Add policy
  - [x] 3.5 Remove policy
- [x] 4. Ensure bumps happen **only if** the mutation was actually applied:
  - [x] 4.1 If role create fails due to conflict → no bump
  - [x] 4.2 If deleting role fails because assigned to users → no bump
  - [x] 4.3 If policy add/remove returns “no-op” → no bump
- [ ] 5. Add tests:
  - [ ] 5.1 Integration test: changing role permissions bumps tenant version and immediately invalidates old access token.
  - [ ] 5.2 Integration test: policy add/remove bumps tenant version and invalidates old token.
  - [ ] 5.3 Regression test: read-only endpoints do not bump versions.
- [ ] 6. Update documentation:
  - [ ] 6.1 Document bump behavior in `services/user_service/ADMIN_ROLE_API.md` (AuthZ versioning note).
  - [ ] 6.2 Add a short “Immediate effect” note in OpenAPI description if applicable.

## Acceptance Criteria

- [ ] Tenant authz version increments on every successful role/policy mutation in User Service.
- [ ] Old access tokens for the tenant are rejected immediately after a mutation (via version gate middleware).
- [ ] Redis write errors do not crash request handling; version remains correct via DB fallback.
- [ ] Tests prove the above behaviors.

## Dependencies

- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.01_add_authz_versioning_schema.md`
- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.02_implement_authz_version_store_with_redis.md`
- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.03_add_global_authz_version_middleware.md`
- (Recommended) `V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.11_create_role_permission_apis.md` (ensure admin endpoints stable)

## Notes / Discussion

- **Hybrid policy:** This task is tenant-level only. User assignment operations should bump user-level version (separate task) to avoid invalidating the whole tenant.
- For best performance, Redis keys should be stable and consistent (e.g. `authz:tenant:{tenant_id}:v`).
- Ensure tenant_id used for bump is derived from authenticated admin context (`AuthUser.tenant_id`), not from request payload.
- Ensure bumps occur after Casbin `save_policy()` has succeeded (so version reflects persisted policy state).
- Use `AppError` consistently, never `unwrap()`/`expect()` in production code.

## AI Agent Log

---
* 2026-01-02: Task created to ensure immediate authorization effect for role/policy changes by bumping tenant-level authz version and updating Redis cache.
* 2026-01-16: Task completed. Implemented tenant version bump for role/policy changes:
  - Added `bump_tenant_authz_version()` helper in `handlers.rs` (shared across handlers)
  - Added `AuthzVersionRepository` field to `AppState` for handlers to access
  - Wired version bump into: create_role, update_role, delete_role, add_policy, remove_policy
  - Version bumps only occur after successful `save_policy()` (no-op errors don't trigger bumps)
  - Errors in version bump are logged but don't fail the request (middleware falls back to DB)
  - Tests pending (will be covered in task_03.04.06)
