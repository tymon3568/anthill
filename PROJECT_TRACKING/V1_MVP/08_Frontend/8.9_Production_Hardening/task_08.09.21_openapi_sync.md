# Task: Sync Frontend API Client with Backend OpenAPI Spec

**Task ID:** V1_MVP/08_Frontend/8.9_Production_Hardening/task_08.09.21_openapi_sync.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.9_Production_Hardening
**Priority:** P1
**Status:** Todo
**Assignee:** Unassigned
**Created Date:** 2026-01-20
**Last Updated:** 2026-01-20

## Detailed Description:
Generate TypeScript API client from backend OpenAPI specification to ensure frontend and backend types are always in sync.

## Specific Sub-tasks:
- [ ] 1. Export OpenAPI spec from backend services
- [ ] 2. Choose and configure API client generator (openapi-typescript)
- [ ] 3. Generate TypeScript types from OpenAPI spec
- [ ] 4. Generate API client functions
- [ ] 5. Integrate generated client with existing code
- [ ] 6. Add generation script to package.json
- [ ] 7. Automate generation in CI/CD pipeline
- [ ] 8. Document the sync process

## Acceptance Criteria:
- [ ] API client auto-generated from OpenAPI specification
- [ ] Type definitions match backend exactly
- [ ] API client integrated with existing fetch utilities
- [ ] Generation automated in CI/CD (fails on spec changes)
- [ ] Documentation for updating API client

## Dependencies:
- V1_MVP/03_User_Service (Backend OpenAPI spec)
- V1_MVP/04_Inventory_Service (Backend OpenAPI spec)

## Related Documents:
- `frontend/src/lib/api/` (directory to be enhanced)
- `services/user-service/openapi.yaml` (backend spec)
- `services/inventory-service/openapi.yaml` (backend spec)
- openapi-typescript documentation

## Notes / Discussion:
---
* Use openapi-typescript for type generation
* Consider openapi-fetch for runtime client
* Backend services use Rust + Axum with utoipa for OpenAPI

## AI Agent Log:
---
