# Common Patterns & Anti-patterns

> This document contains patterns that apply to ALL modules in the anthill-windsurf project.
> AI agents MUST read this file before implementing any new module.

---

## CP-1: API Pagination Limits

**Pattern**: Always check backend pagination constraints before implementing frontend calls.

**Anti-pattern**:
```typescript
// WRONG - backend limits pageSize to 100
const result = await categoryApi.list({ pageSize: 1000 });
```

**Correct Pattern**:
```typescript
// Option 1: Use non-paginated endpoint for dropdowns
const result = await categoryApi.getTree();

// Option 2: Implement proper pagination/infinite scroll
const result = await categoryApi.list({ page: 1, pageSize: 100 });
```

**Symptoms of Violation**:
- 400 Bad Request with validation error
- Dropdown shows empty list despite data existing

**Prevention Checklist**:
- [ ] Check backend validation rules for pageSize (usually max 100)
- [ ] Use `getTree()` or `getAll()` endpoints for dropdowns
- [ ] Test with Network tab open to catch validation errors

**Reference**: `docs/archive/BUGS_FIXED_LEGACY.md` Bug #1

---

## CP-2: camelCase vs snake_case Transformation

**Pattern**: Understand apiClient's auto-transformation behavior.

**Key Rules**:
1. **Backend expects**: `snake_case` (Rust convention)
2. **Frontend uses**: `camelCase` (TypeScript convention)
3. **apiClient auto-transforms**: Response `snake_case` → `camelCase`
4. **apiClient does NOT transform**: Request bodies (you must do it manually)

**Anti-pattern**:
```typescript
// WRONG - sending camelCase, backend expects snake_case
await apiClient.post('/adjustments', {
  productId: 'xxx',      // Backend rejects!
  warehouseId: 'yyy'
});

// WRONG - accessing snake_case but apiClient already converted
function transform(adj: Record<string, unknown>) {
  return {
    adjustmentId: adj.adjustment_id,  // undefined!
  };
}
```

**Correct Pattern**:
```typescript
// Request: Transform to snake_case manually
function transformToSnakeCase(data: CreateRequest) {
  return {
    product_id: data.productId,
    warehouse_id: data.warehouseId,
  };
}
await apiClient.post('/adjustments', transformToSnakeCase(data));

// Response: Access camelCase (already transformed by apiClient)
function transformFromBackend(adj: Record<string, unknown>) {
  return {
    adjustmentId: adj.adjustmentId as string,  // Correct!
  };
}
```

**Symptoms of Violation**:
- 422 Unprocessable Entity on POST/PUT
- Undefined values in response handling
- Table/list displays empty or crashes

**Prevention Checklist**:
- [ ] Create `transformToSnakeCase()` for all POST/PUT requests
- [ ] Access camelCase fields in response transforms
- [ ] Test full CRUD cycle: create → verify in list → edit → delete

**Reference**: `docs/archive/BUGS_FIXED_LEGACY.md` Bug #7

---

## CP-3: Defensive Null Checks

**Pattern**: Always null-check values before calling methods on them.

**Anti-pattern**:
```typescript
// WRONG - crashes if warehouseId is undefined
function getWarehouseName(warehouseId: string): string {
  return warehouse?.name || warehouseId.slice(0, 8); // CRASH!
}
```

**Correct Pattern**:
```typescript
function getWarehouseName(warehouseId: string | undefined | null): string {
  if (!warehouseId) return '-';
  const warehouse = warehouses.find(w => w.id === warehouseId);
  return warehouse?.name || warehouseId.slice(0, 8);
}
```

**Symptoms of Violation**:
- `Cannot read properties of undefined (reading 'slice')`
- `Cannot read properties of undefined (reading 'map')`
- UI crashes after API responses

**Prevention Checklist**:
- [ ] Use union types: `| undefined | null` for optional values
- [ ] Add null check as first line: `if (!value) return fallback;`
- [ ] Provide meaningful fallback values (`'-'`, `[]`, `{}`)

**Reference**: `docs/archive/BUGS_FIXED_LEGACY.md` Bug #8

---

## CP-4: Svelte Each Block Keys

**Pattern**: Use fallback keys to handle items without valid IDs.

**Anti-pattern**:
```svelte
<!-- WRONG - multiple items with empty id cause duplicate key error -->
{#each items as item (item.id)}
```

**Correct Pattern**:
```svelte
<!-- Fallback to index-based key when id is empty/undefined -->
{#each items as item, index (item.id || `temp-${index}`)}
```

**Symptoms of Violation**:
- Console error: `each_key_duplicate`
- Table/list renders incorrectly with duplicate or missing rows
- Items disappear or duplicate after CRUD operations

**Prevention Checklist**:
- [ ] Always use fallback key pattern: `(item.id || \`temp-${index}\`)`
- [ ] After creating items, reload from API to get real IDs
- [ ] Don't add incomplete items to local state

**Reference**: `docs/archive/BUGS_FIXED_LEGACY.md` Bug #9

---

## CP-5: API Response Format Handling

**Pattern**: Handle cases where backend response format differs from frontend expectations.

**Anti-pattern**:
```typescript
// WRONG - assuming backend returns wrapped response
async list(): Promise<ApiResponse<WarehouseListResponse>> {
  const response = await apiClient.get<WarehouseListResponse>('/warehouses');
  // Backend returns: [{...}, {...}]
  // Frontend expects: { warehouses: [...], pagination: {...} }
  return response; // Breaks!
}
```

**Correct Pattern**:
```typescript
async list(): Promise<ApiResponse<WarehouseListResponse>> {
  // Expect raw array from backend
  const response = await apiClient.get<WarehouseResponse[]>('/warehouses');
  
  if (response.success && response.data) {
    const items = response.data;
    return {
      success: true,
      data: {
        warehouses: items,
        pagination: {
          page: 1,
          pageSize: items.length,
          totalItems: items.length,
          totalPages: 1,
          hasNext: false,
          hasPrev: false
        }
      }
    };
  }
  return { success: false, error: response.error };
}
```

**Symptoms of Violation**:
- List shows "0 items" despite API returning 200
- `Cannot read properties of undefined` when accessing nested data
- Stats/counts show zeros

**Prevention Checklist**:
- [ ] Test API with curl/Postman to see actual response format
- [ ] Transform response in API client layer, not in components
- [ ] Log response.data to verify structure before accessing

**Reference**: `docs/archive/BUGS_FIXED_LEGACY.md` Bug #4

---

## CP-6: Consistent Implementation Across Pages

**Pattern**: Same feature should have same implementation across all pages.

**Anti-pattern**:
- Page A uses API to fetch UOMs
- Page B uses hardcoded static list
- Page C tries to fetch from non-existent endpoint

**Correct Pattern**:
```typescript
// Shared constant or utility
// src/lib/constants/uom.ts
export const UOM_OPTIONS = [
  { uomId: 'piece', uomName: 'Piece', uomCode: 'PC' },
  { uomId: 'box', uomName: 'Box', uomCode: 'BOX' },
  // ...
];

// OR shared store that handles API availability
// src/lib/stores/uom.svelte.ts
export const uomStore = createUomStore(); // Handles both API and fallback
```

**Prevention Checklist**:
- [ ] Before implementing, check how other pages handle same feature
- [ ] Extract shared logic to utilities or stores
- [ ] Add TODO comments when using temporary static data
- [ ] Audit all related pages when fixing a bug

**Reference**: `docs/archive/BUGS_FIXED_LEGACY.md` Bug #2

---

## Quick Reference Table

| Pattern | Symptom | Quick Fix |
|---------|---------|-----------|
| CP-1 | 400 on list with large pageSize | Use `getTree()` or max 100 |
| CP-2 | 422 on POST | Transform request to snake_case |
| CP-2 | Undefined in response | Access camelCase fields |
| CP-3 | `Cannot read properties of undefined` | Add null check |
| CP-4 | `each_key_duplicate` | Use `(id \|\| \`temp-${index}\`)` |
| CP-5 | List shows 0 items | Transform response in API client |
| CP-6 | Feature works on page A, not B | Check consistency, use shared code |

---

**Document Version:** 1.0
**Created:** 2026-01-29
**Author:** Claude (AI Agent)
