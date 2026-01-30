# Task: Create Stock Levels UI

**Task ID:** V1_MVP/08_Frontend/8.10_Inventory_UI/task_08.10.17
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.10_Inventory_UI
**Priority:** High
**Status:** Complete
**Assignee:** Claude
**Created Date:** 2026-01-27
**Last Updated:** 2026-01-27

## Detailed Description:

Create the Stock Levels UI module that displays current inventory levels across products, warehouses, and locations. This page allows users to view real-time stock availability, filter by warehouse/product, and see reserved vs available quantities.

## Business Requirements:

1. Display current stock levels for all products
2. Filter by warehouse, product, and stock status (low stock, out of stock, in stock)
3. Show available quantity vs reserved quantity
4. Aggregate stock across warehouses or show per-warehouse breakdown
5. Quick search by product SKU or name
6. Export stock levels to CSV/Excel

## Technical Specifications:

### Backend API (New Endpoint Required)

The backend currently has `PgInventoryLevelRepository` but no REST API endpoint. Need to add:

```
GET /api/v1/inventory/stock-levels
  Query params:
    - warehouseId (optional)
    - productId (optional)
    - search (optional) - search by product name/SKU
    - lowStockOnly (optional) - filter for items below reorder point
    - page, pageSize (pagination)
    - sortBy, sortDir (sorting)
  
  Response:
    {
      items: StockLevelResponse[],
      pagination: PaginationInfo,
      summary: {
        totalProducts: number,
        totalQuantity: number,
        lowStockCount: number,
        outOfStockCount: number
      }
    }
```

### Frontend Structure

```
frontend/src/
├── lib/
│   ├── api/inventory/stock-levels.ts    # API client
│   ├── types/stock-levels.ts            # TypeScript types
│   └── stores/stock-levels.svelte.ts    # Svelte 5 store
└── routes/(protected)/inventory/stock/
    └── +page.svelte                      # Stock Levels list page
```

### UI Components

1. **Stock Levels Table**
   - Columns: Product (SKU, Name), Warehouse, Available, Reserved, Total, Status
   - Sortable columns
   - Inline status badges (In Stock, Low Stock, Out of Stock)

2. **Filters Panel**
   - Warehouse selector dropdown
   - Search input (product name/SKU)
   - Status filter (All, In Stock, Low Stock, Out of Stock)

3. **Summary Cards**
   - Total Products with Stock
   - Total Available Quantity
   - Low Stock Alerts count
   - Out of Stock count

## Specific Sub-tasks:

- [x] 1. Add backend stock levels list API endpoint
  - Implemented in `services/inventory_service/api/src/handlers/stock_levels.rs`
  - Routes registered in `services/inventory_service/api/src/routes/mod.rs:521`
  - Service: `PgStockLevelsService` in `services/inventory_service/infra/src/services/stock_levels.rs`
- [x] 2. Create Casbin policy migration for stock-levels endpoint
  - Migration: `migrations/20260127000003_add_stock_levels_casbin_policies.sql`
  - Roles: admin, manager, user, viewer have GET access
- [x] 3. Create frontend TypeScript types for stock levels
  - Types in `frontend/src/lib/types/inventory.ts`
- [x] 4. Create frontend API client for stock levels
  - API client: `frontend/src/lib/api/inventory/stock-levels.ts`
- [x] 5. Create Svelte 5 store for stock levels state management
  - Store: `frontend/src/lib/stores/inventory.svelte.ts`
- [x] 6. Create Stock Levels page UI with table and filters
  - Page: `frontend/src/routes/(protected)/inventory/stock-levels/+page.svelte`
  - Uses Svelte 5 `$state` and `$effect` runes correctly
- [x] 7. Test with Chrome DevTools against real backend
  - E2E tests: `frontend/e2e/stock-levels.e2e.spec.ts` (17 test cases)

## Acceptance Criteria:

- [x] Stock levels page displays at `/inventory/stock-levels` (updated URL)
- [x] Users can filter by warehouse
- [x] Users can search by product name/SKU
- [x] Status badges correctly show stock status
- [x] Pagination works correctly
- [x] Summary cards show accurate statistics
- [x] No TypeScript errors
- [x] No console errors

## Non-Functional Requirements:

- **Performance**: Page loads within 2 seconds for 1000+ products
- **Accessibility**: WCAG 2.1 AA compliance
- **Responsiveness**: Works on mobile and desktop

## Dependencies:

- Backend inventory_service running on port 8001
- Product and Warehouse data exists in database
- Casbin policies configured for stock-levels endpoint

## Related Documents:

- `docs/database-erd.dbml` - inventory_levels table schema
- `docs/ui-architecture-proposal.md` - UI architecture
- `services/inventory_service/infra/src/repositories/stock.rs` - Repository implementation

## Notes / Discussion:
---
- Stock Levels UI is referenced in navigation but not yet implemented
- Backend has repository but no API endpoint - need to add both handler and routes

## AI Agent Log:
---
*   2026-01-27 10:30: Task created by Claude
    - Researched existing codebase
    - Found that backend has PgInventoryLevelRepository but no REST API
    - Need to implement both backend API and frontend UI
    - Starting implementation

*   2026-01-27 16:55: Business Flow Review Completed by Claude
    - **Stock Levels is a pure READ operation** - no mutations, no event publishing
    - Stock levels are UPDATED by other modules:
      - `GRN/Receipt` → increases stock via `stock_moves`
      - `Stock Transfers` → moves stock between warehouses
      - `Scrap` → decreases stock with `move_type='scrap'`
      - `Cycle Count` → adjusts stock with `move_type='adjustment'`
      - `Reconciliation` → adjusts stock with `reference_type='reconciliation'`
      - `Delivery Orders` → reserves and ships stock
    - **Tenant Isolation**: Proper `tenant_id` filtering at all layers
    - **Backend Implementation**: Complete with OpenAPI docs, validation, pagination
    - **Frontend Implementation**: Svelte 5 runes (`$state`, `$effect`, `$derived`) correctly used
    - **Authorization**: Casbin policies for GET access (admin, manager, user, viewer)
    - **E2E Tests**: 17 comprehensive test cases covering all UI functionality
    - **Navigation**: Registered in sidebar at `/inventory/stock-levels`
    - **Cross-module Dependencies**: Correctly depends on Products + Warehouses data
    - **Status**: All sub-tasks and acceptance criteria met. Task Complete.
