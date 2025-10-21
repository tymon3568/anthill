# Task: Implement Axum Authorization Middleware

**Task ID:** V1_MVP/01_Infrastructure_Setup/1.4_Auth_Library/task_01.04.05_implement_axum_middleware.md
**Version:** V1_MVP
**Phase:** 01_Infrastructure_Setup
**Module:** 1.4_Auth_Library
**Priority:** High
**Status:** Todo
**Assignee:**
**Created Date:** 2025-01-21
**Last Updated:** 2025-01-21

## Detailed Description:
Implement Axum middleware for Casbin authorization that checks JWT tokens and enforces multi-tenant RBAC policies on incoming requests.

## Specific Sub-tasks:
- [ ] 1. Create `src/middleware.rs` in `shared/auth`
- [ ] 2. Implement `casbin_middleware()` function
- [ ] 3. Extract JWT claims (user_id, tenant_id, role) from Authorization header
- [ ] 4. Check permissions with Casbin enforcer (sub, dom, obj, act)
- [ ] 5. Return 403 Forbidden if access denied
- [ ] 6. Handle malformed or expired tokens gracefully
- [ ] 7. Add proper error logging for authorization failures
- [ ] 8. Create unit tests for middleware functionality

## Acceptance Criteria:
- [ ] Middleware function implemented and exported from `shared/auth`
- [ ] JWT extraction works with Bearer token format
- [ ] Casbin permission checking integrated properly
- [ ] Proper error responses for unauthorized access (403)
- [ ] Performance optimized (no unnecessary allocations)
- [ ] Thread-safe implementation for concurrent requests
- [ ] Comprehensive unit tests covering success and failure cases

## Dependencies:
- V1_MVP/01_Infrastructure_Setup/1.4_Auth_Library/task_01.04.04_initialize_casbin_enforcer.md

## Related Documents:
- `shared/auth/src/middleware.rs` (file to be created)
- `shared/jwt/src/lib.rs` (existing JWT utilities)

## Notes / Discussion:
---
* Middleware should be lightweight and not impact request latency significantly
* Consider caching frequently used permissions if performance becomes issue
* Ensure proper integration with existing error handling (AppError)
* Multi-tenant isolation must be strictly enforced at middleware level

## AI Agent Log:
---
* (Log will be automatically updated by AI agent when starting and executing task)