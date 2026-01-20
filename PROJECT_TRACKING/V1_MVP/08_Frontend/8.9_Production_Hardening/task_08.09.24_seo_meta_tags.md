# Task: Add SEO Meta Tags and Open Graph

**Task ID:** V1_MVP/08_Frontend/8.9_Production_Hardening/task_08.09.24_seo_meta_tags.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.9_Production_Hardening
**Priority:** P2
**Status:** Todo
**Assignee:** Unassigned
**Created Date:** 2025-01-21
**Last Updated:** 2025-01-21

## Detailed Description:
Enhance app.html with meta tags, Open Graph, and structured data for better SEO and social sharing.

## Specific Sub-tasks:
- [ ] 1. Create meta tag component/utility
- [ ] 2. Add dynamic page titles and descriptions
- [ ] 3. Implement Open Graph tags for social sharing
- [ ] 4. Add Twitter Card meta tags
- [ ] 5. Implement structured data (JSON-LD) for products
- [ ] 6. Add canonical URLs
- [ ] 7. Create robots.txt and sitemap.xml
- [ ] 8. Test with social media debuggers

## Acceptance Criteria:
- [ ] Meta description and title tags dynamic per page
- [ ] Open Graph tags implemented for all public pages
- [ ] Twitter Card tags added for social sharing
- [ ] Structured data valid (test with Google Rich Results)
- [ ] robots.txt and sitemap.xml generated

## Dependencies:
- V1_MVP/08_Frontend/8.1_Project_Setup/task_08.01.01_setup_sveltekit_project.md

## Related Documents:
- `frontend/src/app.html` (file to be modified)
- `frontend/src/routes/+layout.svelte` (file to be modified)
- `frontend/static/robots.txt` (file to be created)
- Schema.org documentation

## Notes / Discussion:
---
* Use svelte:head for per-page meta tags
* SaaS apps may have limited SEO needs (mostly authenticated)
* Focus on login, landing, and public product pages

## AI Agent Log:
---
