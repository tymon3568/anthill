# Location Architecture Migration Guide

## Module 4.5 - Location Architecture Fix

**Version:** V1_MVP  
**Created:** 2026-01-28  
**Status:** Implemented

---

## Overview

This document describes the migration from the legacy dual-location architecture to the unified location architecture.

### Problem Statement

Previously, the system had two separate location tables:

1. **storage_locations** - Physical storage locations with detailed attributes (aisle, rack, level, etc.)
2. **warehouse_locations** - Logical locations within warehouse zones

This caused:
- `inventory_levels.location_id` referenced `storage_locations` but was **ALWAYS NULL**
- `stock_moves` referenced `warehouse_locations`
- Inconsistent location tracking across the system
- `stock_transfers` only had `warehouse_id`, no zone/location granularity

### Solution

The unified architecture:
1. **Merged** all `storage_locations` columns into `warehouse_locations`
2. **Added** zone/location columns to `stock_transfer_items`
3. **Updated** `inventory_levels.location_id` FK to reference `warehouse_locations`
4. **Dropped** the legacy `storage_locations` table

---

## Database Changes

### Tables Modified

#### `warehouse_locations` (Unified)

New columns added:

| Column | Type | Description |
|--------|------|-------------|
| `aisle` | VARCHAR(50) | Aisle identifier within zone |
| `rack` | VARCHAR(50) | Rack identifier within aisle |
| `level` | INTEGER | Level/shelf number on rack |
| `position` | INTEGER | Position number on level |
| `capacity` | BIGINT | Maximum capacity in base units |
| `current_stock` | BIGINT | Current stock quantity at location |
| `is_quarantine` | BOOLEAN | Whether location is for quarantined stock |
| `is_picking_location` | BOOLEAN | Whether location is used for picking |
| `length_cm` | INTEGER | Location length in centimeters |
| `width_cm` | INTEGER | Location width in centimeters |
| `height_cm` | INTEGER | Location height in centimeters |
| `weight_limit_kg` | INTEGER | Maximum weight limit in kilograms |
| `created_by` | UUID | User who created this location |
| `updated_by` | UUID | User who last updated this location |

#### `stock_transfer_items`

New columns added for location-level transfers:

| Column | Type | Description |
|--------|------|-------------|
| `source_zone_id` | UUID | Source zone within source warehouse (optional) |
| `source_location_id` | UUID | Source location/bin within source warehouse (optional) |
| `destination_zone_id` | UUID | Destination zone within destination warehouse (optional) |
| `destination_location_id` | UUID | Destination location/bin within destination warehouse (optional) |

### Tables Removed

- **`storage_locations`** - Merged into `warehouse_locations`

### New Foreign Keys

```sql
-- stock_transfer_items -> warehouse_zones
fk_transfer_items_source_zone: (tenant_id, source_zone_id) -> warehouse_zones(tenant_id, zone_id)
fk_transfer_items_dest_zone: (tenant_id, destination_zone_id) -> warehouse_zones(tenant_id, zone_id)

-- stock_transfer_items -> warehouse_locations
fk_transfer_items_source_location: (tenant_id, source_location_id) -> warehouse_locations(tenant_id, location_id)
fk_transfer_items_dest_location: (tenant_id, destination_location_id) -> warehouse_locations(tenant_id, location_id)

-- inventory_levels -> warehouse_locations (updated)
inventory_levels.(tenant_id, location_id) -> warehouse_locations.(tenant_id, location_id)
```

---

## Migration Steps

### Prerequisites

1. Ensure all pending transactions are completed
2. Take a full database backup
3. Notify users of planned maintenance window
4. Have rollback script ready

### Running the Migration

```bash
# 1. Set environment variables
export DATABASE_URL="postgres://user:password@host/database"
export BACKUP_DIR="./backups"

# 2. Run migration script
./scripts/migrate_locations.sh
```

The script will:
1. Create backup of current data
2. Run migrations in order
3. Verify migration success

### Verification Checklist

After migration, verify:

- [ ] `storage_locations` table no longer exists
- [ ] `warehouse_locations` has new columns (aisle, rack, level, etc.)
- [ ] `stock_transfer_items` has zone/location columns
- [ ] Existing inventory levels still query correctly
- [ ] Stock moves reference `warehouse_locations`
- [ ] Frontend location selectors work

---

## Rollback Procedure

If issues are found after migration:

```bash
# Run rollback with backup file
./scripts/rollback_locations.sh ./backups/backup_before_location_migration_TIMESTAMP.sql
```

The rollback will:
1. Restore `storage_locations` table from backup
2. Revert migration changes
3. Remove new columns from modified tables

---

## API Changes

### Transfer Creation

Transfer items can now include zone/location:

```json
{
  "sourceWarehouseId": "uuid",
  "destinationWarehouseId": "uuid",
  "items": [
    {
      "productId": "uuid",
      "quantity": 100,
      "sourceZoneId": "uuid",      // NEW - optional
      "sourceLocationId": "uuid",   // NEW - optional
      "destinationZoneId": "uuid",  // NEW - optional
      "destinationLocationId": "uuid" // NEW - optional
    }
  ]
}
```

### Location Response

Locations now include additional fields:

```json
{
  "locationId": "uuid",
  "locationCode": "A-01-01",
  "locationName": "Aisle A, Rack 01, Level 01",
  "locationType": "bin",
  "zoneId": "uuid",
  "aisle": "A",           // NEW
  "rack": "01",           // NEW
  "level": 1,             // NEW
  "position": 1,          // NEW
  "capacity": 1000,       // NEW
  "currentStock": 50,     // NEW
  "isQuarantine": false,  // NEW
  "isPickingLocation": true // NEW
}
```

---

## Backward Compatibility

The migration maintains backward compatibility:

1. **NULL location_id allowed** - Transfers without location specification still work
2. **Fallback to warehouse level** - If no location specified, stock is tracked at warehouse level
3. **Existing data preserved** - All existing stock levels and moves remain valid

---

## Related Files

### Migrations

1. `migrations/20260128300001_unify_location_tables_add_columns.sql` - Add columns to warehouse_locations
2. `migrations/20260128300002_unify_location_tables_migrate_data.sql` - Migrate data from storage_locations
3. `migrations/20260128300003_unify_location_tables_drop_old.sql` - Drop storage_locations table
4. `migrations/20260128300004_add_transfer_item_locations.sql` - Add zone/location to stock_transfer_items
5. `migrations/20260128300005_add_warehouse_zones_tenant_unique.sql` - Add unique constraint for FKs

### Scripts

- `scripts/migrate_locations.sh` - Production migration script
- `scripts/rollback_locations.sh` - Rollback script

### Documentation

- `docs/database-erd.dbml` - Updated database ERD

### Tests

- `services/inventory_service/api/tests/location_architecture_tests.rs` - Integration tests

---

## Troubleshooting

### Migration Fails

1. Check database connectivity
2. Verify backup completed successfully
3. Check for active transactions blocking migration
4. Review migration logs for specific errors

### Rollback Needed

1. Stop application servers
2. Run rollback script with backup file
3. Verify rollback success
4. Restart application servers

### Data Issues After Migration

1. Check `inventory_levels` for orphan `location_id` references
2. Verify `stock_moves` location references are valid
3. Use pre-migration backup for data recovery if needed

---

## Contact

For issues with this migration, contact the development team or open an issue in the project tracker:
- Task: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.5_Location_Architecture_Fix/`
