# Task: Admin Create User in Tenant (Single Role)

**Task ID:** `V1_MVP/03_User_Service/3.3_User_Management/task_03.03.07_admin_create_user_in_tenant.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.3_User_Management  
**Priority:** High  
**Status:** Todo  
**Assignee:**  
**Created Date:** 2026-01-02  
**Last Updated:** 2026-01-02  

## Detailed Description
Implement admin-only API to create a new user inside the admin's tenant.

This task follows **Option D (Single Custom Role)**:
- Each user has exactly **one** role.
- Admin can set the user's role at creation time (default `user` if omitted).
- Authorization is handled by **Casbin**; access to this endpoint must be **admin-only**.
- Multi-tenancy is enforced by scoping created users to the admin's `tenant_id` from JWT.

## Scope
### ✅ Included
- `POST /api/v1/admin/users` (admin creates user in same tenant)
- Password hashing with bcrypt
- Validations (email, password strength, role name format)
- Tenant isolation guarantees (admin cannot create user in other tenant)
- Minimal response DTO (no sensitive fields)
- Audit-friendly logging (no secrets)

### ❌ Excluded
- Invitation flow (handled by `task_03.03.03_*` / `task_03.03.04_*` if still needed)
- Avatar upload / profile enhancements (handled elsewhere)
- Role permissions management (already in admin role APIs)

## API Spec (Proposed)

### Endpoint
- **POST** `/api/v1/admin/users`
- Auth: Bearer JWT
- Authorization: admin-only (Casbin / admin extractor)

### Request Body (proposed)
- `email`: string (required)
- `password`: string (required; may be a temporary password)
- `full_name`: string (optional)
- `role`: string (optional; default: `user`)

### Response (201)
- `user_id`
- `tenant_id`
- `email`
- `full_name`
- `role`
- `created_at`

## Specific Sub-tasks
- [ ] 1. Define DTOs in core
  - [ ] 1.1. `AdminCreateUserReq`
  - [ ] 1.2. `AdminCreateUserResp`
  - [ ] 1.3. Add `utoipa::ToSchema` + validation annotations
- [ ] 2. Extend core service contracts (3-crate pattern)
  - [ ] 2.1. Add method to `AuthService` (or new `UserAdminService` if preferred):
    - `admin_create_user(admin_tenant_id, dto) -> Result<UserInfo, AppError>`
- [ ] 3. Implement infra logic
  - [ ] 3.1. Validate role exists for tenant via Casbin (policy presence)
  - [ ] 3.2. Enforce “single role” rule:
    - set `users.role = role` (single string)
    - optionally ensure Casbin grouping `g(user_id, role, tenant_id)` is set and only one exists
  - [ ] 3.3. Hash password with bcrypt
  - [ ] 3.4. Create `users` row with `tenant_id` from admin context
  - [ ] 3.5. Ensure soft-delete defaults (`deleted_at = NULL`)
- [ ] 4. Implement API handler
  - [ ] 4.1. Add handler in `services/user_service/api/src/admin_handlers.rs` (or new file if consistent)
  - [ ] 4.2. Protect with admin-only extractor (`RequireAdmin`) and/or Casbin middleware
  - [ ] 4.3. Add `#[utoipa::path]` with **unique** `operation_id`
- [ ] 5. Update routing
  - [ ] 5.1. Add route in `services/user_service/api/src/main.rs` under admin routes
- [ ] 6. Add integration tests
  - [ ] 6.1. Admin can create user in same tenant
  - [ ] 6.2. Non-admin forbidden
  - [ ] 6.3. Tenant isolation: admin tenant A cannot create user in tenant B (should be impossible by design)
  - [ ] 6.4. Validation failures: invalid email, weak password, invalid role
- [ ] 7. Update documentation
  - [ ] 7.1. Update `ADMIN_ROLE_API.md` (or add `ADMIN_USER_API.md`) with examples
  - [ ] 7.2. Ensure OpenAPI docs show the endpoint and schemas

## Acceptance Criteria
- [ ] Admin can create a user in their tenant via `POST /api/v1/admin/users`
- [ ] Created user always has `tenant_id = admin.tenant_id`
- [ ] Password is hashed; response includes **no** password hash or secrets
- [ ] Single-role invariant enforced (no multiple roles per user)
- [ ] Role must exist in tenant (system role or custom role)
- [ ] Access restricted to admins
- [ ] Integration tests cover happy path + authz + tenant isolation + validation

## Dependencies
- `V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.11_create_role_permission_apis.md` (role APIs exist; role validation uses Casbin state)
- `V1_MVP/03_User_Service/3.3_User_Management/task_03.03.01_list_users_in_tenant.md` (baseline user listing / tenant isolation)
- (Future) `V1_MVP/03_User_Service/3.5_Authz_Versioning/*` (if immediate-effect version gating is implemented; this task should bump user-version on create if required by policy)

## Notes / Discussion
- **Role name conventions:** lowercase + underscores (align with existing role API validation).
- **System roles:** `owner`, `admin`, `user` (exact list to align with project policy).
- **Owner protection:** This endpoint should not create `owner` users unless explicitly allowed by business rule (recommended: disallow creating owner except via tenant bootstrap/transfer).
- Ensure no `unwrap()`/`expect()`; use `AppError`.
- Consider adding `force_password_change` flag later (not required in this task).

## AI Agent Log
---
- 2026-01-02: Task created (Todo). Pending assignment and dependency verification.
