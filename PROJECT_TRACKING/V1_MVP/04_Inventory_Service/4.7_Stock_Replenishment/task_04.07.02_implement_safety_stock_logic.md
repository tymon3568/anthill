# Task: Implement Safety Stock in Replenishment Logic

**Task ID:** V1_MVP/04_Inventory_Service/4.7_Stock_Replenishment/task_04.07.02_implement_safety_stock_logic.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.7_Stock_Replenishment
**Priority:** Medium
**Status:** ToDo
**Assignee:** Unassigned
**Created Date:** 2025-12-14
**Last Updated:** 2025-12-14

## Detailed Description:
The `safety_stock` field exists in the `reorder_rules` table but is **not currently used** in the replenishment calculation logic. This task implements the safety stock feature to properly affect replenishment decisions.

### Current Behavior:
- `needs_replenishment = projected_quantity < reorder_point`
- `suggested_order_quantity = max_quantity - projected_quantity`
- `safety_stock` is stored but ignored

### Expected Behavior After Implementation:
Safety stock should be used in one of these ways (choose during implementation):

**Option A - Adjust Reorder Point:**
```
effective_reorder_point = reorder_point + safety_stock
needs_replenishment = projected_quantity < effective_reorder_point
```

**Option B - Include Safety Stock in Order Quantity:**
```
target_quantity = max_quantity + safety_stock
suggested_order_quantity = target_quantity - projected_quantity
```

**Option C - Both (recommended):**
- Trigger replenishment earlier when inventory approaches safety stock threshold
- Include safety stock in suggested order calculation

## Specific Sub-tasks:
- [ ] 1. Research best practices for safety stock implementation in inventory systems
- [ ] 2. Decide on implementation approach (Option A, B, or C above)
- [ ] 3. Update `check_product_replenishment` in `services/inventory_service/infra/src/services/replenishment.rs`
- [ ] 4. Update `run_replenishment_check` to use same logic
- [ ] 5. Update tests in `services/inventory_service/api/tests/reorder_rules_tests.rs`:
  - [ ] 5.1. Test: inventory at safety_stock + 1 - verify expected behavior
  - [ ] 5.2. Test: inventory below safety_stock - verify needs_replenishment and suggested_order_quantity
  - [ ] 5.3. Test: verify safety_stock affects suggested_order_quantity calculation
- [ ] 6. Update API documentation if needed

## Acceptance Criteria:
- [ ] `safety_stock` field is used in replenishment calculation logic
- [ ] `needs_replenishment` is triggered appropriately based on safety stock
- [ ] `suggested_order_quantity` accounts for safety stock in its calculation
- [ ] All existing tests pass
- [ ] New tests explicitly validate safety stock behavior
- [ ] Code review comments from PR #103 are addressed

## Dependencies:
- Depends on: `task_04.07.01_implement_automated_replenishment.md` (completed)

## Related Documents:
- `services/inventory_service/infra/src/services/replenishment.rs` - Service implementation
- `services/inventory_service/api/tests/reorder_rules_tests.rs` - Test file
- PR #103 review comments

## Notes / Discussion:
---
- This task was created to address a code review comment in PR #103 noting that `safety_stock` is not used in the actual replenishment logic
- Consider impact on existing clients when changing calculation behavior

## AI Agent Log:
---
- 2025-12-14 07:44: Task created based on PR #103 review feedback about unused safety_stock field
- (Logs will be automatically updated by AI agent when starting and executing tasks)
