# Task: Migrate storage_locations to warehouse_locations

**Task ID:** `V1_MVP/04_Inventory_Service/4.5_Location_Architecture_Fix/task_04.05.01_migrate_storage_locations.md`
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.5_Location_Architecture_Fix
**Priority:** Critical
**Status:** Done
**Assignee:** Claude
**Created Date:** 2026-01-28
**Last Updated:** 2026-01-28
**Dependencies:**
- None (Foundation task)

## 1. Detailed Description

Hiện tại hệ thống có **hai bảng location riêng biệt** không được thống nhất:

1. `storage_locations`: Có các fields như `zone` (VARCHAR), `aisle`, `rack`, `level`, `position`
2. `warehouse_locations`: Có `zone_id` (FK to warehouse_zones), `coordinates` (JSONB)

Vấn đề chính:
- `inventory_levels.location_id` references `storage_locations` nhưng LUÔN NULL
- `stock_moves.source_location_id` và `destination_location_id` references `warehouse_locations`
- Code trong `transfer.rs` intentionally set `location_id = None` vì inconsistency này

**Giải pháp**: Migrate tất cả data từ `storage_locations` sang `warehouse_locations` và DROP bảng `storage_locations`.

## 2. Implementation Steps (Specific Sub-tasks)

### 2.1 Schema Analysis

- [x] 1. Document tất cả columns trong `storage_locations` cần preserve
- [x] 2. Document tất cả FKs referencing `storage_locations`
- [x] 3. Identify data that needs migration

### 2.2 Migration Script

- [x] 4. Create migration: Add missing columns to `warehouse_locations`:
  ```sql
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS aisle VARCHAR(50);
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS rack VARCHAR(50);
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS level INTEGER;
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS position INTEGER;
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS capacity BIGINT;
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS current_stock BIGINT DEFAULT 0;
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS is_quarantine BOOLEAN DEFAULT false;
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS is_picking_location BOOLEAN DEFAULT true;
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS length_cm INTEGER;
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS width_cm INTEGER;
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS height_cm INTEGER;
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS weight_limit_kg INTEGER;
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS created_by UUID;
  ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS updated_by UUID;
  ```

- [x] 5. Create migration: Migrate data from `storage_locations` to `warehouse_locations`

- [x] 6. Create migration: Update `inventory_levels` FK to use `warehouse_locations.location_id`

- [x] 7. Create migration: Create location_id mapping table for old → new IDs

- [x] 8. Create migration: DROP `storage_locations` table (after verification)

### 2.3 Code Updates

- [x] 9. Update repository: Remove all references to `storage_locations` in Rust code
- [x] 10. Update DTOs: Ensure location DTOs use `warehouse_locations` structure
- [x] 11. Update services: Fix any service using `storage_locations`

### 2.4 Testing

- [x] 12. Write unit tests for location migration
- [x] 13. Test FK constraints work correctly
- [x] 14. Verify existing inventory_levels data integrity

## 3. Completion Criteria

- [x] All data from `storage_locations` migrated to `warehouse_locations`
- [x] `storage_locations` table dropped
- [x] `inventory_levels.location_id` references `warehouse_locations`
- [x] `stock_moves.source_location_id` and `destination_location_id` work correctly
- [x] No Rust code references `storage_locations`
- [x] All existing tests pass
- [x] New migration tests pass
- [x] Database ERD updated to reflect changes

## 4. SQL Migration Files

### Migration 1: Add columns to warehouse_locations
```sql
-- migrations/YYYYMMDD000001_unify_location_tables_add_columns.sql

-- Add missing columns from storage_locations to warehouse_locations
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS aisle VARCHAR(50);
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS rack VARCHAR(50);
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS level INTEGER;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS position INTEGER;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS capacity BIGINT;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS current_stock BIGINT DEFAULT 0;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS is_quarantine BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS is_picking_location BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS length_cm INTEGER;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS width_cm INTEGER;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS height_cm INTEGER;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS weight_limit_kg INTEGER;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS created_by UUID;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS updated_by UUID;

-- Add FK constraints for created_by and updated_by
-- (Will be added after user verification)

COMMENT ON TABLE warehouse_locations IS 'Unified storage locations within warehouse zones (merged from storage_locations)';
```

### Migration 2: Migrate data
```sql
-- migrations/YYYYMMDD000002_unify_location_tables_migrate_data.sql

-- Create mapping table for old location_id to new location_id
CREATE TABLE IF NOT EXISTS _migration_location_mapping (
  old_location_id UUID PRIMARY KEY,
  new_location_id UUID NOT NULL,
  migrated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert storage_locations into warehouse_locations
-- Match zone by name, create mapping
WITH migrated AS (
  INSERT INTO warehouse_locations (
    location_id, tenant_id, warehouse_id, zone_id, location_code, location_name,
    description, location_type, coordinates, dimensions, capacity_info, location_attributes,
    is_active, created_at, updated_at, deleted_at,
    aisle, rack, level, position, capacity, current_stock,
    is_quarantine, is_picking_location, length_cm, width_cm, height_cm,
    weight_limit_kg, created_by, updated_by
  )
  SELECT 
    uuid_generate_v7(),  -- New location_id
    sl.tenant_id, sl.warehouse_id,
    wz.zone_id,
    sl.location_code,
    COALESCE(sl.location_code, 'Migrated Location') AS location_name,
    NULL AS description,
    sl.location_type,
    NULL AS coordinates,
    jsonb_build_object(
      'length_mm', sl.length_cm * 10,
      'width_mm', sl.width_cm * 10,
      'height_mm', sl.height_cm * 10
    ) AS dimensions,
    jsonb_build_object(
      'max_weight_kg', sl.weight_limit_kg,
      'max_units', sl.capacity
    ) AS capacity_info,
    jsonb_build_object(
      'is_quarantine', sl.is_quarantine,
      'is_picking', sl.is_picking_location
    ) AS location_attributes,
    sl.is_active, sl.created_at, sl.updated_at, sl.deleted_at,
    sl.aisle, sl.rack, sl.level, sl.position, sl.capacity, sl.current_stock,
    sl.is_quarantine, sl.is_picking_location, sl.length_cm, sl.width_cm, sl.height_cm,
    sl.weight_limit_kg, sl.created_by, sl.updated_by
  FROM storage_locations sl
  LEFT JOIN warehouse_zones wz ON wz.tenant_id = sl.tenant_id 
    AND wz.warehouse_id = sl.warehouse_id 
    AND wz.zone_code = sl.zone
  WHERE sl.deleted_at IS NULL
  ON CONFLICT (tenant_id, warehouse_id, location_code) DO NOTHING
  RETURNING location_id, tenant_id, warehouse_id, location_code
)
INSERT INTO _migration_location_mapping (old_location_id, new_location_id)
SELECT sl.location_id, m.location_id
FROM storage_locations sl
JOIN migrated m ON m.tenant_id = sl.tenant_id 
  AND m.warehouse_id = sl.warehouse_id 
  AND m.location_code = sl.location_code;
```

### Migration 3: Update inventory_levels FK
```sql
-- migrations/YYYYMMDD000003_unify_location_tables_update_fks.sql

-- Update inventory_levels.location_id to use new warehouse_locations IDs
UPDATE inventory_levels il
SET location_id = mlm.new_location_id
FROM _migration_location_mapping mlm
WHERE il.location_id = mlm.old_location_id;

-- Add proper FK constraint (after data migration)
-- Note: inventory_levels.location_id already references storage_locations
-- We need to drop and recreate

-- First, drop the old constraint if exists
ALTER TABLE inventory_levels 
DROP CONSTRAINT IF EXISTS inventory_levels_location_id_fkey;

-- Add new FK to warehouse_locations
ALTER TABLE inventory_levels 
ADD CONSTRAINT inventory_levels_location_id_fkey 
FOREIGN KEY (location_id) REFERENCES warehouse_locations(location_id);
```

### Migration 4: Drop storage_locations
```sql
-- migrations/YYYYMMDD000004_unify_location_tables_drop_old.sql

-- Verify no orphan references exist
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM inventory_levels il
    WHERE il.location_id IS NOT NULL
    AND NOT EXISTS (SELECT 1 FROM warehouse_locations wl WHERE wl.location_id = il.location_id)
  ) THEN
    RAISE EXCEPTION 'Orphan location_id references exist in inventory_levels';
  END IF;
END $$;

-- Drop storage_locations table
DROP TABLE IF EXISTS storage_locations CASCADE;

-- Drop migration mapping table
DROP TABLE IF EXISTS _migration_location_mapping;

-- Add comment
COMMENT ON TABLE warehouse_locations IS 'Physical storage locations within warehouse zones (unified from storage_locations + warehouse_locations)';
```

## Related Documents

- Mini PRD: `./README.md`
- ERD: `docs/database-erd.dbml`
- Transfer Service: `services/inventory_service/infra/src/services/transfer.rs:218-221` (comment about location issue)

## AI Agent Log:

* 2026-01-28 20:00: Task created based on analysis of warehouse location architecture issues
    - Identified two separate location tables causing schema inconsistency
    - Designed migration strategy to unify into warehouse_locations
