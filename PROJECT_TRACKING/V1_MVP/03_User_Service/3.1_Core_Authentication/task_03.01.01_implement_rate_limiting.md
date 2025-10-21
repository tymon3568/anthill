# Task: Implement Rate Limiting and Brute-Force Protection

**Task ID:** V1_MVP/03_User_Service/3.1_Core_Authentication/task_03.01.01_implement_rate_limiting.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.1_Core_Authentication
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Implement rate limiting to protect against brute-force attacks on authentication endpoints.

## Specific Sub-tasks:
- [ ] 1. Choose and add a rate-limiting crate like `tower_governor`.
- [ ] 2. Configure the rate limiter for login: 5 attempts per IP per 5 minutes.
- [ ] 3. Configure the rate limiter for forgot password: 3 attempts per email per day.
- [ ] 4. Use Redis as the backing store for the rate limiter for a distributed environment.
- [ ] 5. Apply the rate-limiting middleware to the relevant Axum routes.
- [ ] 6. Write integration tests to verify that requests are blocked after the limit is exceeded.

## Acceptance Criteria:
- [ ] `tower_governor` dependency is added.
- [ ] A GovernorLayer is configured and applied to the login and forgot-password routes.
- [ ] Requests exceeding the rate limit receive a `429 Too Many Requests` response.
- [ ] Integration tests are written to verify the rate limiting logic.

## Dependencies:
*   (Sẽ được cập nhật khi cần)

## Related Documents:
*   `user_service/api/src/main.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)