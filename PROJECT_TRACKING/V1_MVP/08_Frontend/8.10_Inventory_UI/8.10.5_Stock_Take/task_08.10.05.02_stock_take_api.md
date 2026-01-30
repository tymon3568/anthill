# Task: Create Stock Take API Client

**Task ID:** `V1_MVP/08_Frontend/8.10_Inventory_UI/8.10.5_Stock_Take/task_08.10.05.02_stock_take_api.md`
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.10_Inventory_UI
**Sub-Module:** 8.10.5_Stock_Take
**Priority:** High
**Status:** Done
**Assignee:** AI Agent
**Created Date:** 2026-01-29
**Last Updated:** 2026-01-29
**Dependencies:**
- `V1_MVP/08_Frontend/8.10_Inventory_UI/8.10.5_Stock_Take/task_08.10.05.01_stock_take_types.md`

## 1. Detailed Description

Create API client functions for Stock Take module to communicate with backend:
- List stock takes with pagination and filters
- Get single stock take details
- Create new stock take
- Submit counts (update actual quantities)
- Finalize stock take (complete/cancel)
- Delete stock take

## 2. Implementation Steps (Specific Sub-tasks)

- [x] 1. Create transformers for snake_case ↔ camelCase conversion
- [x] 2. Implement list() function with filters
- [x] 3. Implement get() function for single stock take
- [x] 4. Implement create() function
- [x] 5. Implement submitCounts() function for batch count updates
- [x] 6. Implement finalize() function for complete/cancel transitions
- [x] 7. Implement delete() function
- [x] 8. Export stockTakeApi object with all methods

## 3. Completion Criteria

- [x] API client created at `frontend/src/lib/api/inventory/stock-take.ts`
- [x] All endpoints from PRD section 8.2 covered
- [x] Proper error handling with typed responses
- [x] snake_case/camelCase transformation working
- [x] No TypeScript compilation errors

## Related Documents

- Mini PRD: `./README.md`
- API Endpoints: PRD Section 8.2

## Files Created/Modified

- `frontend/src/lib/api/inventory/stock-take.ts` (Created)
- `frontend/src/lib/api/inventory/index.ts` (Modified - added export)

## AI Agent Log:

* 2026-01-29 14:30: AI Agent - Task started
    - Reviewed API endpoints from Mini PRD
    - Created API client with transformation functions

* 2026-01-29 15:00: AI Agent - Fixed type casting errors
    - Used `as unknown as ApiResponse<T>` pattern for snake_case to camelCase casting
    - This pattern is documented in COMMON_PATTERNS.md

* 2026-01-29 15:15: AI Agent - Task completed
    - All API methods implemented
    - Transformers handle snake_case/camelCase conversion
    - Status changed to Done
