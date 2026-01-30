# Inventory Service - Bugs Fixed

> This document contains bugs that are SPECIFIC to the Inventory Service (Rust backend).
> For cross-module bugs (API format, Casbin, etc.), see `docs/INTEGRATION_BUGS.md`.
> For common patterns, see `docs/COMMON_PATTERNS.md`.

---

## INV-1: [Template for Future Inventory Bugs]

**Date discovered:** YYYY-MM-DD

**Sub-module:** [e.g., Stock Adjustments, Warehouses, Products]

### Bug Description
[Describe what the user or system experiences]

### Root Cause
[Explain the technical cause in the Rust backend]

### Fix
```rust
// Before
[buggy code]

// After
[fixed code]
```

### Related Files
- `services/inventory_service/api/src/handlers/[module].rs`
- `services/inventory_service/core/src/services/[module].rs`
- `services/inventory_service/infra/src/repositories/[module].rs`

### Lessons Learned
- [key takeaways]

---

## Common Inventory Service Issues Reference

> These are NOT bugs but common issues to watch for in Inventory Service development.

### Issue: Stock Calculation Precision
- Use `Decimal` type for quantities, not `f64`
- SQLx: Use `sqlx::types::Decimal` or `rust_decimal::Decimal`

### Issue: Concurrent Stock Updates
- Use database transactions for stock level changes
- Consider using `SELECT ... FOR UPDATE` for row-level locking
- Implement optimistic locking with version field if high concurrency expected

### Issue: Audit Trail
- All stock movements must be logged
- Include: who, when, what, from_qty, to_qty, reason

---

**Document Version:** 1.0
**Created:** 2026-01-29
**Author:** Claude (AI Agent)
