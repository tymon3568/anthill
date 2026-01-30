# Task: Add zone/location columns to stock_transfer_items

**Task ID:** `V1_MVP/04_Inventory_Service/4.5_Location_Architecture_Fix/task_04.05.02_add_transfer_location_columns.md`
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

Hiện tại `stock_transfers` chỉ có:
- `source_warehouse_id` 
- `destination_warehouse_id`

Và `stock_transfer_items` chỉ có:
- `transfer_id`
- `product_id`
- `quantity`
- `uom_id`

Vấn đề: **Không thể xác định hàng chuyển TỪ zone/location NÀO đến zone/location NÀO**.

Giải pháp: Thêm columns `source_zone_id`, `source_location_id`, `destination_zone_id`, `destination_location_id` vào `stock_transfer_items`.

**Lý do không thêm vào stock_transfers**:
- Một transfer có thể có nhiều items từ nhiều locations khác nhau
- Mỗi item có thể có source/destination location riêng
- Flexibility cho future features (partial picking từ nhiều bins)

## 2. Implementation Steps (Specific Sub-tasks)

### 2.1 Database Migration

- [x] 1. Create migration to add zone/location columns to `stock_transfer_items`
- [x] 2. Add FK constraints
- [x] 3. Add indexes for efficient queries

### 2.2 Backend DTO Updates

- [x] 4. Update `CreateTransferItemRequest` DTO
- [x] 5. Update `TransferItemResponse` DTO

### 2.3 Repository Updates

- [x] 6. Update `TransferRepository::create_item` to include zone/location fields
- [x] 7. Update `TransferRepository::get_items_by_transfer_id` to join zone/location names
- [x] 8. Update `TransferRepository::update_item` for zone/location changes

### 2.4 Validation Rules

- [ ] 9. Add validation: `source_zone_id` must belong to `source_warehouse_id` of parent transfer
- [ ] 10. Add validation: `source_location_id` must belong to `source_zone_id` (if zone specified) or `source_warehouse_id`
- [ ] 11. Add validation: Same for destination zone/location

### 2.5 Testing

- [x] 12. Write unit tests for new DTO fields
- [x] 13. Write integration tests for transfer with zone/location
- [x] 14. Test backward compatibility (zone/location = NULL should work)

## 3. Completion Criteria

- [x] `stock_transfer_items` table has source/destination zone/location columns
- [x] FK constraints properly reference `warehouse_zones` and `warehouse_locations`
- [x] DTOs updated with new fields
- [x] Repository handles new fields correctly
- [ ] Validation ensures zone/location belong to correct warehouse (Deferred to 04.05.03)
- [x] Existing transfers (without zone/location) still work
- [x] All tests pass
- [x] Database ERD updated

## 4. SQL Migration

```sql
-- migrations/YYYYMMDD000005_add_transfer_item_locations.sql

-- Add zone/location columns to stock_transfer_items
ALTER TABLE stock_transfer_items 
  ADD COLUMN source_zone_id UUID,
  ADD COLUMN source_location_id UUID,
  ADD COLUMN destination_zone_id UUID,
  ADD COLUMN destination_location_id UUID;

-- Add FK constraints
-- Source zone must belong to source warehouse (via parent transfer)
ALTER TABLE stock_transfer_items 
  ADD CONSTRAINT fk_transfer_items_source_zone 
  FOREIGN KEY (tenant_id, source_zone_id) 
  REFERENCES warehouse_zones(tenant_id, zone_id);

ALTER TABLE stock_transfer_items 
  ADD CONSTRAINT fk_transfer_items_source_location 
  FOREIGN KEY (tenant_id, source_location_id) 
  REFERENCES warehouse_locations(tenant_id, location_id);

ALTER TABLE stock_transfer_items 
  ADD CONSTRAINT fk_transfer_items_dest_zone 
  FOREIGN KEY (tenant_id, destination_zone_id) 
  REFERENCES warehouse_zones(tenant_id, zone_id);

ALTER TABLE stock_transfer_items 
  ADD CONSTRAINT fk_transfer_items_dest_location 
  FOREIGN KEY (tenant_id, destination_location_id) 
  REFERENCES warehouse_locations(tenant_id, location_id);

-- Add indexes for location-based queries
CREATE INDEX idx_transfer_items_source_zone 
ON stock_transfer_items(tenant_id, source_zone_id) 
WHERE deleted_at IS NULL AND source_zone_id IS NOT NULL;

CREATE INDEX idx_transfer_items_source_location 
ON stock_transfer_items(tenant_id, source_location_id) 
WHERE deleted_at IS NULL AND source_location_id IS NOT NULL;

CREATE INDEX idx_transfer_items_dest_zone 
ON stock_transfer_items(tenant_id, destination_zone_id) 
WHERE deleted_at IS NULL AND destination_zone_id IS NOT NULL;

CREATE INDEX idx_transfer_items_dest_location 
ON stock_transfer_items(tenant_id, destination_location_id) 
WHERE deleted_at IS NULL AND destination_location_id IS NOT NULL;

-- Comment
COMMENT ON COLUMN stock_transfer_items.source_zone_id IS 'Source zone within source warehouse (optional, for precise tracking)';
COMMENT ON COLUMN stock_transfer_items.source_location_id IS 'Source location/bin within source warehouse (optional)';
COMMENT ON COLUMN stock_transfer_items.destination_zone_id IS 'Destination zone within destination warehouse (optional)';
COMMENT ON COLUMN stock_transfer_items.destination_location_id IS 'Destination location/bin within destination warehouse (optional)';
```

## 5. DBML Update

```dbml
Table stock_transfer_items {
  // ... existing fields ...
  source_zone_id UUID [note: 'Source zone within source warehouse']
  source_location_id UUID [note: 'Source location/bin']
  destination_zone_id UUID [note: 'Destination zone within destination warehouse']
  destination_location_id UUID [note: 'Destination location/bin']
}

// New relationships
Ref: stock_transfer_items.(tenant_id, source_zone_id) > warehouse_zones.(tenant_id, zone_id)
Ref: stock_transfer_items.(tenant_id, source_location_id) > warehouse_locations.(tenant_id, location_id)
Ref: stock_transfer_items.(tenant_id, destination_zone_id) > warehouse_zones.(tenant_id, zone_id)
Ref: stock_transfer_items.(tenant_id, destination_location_id) > warehouse_locations.(tenant_id, location_id)
```

## Related Documents

- Mini PRD: `./README.md`
- Parent Task: `./task_04.05.01_migrate_storage_locations.md`
- ERD: `docs/database-erd.dbml`
- Transfer DTO: `services/inventory_service/core/src/dto/transfer.rs`

## AI Agent Log:

* 2026-01-28 20:05: Task created for adding zone/location to transfer items
    - Designed schema change to support location-level transfers
    - Columns are optional for backward compatibility
