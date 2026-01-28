# Task 08.11.01: Price List Management UI

**Status:** InProgress_By_Claude
**Priority:** High
**Assignee:** Claude
**Created:** 2026-01-24
**Last Updated:** 2026-01-24

## Objective

Create a complete UI for managing price lists including CRUD operations, price list items with quantity breaks, and customer assignment.

## Database Reference

See `docs/database-erd.dbml` - Tables:
- `price_lists`
- `price_list_items`
- `customer_price_lists`

## User Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    PRICE LIST MANAGEMENT FLOW                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │  Price List  │───▶│  Add Items   │───▶│   Assign     │       │
│  │    List      │    │  (Products)  │    │  Customers   │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│         │                   │                   │                │
│         ▼                   ▼                   ▼                │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │ • View all   │    │ • Fixed price│    │ • Select     │       │
│  │ • Filter by  │    │ • % discount │    │   customers  │       │
│  │   type/status│    │ • Qty breaks │    │ • Set dates  │       │
│  │ • Search     │    │ • By category│    │ • Priority   │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## UI Specifications

### 1. Price List Index Page (`/inventory/pricing/price-lists`)

```
┌─────────────────────────────────────────────────────────────────┐
│  Price Lists                                    [+ New Price List]│
│  Manage pricing for different customer groups                    │
├─────────────────────────────────────────────────────────────────┤
│  [Search...]  [Type ▼] [Status ▼] [Currency ▼]                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ ☐ │ Name          │ Code    │ Type │ Items │ Valid    │ ⋮  ││
│  ├───┼───────────────┼─────────┼──────┼───────┼──────────┼────┤│
│  │ ☐ │ Retail Price  │ RETAIL  │ Sale │ 150   │ Always   │ ⋮  ││
│  │   │ ⭐ Default     │         │      │       │          │    ││
│  ├───┼───────────────┼─────────┼──────┼───────┼──────────┼────┤│
│  │ ☐ │ Wholesale     │ WHOLE   │ Sale │ 150   │ Always   │ ⋮  ││
│  │   │ -15% from base│         │      │       │          │    ││
│  ├───┼───────────────┼─────────┼──────┼───────┼──────────┼────┤│
│  │ ☐ │ VIP Members   │ VIP     │ Sale │ 45    │ Always   │ ⋮  ││
│  │   │ -20% from base│         │      │       │          │    ││
│  ├───┼───────────────┼─────────┼──────┼───────┼──────────┼────┤│
│  │ ☐ │ Tết 2026      │ TET2026 │ Sale │ 30    │ 25/1-10/2│ ⋮  ││
│  │   │ Seasonal promo│         │      │       │ 🟢 Active │    ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
│  Showing 1-4 of 4          [10 ▼] per page    [« ‹] 1 [› »]     │
└─────────────────────────────────────────────────────────────────┘
```

**Features:**
- Search by name/code
- Filter by type (Sale/Purchase), status (Active/Inactive), currency
- Bulk actions (activate, deactivate, delete)
- Quick view of item count and validity
- Default price list indicator

### 2. Create/Edit Price List (`/inventory/pricing/price-lists/new`)

```
┌─────────────────────────────────────────────────────────────────┐
│  ← Back                                                          │
│  New Price List                                       [Cancel]   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─ Basic Information ─────────────────────────────────────────┐│
│  │                                                              ││
│  │  Name *                    Code *                            ││
│  │  [Wholesale Pricing    ]   [WHOLESALE  ]                     ││
│  │                                                              ││
│  │  Description                                                 ││
│  │  [Special pricing for wholesale customers              ]     ││
│  │                                                              ││
│  │  Type *                    Currency *                        ││
│  │  [Sale           ▼]        [VND           ▼]                 ││
│  │                                                              ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                  │
│  ┌─ Pricing Method ────────────────────────────────────────────┐│
│  │                                                              ││
│  │  Based On *                                                  ││
│  │  ○ Fixed prices (set specific prices for each product)      ││
│  │  ● Base price with adjustment                                ││
│  │  ○ Another price list                                        ││
│  │                                                              ││
│  │  ┌─ Adjustment ─────────────────────────────────────────┐   ││
│  │  │  Default adjustment: [-15    ] %                     │   ││
│  │  │  (Negative = discount, positive = markup)            │   ││
│  │  └──────────────────────────────────────────────────────┘   ││
│  │                                                              ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                  │
│  ┌─ Validity ──────────────────────────────────────────────────┐│
│  │                                                              ││
│  │  ☐ Always valid                                              ││
│  │                                                              ││
│  │  Valid From                  Valid To                        ││
│  │  [📅 2026-01-25        ]     [📅 2026-02-10        ]         ││
│  │                                                              ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                  │
│  ┌─ Settings ──────────────────────────────────────────────────┐│
│  │                                                              ││
│  │  Priority: [100     ] (lower = higher priority)              ││
│  │                                                              ││
│  │  ☑ Active                                                    ││
│  │  ☐ Set as default price list                                 ││
│  │                                                              ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                  │
│                                              [Cancel] [Create]   │
└─────────────────────────────────────────────────────────────────┘
```

### 3. Price List Detail / Items (`/inventory/pricing/price-lists/[id]`)

```
┌─────────────────────────────────────────────────────────────────┐
│  ← Back to Price Lists                                           │
│  Wholesale Pricing                              [Edit] [Delete]  │
│  WHOLESALE • Sale • VND • -15% from base                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  [Items] [Customers] [History]                                   │
│  ─────────────────────────────                                   │
│                                                                  │
│  Price List Items (150)                        [+ Add Item]      │
│  ┌──────────────────────────────────────────────────────────────┐
│  │ [Search product...]  [Apply To ▼] [Method ▼]                 │
│  ├──────────────────────────────────────────────────────────────┤
│  │                                                               │
│  │ ┌───────────────────────────────────────────────────────────┐│
│  │ │ Product/Category  │ Min Qty │ Method      │ Value   │ ⋮  ││
│  │ ├───────────────────┼─────────┼─────────────┼─────────┼────┤│
│  │ │ 📦 Laptop Pro 15" │    1    │ Percentage  │ -15%    │ ⋮  ││
│  │ │                   │   10    │ Percentage  │ -20%    │ ⋮  ││
│  │ │                   │   50    │ Percentage  │ -25%    │ ⋮  ││
│  │ ├───────────────────┼─────────┼─────────────┼─────────┼────┤│
│  │ │ 📦 Wireless Mouse │    1    │ Fixed       │ 400,000₫│ ⋮  ││
│  │ ├───────────────────┼─────────┼─────────────┼─────────┼────┤│
│  │ │ 📁 Electronics    │    1    │ Percentage  │ -10%    │ ⋮  ││
│  │ │ (Category)        │         │             │         │    ││
│  │ └───────────────────────────────────────────────────────────┘│
│  │                                                               │
│  └──────────────────────────────────────────────────────────────┘
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 4. Add Price List Item Modal

```
┌─────────────────────────────────────────────────────────────────┐
│  Add Price List Item                                      [×]   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Apply To *                                                      │
│  ○ Specific Product                                              │
│  ○ Product Variant                                               │
│  ● Product Category                                              │
│  ○ All Products                                                  │
│                                                                  │
│  Select Category *                                               │
│  [🔍 Search categories...                              ▼]        │
│  ┌──────────────────────────────────────────────────────────────┐
│  │ ☑ Electronics                                                │
│  │   ☐ Laptops                                                  │
│  │   ☐ Accessories                                              │
│  │ ☐ Office Supplies                                            │
│  └──────────────────────────────────────────────────────────────┘
│                                                                  │
│  ┌─ Quantity Tiers ────────────────────────────────────────────┐│
│  │                                                              ││
│  │  [+ Add Tier]                                                ││
│  │                                                              ││
│  │  Tier 1: Min Qty [1    ] Method [Percentage ▼] Value [-10 ]%││
│  │  Tier 2: Min Qty [10   ] Method [Percentage ▼] Value [-15 ]%││
│  │  Tier 3: Min Qty [50   ] Method [Percentage ▼] Value [-20 ]%││
│  │                                                              ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                  │
│  ┌─ Price Preview ─────────────────────────────────────────────┐│
│  │                                                              ││
│  │  Sample product: Laptop Pro 15" (Base: 25,000,000₫)          ││
│  │                                                              ││
│  │  Qty 1-9:   22,500,000₫ (save 2,500,000₫)                   ││
│  │  Qty 10-49: 21,250,000₫ (save 3,750,000₫)                   ││
│  │  Qty 50+:   20,000,000₫ (save 5,000,000₫)                   ││
│  │                                                              ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                  │
│                                              [Cancel] [Add]      │
└─────────────────────────────────────────────────────────────────┘
```

### 5. Customer Assignment Tab

```
┌─────────────────────────────────────────────────────────────────┐
│  [Items] [Customers] [History]                                   │
│           ──────────                                             │
│                                                                  │
│  Assigned Customers (12)                    [+ Assign Customer]  │
│  ┌──────────────────────────────────────────────────────────────┐
│  │                                                               │
│  │ ┌───────────────────────────────────────────────────────────┐│
│  │ │ Customer          │ Priority │ Valid From │ Valid To │ ⋮  ││
│  │ ├───────────────────┼──────────┼────────────┼──────────┼────┤│
│  │ │ ABC Corporation   │    0     │     -      │    -     │ ⋮  ││
│  │ │ abc@example.com   │          │            │          │    ││
│  │ ├───────────────────┼──────────┼────────────┼──────────┼────┤│
│  │ │ XYZ Trading       │    0     │ 2026-01-01 │ 2026-12-31│ ⋮  ││
│  │ │ xyz@example.com   │          │            │          │    ││
│  │ └───────────────────────────────────────────────────────────┘│
│  │                                                               │
│  └──────────────────────────────────────────────────────────────┘
└─────────────────────────────────────────────────────────────────┘
```

## Component Structure

```
src/lib/components/pricing/
├── PriceListCard.svelte           # Card view for price list
├── PriceListTable.svelte          # Table view with items
├── PriceListItemForm.svelte       # Add/edit item modal
├── PriceListItemRow.svelte        # Editable row for item
├── QuantityTierEditor.svelte      # Manage quantity breaks
├── CustomerAssignmentModal.svelte # Assign customers
├── PricePreview.svelte            # Real-time price preview
└── PriceMethodSelect.svelte       # Compute method selector
```

## TypeScript Types

```typescript
// src/lib/types/pricing.ts

export type PriceListType = 'sale' | 'purchase';
export type BasedOn = 'fixed' | 'base_price' | 'other_pricelist';
export type ComputeMethod = 'fixed' | 'percentage' | 'formula' | 'margin';
export type ApplyTo = 'product' | 'variant' | 'category' | 'all';
export type RoundingMethod = 'none' | 'round_up' | 'round_down' | 'round_nearest' | 'round_to_99';

export interface PriceList {
  priceListId: string;
  tenantId: string;
  name: string;
  code: string;
  description?: string;
  currencyCode: string;
  priceListType: PriceListType;
  basedOn: BasedOn;
  parentPriceListId?: string;
  defaultPercentage: number;
  validFrom?: Date;
  validTo?: Date;
  priority: number;
  isDefault: boolean;
  isActive: boolean;
  createdAt: Date;
  updatedAt: Date;
  
  // Computed
  itemCount?: number;
  customerCount?: number;
}

export interface PriceListItem {
  itemId: string;
  tenantId: string;
  priceListId: string;
  applyTo: ApplyTo;
  productId?: string;
  variantId?: string;
  categoryId?: string;
  minQuantity: number;
  maxQuantity?: number;
  computeMethod: ComputeMethod;
  fixedPrice?: number;
  percentage?: number;
  marginPercentage?: number;
  formula?: string;
  roundingMethod: RoundingMethod;
  roundingPrecision: number;
  validFrom?: Date;
  validTo?: Date;
  
  // Joined data
  product?: { name: string; sku: string };
  variant?: { sku: string; attributes: Record<string, string> };
  category?: { name: string };
}

export interface CustomerPriceList {
  id: string;
  tenantId: string;
  customerId: string;
  priceListId: string;
  priority: number;
  validFrom?: Date;
  validTo?: Date;
  
  // Joined
  customer?: { name: string; email: string };
}
```

## API Endpoints (Mock)

```typescript
// src/lib/api/pricing.ts

export const priceListsApi = {
  list: (params?: ListParams) => Promise<PaginatedResponse<PriceList>>,
  get: (id: string) => Promise<PriceList>,
  create: (data: CreatePriceListInput) => Promise<PriceList>,
  update: (id: string, data: UpdatePriceListInput) => Promise<PriceList>,
  delete: (id: string) => Promise<void>,
  setDefault: (id: string) => Promise<void>,
};

export const priceListItemsApi = {
  list: (priceListId: string, params?: ListParams) => Promise<PaginatedResponse<PriceListItem>>,
  create: (priceListId: string, data: CreateItemInput) => Promise<PriceListItem>,
  update: (itemId: string, data: UpdateItemInput) => Promise<PriceListItem>,
  delete: (itemId: string) => Promise<void>,
  bulkCreate: (priceListId: string, items: CreateItemInput[]) => Promise<PriceListItem[]>,
};

export const customerPriceListsApi = {
  list: (priceListId: string) => Promise<CustomerPriceList[]>,
  assign: (priceListId: string, customerId: string, data: AssignInput) => Promise<void>,
  unassign: (priceListId: string, customerId: string) => Promise<void>,
};
```

## Acceptance Criteria

1. [ ] User can view list of all price lists with search/filter
2. [ ] User can create a new price list with all options
3. [ ] User can add items to price list (product, variant, category, all)
4. [ ] User can set quantity breaks (multiple tiers per item)
5. [ ] User can see real-time price preview
6. [ ] User can assign/unassign customers to price lists
7. [ ] User can set validity dates
8. [ ] User can set one price list as default
9. [ ] All forms have proper validation
10. [ ] Mobile responsive design

## Dependencies

- Task 08.04 (Product Management) - For product/category selectors
- shadcn-svelte components: Table, Dialog, Select, Input, Button, Tabs, DatePicker

---

## Sub-Tasks

### Phase 1: Foundation
- [x] Create TypeScript types (`frontend/src/lib/types/pricing.ts`)
- [x] Create pricing API client (`frontend/src/lib/api/pricing/`)
- [x] Create pricing stores (`frontend/src/lib/stores/pricing.svelte.ts`)

### Phase 2: Core Pages
- [x] Create Price Lists index page (`/inventory/pricing/price-lists`)
- [x] Create New Price List page (`/inventory/pricing/price-lists/new`)
- [x] Create Price List detail page (`/inventory/pricing/price-lists/[id]`)
- [x] Create Edit Price List page (`/inventory/pricing/price-lists/[id]/edit`)

### Phase 3: Components
- [x] Create `PriceListTable.svelte` component
- [x] Create `PriceListForm.svelte` component
- [x] Create `PriceListItemForm.svelte` modal
- [x] Create `CustomerAssignmentModal.svelte` component
- [x] Create `PricePreview.svelte` component

### Phase 4: API Routes
- [x] Create `/api/v1/pricing/price-lists` endpoint (CRUD)
- [x] Create `/api/v1/pricing/price-lists/[id]/items` endpoint
- [x] Create `/api/v1/pricing/price-lists/[id]/customers` endpoint

### Phase 5: Testing & Polish
- [ ] Test all CRUD operations in browser
- [ ] Verify mobile responsive design
- [ ] Add loading states and error handling

---

## AI Agent Log

| Timestamp | Agent | Action | Notes |
|-----------|-------|--------|-------|
| 2026-01-24 21:43 | Claude | Started task | Created TypeScript types in `pricing.ts` |
| 2026-01-24 21:44 | Claude | Updated task file | Added sub-tasks and AI Agent Log per folder-tasks workflow |
| 2026-01-24 21:50 | Claude | Completed Phase 1 | Created API clients (price-lists.ts, pricing-rules.ts, calculation.ts) and stores (pricing.svelte.ts) |
| 2026-01-24 22:10 | Claude | Completed Phase 2 | Created all Price List pages (index, new, detail, edit) with Svelte 5 runes. All pages verified with svelte-autofixer |
| 2026-01-24 22:25 | Claude | Completed Phase 3 | Created 5 reusable components: PriceListTable, PriceListForm, PriceListItemForm, CustomerAssignmentModal, PricePreview. All verified with svelte-autofixer |
| 2026-01-24 22:35 | Claude | Completed Phase 4 | Created mock API routes for price-lists CRUD, items, and customer assignments |
