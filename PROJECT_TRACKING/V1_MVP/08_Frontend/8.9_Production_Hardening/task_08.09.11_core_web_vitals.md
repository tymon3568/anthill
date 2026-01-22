# Task: Add Performance Monitoring (Core Web Vitals)

**Task ID:** V1_MVP/08_Frontend/8.9_Production_Hardening/task_08.09.11_core_web_vitals.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.9_Production_Hardening
**Priority:** P1
**Status:** Todo
**Assignee:** Unassigned
**Created Date:** 2026-01-20
**Last Updated:** 2026-01-20

## Detailed Description:
Implement performance monitoring for LCP, FID, CLS metrics (Core Web Vitals). This ensures the application meets Google's performance standards and provides good user experience.

## Specific Sub-tasks:
- [ ] 1. Install web-vitals package
- [ ] 2. Create performance monitoring utility
- [ ] 3. Track LCP (Largest Contentful Paint)
- [ ] 4. Track FID (First Input Delay) / INP (Interaction to Next Paint)
- [ ] 5. Track CLS (Cumulative Layout Shift)
- [ ] 6. Send metrics to monitoring service (Sentry/custom)
- [ ] 7. Define performance budgets
- [ ] 8. Add Lighthouse CI to build pipeline

## Acceptance Criteria:
- [ ] Core Web Vitals (LCP, FID/INP, CLS) tracked
- [ ] Performance data sent to monitoring service
- [ ] Performance budgets defined (LCP < 2.5s, FID < 100ms, CLS < 0.1)
- [ ] Performance regression detection in CI/CD
- [ ] Dashboard available for performance monitoring

## Dependencies:
- V1_MVP/08_Frontend/8.9_Production_Hardening/task_08.09.10_sentry_integration.md

## Related Documents:
- `frontend/src/lib/utils/performance.ts` (file to be created)
- `frontend/src/hooks.client.ts` (file to be modified)
- web.dev Core Web Vitals documentation

## Notes / Discussion:
---
* Use web-vitals library for accurate measurement
* Consider using Sentry's performance monitoring
* Add performance marks for custom timing

## AI Agent Log:
---
