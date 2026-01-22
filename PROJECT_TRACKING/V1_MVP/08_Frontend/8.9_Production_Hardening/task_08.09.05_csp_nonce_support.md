# Task: Add CSP Nonce Support for Inline Scripts

**Task ID:** V1_MVP/08_Frontend/8.9_Production_Hardening/task_08.09.05_csp_nonce_support.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.9_Production_Hardening
**Priority:** P0
**Status:** Todo
**Assignee:** Unassigned
**Created Date:** 2026-01-20
**Last Updated:** 2026-01-20

## Detailed Description:
Implement nonce-based CSP to remove 'unsafe-inline' from script-src directive. This significantly improves security by preventing XSS attacks through inline script injection.

## Specific Sub-tasks:
- [ ] 1. Generate unique nonce per request in hooks.server.ts
- [ ] 2. Pass nonce to app.html via locals
- [ ] 3. Update app.html to use %sveltekit.nonce% placeholder
- [ ] 4. Configure CSP header with nonce in svelte.config.js
- [ ] 5. Remove 'unsafe-inline' from script-src directive
- [ ] 6. Test all pages for CSP violations
- [ ] 7. Add CSP violation reporting (task 08.09.25)

## Acceptance Criteria:
- [ ] CSP nonce generated uniquely per request
- [ ] Nonce properly added to all inline scripts
- [ ] script-src uses nonce instead of 'unsafe-inline'
- [ ] No CSP violations in browser console
- [ ] All application functionality works correctly

## Dependencies:
- V1_MVP/08_Frontend/8.1_Project_Setup/task_08.01.02_upgrade_sveltekit_security_cves.md

## Related Documents:
- `frontend/src/hooks.server.ts` (file to be modified)
- `frontend/src/app.html` (file to be modified)
- `frontend/svelte.config.js` (file to be modified)
- SvelteKit CSP documentation

## Notes / Discussion:
---
* SvelteKit supports %sveltekit.nonce% placeholder in app.html
* Use crypto.randomUUID() or similar for nonce generation
* Consider report-only mode first (task 08.09.25)

## AI Agent Log:
---
