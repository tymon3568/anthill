# Module: 4.5 Location Architecture Fix

## Overview

Module này giải quyết các vấn đề kiến trúc nghiêm trọng về warehouse location management:

1. **Hai bảng location riêng biệt**: `storage_locations` và `warehouse_locations` không được thống nhất
2. **Transfer không hỗ trợ zone/location**: `stock_transfers` chỉ có `source_warehouse_id` và `destination_warehouse_id`, không chi tiết đến zone/location
3. **inventory_levels.location_id luôn NULL**: Stock chỉ được track ở mức warehouse, không chi tiết đến location

## Problem Analysis

### Current Schema Issues

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CURRENT ARCHITECTURE (BROKEN)                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  storage_locations                 warehouse_locations                      │
│  ├── location_id (PK)              ├── location_id (PK)                     │
│  ├── warehouse_id (FK)             ├── warehouse_id (FK)                    │
│  ├── location_code                 ├── zone_id (FK) ← warehouse_zones       │
│  ├── zone (VARCHAR)  ← String!     ├── location_code                        │
│  └── ...                           └── ...                                  │
│       │                                  │                                  │
│       ▼                                  ▼                                  │
│  inventory_levels.location_id      stock_moves.source_location_id           │
│  (ALWAYS NULL!)                    stock_moves.destination_location_id      │
│                                                                             │
│  ⚠️ INCONSISTENCY: Two separate tables, neither properly used               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Impact

| Issue | Impact |
|-------|--------|
| `inventory_levels.location_id` always NULL | Stock chỉ track ở warehouse level, không biết hàng nằm ở đâu trong warehouse |
| Transfers no zone/location | Không thể chuyển hàng giữa các zone/location cụ thể |
| Two location tables | Code confusion, maintenance nightmare |
| Cannot generate reports by location | Business reporting bị giới hạn |

## Solution Design

### Phase 1: Unify Location Tables

Consolidate `storage_locations` và `warehouse_locations` thành một bảng duy nhất `warehouse_locations`:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        TARGET ARCHITECTURE (FIXED)                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  warehouses                                                                 │
│       │                                                                     │
│       ├──▶ warehouse_zones (1:N)                                            │
│       │         │                                                           │
│       │         └──▶ warehouse_locations (1:N)                              │
│       │                    │                                                │
│       │                    ├──▶ inventory_levels.location_id                │
│       │                    ├──▶ stock_moves.source_location_id              │
│       │                    ├──▶ stock_moves.destination_location_id         │
│       │                    └──▶ stock_transfer_items.source_location_id     │
│       │                         stock_transfer_items.destination_location_id│
│       │                                                                     │
│       └──▶ stock_transfers (source/destination warehouse only)              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Phase 2: Add Zone/Location to Transfers

```sql
-- Add zone/location columns to stock_transfer_items
ALTER TABLE stock_transfer_items ADD COLUMN source_zone_id UUID;
ALTER TABLE stock_transfer_items ADD COLUMN source_location_id UUID;
ALTER TABLE stock_transfer_items ADD COLUMN destination_zone_id UUID;
ALTER TABLE stock_transfer_items ADD COLUMN destination_location_id UUID;
```

### Phase 3: Fix inventory_levels to Track Location

Update services to properly set `location_id` when receiving/moving stock.

## Task Summary

| Task ID | Task Name | Priority | Status | Dependencies |
|---------|-----------|----------|--------|--------------|
| 04.05.01 | Migrate storage_locations to warehouse_locations | Critical | Todo | - |
| 04.05.02 | Add zone/location columns to stock_transfer_items | High | Todo | 04.05.01 |
| 04.05.03 | Update transfer service for zone/location | High | Todo | 04.05.02 |
| 04.05.04 | Fix inventory_levels location tracking | High | Todo | 04.05.01 |
| 04.05.05 | Update stock_moves location references | High | Todo | 04.05.01 |
| 04.05.06 | Update frontend for zone/location selection | Medium | Todo | 04.05.03 |
| 04.05.07 | Add location-level reports | Medium | Todo | 04.05.04 |
| 04.05.08 | Integration tests and data migration | High | Todo | All above |

## Implementation Order

```
Phase 1: Schema Unification (Tasks 01)
    │
    ▼
Phase 2: Transfer Enhancement (Tasks 02, 03)
    │
    ▼
Phase 3: Inventory Tracking Fix (Tasks 04, 05)
    │
    ▼
Phase 4: Frontend + Reports (Tasks 06, 07)
    │
    ▼
Phase 5: Testing + Migration (Task 08)
```

## Database Schema Changes

### Tables to Modify

| Table | Change |
|-------|--------|
| `storage_locations` | **DROP** - Migrate data to warehouse_locations |
| `warehouse_locations` | Add missing columns from storage_locations |
| `inventory_levels` | Update FK to reference warehouse_locations |
| `stock_transfer_items` | Add source/destination zone/location columns |
| `stock_moves` | Verify FK references are correct |

### New Indexes

```sql
-- For location-level inventory queries
CREATE INDEX idx_inventory_levels_location 
ON inventory_levels (tenant_id, location_id, product_id) 
WHERE deleted_at IS NULL AND location_id IS NOT NULL;

-- For transfer item location tracking
CREATE INDEX idx_stock_transfer_items_locations
ON stock_transfer_items (tenant_id, source_location_id, destination_location_id)
WHERE deleted_at IS NULL;
```

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Data migration errors | High | Backup before migration, run in transaction, test on staging |
| Existing code breaks | High | Update all services that reference storage_locations |
| Frontend breaks | Medium | Update forms to use new location selector |
| Performance regression | Low | Add proper indexes, test with production-like data |

## Reference Documents

- Database ERD: `docs/database-erd.dbml`
- Transfer Service: `services/inventory_service/infra/src/services/transfer.rs`
- Stock Levels Service: `services/inventory_service/infra/src/services/stock_levels.rs`
- Warehouse Repository: `services/inventory_service/infra/src/repositories/warehouse.rs`

## Notes

- Migration MUST be backwards compatible (allow NULL location_id during transition)
- Update database ERD (`docs/database-erd.dbml`) after completing Phase 1
- Run full E2E tests after each phase
