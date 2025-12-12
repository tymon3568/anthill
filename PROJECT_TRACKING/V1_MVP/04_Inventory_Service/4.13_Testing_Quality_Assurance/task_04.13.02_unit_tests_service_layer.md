# Task: Implement Unit Tests for Service Layer

**Task ID:** V1_MVP/04_Inventory_Service/4.13_Testing_Quality_Assurance/task_04.13.02_unit_tests_service_layer.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.13_Testing_Quality_Assurance
**Priority:** High
**Status:** InProgress
**Assignee:** AI_Agent
**Created Date:** 2025-12-12
**Last Updated:** 2025-12-12

## Detailed Description:
Implement unit tests for service layer implementations in `inventory_service_infra`, using mock repositories to test business logic in CategoryService, ProductService, WarehouseService, and other core services.

## Specific Sub-tasks:
- [ ] 1. Set up mock infrastructure using mockall crate
    - [ ] 1.1. Create MockCategoryRepository
    - [ ] 1.2. Create MockProductRepository
    - [ ] 1.3. Create MockWarehouseRepository
- [ ] 2. Write tests for CategoryService
    - [ ] 2.1. create_category (validation, parent existence check)
    - [ ] 2.2. update_category (field updates, name uniqueness)
    - [ ] 2.3. delete_category (soft delete, has children check)
    - [ ] 2.4. get_category_tree (hierarchy building)
    - [ ] 2.5. bulk_activate/deactivate_categories
- [ ] 3. Write tests for ProductService
    - [ ] 3.1. search_products (filters, pagination)
    - [ ] 3.2. get_product_by_sku
    - [ ] 3.3. Product search analytics
- [ ] 4. Write tests for WarehouseService
    - [ ] 4.1. CRUD operations
    - [ ] 4.2. Hierarchy management
    - [ ] 4.3. Zone and location management

## Acceptance Criteria:
- [ ] Mock repositories created for all core repositories
- [ ] Service tests cover all major business rules
- [ ] Test coverage > 70% for service layer
- [ ] Edge cases tested (not found, validation errors, conflicts)

## Dependencies:
* task_04.13.01_unit_tests_business_logic.md (Done)

## Related Documents:
* services/inventory_service/infra/src/services/category_tests.rs (existing example)

## Notes / Discussion:
---
* Follow existing pattern in category_tests.rs for mock setup
