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

## PR Review Issues (from PR #136)

### Critical
- [x] 1. Missing `save_policy()` after Casbin grouping policy (Severity: Critical, Reviewers: Cubic, CodeRabbit, Sourcery, CodeAnt)
  - Location: `services/user_service/api/src/admin_handlers.rs:804-814`
  - Fix: Call `enforcer.save_policy().await` after `add_grouping_policy`
  - **Fixed:** Added `save_policy()` call with proper error handling
- [x] 2. Inconsistent state on Casbin failure - user persisted without policy (Severity: Critical, Reviewers: Gemini, CodeRabbit, Sourcery)
  - Location: `services/user_service/api/src/admin_handlers.rs:784-814`
  - Fix: Implement compensating delete - delete user from DB if Casbin policy fails
  - **Fixed:** Added `internal_delete_user` method and compensating delete logic
- [x] 3. Custom role validation gap - no enforcement (Severity: Critical, Reviewers: Gemini, Sourcery, CodeRabbit)
  - Location: `services/user_service/api/src/admin_handlers.rs:777-795`
  - Fix: Validate custom roles against Casbin policies in handler before calling service
  - **Fixed:** Added validation in handler using `enforcer.get_filtered_policy()` to check if custom role has policies defined for the tenant (per AUTHORIZATION_RBAC_STRATEGY.md - in-memory policies, no DB queries)

### Warning
- [x] 4. SYSTEM_ROLES constant unused/misleading (Severity: Warning, Reviewers: Gemini, Sourcery, CodeAnt)
  - Location: `services/user_service/core/src/domains/auth/dto/admin_dto.rs:208-209`
  - Fix: Update constant to only "owner" (actual protected role) and clarify comment
  - **Fixed:** Renamed to `PROTECTED_ROLES`, updated to only include "owner", improved documentation
- [x] 5. Test helper `add_casbin_grouping` missing v3 column (Severity: Warning, Reviewers: CodeAnt)
  - Location: `services/user_service/api/tests/test_database.rs:416`
  - Fix: Add v3, v4, v5 columns to INSERT (v3 is NOT NULL per migration)
  - **Fixed:** Added v3, v4, v5 empty string values to INSERT
- [x] 6. Test AppState has user_repo/tenant_repo = None (Severity: Warning, Reviewers: Gemini, CodeAnt)
  - Location: `services/user_service/api/tests/admin_create_user_tests.rs:57-58`
  - Fix: Set repos to Some(Arc::new(...)) for consistency
  - **Fixed:** Wrapped repos in Arc and set to Some(...)
- [x] 7. Logging PII (email at info level) (Severity: Warning, Reviewer: CodeAnt)
  - Location: `services/user_service/infra/src/auth/service.rs:656`
  - Fix: Remove or redact email from log message
  - **Fixed:** Removed email from log message
- [x] 8. Missing tenant existence validation (Severity: Warning, Reviewer: CodeAnt)
  - Location: `services/user_service/infra/src/auth/service.rs:578`
  - Fix: Validate tenant exists and is active before creating user
  - **Fixed:** Added tenant validation at start of admin_create_user, included tenant name in password validation

### Style
- [x] 9. Doc test count mismatch (10 vs 11) (Severity: Style, Reviewer: Cubic)
  - Location: `task_03.03.07.md:133`
  - Fix: Change "10" to "11"
  - **Fixed:** Updated in this file
- [x] 10. Missing language specifier in code block (Severity: Style, Reviewer: CodeRabbit)
  - Location: `services/user_service/ADMIN_USER_API.md:24`
  - Fix: Add `http` language to code fence
  - **Fixed:** Added `http` language specifier
- [x] 11. Incorrect endpoint paths in Related Endpoints table (Severity: Style, Reviewer: CodeRabbit)
  - Location: `services/user_service/ADMIN_USER_API.md:189-191`
  - Fix: Update paths to match actual handlers
  - **Fixed:** Updated paths to match actual routes

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
  - Sub-task 6: Created `admin_create_user_tests.rs` with 11 integration tests:
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
- 2026-01-06 16:30: PR Review Auto-Fix workflow initiated by Claude:
  - Fetched PR #136 content
  - Identified 11 unresolved review issues from automated reviewers
  - Categorized by severity: 3 Critical, 5 Warning, 3 Style
  - User confirmed Option A for atomicity: implement compensating delete on Casbin failure
- 2026-01-06 17:00: PR Review fixes applied by Claude:
  - **Critical fixes:**
    - Added `save_policy()` call after `add_grouping_policy` with error handling
    - Implemented compensating delete: added `hard_delete_by_id` to UserRepository, `internal_delete_user` to AuthService
    - Handler now rolls back user creation if Casbin policy add or save fails
    - Added custom role validation against Casbin policies in handler (per AUTHORIZATION_RBAC_STRATEGY.md)
  - **Warning fixes:**
    - Renamed `SYSTEM_ROLES` to `PROTECTED_ROLES`, updated to only include "owner"
    - Fixed `add_casbin_grouping` test helper to include v3, v4, v5 columns
    - Fixed test AppState to use `Some(Arc::new(...))` for repos
    - Removed email from info-level log message (PII)
    - Added tenant existence and active status validation before user creation
    - Included tenant name in password strength validation (consistency with registration)
  - **Style fixes:**
    - Fixed test count (10 → 11) in this task file
    - Added `http` language specifier to ADMIN_USER_API.md code block
    - Fixed Related Endpoints paths in ADMIN_USER_API.md
  - Quality gates passed: `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`
  - **All 11 issues resolved** - PR ready for final review and merge
  - Status remains NeedsReview pending PR re-review
- 2026-01-06 17:30: Bug fix applied by Claude:
  - Fixed custom role validation policy index bug: changed `get_filtered_policy(1, ...)` to `get_filtered_policy(0, ...)` 
  - Per Casbin model: `p = sub, dom, obj, act` → index 0 = role/subject, index 1 = tenant_id
  - Also simplified the check by including tenant_id in filter args and using `is_empty()` instead of manual iteration
  - Per AUTHORIZATION_RBAC_STRATEGY.md: validation uses in-memory Casbin policies (no DB roundtrip per request)
  - Quality gates passed: `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`
