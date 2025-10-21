# Task: Create `stock_moves` Table (Stock Ledger)

**Task ID:** V1_MVP/04_Inventory_Service/4.3_Stock_Operations/task_04.03.01_create_stock_moves_table.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.3_Stock_Operations
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create the `stock_moves` table, which will act as an **immutable** stock ledger. This is one of the most critical tables in the inventory system, providing a complete audit trail of all stock movements.

## Specific Sub-tasks:
- [ ] 1. Create a new SQL migration file for the `stock_moves` table.
- [ ] 2. Define all columns as specified: `move_id`, `tenant_id`, `product_id`, `source_location_id`, `destination_location_id`, `move_type`, `quantity`, `unit_cost`, `reference_type`, `reference_id`, `idempotency_key`, etc.
- [ ] 3. Ensure the table is designed to be immutable (no application logic should ever `UPDATE` a row).
- [ ] 4. Add critical indexes on (`tenant_id`, `product_id`, `move_date`) and (`reference_type`, `reference_id`).

## Acceptance Criteria:
- [ ] A new SQL migration is created for the `stock_moves` table.
- [ ] The table schema is implemented as specified.
- [ ] Critical indexes are created for performance.
- [ ] The migration runs successfully.

## Dependencies:
*   Task: `task_04.01.01_create_products_table.md`

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   This table is the foundation for almost all inventory reporting.

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)