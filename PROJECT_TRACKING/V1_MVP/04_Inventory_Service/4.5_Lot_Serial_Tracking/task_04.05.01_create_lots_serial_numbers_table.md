# Task: Create `lots_serial_numbers` Table

**Task ID:** V1_MVP/04_Inventory_Service/4.5_Lot_Serial_Tracking/task_04.05.01_create_lots_serial_numbers_table.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.5_Lot_Serial_Tracking
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create the `lots_serial_numbers` table to enable traceability by tracking individual product units (serial numbers) or batches (lot numbers).

## Specific Sub-tasks:
- [ ] 1. Create a new SQL migration file.
- [ ] 2. Define all columns: `lot_serial_id`, `tenant_id`, `product_id`, `tracking_type`, `lot_number`/`serial_number`, `expiry_date`, `status`, etc.
- [ ] 3. Add indexes for efficient querying on `lot_number`/`serial_number` and `product_id`.

## Acceptance Criteria:
- [ ] A new SQL migration is created for the `lots_serial_numbers` table.
- [ ] The table schema is implemented as specified.
- [ ] The migration runs successfully.

## Dependencies:
*   (Sẽ được cập nhật khi cần)

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
