# Task: Create Critical Tenant Isolation Security Test

**Task ID:** V1_MVP/03_User_Service/3.3_User_Management/task_03.03.02_tenant_isolation_security_test.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.3_User_Management
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
This is a critical security test to ensure tenant isolation is unbreakable. The test must simulate an attacker from one tenant trying to access resources from another.

## Specific Sub-tasks:
- [ ] 1. Create a new integration test file, e.g., `tests/security.rs`.
- [ ] 2. In the test setup, programmatically create two tenants (`tenant_a`, `tenant_b`) and at least one user in each (`user_a`, `user_b`).
- [ ] 3. Create a resource (e.g., a product, an order, or user `user_b` itself) that belongs to `tenant_b`.
- [ ] 4. Log in as `user_a` to get a valid JWT for `tenant_a`.
- [ ] 5. Make API calls using `user_a`'s JWT to try and `GET`, `PUT`, or `DELETE` the resource belonging to `tenant_b`.
- [ ] 6. Assert that all responses are `403 Forbidden` or `404 Not Found`.

## Acceptance Criteria:
- [ ] An integration test file dedicated to tenant isolation is created.
- [ ] The test sets up at least two separate tenants and users.
- [ ] The test systematically attempts to perform cross-tenant data access (read, write, delete).
- [ ] All attempts for cross-tenant access must fail with a `403` or `404` status code.
- [ ] The test must be part of the main CI/CD pipeline.

## Dependencies:
*   (Core user and tenant creation logic must be complete)

## Related Documents:
*   `user_service/api/tests/`

## Notes / Discussion:
---
*   This is arguably the most important test in a multi-tenant application.

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)