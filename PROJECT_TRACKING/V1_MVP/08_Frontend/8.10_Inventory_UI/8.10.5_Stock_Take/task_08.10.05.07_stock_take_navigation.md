# Task: Add Navigation Link to Stock Take

**Task ID:** `V1_MVP/08_Frontend/8.10_Inventory_UI/8.10.5_Stock_Take/task_08.10.05.07_stock_take_navigation.md`
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.10_Inventory_UI
**Sub-Module:** 8.10.5_Stock_Take
**Priority:** Medium
**Status:** Done
**Assignee:** AI Agent
**Created Date:** 2026-01-29
**Last Updated:** 2026-01-29
**Dependencies:**
- `V1_MVP/08_Frontend/8.10_Inventory_UI/8.10.5_Stock_Take/task_08.10.05.04_stock_take_list.md`

## 1. Detailed Description

Add Stock Takes navigation link to the sidebar under the Inventory section. The link should use plural form "Stock Takes" to match other navigation items.

## 2. Implementation Steps (Specific Sub-tasks)

- [x] 1. Locate navigation configuration file
- [x] 2. Find Inventory section items
- [x] 3. Update "Stock Take" to "Stock Takes" (plural)
- [x] 4. Update URL from `/inventory/stock-take` to `/inventory/stock-takes` (plural)
- [x] 5. Verify navigation renders correctly

## 3. Completion Criteria

- [x] Navigation link appears under Inventory section
- [x] Link text is "Stock Takes" (plural)
- [x] Link URL is `/inventory/stock-takes` (plural)
- [x] Clicking link navigates to list page

## Related Documents

- UI Architecture: `docs/ui-architecture-proposal.md`

## Files Created/Modified

- `frontend/src/lib/config/navigation.ts` (Modified)

## Notes

The navigation item was already present but used singular form. Updated to match the plural convention used by other navigation items and route paths.

## AI Agent Log:

* 2026-01-29 17:30: AI Agent - Task started
    - Found existing navigation config
    - Updated from singular to plural form

* 2026-01-29 17:35: AI Agent - Task completed
    - Changed "Stock Take" → "Stock Takes"
    - Changed "/inventory/stock-take" → "/inventory/stock-takes"
    - Removed old singular route directory
    - Status changed to Done
