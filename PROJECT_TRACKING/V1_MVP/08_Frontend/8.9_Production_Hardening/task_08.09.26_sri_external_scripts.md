# Task: Add Subresource Integrity (SRI) for External Scripts

**Task ID:** V1_MVP/08_Frontend/8.9_Production_Hardening/task_08.09.26_sri_external_scripts.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.9_Production_Hardening
**Priority:** P2
**Status:** Todo
**Assignee:** Unassigned
**Created Date:** 2026-01-20
**Last Updated:** 2026-01-20

## Detailed Description:
Implement SRI for any external CDN resources to prevent tampering with third-party scripts.

## Specific Sub-tasks:
- [ ] 1. Audit external script/style resources
- [ ] 2. Generate SRI hashes for external resources
- [ ] 3. Add integrity attributes to script/link tags
- [ ] 4. Implement fallback mechanism for SRI failures
- [ ] 5. Automate SRI hash generation for updates
- [ ] 6. Test SRI validation in browsers
- [ ] 7. Document external resource policy

## Acceptance Criteria:
- [ ] SRI hashes generated for all external scripts
- [ ] Integrity attributes added to external resources
- [ ] Fallback mechanism works when SRI fails
- [ ] SRI hash updates automated for dependency changes
- [ ] No external resources loaded without SRI

## Dependencies:
- V1_MVP/08_Frontend/8.9_Production_Hardening/task_08.09.05_csp_nonce_support.md

## Related Documents:
- `frontend/src/app.html` (file to audit)
- MDN Subresource Integrity documentation

## Notes / Discussion:
---
* SRI only works for resources served with CORS headers
* Most dependencies are bundled, so external resources may be minimal
* Consider self-hosting critical external resources

## AI Agent Log:
---
