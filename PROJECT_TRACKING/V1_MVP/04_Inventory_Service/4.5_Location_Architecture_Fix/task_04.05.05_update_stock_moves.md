# Task: Update stock_moves location references

**Task ID:** `V1_MVP/04_Inventory_Service/4.5_Location_Architecture_Fix/task_04.05.05_update_stock_moves.md`
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.5_Location_Architecture_Fix
**Priority:** High
**Status:** Done
**Assignee:** Claude
**Created Date:** 2026-01-28
**Last Updated:** 2026-01-28
**Dependencies:**
- `V1_MVP/04_Inventory_Service/4.5_Location_Architecture_Fix/task_04.05.01_migrate_storage_locations.md`

## 1. Detailed Description

`stock_moves` là bảng audit trail cho tất cả stock movements. Hiện tại nó đã có:
- `source_location_id` (FK to `warehouse_locations`)
- `destination_location_id` (FK to `warehouse_locations`)

Nhưng sau khi unify location tables (Task 04.05.01), cần verify:
1. FK references are correct
2. Services create stock_moves with location_id
3. Queries for movement history work correctly

## 2. Implementation Steps (Specific Sub-tasks)

### 2.1 Verify Schema

- [ ] 1. Verify FK constraints point to `warehouse_locations`:
  ```sql
  -- Check existing FKs
  SELECT conname, pg_get_constraintdef(oid) 
  FROM pg_constraint 
  WHERE conrelid = 'stock_moves'::regclass 
  AND contype = 'f';
  ```

- [ ] 2. If needed, update FK constraints after location unification

### 2.2 Service Updates

- [ ] 3. Update `create_stock_move` function to require location_id when available:
  ```rust
  pub async fn create_stock_move(
      &self,
      tenant_id: Uuid,
      product_id: Uuid,
      source_location_id: Option<Uuid>,
      destination_location_id: Option<Uuid>,
      move_type: MoveType,
      quantity: i64,
      reference_type: &str,
      reference_id: Uuid,
  ) -> Result<StockMove, DomainError>
  ```

- [ ] 4. Update GRN receive to create move with destination_location
- [ ] 5. Update DO ship to create move with source_location
- [ ] 6. Update Transfer ship/receive to create moves with both locations

### 2.3 Query Updates

- [ ] 7. Add query for movement history by location:
  ```sql
  SELECT sm.*, 
    sl.location_code as source_location_code,
    dl.location_code as destination_location_code
  FROM stock_moves sm
  LEFT JOIN warehouse_locations sl ON sl.location_id = sm.source_location_id
  LEFT JOIN warehouse_locations dl ON dl.location_id = sm.destination_location_id
  WHERE sm.tenant_id = $1 
    AND (sm.source_location_id = $2 OR sm.destination_location_id = $2)
  ORDER BY sm.move_date DESC;
  ```

- [ ] 8. Add query for movement history by zone:
  ```sql
  SELECT sm.*, 
    sz.zone_code as source_zone_code,
    dz.zone_code as destination_zone_code
  FROM stock_moves sm
  LEFT JOIN warehouse_locations sl ON sl.location_id = sm.source_location_id
  LEFT JOIN warehouse_zones sz ON sz.zone_id = sl.zone_id
  LEFT JOIN warehouse_locations dl ON dl.location_id = sm.destination_location_id
  LEFT JOIN warehouse_zones dz ON dz.zone_id = dl.zone_id
  WHERE sm.tenant_id = $1 
    AND (sl.zone_id = $2 OR dl.zone_id = $2)
  ORDER BY sm.move_date DESC;
  ```

### 2.4 DTO Updates

- [ ] 9. Update `StockMoveResponse` to include location details:
  ```rust
  pub struct StockMoveResponse {
      pub move_id: Uuid,
      pub product_id: Uuid,
      pub product_name: String,
      pub source_location_id: Option<Uuid>,
      pub source_location_code: Option<String>,
      pub source_zone_name: Option<String>,
      pub destination_location_id: Option<Uuid>,
      pub destination_location_code: Option<String>,
      pub destination_zone_name: Option<String>,
      pub move_type: String,
      pub quantity: i64,
      pub reference_type: String,
      pub reference_id: Uuid,
      pub move_date: DateTime<Utc>,
  }
  ```

### 2.5 API Endpoints

- [ ] 10. Add endpoint: `GET /api/v1/inventory/stock-moves/by-location/{location_id}`
- [ ] 11. Add endpoint: `GET /api/v1/inventory/stock-moves/by-zone/{zone_id}`

### 2.6 Testing

- [ ] 12. Unit test: Create stock move with locations
- [ ] 13. Unit test: Query moves by location
- [ ] 14. Unit test: Query moves by zone
- [ ] 15. Integration test: Full audit trail for transfer

## 3. Completion Criteria

- [ ] FK constraints correctly reference `warehouse_locations`
- [ ] All stock operations create moves with location_id when available
- [ ] Movement history queryable by location
- [ ] Movement history queryable by zone
- [ ] StockMoveResponse includes location/zone details
- [ ] All tests pass

## 4. SQL Migrations (if needed)

```sql
-- migrations/YYYYMMDD000006_verify_stock_moves_fks.sql

-- Verify/update FK for source_location_id
ALTER TABLE stock_moves
DROP CONSTRAINT IF EXISTS stock_moves_source_location_id_fkey;

ALTER TABLE stock_moves
ADD CONSTRAINT stock_moves_source_location_id_fkey
FOREIGN KEY (tenant_id, source_location_id) 
REFERENCES warehouse_locations(tenant_id, location_id);

-- Verify/update FK for destination_location_id
ALTER TABLE stock_moves
DROP CONSTRAINT IF EXISTS stock_moves_destination_location_id_fkey;

ALTER TABLE stock_moves
ADD CONSTRAINT stock_moves_destination_location_id_fkey
FOREIGN KEY (tenant_id, destination_location_id) 
REFERENCES warehouse_locations(tenant_id, location_id);

-- Add indexes for location-based queries
CREATE INDEX IF NOT EXISTS idx_stock_moves_source_location
ON stock_moves(tenant_id, source_location_id, move_date DESC)
WHERE source_location_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_stock_moves_dest_location
ON stock_moves(tenant_id, destination_location_id, move_date DESC)
WHERE destination_location_id IS NOT NULL;
```

## Related Documents

- Mini PRD: `./README.md`
- Parent Task: `./task_04.05.01_migrate_storage_locations.md`
- Database ERD: `docs/database-erd.dbml` (stock_moves table)
- Stock Repository: `services/inventory_service/infra/src/repositories/stock.rs`

## AI Agent Log:

* 2026-01-28 20:20: Task created for stock_moves location updates
    - Verified current schema has location FKs
    - Added queries for location/zone-based movement history
