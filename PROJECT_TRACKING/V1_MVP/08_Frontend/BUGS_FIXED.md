# Frontend Module - Bugs Fixed

> This document contains bugs that are SPECIFIC to the Frontend module (SvelteKit, Svelte 5, stores, components).
> For cross-module bugs, see `docs/INTEGRATION_BUGS.md`.
> For common patterns, see `docs/COMMON_PATTERNS.md`.

---

## FB-1: UOM Loading Forever in Product Detail Page

**Date discovered:** 2026-01-28

**Sub-module:** Products UI (`8.x_Products_UI`)

### Bug Description
- In the Product Detail page (`/inventory/products/[id]`), the "Default UOM" field always displays "Loading..."
- Users cannot view the product's UOM information

### Root Cause
- Frontend called `uomApi.list()` to fetch UOM list from endpoint `/inventory/uom`
- **Backend has NOT implemented the `/inventory/uom` endpoint** - this endpoint does not exist
- API call failed (404), but UI still displayed "Loading..."
- Other pages (new, edit) used static UOM list - inconsistency

### Fix
Use static UOM list like other pages for consistency:

```typescript
// Before (calling non-existent API)
async function loadUoms() {
  const result = await uomApi.list();  // 404!
  if (result.success) uomOptions = result.data;
}

// After (static list, consistent with other pages)
const uomOptions = [
  { uomId: 'piece', uomName: 'Piece', uomCode: 'PC' },
  { uomId: 'box', uomName: 'Box', uomCode: 'BOX' },
  { uomId: 'kg', uomName: 'Kilogram', uomCode: 'KG' },
  // ...
];
```

### Related Files
- `frontend/src/routes/(protected)/inventory/products/[id]/+page.svelte`

### Lessons Learned
- Verify backend endpoint exists before calling from frontend
- Keep consistent approach across all pages (if static, all pages use static)
- Add TODO comments when using temporary static data

---

## FB-2: [Template for Future Frontend Bugs]

**Date discovered:** YYYY-MM-DD

**Sub-module:** [Sub-module name]

### Bug Description
[Describe what the user experiences]

### Root Cause
[Explain the technical cause]

### Fix
```typescript
// Before
[buggy code]

// After
[fixed code]
```

### Related Files
- [list files]

### Lessons Learned
- [key takeaways]

---

**Document Version:** 1.0
**Created:** 2026-01-29
**Author:** Claude (AI Agent)
