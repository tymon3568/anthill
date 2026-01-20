# Task: Implement Session Timeout Warning

**Task ID:** V1_MVP/08_Frontend/8.9_Production_Hardening/task_08.09.27_session_timeout_warning.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.9_Production_Hardening
**Priority:** P1
**Status:** Todo
**Assignee:** Unassigned
**Created Date:** 2025-01-21
**Last Updated:** 2025-01-21

## Detailed Description:
Show warning before session expires and auto-logout on timeout. This improves user experience and security for inactive sessions.

## Specific Sub-tasks:
- [ ] 1. Create session timeout tracking utility
- [ ] 2. Implement warning modal component
- [ ] 3. Show warning 5 minutes before session expiry
- [ ] 4. Add countdown timer in warning modal
- [ ] 5. Implement "Extend Session" action
- [ ] 6. Auto-logout when timeout reached
- [ ] 7. Save unsaved work before logout (optional)
- [ ] 8. Test timeout behavior across tabs

## Acceptance Criteria:
- [ ] Warning shown 5 minutes before session expiry
- [ ] Countdown timer displayed in warning modal
- [ ] "Extend Session" option refreshes tokens
- [ ] Auto-logout after timeout with redirect to login
- [ ] Session timeout synced across browser tabs

## Dependencies:
- V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.04_session_management.md

## Related Documents:
- `frontend/src/lib/components/SessionTimeout.svelte` (file to be created)
- `frontend/src/lib/auth/token-manager.ts` (file to be modified)

## Notes / Discussion:
---
* Use BroadcastChannel API for cross-tab sync
* Consider activity detection to reset timeout
* Token expiry time should drive the warning timing

## AI Agent Log:
---
