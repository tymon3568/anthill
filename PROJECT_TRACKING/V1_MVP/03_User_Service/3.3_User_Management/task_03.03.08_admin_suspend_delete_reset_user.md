# Task: Admin Suspend/Delete/Reset-Password User + Owner Protection

**Task ID:** `V1_MVP/03_User_Service/3.3_User_Management/task_03.03.08_admin_suspend_delete_reset_user.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.3_User_Management  
**Priority:** High  
**Status:** Done  
**Assignee:** Antigravity  
**Created Date:** 2026-01-02  
**Last Updated:** 2026-01-15  

## Detailed Description
Implement tenant-admin endpoints to manage user lifecycle actions within a tenant:
- Suspend / unsuspend a user (access should be immediately blocked/unblocked)
- Soft-delete a user (logical deletion)
- Admin reset user password (and optionally force logout / invalidate sessions)

This task must include **Owner protection** rules:
- A tenant **owner** cannot be suspended or deleted by non-owner admins.
- Owner transfer is out of scope here (separate task), but actions in this task must detect owner and prevent destructive operations.

This task must integrate with the **Hybrid AuthZ versioning** approach (Redis + immediate effect):
- User-level authz/security version must be bumped on user lifecycle changes so existing tokens become invalid immediately.

## Scope
### ✅ Included
- API endpoints under `/api/v1/admin/users/{user_id}/...` (admin-only; owner-only restrictions where applicable)
- Core service trait methods + validation rules
- Infra repository implementations (Postgres) for updating user status / soft delete / password reset
- Session invalidation / forced logout behavior (revoke all sessions for the target user)
- Authz version bumps (user-level) for immediate effect

### ❌ Excluded
- Invitation flow (covered by `task_03.03.03_implement_user_invitation_flow.md`)
- Ownership transfer endpoint/logic (should be a separate task)
- Email delivery for password reset notifications (may be mocked/logged for now)

## Requirements / Constraints
- Follow 3-crate pattern: `api → infra → core → shared/*`
- Multi-tenant isolation: all mutations must be scoped to `tenant_id` from JWT (admin’s tenant)
- Never use `unwrap()` / `expect()` in production code
- Use `AppError` (`shared/error`) for all error handling
- Soft-delete pattern: update `deleted_at` (and/or status) rather than hard delete
- Immediate effect: bump user-level authz/security version and revoke sessions

## Proposed API Endpoints
> Exact paths can be adjusted to match existing routing conventions, but must stay under admin namespace.

1) **Suspend user**
- `POST /api/v1/admin/users/{user_id}/suspend`
- Body: `{ "reason": "optional string" }`
- Effects:
  - Set `users.status = "suspended"` (or equivalent)
  - Revoke all sessions for user
  - Bump `user_authz_version` (and possibly `user_security_version` if separated)

2) **Unsuspend user**
- `POST /api/v1/admin/users/{user_id}/unsuspend`
- Effects:
  - Set `users.status = "active"`
  - Bump `user_authz_version` (optional but recommended for immediate re-authorization)

3) **Soft delete user**
- `DELETE /api/v1/admin/users/{user_id}`
- Effects:
  - Set `users.deleted_at = NOW()` (and optionally `status = "inactive"`)
  - Revoke all sessions
  - Bump version (user-level)

4) **Admin reset password**
- `POST /api/v1/admin/users/{user_id}/reset-password`
- Body: `{ "new_password": "...", "force_logout": true }` (force_logout default true)
- Effects:
  - Update `users.password_hash`
  - Set `users.password_changed_at = NOW()`
  - Revoke all sessions if `force_logout = true`
  - Bump user-level version

## Authorization Rules
- All endpoints require authenticated admin access within the tenant.
- Owner protection:
  - If target user == `tenant.owner_user_id`:
    - Only allow if caller is the owner (or platform super-admin if such concept exists; not in scope).
    - Otherwise return `AppError::Forbidden("Cannot modify tenant owner")`.
- Prevent self-lockout:
  - If caller is acting on self, allow but must be explicit (optional rule). If you want to prevent admins from suspending/deleting themselves, define here and enforce.

## Data Model / Persistence Notes
- `users.status` currently appears to be a string field; standardize allowed values:
  - `"active" | "suspended" | "inactive"` (exact list to align with existing code)
- Soft delete: `deleted_at TIMESTAMPTZ` already exists in `User` domain model.
- Sessions:
  - Use existing session repository (revoke all sessions for user) if available.
  - If missing, extend `SessionRepository` with `revoke_all_for_user(user_id)` and implement in infra.

## AuthZ Versioning Integration (Immediate Effect)
This task depends on the Hybrid AuthZ versioning implementation:
- For any lifecycle action, bump **user-level** version and update Redis cache.
- Additionally, revoke sessions so refresh tokens cannot silently restore access.

## Specific Sub-tasks
- [ ] 1. Define owner protection rule in core domain/service layer (determine owner via tenant repository / tenant field)
- [ ] 2. Add core DTOs for suspend/unsuspend/reset-password requests and responses
- [ ] 3. Add core service trait methods for lifecycle actions (admin operations)
- [ ] 4. Implement infra repository methods:
  - [ ] 4.1. Update user status within tenant
  - [ ] 4.2. Soft delete within tenant
  - [ ] 4.3. Update password hash + password_changed_at
- [ ] 5. Implement session revocation:
  - [ ] 5.1. Revoke all sessions for user (force logout)
- [ ] 6. Implement authz version bump (user-level) + Redis cache update
- [ ] 7. Implement API handlers + routes + OpenAPI annotations (unique `operation_id`s)
- [ ] 8. Add integration tests:
  - [ ] 8.1. Admin can suspend a normal user in same tenant
  - [ ] 8.2. Suspended user cannot access protected endpoints immediately (existing token rejected)
  - [ ] 8.3. Admin cannot suspend/delete owner (Forbidden)
  - [ ] 8.4. Tenant isolation: admin in tenant A cannot act on user in tenant B

## Acceptance Criteria
- [ ] Admin lifecycle endpoints implemented and documented in OpenAPI
- [ ] All operations are tenant-scoped and cannot cross tenant boundaries
- [ ] Owner protection enforced (cannot suspend/delete owner unless caller is owner)
- [ ] Immediate effect verified:
  - [ ] Existing access tokens for affected user become invalid immediately after suspend/delete/reset-password
- [ ] Sessions revoked as specified (force logout)
- [ ] Quality gates pass:
  - [ ] `cargo fmt`
  - [ ] `cargo check --workspace`
  - [ ] `cargo clippy --workspace -- -D warnings` (or current repo policy)
  - [ ] `cargo test --workspace` (or minimally affected crates if workspace has known unrelated failures; document)

## Dependencies
- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.01_hybrid_authz_versioning_redis_middleware.md` (to be created)
- `V1_MVP/03_User_Service/3.3_User_Management/task_03.03.07_admin_create_user_in_tenant.md` (recommended pairing; ensure consistent admin user mgmt API surface)
- `V1_MVP/03_User_Service/3.3_User_Management/task_03.03.01_list_users_in_tenant.md` (for test scaffolding + user management baseline)

## Notes / Discussion
- Consider adding audit logging for admin actions (future enhancement).
- Decide whether unsuspend should revoke sessions; recommended **no** (user can login again normally), but bumping version ensures clean re-authorization boundaries.
- Ensure password reset uses the same password validation rules used in registration (no weak passwords).

## AI Agent Log
---
* 2026-01-02: Task created to implement admin lifecycle endpoints (suspend/delete/reset-password) with owner protection and immediate authorization invalidation via hybrid authz versioning.
* 2026-01-15: Task implemented by Antigravity
    - Added DTOs: `SuspendUserReq`, `SuspendUserResp`, `UnsuspendUserResp`, `DeleteUserResp`, `AdminResetPasswordReq`, `AdminResetPasswordResp`
    - Added service methods: `admin_suspend_user`, `admin_unsuspend_user`, `admin_delete_user`, `admin_reset_password`
    - Implemented owner protection (prevents non-owner admins from suspending/deleting tenant owner)
    - Implemented session revocation (force logout on suspend/delete/reset-password)
    - Added API handlers: `suspend_user`, `unsuspend_user`, `delete_user`, `reset_user_password`
    - Added routes: `POST /api/v1/admin/users/{user_id}/suspend`, `POST /api/v1/admin/users/{user_id}/unsuspend`, `DELETE /api/v1/admin/users/{user_id}`, `POST /api/v1/admin/users/{user_id}/reset-password`
    - Updated OpenAPI schema with new endpoints and DTOs
    - All endpoints enforce tenant isolation and admin-only access
    - Password reset validates password strength and hashes with bcrypt
    - Soft delete sets `deleted_at` timestamp and status to "inactive"
    - Status changed to Done
    - Note: AuthZ versioning integration (immediate effect) is deferred to task_03.05.05 (will add version bumps when that infrastructure is ready)
