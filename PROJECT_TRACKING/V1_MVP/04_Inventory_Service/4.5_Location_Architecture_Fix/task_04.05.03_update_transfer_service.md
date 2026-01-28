# Task: Update transfer service for zone/location support

**Task ID:** `V1_MVP/04_Inventory_Service/4.5_Location_Architecture_Fix/task_04.05.03_update_transfer_service.md`
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.5_Location_Architecture_Fix
**Priority:** High
**Status:** Done
**Assignee:** Claude
**Created Date:** 2026-01-28
**Last Updated:** 2026-01-28
**Dependencies:**
- `V1_MVP/04_Inventory_Service/4.5_Location_Architecture_Fix/task_04.05.02_add_transfer_location_columns.md`

## 1. Detailed Description

Sau khi thêm zone/location columns vào `stock_transfer_items`, cần cập nhật Transfer Service để:

1. **Accept zone/location** trong create/update transfer item requests
2. **Validate zone/location** thuộc đúng warehouse
3. **Deduct stock từ source_location** khi ship
4. **Add stock vào destination_location** khi receive
5. **Create stock_moves** với đúng source/destination location

## 2. Implementation Steps (Specific Sub-tasks)

### 2.1 Service Layer Updates

- [x] 1. Update `TransferService::create` to accept zone/location in items
- [x] 2. Update `TransferService::add_item` to validate zone/location
- [x] 3. Update `TransferService::ship` to:
  - Deduct from `inventory_levels` at `source_location_id`
  - Create `stock_moves` with `source_location_id`
- [x] 4. Update `TransferService::receive` to:
  - Add to `inventory_levels` at `destination_location_id`
  - Create `stock_moves` with `destination_location_id`

### 2.2 Validation Logic

- [ ] 5. Add validation function `validate_zone_in_warehouse`:
  ```rust
  async fn validate_zone_in_warehouse(
      &self,
      zone_id: Uuid,
      warehouse_id: Uuid,
      tenant_id: Uuid,
  ) -> Result<(), DomainError> {
      let zone = self.zone_repo.get_by_id(tenant_id, zone_id).await?
          .ok_or(DomainError::NotFound { 
              entity_type: "Zone", 
              id: zone_id.to_string() 
          })?;
      
      if zone.warehouse_id != warehouse_id {
          return Err(DomainError::Validation(format!(
              "Zone {} does not belong to warehouse {}",
              zone_id, warehouse_id
          )));
      }
      Ok(())
  }
  ```

- [ ] 6. Add validation function `validate_location_in_zone_or_warehouse`:
  ```rust
  async fn validate_location(
      &self,
      location_id: Uuid,
      zone_id: Option<Uuid>,
      warehouse_id: Uuid,
      tenant_id: Uuid,
  ) -> Result<(), DomainError> {
      let location = self.location_repo.get_by_id(tenant_id, location_id).await?
          .ok_or(DomainError::NotFound { 
              entity_type: "Location", 
              id: location_id.to_string() 
          })?;
      
      // Must belong to warehouse
      if location.warehouse_id != warehouse_id {
          return Err(DomainError::Validation(format!(
              "Location {} does not belong to warehouse {}",
              location_id, warehouse_id
          )));
      }
      
      // If zone specified, must belong to that zone
      if let Some(zone_id) = zone_id {
          if location.zone_id != Some(zone_id) {
              return Err(DomainError::Validation(format!(
                  "Location {} does not belong to zone {}",
                  location_id, zone_id
              )));
          }
      }
      
      Ok(())
  }
  ```

### 2.3 Stock Operations

- [ ] 7. Update `ship` method:
  ```rust
  pub async fn ship(&self, tenant_id: Uuid, transfer_id: Uuid, user_id: Uuid) -> Result<TransferResponse, DomainError> {
      let transfer = self.get_transfer(tenant_id, transfer_id).await?;
      transfer.validate_can_ship()?;
      
      // For each item, deduct from source_location
      for item in &transfer.items {
          // Use source_location_id if specified, otherwise warehouse-level
          self.inventory_service.deduct_stock(
              tenant_id,
              item.product_id,
              transfer.source_warehouse_id,
              item.source_location_id,  // NEW: Location-level deduction
              item.quantity,
          ).await?;
          
          // Create stock_move with location
          self.create_stock_move(
              tenant_id,
              item.product_id,
              item.source_location_id,  // Source location
              None,                      // Destination (in transit)
              MoveType::Transfer,
              -item.quantity,            // Negative for outbound
              "transfer",
              transfer_id,
          ).await?;
      }
      
      // Update status to Shipped
      // ...
  }
  ```

- [ ] 8. Update `receive` method:
  ```rust
  pub async fn receive(&self, tenant_id: Uuid, transfer_id: Uuid, user_id: Uuid) -> Result<TransferResponse, DomainError> {
      let transfer = self.get_transfer(tenant_id, transfer_id).await?;
      transfer.validate_can_receive()?;
      
      // For each item, add to destination_location
      for item in &transfer.items {
          self.inventory_service.add_stock(
              tenant_id,
              item.product_id,
              transfer.destination_warehouse_id,
              item.destination_location_id,  // NEW: Location-level addition
              item.quantity,
          ).await?;
          
          // Create stock_move with location
          self.create_stock_move(
              tenant_id,
              item.product_id,
              None,                          // Source (from transit)
              item.destination_location_id,  // Destination location
              MoveType::Transfer,
              item.quantity,                 // Positive for inbound
              "transfer",
              transfer_id,
          ).await?;
      }
      
      // Update status to Received
      // ...
  }
  ```

### 2.4 Handler Updates

- [ ] 9. Update API handler to accept zone/location in request body
- [ ] 10. Update OpenAPI spec with new request/response fields

### 2.5 Testing

- [ ] 11. Unit test: Create transfer with zone/location
- [ ] 12. Unit test: Validate zone not in warehouse → error
- [ ] 13. Unit test: Validate location not in zone → error
- [ ] 14. Integration test: Full transfer flow with locations
- [ ] 15. Integration test: Backward compat (no location specified)

## 3. Completion Criteria

- [ ] Transfer service accepts zone/location in item requests
- [ ] Validation ensures zone/location belong to correct warehouse
- [ ] Ship operation deducts from source_location
- [ ] Receive operation adds to destination_location
- [ ] stock_moves have correct location references
- [ ] Backward compatibility maintained (NULL zone/location works)
- [ ] All existing tests still pass
- [ ] New tests for location-aware transfers pass

## 4. Code Examples

### Request Body Example

```json
{
  "source_warehouse_id": "uuid-source-wh",
  "destination_warehouse_id": "uuid-dest-wh",
  "priority": "normal",
  "items": [
    {
      "product_id": "uuid-product",
      "quantity": 100,
      "uom_id": "uuid-uom",
      "source_zone_id": "uuid-source-zone",       // NEW (optional)
      "source_location_id": "uuid-source-loc",     // NEW (optional)
      "destination_zone_id": "uuid-dest-zone",     // NEW (optional)
      "destination_location_id": "uuid-dest-loc"   // NEW (optional)
    }
  ]
}
```

### Response Body Example

```json
{
  "transfer_id": "uuid-transfer",
  "transfer_number": "ST-2026-00001",
  "status": "draft",
  "items": [
    {
      "transfer_item_id": "uuid-item",
      "product_id": "uuid-product",
      "product_name": "Widget A",
      "quantity": 100,
      "source_zone_id": "uuid-source-zone",
      "source_zone_name": "Zone A",
      "source_location_id": "uuid-source-loc",
      "source_location_code": "A-01-01-01",
      "destination_zone_id": "uuid-dest-zone",
      "destination_zone_name": "Zone B",
      "destination_location_id": "uuid-dest-loc",
      "destination_location_code": "B-02-01-01"
    }
  ]
}
```

## Related Documents

- Mini PRD: `./README.md`
- Parent Task: `./task_04.05.02_add_transfer_location_columns.md`
- Transfer Service: `services/inventory_service/infra/src/services/transfer.rs`
- Transfer Handler: `services/inventory_service/api/src/handlers/transfer.rs`

## AI Agent Log:

* 2026-01-28 20:10: Task created for transfer service location support
    - Detailed validation logic for zone/location hierarchy
    - Stock operations updated for location-level tracking
