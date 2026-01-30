# Task 08.10.16: Create Variants Management UI

## Task Overview

| Field | Value |
|-------|-------|
| Task ID | 08.10.16 |
| Task Name | Create Variants Management UI |
| Module | 8.10 Inventory UI |
| Priority | High |
| Status | ✅ Done |
| Dependencies | 08.04.01, 08.04.02 (Product Management) |
| Estimated Effort | 4 hours |

## Description

Create a dedicated Variants management page under `/inventory/variants` that provides a centralized view of all product variants across the inventory. This complements the existing variant CRUD in the Product Detail page by providing a cross-product variant search and management experience.

## Database Schema Reference

From `docs/database-erd.dbml`:

```sql
Table product_variants {
  variant_id UUID [pk, default: `uuid_generate_v7()`]
  tenant_id UUID [not null]
  parent_product_id UUID [not null]
  variant_attributes JSONB [not null, default: '{}', note: 'e.g., {color: "red", size: "L"}']
  sku TEXT [not null]
  barcode TEXT
  price_difference BIGINT [not null, default: 0, note: 'Price delta from parent product']
  is_active BOOLEAN [not null, default: true]
  created_at TIMESTAMPTZ [not null, default: `NOW()`]
  updated_at TIMESTAMPTZ [not null, default: `NOW()`]
  deleted_at TIMESTAMPTZ

  Indexes {
    (tenant_id, sku) [unique, name: 'product_variants_unique_sku_per_tenant']
    (tenant_id, parent_product_id, variant_attributes) [unique]
    (tenant_id, parent_product_id) [name: 'idx_product_variants_tenant_parent']
  }
}
```

## Acceptance Criteria

### Must Have
- [x] Variant types defined in `frontend/src/lib/types/inventory.ts`
- [x] Variant API client at `frontend/src/lib/api/inventory/variants.ts`
- [x] variantStore in `frontend/src/lib/stores/inventory.svelte.ts`
- [x] Variants list page at `/inventory/variants`
- [x] Search by SKU, barcode, parent product name
- [x] Filter by active status
- [x] Link to parent product
- [x] Unit tests for API client

### Should Have
- [ ] Bulk activate/deactivate variants
- [ ] Export variants to CSV

### Could Have
- [ ] Barcode scanning integration
- [ ] Variant image upload

## Technical Implementation

### Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `frontend/src/lib/types/inventory.ts` | Modify | Add Variant types |
| `frontend/src/lib/api/inventory/variants.ts` | Create | Variant API client |
| `frontend/src/lib/stores/inventory.svelte.ts` | Modify | Add variantStore |
| `frontend/src/routes/(protected)/inventory/variants/+page.svelte` | Create | Variants list page |
| `frontend/src/lib/api/inventory/variants.test.ts` | Create | Unit tests |

### Type Definitions

```typescript
// VariantResponse - matches API response
export interface VariantResponse {
  variantId: string;
  tenantId: string;
  parentProductId: string;
  variantAttributes: Record<string, string>;
  sku: string;
  barcode?: string | null;
  priceDifference: number; // BIGINT as cents
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
  // Joined fields from API
  parentProductName?: string;
  parentProductSku?: string;
}

export interface VariantCreateRequest {
  parentProductId: string;
  sku: string;
  barcode?: string | null;
  variantAttributes: Record<string, string>;
  priceDifference?: number;
  isActive?: boolean;
}

export interface VariantUpdateRequest {
  sku?: string | null;
  barcode?: string | null;
  variantAttributes?: Record<string, string> | null;
  priceDifference?: number | null;
  isActive?: boolean | null;
}

export interface VariantListParams extends PaginationParams {
  parentProductId?: string | null;
  isActive?: boolean | null;
  search?: string | null; // Search SKU, barcode, parent product name
}

export interface VariantListResponse {
  variants: VariantResponse[];
  pagination: PaginationInfo;
}
```

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/inventory/variants` | List all variants (paginated) |
| GET | `/inventory/variants/{id}` | Get variant by ID |
| POST | `/inventory/variants` | Create variant |
| PUT | `/inventory/variants/{id}` | Update variant |
| DELETE | `/inventory/variants/{id}` | Delete variant (soft) |
| GET | `/inventory/variants/by-sku/{sku}` | Get variant by SKU |
| GET | `/inventory/variants/by-barcode/{barcode}` | Get variant by barcode |

**Note:** These endpoints are in addition to the existing `/products/{id}/variants` endpoints which are product-scoped.

## Route Structure

```
/inventory/variants
├── /                      # Variant List (this task)
└── → Links to /inventory/products/[id] for editing
```

**Design Decision:** Variants are always edited in the context of their parent product, so no separate edit page. The list page provides a cross-product search view.

## Quality Gates

```bash
# Unit tests (2026-01-27)
$ bun run test:unit -- variants

✓ variants.test.ts (32 tests)
Total: 32 tests PASS
Duration: 779ms

# Lint check
$ bunx eslint variants.ts +page.svelte
✓ No errors
```

## AI Agent Log

| Timestamp | Agent | Action | Result |
|-----------|-------|--------|--------|
| 2026-01-27 06:30 | Claude | Created task file | ✅ |
| 2026-01-27 06:30 | Claude | Added Variant types to inventory.ts | ✅ |
| 2026-01-27 06:30 | Claude | Created variants.ts API client | ✅ |
| 2026-01-27 06:30 | Claude | Added variantStore to store | ✅ |
| 2026-01-27 06:30 | Claude | Created variants list page | ✅ |
| 2026-01-27 06:30 | Claude | Created variants.test.ts | ✅ |

## Failure Analysis Log

| Timestamp | Error Description | Root Cause | Resolution |
|-----------|-------------------|------------|------------|
| - | - | - | - |

## Notes

- Variants are tenant-isolated via tenant_id
- SKU must be unique per tenant across all variants
- price_difference is stored as BIGINT (cents) for precision
- variant_attributes is flexible JSONB for any attribute combinations
- Existing `variantsApi` in `products.ts` handles product-scoped variant operations
- New `variantApi` in `inventory/variants.ts` handles cross-product variant operations
