# Task: Expand E2E Test Coverage Beyond Auth Flow

**Task ID:** V1_MVP/08_Frontend/8.9_Production_Hardening/task_08.09.13_e2e_test_coverage.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.9_Production_Hardening
**Priority:** P1
**Status:** Todo
**Assignee:** Unassigned
**Created Date:** 2025-01-21
**Last Updated:** 2025-01-21

## Detailed Description:
Add comprehensive E2E tests for inventory operations, tenant management, and admin features. Currently only auth.e2e.spec.ts exists - need broader coverage.

## Specific Sub-tasks:
- [ ] 1. Create inventory CRUD E2E tests
- [ ] 2. Create product management E2E tests
- [ ] 3. Create tenant switching E2E tests
- [ ] 4. Create admin user management E2E tests
- [ ] 5. Create settings page E2E tests
- [ ] 6. Add visual regression testing
- [ ] 7. Configure test data seeding
- [ ] 8. Add E2E tests to CI/CD pipeline

## Acceptance Criteria:
- [ ] Login/logout flow tested (existing)
- [ ] Inventory CRUD operations covered (create, read, update, delete)
- [ ] Tenant switching scenarios tested
- [ ] Admin workflows validated
- [ ] E2E tests run in CI/CD with test reports
- [ ] Test coverage for critical user journeys > 80%

## Dependencies:
- V1_MVP/08_Frontend/8.1_Project_Setup/task_08.01.01_setup_sveltekit_project.md

## Related Documents:
- `frontend/tests/e2e/auth.e2e.spec.ts` (existing)
- `frontend/tests/e2e/inventory.e2e.spec.ts` (file to be created)
- `frontend/tests/e2e/admin.e2e.spec.ts` (file to be created)
- `frontend/playwright.config.ts` (existing)

## Notes / Discussion:
---
* Playwright already configured in project
* Consider using page object pattern for maintainability
* Use test fixtures for common setup/teardown

## AI Agent Log:
---
