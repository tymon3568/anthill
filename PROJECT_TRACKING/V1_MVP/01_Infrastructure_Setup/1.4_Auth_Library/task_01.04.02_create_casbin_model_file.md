# Task: Create Casbin Model Configuration File

**Task ID:** V1_MVP/01_Infrastructure_Setup/1.4_Auth_Library/task_01.04.02_create_casbin_model_file.md
**Version:** V1_MVP
**Phase:** 01_Infrastructure_Setup
**Module:** 1.4_Auth_Library
**Priority:** High
**Status:** Done
**Assignee:**
**Created Date:** 2025-01-21
**Last Updated:** 2025-01-21

## Detailed Description:
Create the Casbin model configuration file at `shared/auth/model.conf`. This model will define the multi-tenant RBAC structure using `sub, dom, obj, act` for tenant isolation.

## Specific Sub-tasks:
- [ ] 1. Create a new file named `model.conf` inside the `shared/auth` directory
- [ ] 2. Add the `[request_definition]` with `r = sub, dom, obj, act`
- [ ] 3. Add the `[policy_definition]` with `p = sub, dom, obj, act`
- [ ] 4. Add the `[role_definition]` with `g = _, _, _`
- [ ] 5. Add the `[policy_effect]` with `e = some(where (p.eft == allow))`
- [ ] 6. Add the `[matchers]` with `m = g(r.sub, p.sub, r.dom) && r.dom == p.dom && r.obj == p.obj && r.act == p.act`

## Acceptance Criteria:
- [x] The file `shared/auth/model.conf` is created with the exact content specified
- [x] The model correctly defines request, policy, role, effect, and matchers for multi-tenant RBAC
- [x] Multi-tenant isolation is enforced through `r.dom == p.dom` matcher

## Dependencies:
- V1_MVP/01_Infrastructure_Setup/1.4_Auth_Library/task_01.04.01_add_casbin_dependencies.md

## Related Documents:
- `shared/auth/model.conf` (file to be created)

## Notes / Discussion:
---
* The matcher `r.dom == p.dom` is the key to enforcing multi-tenant data isolation at the policy level
* This ensures users can only access resources within their own tenant
* Model structure: subject (user/role), domain (tenant), object (resource), action (permission)

## AI Agent Log:
---
*   2025-11-05 10:56: Task status updated by Claude
    - Casbin model file created early in project setup
    - Still valid and in use for multi-tenant RBAC after Kanidm migration
    - Status: Done ✓
