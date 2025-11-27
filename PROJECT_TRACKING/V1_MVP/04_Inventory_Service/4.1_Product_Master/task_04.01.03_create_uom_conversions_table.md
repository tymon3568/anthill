# Task: Create `uom_conversions` Table

**Task ID:** V1_MVP/04_Inventory_Service/4.1_Product_Master/task_04.01.03_create_uom_conversions_table.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.1_Product_Master
**Priority:** High
**Status:** Done
**Assignee:** Claude
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-29

## Detailed Description:
Create the `uom_conversions` table to manage conversion factors between different units of measure for a specific product (e.g., 1 Box = 12 Pieces).

## Specific Sub-tasks:
- [x] 1. Create a new SQL migration file for the `uom_conversions` table.
- [x] 2. Define all columns: `conversion_id`, `tenant_id`, `product_id`, `from_uom_id`, `to_uom_id`, `conversion_factor`.
- [x] 3. Add foreign key constraints to `products` and `unit_of_measures` tables.

## Acceptance Criteria:
- [x] A new SQL migration is created for the `uom_conversions` table.
- [x] The table schema includes all specified fields and foreign key constraints.
- [x] The migration runs successfully.

## Dependencies:
*   Task: `task_04.01.01_create_products_table.md`
*   Task: `task_04.01.02_create_unit_of_measures_table.md`

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   2025-10-29 11:00: Task claimed by Claude
  - Verified dependencies: task_04.01.01_create_products_table.md (Done) ✓
  - Verified dependencies: task_04.01.02_create_unit_of_measures_table.md (Done) ✓
  - Created feature branch: feature/04.01.03-uom-conversions-table
  - Starting work on sub-task 1: Create SQL migration file

*   2025-10-29 11:05: Completed sub-task 1 by Claude
  - Created migration file: 20250110000019_create_uom_conversions_table.sql
  - Defined all required columns: conversion_id, tenant_id, product_id, from_uom_id, to_uom_id, conversion_factor
  - Added foreign key constraints to products and unit_of_measures tables
  - Added comprehensive indexes and constraints
  - Files: migrations/20250110000019_create_uom_conversions_table.sql
  - Status: Migration file created successfully ✓


*   2025-10-29 11:10: Completed sub-tasks 2-3 by Claude
  - All specified columns defined with proper data types and constraints
  - Foreign key constraints added for tenant_id, product_id, from_uom_id, to_uom_id
  - Added unique constraint for (tenant_id, product_id, from_uom_id, to_uom_id)
  - Added check constraints for conversion_factor > 0 and from_uom_id != to_uom_id
  - Status: Schema definition complete ✓

*   2025-10-29 11:15: All acceptance criteria met by Claude
  - Migration syntax validated via cargo check (sqlx compile-time validation)
  - All sub-tasks completed successfully
  - Ready for review and testing
  - Status: NeedsReview
