# Task: Implement Error Tracking Service (Sentry)

**Task ID:** V1_MVP/08_Frontend/8.9_Production_Hardening/task_08.09.10_sentry_integration.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.9_Production_Hardening
**Priority:** P1
**Status:** Todo
**Assignee:** Unassigned
**Created Date:** 2025-01-21
**Last Updated:** 2025-01-21

## Detailed Description:
Integrate Sentry for client and server error tracking. This provides real-time visibility into application errors and helps with debugging production issues.

## Specific Sub-tasks:
- [ ] 1. Install @sentry/sveltekit package
- [ ] 2. Configure Sentry DSN in environment variables
- [ ] 3. Initialize Sentry in hooks.client.ts
- [ ] 4. Initialize Sentry in hooks.server.ts
- [ ] 5. Configure source maps upload for production builds
- [ ] 6. Add user context to error reports
- [ ] 7. Set up error filtering (ignore expected errors)
- [ ] 8. Configure performance monitoring sampling

## Acceptance Criteria:
- [ ] Sentry SDK installed and configured
- [ ] Client-side errors captured and reported to Sentry
- [ ] Server-side errors captured with request context
- [ ] User information (ID, email, tenant) attached to error reports
- [ ] Source maps available for stack trace deobfuscation
- [ ] Performance transactions sampled appropriately

## Dependencies:
- V1_MVP/08_Frontend/8.9_Production_Hardening/task_08.09.02_handle_server_error_hook.md

## Related Documents:
- `frontend/src/hooks.client.ts` (file to be created/modified)
- `frontend/src/hooks.server.ts` (file to be modified)
- `frontend/vite.config.ts` (file to be modified)
- Sentry SvelteKit documentation

## Notes / Discussion:
---
* Use @sentry/sveltekit for SvelteKit-specific integration
* Configure sampling rate based on traffic (e.g., 0.1 for 10%)
* Consider using Sentry's session replay for debugging

## AI Agent Log:
---
