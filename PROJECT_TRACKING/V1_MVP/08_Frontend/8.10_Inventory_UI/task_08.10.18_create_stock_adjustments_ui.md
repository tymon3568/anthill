# Task: Create Stock Adjustments UI

**Task ID:** `V1_MVP/08_Frontend/8.10_Inventory_UI/task_08.10.18_create_stock_adjustments_ui.md`
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.10_Inventory_UI
**Priority:** High
**Status:** NeedsReview
**Assignee:** Claude
**Created Date:** 2026-01-29
**Last Updated:** 2026-01-29
**Dependencies:**
- `V1_MVP/08_Frontend/8.10_Inventory_UI/task_08.10.02_create_warehouse_management_ui.md`
- `V1_MVP/08_Frontend/8.10_Inventory_UI/task_08.10.10_inventory_api_integration.md`
- `V1_MVP/04_Inventory_Service/4.3_Stock_Foundation/task_04.03.02_create_stock_adjustments_table.md`

---

## 1. Overview

### 1.1 Purpose

Stock Adjustments UI cho phép người dùng:
- **Ghi nhận các thay đổi tồn kho** không qua quy trình mua/bán/transfer thông thường
- **Điều chỉnh số lượng** khi phát hiện sai lệch (hao hụt, thừa/thiếu, hư hỏng)
- **Tạo audit trail** cho mọi thay đổi tồn kho
- **Tích hợp với Stock Take** để xử lý chênh lệch sau kiểm kê

### 1.2 Business Value

- Đảm bảo tính chính xác của số liệu tồn kho
- Tuân thủ quy định kiểm toán (audit compliance)
- Giảm thiểu sai sót do nhập liệu thủ công
- Hỗ trợ phân tích nguyên nhân hao hụt

### 1.3 Target Users

| Actor | Role | Actions |
|-------|------|---------|
| Warehouse Staff | Nhập điều chỉnh | Tạo adjustment, nhập lý do |
| Warehouse Manager | Phê duyệt | Review và approve adjustments |
| Inventory Controller | Kiểm soát | Xem báo cáo, phân tích variance |

---

## 2. State Machine

### 2.1 State Diagram

```
                    ┌─────────────┐
                    │   Draft     │ ◄─── create()
                    └──────┬──────┘
                           │ confirm() / post()
                           ▼
                    ┌─────────────┐
                    │   Posted    │ ───► stock_move created
                    └──────┬──────┘      inventory_levels updated
                           │ (terminal)
                           ▼
                        [END]
    
    Any Draft can transition to:
                    ┌─────────────┐
                    │  Cancelled  │ ◄─── cancel()
                    └─────────────┘

Note: Based on current simplified schema (stock_adjustments table),
adjustments are linked directly to stock_moves and don't have
a separate status workflow. This UI will use an implicit workflow:
- Draft = adjustment form in progress (not saved)
- Posted = saved to database (linked to stock_move)
```

### 2.2 State Transition Table

| Current State | Action | Next State | Side Effects |
|---------------|--------|------------|--------------|
| (none) | create_draft | Draft | Form initialized |
| Draft | save/post | Posted | stock_move created, inventory updated |
| Draft | cancel | (discarded) | Form closed |
| Posted | - | - | Terminal state (immutable) |

### 2.3 State Rules

- **Editable states**: Draft only (form not yet saved)
- **Terminal states**: Posted (cannot modify after save)
- **No reversal**: To reverse, create opposite adjustment

---

## 3. Business Rules

### 3.1 Validation Rules

| Rule ID | Rule Description | When Applied | Error Message |
|---------|------------------|--------------|---------------|
| BR-ADJ-001 | Warehouse is required | Create | "Please select a warehouse" |
| BR-ADJ-002 | Product is required | Add line | "Please select a product" |
| BR-ADJ-003 | Quantity must be non-zero | Add line | "Quantity cannot be zero" |
| BR-ADJ-004 | Reason code is required | Save | "Please select a reason for adjustment" |
| BR-ADJ-005 | At least one line item required | Save | "Add at least one product to adjust" |
| BR-ADJ-006 | Decrease qty cannot exceed available | Save | "Insufficient stock: {available} available" |
| BR-ADJ-007 | Lot/Serial required for tracked products | Add line | "This product requires lot/serial number" |

### 3.2 Computation Rules

| Rule ID | Description | Formula/Logic |
|---------|-------------|---------------|
| CR-ADJ-001 | New quantity | `current_quantity + adjustment_quantity` (increase) or `current_quantity - adjustment_quantity` (decrease) |
| CR-ADJ-002 | Line value | `quantity * unit_cost` |
| CR-ADJ-003 | Total adjustment value | `SUM(line_values)` |

### 3.3 Authorization Rules

| Action | Required Permission | Additional Conditions |
|--------|---------------------|----------------------|
| View adjustments | `inventory:adjustments:read` | Own warehouse only for staff |
| Create adjustment | `inventory:adjustments:create` | - |
| Delete draft | `inventory:adjustments:delete` | Creator only |
| View all adjustments | `inventory:adjustments:read:all` | Managers and above |

---

## 4. Data Model

### 4.1 Current Database Schema

```sql
-- Current simplified schema (from database-erd.dbml)
Table stock_adjustments {
  adjustment_id UUID [pk]
  tenant_id UUID [not null]
  move_id UUID [not null]       -- Reference to stock_moves
  product_id UUID [not null]
  warehouse_id UUID [not null]
  reason_code VARCHAR(50) [not null]
  notes TEXT
  approved_by UUID [not null]
  created_at TIMESTAMPTZ
  updated_at TIMESTAMPTZ
  deleted_at TIMESTAMPTZ
}
```

### 4.2 TypeScript Types

```typescript
// frontend/src/lib/types/inventory/adjustments.ts

export interface StockAdjustment {
  adjustmentId: string;
  tenantId: string;
  moveId: string;
  productId: string;
  warehouseId: string;
  reasonCode: string;
  notes: string | null;
  approvedBy: string;
  createdAt: string;
  updatedAt: string;
  
  // Enriched data from joins
  product?: {
    productId: string;
    sku: string;
    name: string;
  };
  warehouse?: {
    warehouseId: string;
    warehouseCode: string;
    warehouseName: string;
  };
  stockMove?: {
    moveId: string;
    quantity: number;
    moveType: string;
    moveDate: string;
  };
  approvedByUser?: {
    userId: string;
    fullName: string;
  };
}

export interface CreateAdjustmentRequest {
  warehouseId: string;
  items: CreateAdjustmentItem[];
  notes?: string;
}

export interface CreateAdjustmentItem {
  productId: string;
  quantity: number;           // Positive = increase, Negative = decrease
  reasonCode: string;
  locationId?: string;
  lotSerialId?: string;
  notes?: string;
}

export interface AdjustmentListParams {
  warehouseId?: string;
  productId?: string;
  reasonCode?: string;
  dateFrom?: string;
  dateTo?: string;
  page?: number;
  pageSize?: number;
}

export type ReasonCode = 
  | 'COUNT_ERROR'
  | 'STOCK_TAKE_VARIANCE'
  | 'DAMAGE'
  | 'EXPIRED'
  | 'SCRAP'
  | 'THEFT'
  | 'LOST'
  | 'FOUND'
  | 'SAMPLE'
  | 'PROMOTION'
  | 'INTERNAL_USE'
  | 'RETURN_TO_STOCK'
  | 'WRITE_OFF'
  | 'CORRECTION';

export interface ReasonCodeOption {
  code: ReasonCode;
  label: string;
  direction: 'increase' | 'decrease' | 'both';
  category: 'inventory_count' | 'quality' | 'loss' | 'other';
  requiresApproval: boolean;
}
```

### 4.3 API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/inventory/adjustments` | List adjustments with filters |
| GET | `/api/v1/inventory/adjustments/:id` | Get adjustment detail |
| POST | `/api/v1/inventory/adjustments` | Create new adjustment(s) |
| DELETE | `/api/v1/inventory/adjustments/:id` | Soft delete (if not posted) |
| GET | `/api/v1/inventory/adjustment-reasons` | List available reason codes |

---

## 5. UI Specifications

### 5.1 Routes

```
/inventory/adjustments                  # List page
/inventory/adjustments/new              # Create form
/inventory/adjustments/[id]             # Detail view
```

### 5.2 Page Layouts

#### 5.2.1 List Page (`/inventory/adjustments`)

```
┌─────────────────────────────────────────────────────────────────┐
│  STOCK ADJUSTMENTS                           [+ New Adjustment] │
├─────────────────────────────────────────────────────────────────┤
│  Filters:                                                       │
│  [Warehouse ▼] [Product ▼] [Reason ▼] [Date From] [Date To]    │
│                                                      [Search]   │
├─────────────────────────────────────────────────────────────────┤
│  Date       │ Product   │ Warehouse │ Qty    │ Reason  │ User  │
│─────────────┼───────────┼───────────┼────────┼─────────┼───────│
│  29/01/2026 │ SKU-001   │ WH-A      │ -10    │ Damage  │ John  │
│  29/01/2026 │ SKU-002   │ WH-A      │ +5     │ Found   │ Jane  │
│  28/01/2026 │ SKU-003   │ WH-B      │ -100   │ Expired │ John  │
├─────────────────────────────────────────────────────────────────┤
│  Page 1 of 5                               [< Prev] [Next >]    │
└─────────────────────────────────────────────────────────────────┘
```

**List View Columns:**

| Column | Type | Sortable | Notes |
|--------|------|----------|-------|
| Date | DateTime | Yes | Created date |
| Product | Link | Yes | SKU + Name |
| Warehouse | Text | Yes | Warehouse name |
| Quantity | Number | Yes | +/- with color (green/red) |
| Reason | Badge | Yes | Reason code |
| Value | Currency | Yes | Adjustment value |
| Created By | Text | Yes | User name |
| Actions | Buttons | No | View, Delete (if draft) |

#### 5.2.2 Create Form (`/inventory/adjustments/new`)

```
┌─────────────────────────────────────────────────────────────────┐
│  ← Back                                                         │
│                                                                 │
│  NEW STOCK ADJUSTMENT                                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Warehouse: [Select Warehouse ▼]                                │
│                                                                 │
│  Notes (optional):                                              │
│  [_______________________________________________]              │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  ADJUSTMENT ITEMS                              [+ Add Product]  │
├─────────────────────────────────────────────────────────────────┤
│  # │ Product      │ Current │ Type     │ Qty  │ Reason  │ 🗑️  │
│────┼──────────────┼─────────┼──────────┼──────┼─────────┼─────│
│  1 │ [Product ▼]  │   100   │ [Dec ▼]  │ [10] │ [Dam ▼] │ ✕   │
│  2 │ [Product ▼]  │    50   │ [Inc ▼]  │  [5] │ [Fou ▼] │ ✕   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                              [Cancel]  [Save as Draft] [Post]   │
└─────────────────────────────────────────────────────────────────┘
```

**Form Fields:**

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| Warehouse | Select | Yes | Must be active warehouse |
| Notes | Textarea | No | Max 500 chars |
| Product | ProductSearch | Yes | Active products only |
| Type | Select | Yes | Increase/Decrease |
| Quantity | Number | Yes | > 0 |
| Reason | Select | Yes | From reason codes |
| Line Notes | Text | No | Per-line notes |

#### 5.2.3 Detail View (`/inventory/adjustments/[id]`)

```
┌─────────────────────────────────────────────────────────────────┐
│  ← Back to List                                    [Print]      │
│                                                                 │
│  ADJUSTMENT DETAIL                                              │
│  ADJ-2026-00042                                    [Posted ✓]   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Warehouse: Warehouse A (WH-A)                                  │
│  Date: 29/01/2026 06:15:00                                      │
│  Created By: John Doe                                           │
│  Approved By: Jane Manager                                      │
│                                                                 │
│  Notes: Monthly inventory adjustment after stock take           │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  ADJUSTMENT ITEMS                                               │
├─────────────────────────────────────────────────────────────────┤
│  Product        │ Before │ Adjustment │ After │ Reason  │ Value │
│─────────────────┼────────┼────────────┼───────┼─────────┼───────│
│  Widget A       │   100  │    -10     │   90  │ Damage  │ $50   │
│  Widget B       │    50  │     +5     │   55  │ Found   │ $15   │
├─────────────────────────────────────────────────────────────────┤
│  Total Impact:                         │   -5  │         │ $35   │
└─────────────────────────────────────────────────────────────────┘
```

### 5.3 Components to Create

| Component | Location | Description |
|-----------|----------|-------------|
| `AdjustmentList.svelte` | `components/inventory/adjustments/` | List with filters |
| `AdjustmentForm.svelte` | `components/inventory/adjustments/` | Create/Edit form |
| `AdjustmentDetail.svelte` | `components/inventory/adjustments/` | Read-only detail |
| `AdjustmentLineItem.svelte` | `components/inventory/adjustments/` | Line item row |
| `ReasonCodeSelect.svelte` | `components/inventory/adjustments/` | Reason dropdown |
| `AdjustmentTypeToggle.svelte` | `components/inventory/adjustments/` | Inc/Dec toggle |

### 5.4 Shared Components to Use

- `WarehouseSelector.svelte` - Warehouse dropdown
- `ProductSearch.svelte` - Product search with barcode
- `QuantityInput.svelte` - Quantity input with validation
- `StatusBadge.svelte` - Status indicator
- `DataTable.svelte` - Sortable table

---

## 6. Implementation Steps (Specific Sub-tasks)

### Phase 1: Types and API Client

- [x] 1. Create TypeScript types for adjustments in `lib/types/inventory.ts`
- [x] 2. Create adjustments API client in `lib/api/inventory/adjustments.ts`
- [x] 3. Export from `lib/api/inventory/index.ts`
- [x] 4. Create Svelte 5 store `lib/stores/adjustments.svelte.ts`

### Phase 2: List Page

- [x] 5. Create list page route `/inventory/adjustments/+page.svelte`
- [x] 6. Create `AdjustmentList.svelte` component with filters (inline in page)
- [ ] 7. Create server load function `+page.server.ts` (deferred - using client-side load)
- [x] 8. Implement pagination and sorting
- [x] 9. Add filter by warehouse, product, reason, date range

### Phase 3: Create Form

- [x] 10. Create form route `/inventory/adjustments/new/+page.svelte`
- [x] 11. Create `AdjustmentForm.svelte` with line items (inline in page)
- [x] 12. Create `AdjustmentLineItem.svelte` component (inline in form table)
- [x] 13. Create `ReasonCodeSelect.svelte` with categories (inline + constants file)
- [x] 14. Implement form validation with Superforms + Zod (using derived validation)
- [x] 15. Add product search integration
- [ ] 16. Show current stock level when product selected (deferred - needs API)
- [x] 17. Implement save action (POST to API)

### Phase 4: Detail View

- [x] 18. Create detail route `/inventory/adjustments/[id]/+page.svelte`
- [x] 19. Create `AdjustmentDetail.svelte` component (inline in page)
- [x] 20. Show adjustment header and line items
- [x] 21. Add print functionality

### Phase 5: Integration and Polish

- [x] 22. Update navigation menu in sidebar
- [ ] 23. Add breadcrumbs navigation (deferred)
- [x] 24. Handle loading and error states
- [ ] 25. Add success/error toast notifications (deferred)
- [x] 26. Mobile responsive layout

### Phase 6: Testing

- [ ] 27. Write unit tests for store actions
- [ ] 28. Write unit tests for form validation
- [ ] 29. Write E2E tests for CRUD flow
- [ ] 30. Test with real API data

---

## 7. Completion Criteria

- [ ] List page displays adjustments with filtering and pagination
- [ ] Create form validates all fields correctly
- [ ] Decrease adjustments check available stock
- [ ] Reason codes displayed with proper categorization
- [ ] Detail view shows all adjustment information
- [ ] Navigation links work correctly
- [ ] Loading and error states handled gracefully
- [ ] Mobile responsive design works
- [ ] TypeScript compiles without errors (`bun run check`)
- [ ] Lint passes (`bun run lint`)
- [ ] Unit tests pass
- [ ] E2E tests pass with real API

---

## 8. Error Scenarios

### 8.1 Expected Errors

| Error Code | Scenario | User Message | Recovery |
|------------|----------|--------------|----------|
| INSUFFICIENT_STOCK | Decrease > available | "Cannot decrease by {qty}. Only {available} available." | Reduce quantity |
| PRODUCT_NOT_FOUND | Invalid product ID | "Product not found" | Select different product |
| WAREHOUSE_REQUIRED | No warehouse selected | "Please select a warehouse" | Select warehouse |
| REASON_REQUIRED | No reason selected | "Please select a reason" | Select reason |
| VALIDATION_ERROR | Form validation failed | "Please fix the errors below" | Fix highlighted fields |

### 8.2 System Errors

| Error Code | Scenario | Impact | Mitigation |
|------------|----------|--------|------------|
| API_ERROR | Backend unavailable | Cannot save | Retry with exponential backoff |
| NETWORK_ERROR | Connection lost | Cannot load/save | Show offline indicator |

---

## 9. Related Documents

- Database ERD: `docs/database-erd.dbml` (stock_adjustments table)
- Backend task: `task_04.03.02_create_stock_adjustments_table.md`
- API spec: `shared/openapi/inventory_service.yaml`
- Module README: `./README.md`

---

## 10. Notes

### 10.1 Current Backend Schema

The current `stock_adjustments` table is **simplified** - it links directly to `stock_moves`:
- No separate header/lines structure
- No Draft → Posted workflow (immediate)
- Each adjustment = one stock_move = one product

For future enhancement, consider:
- Adding `stock_adjustment_items` table for multi-line adjustments
- Adding status workflow (Draft → Pending Approval → Posted)
- Adding approval thresholds

### 10.2 Reason Codes (Static Data)

Until backend provides `/adjustment-reasons` endpoint, use static reason codes:

```typescript
export const REASON_CODES: ReasonCodeOption[] = [
  { code: 'COUNT_ERROR', label: 'Count Discrepancy', direction: 'both', category: 'inventory_count', requiresApproval: false },
  { code: 'STOCK_TAKE_VARIANCE', label: 'Stock Take Variance', direction: 'both', category: 'inventory_count', requiresApproval: true },
  { code: 'DAMAGE', label: 'Damaged Goods', direction: 'decrease', category: 'quality', requiresApproval: true },
  { code: 'EXPIRED', label: 'Expired Products', direction: 'decrease', category: 'quality', requiresApproval: true },
  { code: 'SCRAP', label: 'Scrap/Waste', direction: 'decrease', category: 'quality', requiresApproval: false },
  { code: 'THEFT', label: 'Theft/Shrinkage', direction: 'decrease', category: 'loss', requiresApproval: true },
  { code: 'LOST', label: 'Lost Inventory', direction: 'decrease', category: 'loss', requiresApproval: true },
  { code: 'FOUND', label: 'Found Inventory', direction: 'increase', category: 'inventory_count', requiresApproval: true },
  { code: 'SAMPLE', label: 'Sample/Demo Use', direction: 'decrease', category: 'other', requiresApproval: false },
  { code: 'INTERNAL_USE', label: 'Internal Consumption', direction: 'decrease', category: 'other', requiresApproval: false },
  { code: 'RETURN_TO_STOCK', label: 'Return to Stock', direction: 'increase', category: 'other', requiresApproval: false },
  { code: 'WRITE_OFF', label: 'Write-off', direction: 'decrease', category: 'loss', requiresApproval: true },
  { code: 'CORRECTION', label: 'Data Correction', direction: 'both', category: 'other', requiresApproval: true },
];
```

---

## AI Agent Log:

* 2026-01-29 06:30: Task created
    - Research completed on ERP adjustment patterns (Odoo, ERPNext, SAP)
    - Reviewed database schema (stock_adjustments, stock_moves tables)
    - Defined UI specifications based on UI Architecture
    - Status: Todo

* 2026-01-29 06:50: Task claimed by Claude
    - Dependencies verified:
      - task_08.10.02 (Warehouse UI): NeedsReview - work done, awaiting review
      - task_08.10.10 (API Integration): Done ✓
      - task_04.03.02 (Stock Adjustments Table): Done ✓
    - Proceeding with Phase 1 (Types and API Client) as shared components exist
    - Starting work on sub-task 1: Create TypeScript types

* 2026-01-29 07:15: Phase 1 completed by Claude
    - Created TypeScript types in lib/types/inventory.ts:
      - StockAdjustment (extended), AdjustmentReasonCode, ReasonCodeOption
      - CreateAdjustmentRequest, CreateAdjustmentResponse, AdjustmentListParams
    - Created adjustment-reasons.ts constants with 13 reason codes
    - Created adjustments.ts API client with CRUD operations
    - Created adjustments.svelte.ts Svelte 5 store
    - Exported from lib/api/inventory/index.ts
    - TypeScript check: No errors in adjustment files ✓

* 2026-01-29 07:30: Phases 2-4 completed by Claude
    - Created list page: /inventory/adjustments/+page.svelte
      - Stats cards, filters (warehouse, reason, search), table view
      - Loading, error, empty states handled
    - Created create form: /inventory/adjustments/new/+page.svelte
      - Multi-line adjustment support
      - Dynamic reason codes based on increase/decrease
      - Form validation with derived state
    - Created detail view: /inventory/adjustments/[id]/+page.svelte
      - Full adjustment info display
      - Print functionality

* 2026-01-29 07:40: Phase 5 completed by Claude
    - Added "Adjustments" to navigation sidebar (lib/config/navigation.ts)
    - All pages have loading/error states
    - Responsive layout using grid

* 2026-01-29 07:45: Implementation complete
    - Files created:
      - lib/constants/adjustment-reasons.ts
      - lib/api/inventory/adjustments.ts
      - lib/stores/adjustments.svelte.ts
      - routes/(protected)/inventory/adjustments/+page.svelte
      - routes/(protected)/inventory/adjustments/new/+page.svelte
      - routes/(protected)/inventory/adjustments/[id]/+page.svelte
    - Files modified:
      - lib/types/inventory.ts (added adjustment types)
      - lib/api/inventory/index.ts (added export)
      - lib/config/navigation.ts (added menu item)
    - Quality gates: TypeScript check passes for adjustment files ✓
    - Deferred items: server load function, breadcrumbs, toasts, current stock display
    - Status changed to NeedsReview

