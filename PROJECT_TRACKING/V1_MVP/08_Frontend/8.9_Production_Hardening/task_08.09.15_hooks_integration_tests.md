# Task: Add Integration Tests for Server Hooks

**Task ID:** V1_MVP/08_Frontend/8.9_Production_Hardening/task_08.09.15_hooks_integration_tests.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.9_Production_Hardening
**Priority:** P2
**Status:** Todo
**Assignee:** Unassigned
**Created Date:** 2026-01-20
**Last Updated:** 2026-01-20

## Detailed Description:
Test authentication flow, tenant resolution, and error handling in hooks.server.ts. These integration tests ensure the server hooks work correctly together.

## Specific Sub-tasks:
- [ ] 1. Set up integration test environment
- [ ] 2. Create mock request/response utilities
- [ ] 3. Test authentication token validation
- [ ] 4. Test token refresh flow
- [ ] 5. Test tenant resolution from headers/cookies
- [ ] 6. Test public route enforcement
- [ ] 7. Test error response format
- [ ] 8. Test hook sequence execution order

## Acceptance Criteria:
- [ ] Token refresh flow tested with mock auth server
- [ ] Tenant resolution validated for all scenarios
- [ ] Public route enforcement verified
- [ ] Error responses tested for correct format
- [ ] Integration tests run in CI/CD

## Dependencies:
- V1_MVP/08_Frontend/8.9_Production_Hardening/task_08.09.04_sequence_hooks_pattern.md

## Related Documents:
- `frontend/src/hooks.server.ts` (file to test)
- `frontend/tests/integration/hooks.test.ts` (file to be created)

## Notes / Discussion:
---
* Use msw (Mock Service Worker) for API mocking
* Test each hook in isolation and in sequence
* Consider using supertest for HTTP-level testing

## AI Agent Log:
---
