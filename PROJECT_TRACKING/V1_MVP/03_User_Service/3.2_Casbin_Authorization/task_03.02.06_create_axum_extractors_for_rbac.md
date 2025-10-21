# Task: Create Axum Extractors for RBAC

**Task ID:** V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.06_create_axum_extractors_for_rbac.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.2_Casbin_Authorization
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Implement custom Axum extractors to simplify role-based and permission-based checks directly in API handlers, making the code cleaner and more declarative.

## Specific Sub-tasks:
- [ ] 1. Implement a `RequireRole` extractor that takes a role name (e.g., `RequireRole("admin")`).
- [ ] 2. Implement a `RequirePermission` extractor that takes a resource and action (e.g., `RequirePermission { resource: "products", action: "write" }`).
- [ ] 3. Both extractors should use the Casbin enforcer from an extension to perform the check.
- [ ] 4. If the check fails, the extractor should return an appropriate `Rejection` (e.g., a 403 Forbidden status).

## Acceptance Criteria:
- [ ] `RequireRole` extractor is implemented and can be used as a handler parameter.
- [ ] `RequirePermission` extractor is implemented.
- [ ] The extractors integrate with the Casbin enforcer to perform checks.
- [ ] Handlers using these extractors are protected as expected.

## Dependencies:
*   Task: `task_03.02.05_implement_axum_authorization_middleware.md`

## Related Documents:
*   `shared/auth/src/extractors.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)