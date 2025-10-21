# Task: Create Casbin Model File

**Task ID:** V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.02_create_casbin_model_file.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.2_Casbin_Authorization
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create the Casbin model configuration file at `shared/auth/model.conf`. This model will define the multi-tenant RBAC structure using `sub, dom, obj, act`.

## Specific Sub-tasks:
- [ ] 1. Create a new file named `model.conf` inside the `shared/auth` directory.
- [ ] 2. Add the `[request_definition]` with `r = sub, dom, obj, act`.
- [ ] 3. Add the `[policy_definition]` with `p = sub, dom, obj, act`.
- [ ] 4. Add the `[role_definition]` with `g = _, _, _`.
- [ ] 5. Add the `[policy_effect]` with `e = some(where (p.eft == allow))`.
- [ ] 6. Add the `[matchers]` with `m = g(r.sub, p.sub, r.dom) && r.dom == p.dom && r.obj == p.obj && r.act == p.act`.

## Acceptance Criteria:
- [ ] The file `shared/auth/model.conf` is created with the exact content specified.
- [ ] The model correctly defines request, policy, role, effect, and matchers for multi-tenant RBAC.

## Dependencies:
*   (Sẽ được cập nhật khi cần)

## Related Documents:
*   `shared/auth/model.conf`

## Notes / Discussion:
---
*   The matcher `r.dom == p.dom` is the key to enforcing multi-tenant data isolation at the policy level.

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)