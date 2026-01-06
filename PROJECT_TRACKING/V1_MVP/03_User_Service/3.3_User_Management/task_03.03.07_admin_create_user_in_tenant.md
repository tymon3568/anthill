# Task: Admin Create User in Tenant (Single Role)

**Task ID:** `V1_MVP/03_User_Service/3.3_User_Management/task_03.03.07_admin_create_user_in_tenant.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.3_User_Management  
**Priority:** High  
**Status:** NeedsReview  
**Assignee:** Claude  
**Created Date:** 2026-01-02  
**Last Updated:** 2026-01-06  

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
- [x] 1. Define DTOs in core
  - [x] 1.1. `AdminCreateUserReq`
  - [x] 1.2. `AdminCreateUserResp`
  - [x] 1.3. Add `utoipa::ToSchema` + validation annotations
- [x] 2. Extend core service contracts (3-crate pattern)
  - [x] 2.1. Add method to `AuthService` (or new `UserAdminService` if preferred):
    - `admin_create_user(admin_tenant_id, dto) -> Result<UserInfo, AppError>`
- [x] 3. Implement infra logic
  - [x] 3.1. Validate role exists for tenant via Casbin (policy presence)
  - [x] 3.2. Enforce "single role" rule:
    - set `users.role = role` (single string)
    - optionally ensure Casbin grouping `g(user_id, role, tenant_id)` is set and only one exists
  - [x] 3.3. Hash password with bcrypt
  - [x] 3.4. Create `users` row with `tenant_id` from admin context
  - [x] 3.5. Ensure soft-delete defaults (`deleted_at = NULL`)
- [x] 4. Implement API handler
  - [x] 4.1. Add handler in `services/user_service/api/src/admin_handlers.rs` (or new file if consistent)
  - [x] 4.2. Protect with admin-only extractor (`RequireAdmin`) and/or Casbin middleware
  - [x] 4.3. Add `#[utoipa::path]` with **unique** `operation_id`
- [x] 5. Update routing
  - [x] 5.1. Add route in `services/user_service/api/src/main.rs` under admin routes
- [x] 6. Add integration tests
  - [x] 6.1. Admin can create user in same tenant
  - [x] 6.2. Non-admin forbidden
  - [x] 6.3. Tenant isolation: admin tenant A cannot create user in tenant B (should be impossible by design)
  - [x] 6.4. Validation failures: invalid email, weak password, invalid role
- [x] 7. Update documentation
  - [x] 7.1. Update `ADMIN_ROLE_API.md` (or add `ADMIN_USER_API.md`) with examples
  - [x] 7.2. Ensure OpenAPI docs show the endpoint and schemas

## Acceptance Criteria
- [x] Admin can create a user in their tenant via `POST /api/v1/admin/users`
- [x] Created user always has `tenant_id = admin.tenant_id`
- [x] Password is hashed; response includes **no** password hash or secrets
- [x] Single-role invariant enforced (no multiple roles per user)
- [x] Role must exist in tenant (system role or custom role)
- [x] Access restricted to admins
- [x] Integration tests cover happy path + authz + tenant isolation + validation

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
- 2026-01-06 15:10: Task claimed by Claude. Dependencies verified:
  - `task_03.02.11_create_role_permission_apis.md` - Done (PR #13 merged)
  - `task_03.03.01_list_users_in_tenant.md` - Done (integration tests verified)
  - Status updated to InProgress_By_Claude.
  - Plan: (1) Define DTOs in core, (2) Add service method, (3) Implement infra logic, (4) Add API handler, (5) Add tests, (6) Update docs.
- 2026-01-06 15:45: Implementation completed by Claude:
  - Sub-task 1: Added `AdminCreateUserReq` and `AdminCreateUserResp` DTOs to `admin_dto.rs`
    - Includes validation annotations and ToSchema for OpenAPI
    - Role name regex validation (lowercase alphanumeric + underscores)
  - Sub-task 2: Extended `AuthService` trait with `admin_create_user` method in `service.rs`
  - Sub-task 3: Implemented `admin_create_user` in `AuthServiceImpl`:
    - Password hashing with bcrypt
    - Owner role protection (cannot create owner via this endpoint)
    - UUID v7 for user_id
    - Tenant isolation via admin_tenant_id parameter
    - Password strength validation via zxcvbn
  - Sub-task 4: Added `admin_create_user` handler in `admin_handlers.rs`:
    - Protected with `RequireAdmin` extractor
    - Adds Casbin grouping policy for new user
    - OpenAPI annotations with unique operation_id `admin_create_user`
  - Sub-task 5: Added route `POST /api/v1/admin/users` in `main.rs`
  - Sub-task 6: Created `admin_create_user_tests.rs` with 10 integration tests:
    - `test_admin_create_user_success` - basic creation
    - `test_admin_create_user_with_role` - custom role assignment
    - `test_admin_create_user_owner_role_forbidden` - owner protection
    - `test_non_admin_cannot_create_user` - authz check
    - `test_admin_create_user_duplicate_email` - conflict handling
    - `test_admin_create_user_invalid_email` - validation
    - `test_admin_create_user_weak_password` - password strength
    - `test_admin_create_user_tenant_isolation` - cross-tenant check
    - `test_admin_create_user_unauthorized` - no auth
    - `test_admin_create_user_invalid_role_format` - role validation
    - `test_admin_create_user_without_full_name` - optional field
  - Sub-task 7: Created `ADMIN_USER_API.md` documentation
  - Updated OpenAPI in `openapi.rs`
  - Quality gates passed: cargo check, cargo clippy
  - Status updated to NeedsReview
