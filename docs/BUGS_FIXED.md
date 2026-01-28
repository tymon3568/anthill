# Bugs Fixed Log

This document records critical bugs that were discovered and fixed during development.
The purpose is to prevent similar bugs from occurring in the future.

---

## Bug #1: Categories not loading in Product dropdowns

**Date discovered:** 2026-01-28

### Bug Description
- Category dropdown in Product pages (List, New, Edit, Detail) does not display the category list
- Only shows "All Categories" or "No category" without actual categories

### Root Cause
- Frontend called `categoryApi.list({ pageSize: 1000 })` to fetch all categories
- Backend has validation limiting `pageSize` to a maximum of **100**
- API returned 400 Bad Request error: `VALIDATION_ERROR: pageSize range is 1-100`
- Due to this error, categories were not loaded and the dropdown remained empty

### Fix
Changed from `categoryApi.list({ pageSize: 1000 })` to `categoryApi.getTree()`:
- `getTree()` returns all categories in tree format without pagination
- This is the appropriate API for displaying a category selection dropdown

```typescript
// Before (incorrect)
async function fetchCategories() {
  const result = await categoryApi.list({ pageSize: 1000 });
  if (result.success && result.data) {
    categories = result.data.categories;
  }
}

// After (correct)
async function fetchCategories() {
  // Use getTree() to fetch all categories - list() has a max pageSize of 100
  const result = await categoryApi.getTree();
  if (result.success && result.data) {
    categories = result.data;
  }
}
```

### Related Files
1. `frontend/src/routes/(protected)/inventory/products/+page.svelte` - Product list page
2. `frontend/src/routes/(protected)/inventory/products/new/+page.svelte` - New product page
3. `frontend/src/routes/(protected)/inventory/products/[id]/+page.svelte` - Product detail page
4. `frontend/src/routes/(protected)/inventory/products/[id]/edit/+page.svelte` - Edit product page

### Lessons Learned
- **Always check API constraints**: When fetching data with pagination, know the backend limits
- **Use the appropriate API**: For dropdowns that need all items, use non-paginated endpoints like `getTree()`
- **Test with network tab**: Check console/network to detect API errors early

---

## Bug #2: Default UOM displays "Loading..." forever

**Date discovered:** 2026-01-28

### Bug Description
- In the Product Detail page (`/inventory/products/[id]`), the "Default UOM" field always displays "Loading..."
- Users cannot view the product's UOM information

### Root Cause
- Frontend called `uomApi.list()` to fetch UOM list from endpoint `/inventory/uom`
- **Backend has not implemented the `/inventory/uom` endpoint** - this endpoint does not exist
- API call failed (404 or no response), but UI still displayed "Loading..."
- Although there was `finally { isLoadingUoms = false }`, the `getUomDisplayText()` function checked `isLoadingUoms` first

### Current State
- Pages `new/+page.svelte` and `edit/+page.svelte` use a **static UOM list** (hardcoded) - works correctly
- Page `[id]/+page.svelte` (Detail) attempted to fetch from a non-existent API - causing the bug

### Fix
Use static UOM list like the new/edit pages for consistency:

```typescript
// Before (fetching from non-existent API)
import { variantsApi, uomApi } from '$lib/api/products';
import type { ProductVariant, UnitOfMeasure } from '$lib/types/products';

let uomOptions = $state<UnitOfMeasure[]>([]);
let isLoadingUoms = $state(false);

// In $effect
loadUoms();

async function loadUoms() {
  isLoadingUoms = true;
  try {
    const result = await uomApi.list();
    if (result.success && result.data) {
      uomOptions = result.data;
    }
  } catch {
    // Silently fail
  } finally {
    isLoadingUoms = false;
  }
}

function getUomDisplayText(): string {
  if (isLoadingUoms) return 'Loading...';  // <-- Bug: displays Loading forever
  return getUomName(product?.defaultUomId);
}
```

```typescript
// After (static list, consistent with other pages)
import { variantsApi } from '$lib/api/products';
import type { ProductVariant } from '$lib/types/products';

// UOM options - static list (UOM API endpoint not implemented yet)
const uomOptions = [
  { uomId: '', uomName: 'None', uomCode: '-' },
  { uomId: 'piece', uomName: 'Piece', uomCode: 'PC' },
  { uomId: 'box', uomName: 'Box', uomCode: 'BOX' },
  { uomId: 'kg', uomName: 'Kilogram', uomCode: 'KG' },
  { uomId: 'meter', uomName: 'Meter', uomCode: 'M' }
];

function getUomDisplayText(): string {
  return getUomName(product?.defaultUomId);
}
```

### Related Files
1. `frontend/src/routes/(protected)/inventory/products/[id]/+page.svelte` - Product detail page

### Lessons Learned
- **Verify backend endpoints exist**: Before calling an API, confirm the endpoint has been implemented
- **Consistency in codebase**: If a feature doesn't have an API yet, all pages should use the same solution (static data or mock)
- **Graceful degradation**: When API is not available, UI should still display useful information instead of "Loading..."
- **TODO tracking**: Leave clear comments when using temporary static data, making it easy to update when the API is ready

---

## Bug #3: Warehouses API returns 403 Forbidden

**Date discovered:** 2026-01-28

### Bug Description
- In the Stock Levels page (`/inventory/stock-levels`), the warehouse filter dropdown only shows "All Warehouses"
- Network request to `/api/v1/inventory/warehouses` returns 403 Forbidden
- Stock levels data loads correctly, but warehouse dropdown cannot be populated

### Root Cause
**Two issues combined:**

1. **Casbin policies used tenant slug instead of tenant UUID**
   - Existing migrations created policies with `tenant_slug` (e.g., `tymon`)
   - But Casbin enforcer uses `tenant_id` UUID from JWT token
   - Example mismatch:
     - Policy: `('p', 'owner', 'tymon', '/api/v1/inventory/warehouses', 'GET')`
     - Enforcer request: `(user_id, '019be656-dd5a-74d2-a07a-48d9164b4199', '/api/v1/inventory/warehouses', 'GET')`

2. **RequirePermission extractor used wrong URI path for nested routes**
   - In handlers using `RequirePermission` extractor, `parts.uri.path()` returned relative path (`/`) after Axum route matching
   - This caused Casbin to check permission for `/` instead of `/api/v1/inventory/warehouses`
   - Casbin middleware passed (checked full path), but extractor failed (checked relative path)

### Fix

**Fix 1: Use tenant_id instead of slug in Casbin policies**
```sql
-- Before (incorrect - using slug)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.slug, '/api/v1/inventory/warehouses', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL;

-- After (correct - using tenant_id)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL;
```

**Fix 2: Use OriginalUri in RequirePermission extractor**
```rust
// Before (incorrect - using parts.uri which is modified by nested routes)
let resource = parts.uri.path().to_string();

// After (correct - using OriginalUri which preserves full path)
let resource = parts
    .extensions
    .get::<OriginalUri>()
    .map(|uri| uri.path().to_string())
    .unwrap_or_else(|| parts.uri.path().to_string());
```

### Related Files
1. `migrations/20260128000001_add_warehouses_casbin_policies.sql` - New migration for warehouse policies
2. `shared/auth/src/extractors.rs` - RequirePermission extractor fix
3. `frontend/src/routes/(protected)/inventory/stock-levels/+page.svelte` - Stock levels page (affected UI)

### Lessons Learned
- **Casbin domain must match JWT token**: Use `tenant_id` UUID consistently, not slug
- **Axum nested routes modify URI**: Use `OriginalUri` extractor to get the full request path
- **Double permission checking**: When using both middleware and extractor for Casbin, ensure both use the same path resolution
- **Check Casbin logs**: The `Enforce Request, Response: true/false` logs show exactly what values are being checked

---

## Bug #4: Warehouses list shows 0 items despite API returning data

**Date discovered:** 2026-01-28

### Bug Description
- Warehouse list page (`/inventory/warehouses`) shows "0 Total Warehouses" and empty list
- Network request to `/api/v1/inventory/warehouses` returns 200 with warehouse data
- Stats cards show all zeros, no warehouse cards displayed

### Root Cause
- **API response format mismatch** between backend and frontend
- Backend returns raw array: `[{...warehouse1...}, {...warehouse2...}]`
- Frontend expected wrapped object: `{ warehouses: [...], pagination: {...} }`
- When frontend tried to access `response.data.warehouses`, it got `undefined`

### Fix
Updated frontend API client to handle raw array response and wrap it in expected format:

```typescript
// Before (expecting wrapped response)
async list(params: PaginationParams = {}): Promise<ApiResponse<WarehouseListResponse>> {
  const query = buildQueryString(toRecord(params));
  return apiClient.get<WarehouseListResponse>(`${BASE_PATH}${query}`);
}

// After (handle raw array, wrap in expected format)
async list(params: PaginationParams = {}): Promise<ApiResponse<WarehouseListResponse>> {
  const query = buildQueryString(toRecord(params));
  const response = await apiClient.get<WarehouseResponse[]>(`${BASE_PATH}${query}`);
  
  // Backend returns raw array, wrap in expected format
  if (response.success && response.data) {
    const warehouses = response.data;
    return {
      success: true,
      data: {
        warehouses,
        pagination: {
          page: 1,
          pageSize: warehouses.length,
          totalItems: warehouses.length,
          totalPages: 1,
          hasNext: false,
          hasPrev: false
        }
      }
    };
  }
  
  return { success: false, error: response.error || 'Failed to load warehouses' };
}
```

### Related Files
1. `frontend/src/lib/api/inventory/warehouses.ts` - Warehouse API client

### Lessons Learned
- **Check API response format**: Always verify backend response matches frontend expectations
- **Handle format differences in API client**: Transform data at the API layer, not in components
- **Test data display after API success**: 200 status doesn't mean data displays correctly

---

## Bug #5: Zones and Locations API returns 403 Forbidden

**Date discovered:** 2026-01-28

### Bug Description
- Creating a zone in warehouse detail page fails with 403 Forbidden
- Creating a location fails with same error
- POST requests to `/api/v1/inventory/warehouses/{id}/zones` and `/api/v1/inventory/warehouses/{id}/locations` return 403

### Root Cause
- **Missing Casbin policies for nested routes**
- Previous migration only added policies for `/api/v1/inventory/warehouses` (GET)
- No policies existed for:
  - POST/PUT/DELETE on warehouses
  - GET/POST/PUT/DELETE on zones (`/warehouses/*/zones`, `/warehouses/*/zones/*`)
  - GET/POST/PUT/DELETE on locations (`/warehouses/*/locations`, `/warehouses/*/locations/*`)

### Fix
Created new migration `20260128000002_add_warehouses_zones_locations_policies.sql` with comprehensive policies:

```sql
-- Zones policies
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones', 'POST', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones/*', 'PUT', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Similar for all roles and locations...
```

### Related Files
1. `migrations/20260128000002_add_warehouses_zones_locations_policies.sql` - New policies migration

### Lessons Learned
- **Plan all endpoints when adding Casbin policies**: List all CRUD operations and nested routes
- **Use wildcard patterns**: `/warehouses/*/zones` matches `/warehouses/{any-uuid}/zones`
- **Role-based access**: Different roles (owner, admin, manager vs user, viewer) need different permissions

---

## Checklist for Developing New Modules

To avoid repeating the above bugs, when developing new modules:

1. **Check API constraints**
   - [ ] Pagination limits (pageSize max)
   - [ ] Required fields
   - [ ] Data format expectations

2. **Verify backend endpoints**
   - [ ] Endpoint exists and works
   - [ ] Response format matches expectations
   - [ ] Error handling works correctly

3. **Consistency check**
   - [ ] Same feature has same implementation across pages
   - [ ] Shared utilities are used instead of duplicate code

4. **Testing**
   - [ ] Open Network tab when testing UI
   - [ ] Check console errors
   - [ ] Test with real data from API
