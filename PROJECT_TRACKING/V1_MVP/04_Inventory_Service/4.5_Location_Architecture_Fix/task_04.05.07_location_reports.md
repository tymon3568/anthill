# Task: Add location-level reports

**Task ID:** `V1_MVP/04_Inventory_Service/4.5_Location_Architecture_Fix/task_04.05.07_location_reports.md`
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.5_Location_Architecture_Fix
**Priority:** Medium
**Status:** Todo
**Assignee:**
**Created Date:** 2026-01-28
**Last Updated:** 2026-01-28
**Dependencies:**
- `V1_MVP/04_Inventory_Service/4.5_Location_Architecture_Fix/task_04.05.04_fix_inventory_levels_location.md`

## 1. Detailed Description

Sau khi inventory_levels track stock ở location level, cần thêm các báo cáo:

1. **Stock by Location Report**: Xem stock breakdown theo warehouse → zone → location
2. **Location Utilization Report**: Xem capacity usage của từng location
3. **Movement by Location Report**: Xem lịch sử movements của location
4. **Empty Locations Report**: Xem locations không có stock

## 2. Implementation Steps (Specific Sub-tasks)

### 2.1 Backend API Endpoints

- [ ] 1. Add endpoint `GET /api/v1/inventory/reports/stock-by-location`:
  ```rust
  pub async fn get_stock_by_location(
      State(state): State<AppState>,
      Query(params): Query<StockByLocationParams>,
  ) -> Result<Json<StockByLocationReport>, AppError>
  
  struct StockByLocationParams {
      warehouse_id: Option<Uuid>,
      zone_id: Option<Uuid>,
      product_id: Option<Uuid>,
      include_empty: bool,
  }
  
  struct StockByLocationReport {
      warehouses: Vec<WarehouseStock>,
  }
  
  struct WarehouseStock {
      warehouse_id: Uuid,
      warehouse_name: String,
      total_stock: i64,
      zones: Vec<ZoneStock>,
  }
  
  struct ZoneStock {
      zone_id: Uuid,
      zone_name: String,
      total_stock: i64,
      locations: Vec<LocationStock>,
  }
  
  struct LocationStock {
      location_id: Uuid,
      location_code: String,
      products: Vec<ProductStock>,
  }
  ```

- [ ] 2. Add endpoint `GET /api/v1/inventory/reports/location-utilization`:
  ```rust
  struct LocationUtilization {
      location_id: Uuid,
      location_code: String,
      zone_name: String,
      warehouse_name: String,
      capacity: i64,
      current_stock: i64,
      utilization_percent: f64,
      products_count: i64,
  }
  ```

- [ ] 3. Add endpoint `GET /api/v1/inventory/reports/location-movements`:
  ```rust
  struct LocationMovementReport {
      location_id: Uuid,
      location_code: String,
      period_start: DateTime<Utc>,
      period_end: DateTime<Utc>,
      inbound_qty: i64,
      outbound_qty: i64,
      net_change: i64,
      movements: Vec<StockMoveResponse>,
  }
  ```

- [ ] 4. Add endpoint `GET /api/v1/inventory/reports/empty-locations`:
  ```rust
  struct EmptyLocation {
      location_id: Uuid,
      location_code: String,
      zone_name: String,
      warehouse_name: String,
      capacity: i64,
      last_movement_at: Option<DateTime<Utc>>,
  }
  ```

### 2.2 SQL Queries

- [ ] 5. Implement stock by location query:
  ```sql
  SELECT 
    w.warehouse_id, w.warehouse_name,
    z.zone_id, z.zone_name,
    l.location_id, l.location_code,
    p.product_id, p.sku, p.name,
    COALESCE(il.available_quantity, 0) as available,
    COALESCE(il.reserved_quantity, 0) as reserved
  FROM warehouse_locations l
  JOIN warehouses w ON w.warehouse_id = l.warehouse_id
  LEFT JOIN warehouse_zones z ON z.zone_id = l.zone_id
  LEFT JOIN inventory_levels il ON il.location_id = l.location_id
  LEFT JOIN products p ON p.product_id = il.product_id
  WHERE l.tenant_id = $1
    AND l.deleted_at IS NULL
    AND (l.warehouse_id = $2 OR $2 IS NULL)
    AND (l.zone_id = $3 OR $3 IS NULL)
  ORDER BY w.warehouse_name, z.zone_name, l.location_code, p.sku;
  ```

- [ ] 6. Implement location utilization query:
  ```sql
  SELECT 
    l.location_id, l.location_code,
    z.zone_name,
    w.warehouse_name,
    l.capacity,
    COALESCE(SUM(il.available_quantity + il.reserved_quantity), 0) as current_stock,
    CASE 
      WHEN l.capacity > 0 THEN 
        ROUND(COALESCE(SUM(il.available_quantity + il.reserved_quantity), 0)::numeric / l.capacity * 100, 2)
      ELSE 0 
    END as utilization_percent,
    COUNT(DISTINCT il.product_id) as products_count
  FROM warehouse_locations l
  JOIN warehouses w ON w.warehouse_id = l.warehouse_id
  LEFT JOIN warehouse_zones z ON z.zone_id = l.zone_id
  LEFT JOIN inventory_levels il ON il.location_id = l.location_id AND il.deleted_at IS NULL
  WHERE l.tenant_id = $1
    AND l.deleted_at IS NULL
  GROUP BY l.location_id, l.location_code, z.zone_name, w.warehouse_name, l.capacity
  ORDER BY utilization_percent DESC;
  ```

### 2.3 Frontend Report Pages

- [ ] 7. Create Stock by Location Report page:
  - Tree view: Warehouse → Zone → Location → Products
  - Expandable/collapsible hierarchy
  - Export to CSV/Excel

- [ ] 8. Create Location Utilization Report page:
  - Table with utilization bars
  - Color coding (red > 90%, yellow > 70%, green < 70%)
  - Sort by utilization %

- [ ] 9. Create Location Movements Report page:
  - Date range selector
  - Location selector
  - Timeline visualization
  - Movement list

- [ ] 10. Create Empty Locations Report page:
  - List of empty locations
  - Filter by warehouse/zone
  - Last movement date

### 2.4 Testing

- [ ] 11. Unit test: Stock by location aggregation
- [ ] 12. Unit test: Utilization calculation
- [ ] 13. Integration test: Report endpoints
- [ ] 14. E2E test: Report pages render correctly

## 3. Completion Criteria

- [ ] 4 report endpoints implemented
- [ ] Reports aggregate data correctly at location/zone/warehouse levels
- [ ] Frontend pages for all reports
- [ ] Export to CSV/Excel works
- [ ] Reports handle empty data gracefully
- [ ] Performance acceptable (< 2s for typical dataset)
- [ ] All tests pass

## 4. Routes

```
/inventory/reports
├── /stock-by-location      # Hierarchical stock view
├── /location-utilization   # Capacity usage
├── /location-movements     # Movement history
└── /empty-locations        # Empty bins
```

## Related Documents

- Mini PRD: `./README.md`
- Parent Task: `./task_04.05.04_fix_inventory_levels_location.md`
- Reports API: `services/inventory_service/api/src/handlers/reports.rs` (if exists)
- Frontend Reports: `frontend/src/routes/(protected)/inventory/reports/`

## AI Agent Log:

* 2026-01-28 20:30: Task created for location-level reports
    - Designed 4 report types for location management
    - Included SQL queries for efficient aggregation
