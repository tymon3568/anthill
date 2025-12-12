# Task: Implement Integration Tests for API Endpoints

**Task ID:** V1_MVP/04_Inventory_Service/4.13_Testing_Quality_Assurance/task_04.13.03_integration_tests_api_endpoints.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.13_Testing_Quality_Assurance
**Priority:** High
**Status:** InProgress
**Assignee:** AI_Agent
**Created Date:** 2025-10-21
**Last Updated:** 2025-12-12
**Related PR:** [#98](https://github.com/tymon3568/anthill/pull/98)

## Detailed Description:
Implement integration tests for inventory API endpoints in `inventory_service_api`, testing full HTTP request/response cycles with a test database.

## Specific Sub-tasks:
- [x] 1. Set up integration testing environment (existing in helpers.rs)
    - [x] 1.1. Configure test database (PostgreSQL)
    - [x] 1.2. Set up test fixtures and data seeding
    - [x] 1.3. Configure axum test client
- [x] 2. Test Category API endpoints (7 tests in category_integration_tests.rs)
    - [x] 2.1. POST /api/v1/categories (create)
    - [x] 2.2. GET /api/v1/categories (list, tree)
    - [x] 2.3. PUT /api/v1/categories/:id (update)
    - [x] 2.4. DELETE /api/v1/categories/:id (soft delete)
- [x] 3. Test Product API endpoints (5 tests in product_integration_tests.rs)
    - [x] 3.1. POST /api/v1/products (N/A - no create endpoint used)
    - [x] 3.2. GET /api/v1/products (search, filter)
    - [x] 3.3. GET /api/v1/products/:id (via search)
    - [x] 3.4. PUT /api/v1/products/:id (N/A - no update endpoint used)
- [x] 4. Test Warehouse API endpoints (8 tests in warehouse_integration_tests.rs)
    - [x] 4.1. CRUD operations for warehouses
    - [x] 4.2. CRUD operations for zones
    - [x] 4.3. CRUD operations for locations
- [ ] 5. Test Stock Operations API
    - [ ] 5.1. POST /api/v1/stock-moves
    - [ ] 5.2. POST /api/v1/stock-adjustments
    - [ ] 5.3. GET /api/v1/stock-levels

## Acceptance Criteria:
- [ ] Integration tests cover all major API endpoints
- [ ] Tests use real database transactions (rollback after test)
- [ ] Authentication/authorization tested
- [ ] Multi-tenancy tested (tenant isolation)
- [ ] Error responses validated

## Dependencies:
* task_04.13.02_unit_tests_service_layer.md

## Related Documents:
* API documentation (OpenAPI spec)
