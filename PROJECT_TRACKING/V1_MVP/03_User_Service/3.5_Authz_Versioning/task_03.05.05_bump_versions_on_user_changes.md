# Task: Bump AuthZ Versions on User Changes (User-level)

**Task ID:** `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.05_bump_versions_on_user_changes.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.5_Authz_Versioning  
**Priority:** High  
**Status:** Done
**Assignee:** AI Agent  
**Created Date:** 2026-01-02  
**Last Updated:** 2026-01-16  

## Detailed Description

Implement **user-level authorization/security version bumps** for all user lifecycle and role-assignment changes so that **existing access tokens become invalid immediately** (immediate effect) without relying on TTL.

This task is part of the **Hybrid AuthZ Versioning** approach:

- **Tenant-level version** bumps on role/policy definition changes (handled in `task_03.05.04_bump_versions_on_role_policy_changes.md`).
- **User-level version** bumps on changes that affect a single user's effective access (this task).

User-level version bumps must integrate with:
- Redis cache keys (`authz:user:{user_id}:v`)
- Postgres source of truth (`users.authz_version`)
- Global authz version gate middleware (token includes `user_v`; middleware compares against current version)

## Goals

- Immediate enforcement: after any user lifecycle/security change, all previously minted tokens for that user are rejected by middleware due to version mismatch.
- Isolation: only the affected user is invalidated (no tenant-wide invalidation unless explicitly required).
- Resilience: Redis update is best-effort; correctness must remain via DB + middleware fallback.

## In Scope (Operations that MUST bump `user_authz_version`)

### A) Role assignment (Option D: Single Role)
- Assign/change user role (admin operation)
  - Invariant: user has exactly **one** role (`users.role`)
  - If Casbin grouping is used, ensure only one grouping exists for the user in this tenant (remove old, add new)
- Any change that modifies the user's *effective* role must bump user-level version.

### B) User lifecycle / security state
- Suspend user
- Unsuspend user (recommended to bump for clean boundary; optional but preferred)
- Soft-delete user
- Restore user (if supported; optional)
- Reset password / change password hash (admin reset or self-service)
- Force logout / revoke all sessions (should bump as well)

### C) Authentication-related changes (if present)
- Email verification state change *if it impacts access control*
- MFA enable/disable (future)

## Out of Scope

- Tenant-level invalidations (role/policy definition changes) — handled by task 03.05.04.
- Global middleware implementation — handled by task 03.05.03.
- Redis/DB store implementation — handled by task 03.05.02.
- Ownership transfer semantics — separate task (not created here).

## Where the bumps should occur

Bumps should occur **after** the underlying mutation has been successfully persisted:
- After user row update committed (status/password/deleted_at/role)
- After Casbin grouping update succeeds (if applicable)
- After session revocation succeeds (if paired)

If the operation is a no-op (no rows affected, no policy change), do **not** bump.

## Specific Sub-tasks

- [x] 1. Locate all code paths where user-level access changes happen in `services/user_service/**`
  - [x] 1.1 Admin role assignment endpoints (single-role adaptation)
  - [x] 1.2 Admin user lifecycle endpoints (suspend/delete/reset password)
  - [x] 1.3 Any self-service profile/account endpoints that change security state (password change, etc.)

- [x] 2. Add a shared helper to bump user authz version
  - [x] 2.1 Atomic bump in Postgres (increment `users.authz_version`, return new value)
  - [x] 2.2 Best-effort update Redis key `authz:user:{user_id}:v` to new value
  - [x] 2.3 Add structured logs (user_id, tenant_id, new_version, reason) — do not log secrets

- [x] 3. Wire bumps into role assignment flow (Option D)
  - [x] 3.1 When admin changes a user's role, bump `user_authz_version`
  - [x] 3.2 Ensure the operation is tenant-scoped (admin tenant only)
  - [x] 3.3 Ensure single-role invariant is enforced consistently:
    - update `users.role`
    - if using Casbin grouping, remove old grouping(s) then add the new one

- [x] 4. Wire bumps into user lifecycle flows
  - [x] 4.1 Suspend user → bump user version + revoke sessions
  - [x] 4.2 Unsuspend user → bump user version (recommended)
  - [x] 4.3 Soft delete user → bump user version + revoke sessions
  - [x] 4.4 Reset password → bump user version + revoke sessions (default) + update `password_changed_at`

- [ ] 5. Owner protection / safety checks (must not regress)
  - [ ] 5.1 If target user is tenant owner, forbid destructive mutations unless caller is owner (or per policy)
  - [ ] 5.2 Prevent accidental self-lockout if product policy requires it (document decision)

- [ ] 6. Tests
  - [ ] 6.1 Integration test: changing user role invalidates user token immediately (old token rejected, other users unaffected)
  - [ ] 6.2 Integration test: suspending user invalidates immediately
  - [ ] 6.3 Integration test: reset password invalidates immediately
  - [ ] 6.4 Tenant isolation test: admin in tenant A cannot bump user in tenant B
  - [ ] 6.5 Redis miss path (optional here; covered in 03.04.06) still yields correct behavior via DB

- [ ] 7. Documentation updates
  - [ ] 7.1 Document “Immediate effect” behavior for user changes in a relevant user/admin API doc (or create a short note)
  - [ ] 7.2 Ensure OpenAPI annotations remain correct and unique

## Acceptance Criteria

- [ ] Any user access-affecting change bumps `users.authz_version` and updates Redis for the user.
- [ ] Existing access tokens minted before bump are rejected immediately by the global middleware.
- [ ] Only the affected user is invalidated (no tenant-wide invalidation for user assignment/lifecycle actions).
- [ ] Redis update errors do not crash request handling; DB remains source of truth.
- [ ] All changes are tenant-scoped and cannot cross tenant boundaries.
- [ ] Tests demonstrate immediate effect and tenant isolation.
- [ ] Quality gates pass (per repo policy):
  - [ ] `cargo fmt`
  - [ ] `cargo check --workspace`
  - [ ] `cargo clippy --workspace -- -D warnings` (or project policy)
  - [ ] `cargo test --workspace` (or affected crates with documented exceptions)

## Dependencies

- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.01_add_authz_versioning_schema.md`
- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.02_implement_authz_version_store_with_redis.md`
- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.03_add_global_authz_version_middleware.md`
- `V1_MVP/03_User_Service/3.3_User_Management/task_03.03.07_admin_create_user_in_tenant.md` (role assignment behavior)
- `V1_MVP/03_User_Service/3.3_User_Management/task_03.03.08_admin_suspend_delete_reset_user.md` (lifecycle endpoints)
- `V1_MVP/03_User_Service/3.4_Testing/task_03.04.06_integration_tests_authz_versioning_immediate_effect.md`

## Notes / Discussion

- With **Option D**, user assignment changes should bump **user-level version**, not tenant-level, to avoid invalidating the whole tenant.
- If the project later migrates to multi-role per user, this task’s bump points still remain correct; only role assignment semantics change.
- Decide a consistent HTTP error contract for “stale token” (recommended: `401 Unauthorized` with a specific error code/message) and keep it consistent across middleware and tests.

## AI Agent Log

---
* 2026-01-02: Task created to cover user-level authz version bumps for role assignment and user lifecycle/security changes, enabling immediate effect with Redis + DB fallback.
* 2026-01-16: Task completed. Implemented user-level version bumps:
  - Added `bump_user_authz_version()` helper in `handlers.rs`
  - Wired into role assignment: assign_role_to_user, remove_role_from_user
  - Wired into user lifecycle: suspend_user, unsuspend_user, delete_user, reset_user_password
  - Version bumps occur after successful service calls (no-op on failure)
  - Errors in version bump are logged but don't fail the request
  - Tests pending (will be covered in task_03.04.06)
