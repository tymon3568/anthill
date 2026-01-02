# Task: Register Bootstrap Owner and Default Role Assignment (Option D)

**Task ID:** `V1_MVP/03_User_Service/3.3_User_Management/task_03.03.06_register_bootstrap_owner_and_default_role.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.3_User_Management  
**Priority:** High  
**Status:** Todo  
**Assignee:**  
**Created Date:** 2026-01-02  
**Last Updated:** 2026-01-02  

## Detailed Description
Implement the agreed registration bootstrap rules for Anthill multi-tenant auth:

1) **When `register()` creates a NEW tenant** → the registering user becomes the **tenant owner** immediately and is assigned the **system role `owner`** (single role per user, Option D).  
2) **When `register()` joins an EXISTING tenant** → the registering user is assigned the default **system role `user`**.

This task also introduces and enforces the invariant that **each user has exactly one effective role** (`users.role`) while still using Casbin for authorization policies. System roles (`owner`, `admin`, `user`) must be protected from deletion/modification per existing admin role APIs.

> Note: This task is focused on the **registration bootstrap** only. Admin CRUD user creation and authz-versioning middleware are handled in separate tasks.

## Scope
### ✅ Included
- Update `register()` logic to assign:
  - `owner` when creating a new tenant
  - `user` when joining an existing tenant
- Add/confirm **tenant ownership** persistence (`tenants.owner_user_id` or equivalent) and ensure correct tenant-user relationship.
- Ensure **Casbin grouping policy** is consistent with `users.role` for bootstrapped users.
- Protect owner from accidental removal/lock in registration logic (basic guardrails where relevant).
- Update OpenAPI / docs to reflect owner bootstrap behavior.

### ❌ Excluded
- Admin endpoints to create/update/suspend/delete users
- Authz versioning (tenant/user version) and Redis version-gate middleware
- Invitation/accept-invite flow
- Email verification flow changes

## Requirements / Rules
- **Multi-tenancy:** All operations must remain tenant-scoped; no cross-tenant access.
- **Option D:** Single role per user. `users.role` is the single effective role.
- **System Roles:** Reserve and protect: `owner`, `admin`, `user`.
- **Owner semantics:** Owner is different from admin:
  - Owner is the tenant’s owner-of-record (used for privileged tenant-level actions).
  - Owner can also be authorized broadly (typically superset of admin) via Casbin policies.
- **No unwrap/expect** in production code.
- Use `AppError` for error handling.

## Proposed Behavior (Acceptance-level Specification)

### Case A: Register creates a new tenant
- Create tenant
- Create user
- Set tenant ownership:
  - `tenants.owner_user_id = created_user.user_id` (or equivalent mechanism)
- Set user role:
  - `users.role = "owner"`
- Ensure Casbin grouping:
  - Add grouping rule: `[user_id, "owner", tenant_id]`
- Issue JWT and session as normal

### Case B: Register joins existing tenant
- Use existing tenant (resolved by slug or identifier as currently implemented in `register()`)
- Create user
- Set user role:
  - `users.role = "user"`
- Ensure Casbin grouping:
  - Add grouping rule: `[user_id, "user", tenant_id]`
- Issue JWT and session as normal

## Specific Sub-tasks
- [ ] 1. Add/confirm tenant ownership field
  - [ ] 1.1 Confirm schema contains `tenants.owner_user_id` (or create a migration if missing)
  - [ ] 1.2 Add repository method(s) to update owner (`set_owner`) in a tenant-scoped way
- [ ] 2. Update `register()` flow in auth service
  - [ ] 2.1 Detect “new tenant created” vs “existing tenant joined”
  - [ ] 2.2 Set `users.role` to `owner` or `user` accordingly
  - [ ] 2.3 Persist tenant owner for new tenant scenario
- [ ] 3. Ensure Casbin consistency on bootstrap
  - [ ] 3.1 Add grouping policy assignment for the bootstrapped role (owner/user)
  - [ ] 3.2 Ensure default policies for `owner` exist (seed if missing; otherwise document dependency)
- [ ] 4. Update docs and OpenAPI
  - [ ] 4.1 Document the bootstrap rule in user-service docs (and/or shared OpenAPI)
  - [ ] 4.2 Verify operation_id uniqueness if new docs/endpoints are touched
- [ ] 5. Add tests (minimal but meaningful)
  - [ ] 5.1 Integration test: register new tenant → response user role is `owner`, tenant owner set
  - [ ] 5.2 Integration test: register into existing tenant → response user role is `user`
  - [ ] 5.3 Tenant isolation sanity check: cannot set owner across tenants

## Acceptance Criteria
- [ ] Registering user that creates a tenant is an **owner** immediately:
  - [ ] `users.role == "owner"`
  - [ ] tenant ownership persisted (`owner_user_id` points to user)
  - [ ] Casbin grouping policy exists for `(user_id, owner, tenant_id)`
- [ ] Registering user joining existing tenant is **user** by default:
  - [ ] `users.role == "user"`
  - [ ] Casbin grouping policy exists for `(user_id, user, tenant_id)`
- [ ] No cross-tenant leakage or ownership drift
- [ ] Tests added and pass for both flows
- [ ] Docs updated

## Dependencies
- `V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.07_seed_default_roles_and_policies.md` (ensure `owner` policies exist or add a follow-up)
- `V1_MVP/03_User_Service/3.3_User_Management/task_03.03.01_list_users_in_tenant.md` (used as baseline for tenant-scoped behavior; tests may reuse setup)

## Notes / Discussion
- If `tenants.owner_user_id` does not exist, prefer adding it (single UUID) over a join table for MVP. Ownership transfer can be added later.
- Keep `owner` a **system role** (non-deletable, non-editable like `admin`/`user`).
- Ensure “single role per user” invariant is honored by admin role assignment endpoints (handled in a separate task).

## AI Agent Log
---
* 2026-01-02: Task created to implement registration bootstrap rules (owner on new tenant; user on existing tenant) aligned with Option D (single role per user).
