# Task: Create Axum Extractors for RBAC

**Task ID:** V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.06_create_axum_extractors_for_rbac.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.2_Casbin_Authorization
**Priority:** High
**Status:** InProgress
**Assignee:** Gemini
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-25

## Detailed Description:
Implement custom Axum extractors to simplify role-based and permission-based checks directly in API handlers, making the code cleaner and more declarative.

## Specific Sub-tasks:
- [ ] 1. Implement a `RequireRole` extractor that takes a role name (e.g., `RequireRole("admin")`).
- [x] 2. Implement a `RequirePermission` extractor that takes a resource and action (e.g., `RequirePermission { resource: "products", action: "write" }`).
- [x] 3. Both extractors should use the Casbin enforcer from an extension to perform the check.
- [x] 4. If the check fails, the extractor should return an appropriate `Rejection` (e.g., a 403 Forbidden status).

## Acceptance Criteria:
- [ ] `RequireRole` extractor is implemented and can be used as a handler parameter.
- [x] `RequirePermission` extractor is implemented.
- [x] The extractors integrate with the Casbin enforcer to perform checks.
- [x] Handlers using these extractors are protected as expected.

## Dependencies:
*   Task: `task_03.02.05_implement_axum_authorization_middleware.md`

## Related Documents:
*   `shared/auth/src/extractors.rs`

## Notes / Discussion:
---
*   `RequireAdmin` is a specific implementation of `RequireRole`. A generic `RequireRole` is not yet implemented.
*   `RequirePermission` is implemented and extracts resource/action from the request.

## AI Agent Log:
---
* 2025-10-25 10:50: Gemini review: Verified `RequireAdmin` and `RequirePermission` extractors in `shared/auth/src/extractors.rs`. `RequireRole` is not generic. `RequirePermission` is implemented. Status updated to InProgress.