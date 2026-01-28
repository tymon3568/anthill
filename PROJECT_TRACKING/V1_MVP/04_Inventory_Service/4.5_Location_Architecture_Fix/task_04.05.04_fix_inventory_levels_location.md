# Task: Fix inventory_levels location tracking

**Task ID:** `V1_MVP/04_Inventory_Service/4.5_Location_Architecture_Fix/task_04.05.04_fix_inventory_levels_location.md`
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

Hiện tại `inventory_levels.location_id` **LUÔN NULL** vì code intentionally set `location_id = None`:

```rust
// services/inventory_service/infra/src/services/transfer.rs:218-221
// Note: We use None for location_id because inventory_levels.location_id references
// storage_locations table, while warehouse_locations is a separate table used for stock_moves.
// Until the schema is unified, we track inventory at the warehouse level only.
```

Sau khi hoàn thành Task 04.05.01 (unify tables), cần cập nhật tất cả services để:

1. **Properly set location_id** khi add/deduct stock
2. **Support location-level queries** for inventory
3. **Maintain warehouse-level aggregates** for backward compatibility

## 2. Implementation Steps (Specific Sub-tasks)

### 2.1 Stock Levels Service Updates

- [ ] 1. Update `StockLevelsService::add_stock` to accept `location_id`:
  ```rust
  pub async fn add_stock(
      &self,
      tenant_id: Uuid,
      product_id: Uuid,
      warehouse_id: Uuid,
      location_id: Option<Uuid>,  // NEW: Optional location
      quantity: i64,
  ) -> Result<InventoryLevel, DomainError> {
      // If location_id provided, update/create location-level record
      // Otherwise, update/create warehouse-level record (location_id = NULL)
  }
  ```

- [ ] 2. Update `StockLevelsService::deduct_stock` similarly

- [ ] 3. Add method `StockLevelsService::get_stock_by_location`:
  ```rust
  pub async fn get_stock_by_location(
      &self,
      tenant_id: Uuid,
      product_id: Uuid,
      location_id: Uuid,
  ) -> Result<Option<InventoryLevel>, DomainError>
  ```

- [ ] 4. Add method `StockLevelsService::get_stock_by_zone`:
  ```rust
  pub async fn get_stock_by_zone(
      &self,
      tenant_id: Uuid,
      product_id: Uuid,
      zone_id: Uuid,
  ) -> Result<Vec<InventoryLevel>, DomainError>
  ```

### 2.2 Repository Updates

- [ ] 5. Update `InventoryRepository::upsert` to handle location_id:
  ```sql
  INSERT INTO inventory_levels (tenant_id, product_id, warehouse_id, location_id, available_quantity)
  VALUES ($1, $2, $3, $4, $5)
  ON CONFLICT (tenant_id, warehouse_id, location_id, product_id) 
  WHERE deleted_at IS NULL
  DO UPDATE SET 
    available_quantity = inventory_levels.available_quantity + EXCLUDED.available_quantity,
    updated_at = NOW()
  RETURNING *;
  ```

- [ ] 6. Add query `get_by_product_location`:
  ```sql
  SELECT * FROM inventory_levels
  WHERE tenant_id = $1 
    AND product_id = $2 
    AND location_id = $3
    AND deleted_at IS NULL;
  ```

- [ ] 7. Add aggregation query `get_warehouse_total`:
  ```sql
  SELECT 
    tenant_id, product_id, warehouse_id,
    SUM(available_quantity) as total_available,
    SUM(reserved_quantity) as total_reserved
  FROM inventory_levels
  WHERE tenant_id = $1 
    AND product_id = $2 
    AND warehouse_id = $3
    AND deleted_at IS NULL
  GROUP BY tenant_id, product_id, warehouse_id;
  ```

### 2.3 GRN Service Updates

- [ ] 8. Update `GrnService::receive_goods` to set location_id:
  ```rust
  // When receiving goods, use default receiving location or specified location
  let receiving_location = self.get_default_receiving_location(
      tenant_id, 
      grn.warehouse_id
  ).await?;
  
  self.stock_service.add_stock(
      tenant_id,
      item.product_id,
      grn.warehouse_id,
      Some(receiving_location.location_id),  // Use location!
      item.received_quantity,
  ).await?;
  ```

### 2.4 Delivery Order Service Updates

- [ ] 9. Update `DoService::pick_items` to deduct from specific location
- [ ] 10. Update `DoService::ship` to finalize location-level deduction

### 2.5 Stock Adjustment Service Updates

- [ ] 11. Update stock adjustment to accept location_id

### 2.6 Backward Compatibility

- [ ] 12. Ensure `location_id = NULL` records still work
- [ ] 13. Add migration to consolidate NULL location records if needed:
  ```sql
  -- Keep one record per warehouse when location is NULL
  -- (Aggregated warehouse-level stock)
  ```

### 2.7 Testing

- [ ] 14. Unit test: Add stock to specific location
- [ ] 15. Unit test: Deduct stock from specific location
- [ ] 16. Unit test: Get stock by location
- [ ] 17. Integration test: GRN with location
- [ ] 18. Integration test: Transfer with location (from task 04.05.03)
- [ ] 19. Test warehouse-level aggregation still works

## 3. Completion Criteria

- [ ] `inventory_levels.location_id` is properly set when location is specified
- [ ] Stock can be queried at location level
- [ ] Stock can be queried at zone level (aggregated)
- [ ] Stock can be queried at warehouse level (aggregated)
- [ ] GRN receives stock into specific locations
- [ ] DO picks stock from specific locations
- [ ] Transfers move stock between specific locations
- [ ] Backward compatibility: NULL location_id still works
- [ ] All tests pass

## 4. Data Model Considerations

### Current State
```
inventory_levels:
  product_id=P1, warehouse_id=W1, location_id=NULL, available=100
  product_id=P1, warehouse_id=W1, location_id=NULL, available=50  ← DUPLICATE BUG!
```

### Target State
```
inventory_levels:
  product_id=P1, warehouse_id=W1, location_id=L1, available=30   ← Location-level
  product_id=P1, warehouse_id=W1, location_id=L2, available=40   ← Location-level
  product_id=P1, warehouse_id=W1, location_id=L3, available=30   ← Location-level
  
  -- Warehouse total = 30 + 40 + 30 = 100 (computed by SUM query)
```

### Migration Strategy

If existing data has warehouse-level records (location_id = NULL), we have two options:

**Option A: Keep warehouse-level records** (Recommended)
- Allow both location-level and warehouse-level records to coexist
- Warehouse-level records represent "unlocated" stock
- New stock goes to locations, old stock stays at warehouse level

**Option B: Migrate to default location**
- Create a "Default" location in each warehouse
- Move all NULL-location stock to this default location
- More complex, but cleaner data model

## 5. SQL Queries

### Get stock at location
```sql
SELECT * FROM inventory_levels
WHERE tenant_id = $1 
  AND product_id = $2 
  AND warehouse_id = $3 
  AND location_id = $4
  AND deleted_at IS NULL;
```

### Get stock at warehouse (aggregated)
```sql
SELECT 
  tenant_id, 
  product_id, 
  warehouse_id,
  SUM(available_quantity) as total_available,
  SUM(reserved_quantity) as total_reserved
FROM inventory_levels
WHERE tenant_id = $1 
  AND product_id = $2 
  AND warehouse_id = $3
  AND deleted_at IS NULL
GROUP BY tenant_id, product_id, warehouse_id;
```

### Get stock at zone (aggregated)
```sql
SELECT 
  il.tenant_id, 
  il.product_id, 
  wl.zone_id,
  SUM(il.available_quantity) as total_available,
  SUM(il.reserved_quantity) as total_reserved
FROM inventory_levels il
JOIN warehouse_locations wl ON wl.location_id = il.location_id
WHERE il.tenant_id = $1 
  AND il.product_id = $2 
  AND wl.zone_id = $3
  AND il.deleted_at IS NULL
GROUP BY il.tenant_id, il.product_id, wl.zone_id;
```

## Related Documents

- Mini PRD: `./README.md`
- Parent Task: `./task_04.05.01_migrate_storage_locations.md`
- Stock Levels Service: `services/inventory_service/infra/src/services/stock_levels.rs`
- Inventory Repository: `services/inventory_service/infra/src/repositories/stock.rs`

## AI Agent Log:

* 2026-01-28 20:15: Task created for fixing inventory_levels location tracking
    - Identified root cause in transfer.rs comment
    - Designed location-level stock tracking solution
