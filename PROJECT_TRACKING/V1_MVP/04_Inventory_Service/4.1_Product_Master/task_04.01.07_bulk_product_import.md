# Task: Implement Bulk Product Import API

**Task ID:** V1_MVP/04_Inventory_Service/4.1_Product_Master/task_04.01.07_bulk_product_import.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.1_Product_Master
**Priority:** Medium
**Status:** Todo
**Assignee:** Unassigned
**Created Date:** 2025-12-13
**Last Updated:** 2025-12-13

## Detailed Description:
Implement bulk operations API for product management to support high-volume data import and mass updates. This includes:
- Bulk product creation/import endpoint
- Mass stock adjustments endpoint
- Performance benchmarks to ensure operations complete within acceptable timeframes

Target performance: Import 10,000 products in < 5 minutes.

## Completion Criteria (Acceptance Criteria):
- [ ] Bulk product import endpoint implemented and tested
- [ ] Mass stock adjustments endpoint implemented
- [ ] Performance benchmarks pass target thresholds
- [ ] API documentation updated
- [ ] Error handling for partial failures (transaction rollback or skip)
- [ ] Rate limiting and request size limits configured

## Specific Sub-tasks:
- [ ] 1. Implement Bulk Product Import API
    - [ ] 1.1. Create `POST /api/v1/products/bulk` endpoint
    - [ ] 1.2. Define `BulkProductImportRequest` and `BulkProductImportResponse` DTOs
    - [ ] 1.3. Implement batch insert with transaction support
    - [ ] 1.4. Handle duplicate SKU conflicts (skip/update/error modes)
    - [ ] 1.5. Support CSV/JSON input formats
    - [ ] 1.6. Add request size limit (max 10,000 products per request)
- [ ] 2. Implement Mass Stock Adjustments API
    - [ ] 2.1. Create `POST /api/v1/stock/adjustments/bulk` endpoint
    - [ ] 2.2. Define `BulkStockAdjustmentRequest` DTO
    - [ ] 2.3. Implement atomic batch updates with row-level locking
    - [ ] 2.4. Validate all SKUs exist before processing
    - [ ] 2.5. Generate stock move records for audit trail
- [ ] 3. Write Integration Tests
    - [ ] 3.1. Test bulk import with 100, 1000, 10000 products
    - [ ] 3.2. Test partial failure handling
    - [ ] 3.3. Test concurrent bulk operations
    - [ ] 3.4. Test validation errors (duplicate SKUs, invalid data)
- [ ] 4. Performance Benchmarks
    - [ ] 4.1. Benchmark: Import 10,000 products < 5 minutes
    - [ ] 4.2. Benchmark: Mass adjust 10,000 stock levels < 2 minutes
    - [ ] 4.3. Add to CI performance regression tests

## Dependencies:
* task_04.01.01_create_products_table.md (Done)
* task_04.01.05_create_product_categories_api.md (Done)
* task_04.13.05_performance_tests.md (InProgress - will consume benchmarks)

## Related Documents:
* Product API documentation
* Performance testing guidelines

## Technical Notes:
### Suggested File Locations:
- Handler: `services/inventory_service/api/src/handlers/product.rs` (new)
- Service: `services/inventory_service/core/src/services/product.rs`
- Repository: `services/inventory_service/infra/src/repositories/product.rs`
- DTOs: `services/inventory_service/core/src/dto/product.rs`

### API Design:
```
POST /api/v1/products/bulk
Content-Type: application/json

{
  "mode": "upsert",  // "insert" | "upsert" | "skip_duplicates"
  "products": [
    {
      "sku": "PROD-001",
      "name": "Product 1",
      "category_id": "uuid",
      "uom_id": "uuid",
      "attributes": {}
    }
  ]
}

Response:
{
  "total": 10000,
  "created": 9500,
  "updated": 400,
  "skipped": 100,
  "errors": [
    {"sku": "PROD-ERR", "error": "Invalid category_id"}
  ]
}
```

## AI Agent Log:
* 2025-12-13 08:28: Task created based on performance test requirements (task_04.13.05)
