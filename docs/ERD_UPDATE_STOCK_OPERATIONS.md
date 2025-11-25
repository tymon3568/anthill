# ERD Update - Stock Operations Tables

**Date:** 2025-11-23  
**Module:** 04_Inventory_Service / 4.4_Stock_Operations  
**Status:** ✅ Completed  
**Commit:** `324dbd4`

## Overview

Updated `docs/database-erd.dbml` to include all Stock Operations tables from module 4.4, along with supporting inventory tables. The ERD is now synchronized with the actual database schema implemented in migrations.

## Tables Added

### Core Inventory Tables (Supporting)

#### 1. `products` - Item Master
- **Primary Key:** `product_id` (UUID v7)
- **Purpose:** Single source of truth for all product data
- **Key Fields:**
  - SKU, name, description
  - Product type (goods/service/consumable)
  - Tracking method (none/lot/serial)
  - Pricing (sale_price, cost_price in cents/xu)
  - Weight, dimensions, attributes
- **Indexes:** Tenant + SKU unique, tenant-scoped lookups
- **References:** Used by all stock operation line items

#### 2. `unit_of_measures` - UOM
- **Primary Key:** `uom_id` (UUID v7)
- **Purpose:** Units of measure for quantity tracking
- **Key Fields:**
  - UOM code (PC, KG, M, etc.)
  - UOM type (unit/weight/length/volume/time)
  - Base unit and conversion factors
- **Indexes:** Tenant + UOM code unique
- **References:** Used by all line item tables for quantity UOM

#### 3. `warehouses` - Warehouse Locations
- **Primary Key:** `warehouse_id` (UUID v4)
- **Purpose:** Physical warehouse locations with hierarchy
- **Key Fields:**
  - Warehouse code, name
  - Warehouse type (main/transit/quarantine/distribution/retail/satellite)
  - Parent warehouse (unlimited hierarchy support)
  - Address, contact, capacity info (JSONB)
- **Indexes:** Tenant + warehouse code unique
- **References:** Used by all stock operations

---

### Stock Operations Tables (Module 4.4)

#### 4. `goods_receipts` - GRN (Goods Receipt Notes)
- **Primary Key:** `receipt_id` (UUID v7)
- **Purpose:** Recording incoming goods from suppliers
- **Auto-generated Number:** `GRN-YYYY-XXXXX`
- **Key Fields:**
  - Receipt number, reference number
  - Warehouse, supplier references
  - Status (draft/confirmed/partially_received/received/cancelled)
  - Expected vs actual delivery dates
  - Total quantity, total value (in cents/xu)
- **Indexes:** 
  - Tenant + receipt number unique
  - Tenant + warehouse, status, date
- **Foreign Keys:**
  - `(tenant_id, warehouse_id)` → warehouses
  - `(tenant_id, created_by)` → users

#### 5. `goods_receipt_items` - GRN Line Items
- **Primary Key:** `receipt_item_id` (UUID v7)
- **Purpose:** Individual products received in GRN
- **Key Fields:**
  - Expected vs received quantities
  - Unit cost, line total
  - Lot number, serial numbers, expiry date
- **Indexes:**
  - Tenant + receipt, product
  - Lot number lookup
- **Foreign Keys:**
  - `(tenant_id, receipt_id)` → goods_receipts
  - `(tenant_id, product_id)` → products
  - `(tenant_id, uom_id)` → unit_of_measures

#### 6. `delivery_orders` - DO (Delivery Orders)
- **Primary Key:** `delivery_id` (UUID v7)
- **Purpose:** Outbound shipments to customers
- **Auto-generated Number:** `DO-YYYY-XXXXX`
- **Key Fields:**
  - Delivery number, reference number
  - Warehouse, order, customer references
  - Status (draft/confirmed/partially_picked/picked/partially_shipped/shipped/cancelled)
  - Shipping method, carrier, tracking number
  - Shipping cost, total value
- **Indexes:**
  - Tenant + delivery number unique
  - Tenant + warehouse, status, tracking number
- **Foreign Keys:**
  - `(tenant_id, warehouse_id)` → warehouses
  - `(tenant_id, created_by)` → users

#### 7. `delivery_order_items` - DO Line Items
- **Primary Key:** `delivery_item_id` (UUID v7)
- **Purpose:** Individual products shipped in DO
- **Key Fields:**
  - Ordered, picked, delivered quantities
  - Unit price, line total
  - Lot number, serial numbers
- **Indexes:**
  - Tenant + delivery, product
- **Foreign Keys:**
  - `(tenant_id, delivery_id)` → delivery_orders
  - `(tenant_id, product_id)` → products

#### 8. `stock_transfers` - ST (Stock Transfers)
- **Primary Key:** `transfer_id` (UUID v7)
- **Purpose:** Internal transfers between warehouses
- **Auto-generated Number:** `ST-YYYY-XXXXX`
- **Key Fields:**
  - Transfer number, reference number
  - Source and destination warehouses
  - Status (draft/confirmed/picked/shipped/received/cancelled)
  - Transfer type (manual/auto_replenishment/emergency/consolidation)
  - Priority (low/normal/high/urgent)
  - Approval workflow (approved_by, approved_at)
  - Shipping details (method, carrier, tracking, cost)
- **Indexes:**
  - Tenant + transfer number unique
  - Tenant + source/destination warehouses
  - Status, type, priority lookups
- **Foreign Keys:**
  - `(tenant_id, source_warehouse_id)` → warehouses
  - `(tenant_id, destination_warehouse_id)` → warehouses
  - `(tenant_id, created_by/updated_by/approved_by)` → users

#### 9. `stock_transfer_items` - ST Line Items
- **Primary Key:** `transfer_item_id` (UUID v7)
- **Purpose:** Individual products in stock transfers
- **Key Fields:**
  - Quantity, UOM
  - Unit cost, line total (auto-calculated)
  - Line number for ordering
- **Indexes:**
  - Tenant + transfer + line number unique
  - Tenant + transfer, product
- **Foreign Keys:**
  - `(tenant_id, transfer_id)` → stock_transfers
  - `(tenant_id, product_id)` → products
  - `(tenant_id, uom_id)` → unit_of_measures

#### 10. `stock_takes` - STK (Stock Takes)
- **Primary Key:** `stock_take_id` (UUID v7)
- **Purpose:** Physical inventory counting sessions
- **Auto-generated Number:** `STK-YYYY-XXXXX`
- **Key Fields:**
  - Stock take number, reference number
  - Warehouse reference
  - Status (Draft/Scheduled/InProgress/Completed/Cancelled)
  - Scheduled/started/completed dates
  - User assignments (created_by, assigned_to)
- **Indexes:**
  - Tenant + stock take number unique
  - Tenant + warehouse, status, type
  - User assignments
- **Foreign Keys:**
  - `(tenant_id, warehouse_id)` → warehouses
  - `(tenant_id, created_by/assigned_to)` → users

#### 11. `stock_take_lines` - STK Line Items
- **Primary Key:** `line_id` (UUID v7)
- **Purpose:** Individual product counts in stock takes
- **Key Fields:**
  - Expected vs actual quantities
  - **difference_quantity** (GENERATED COLUMN: actual - expected)
  - Counted by user and timestamp
- **Indexes:**
  - Tenant + stock_take + product unique
  - Tenant + stock_take, product
  - Counted by user
- **Foreign Keys:**
  - `(tenant_id, stock_take_id)` → stock_takes
  - `(tenant_id, product_id)` → products
  - `(tenant_id, counted_by)` → users

---

## Key Design Patterns Documented

### 1. Multi-Tenancy
- All tables have `tenant_id` field
- Composite foreign keys: `(tenant_id, parent_id)`
- All indexes scoped by tenant for performance
- Application-level filtering (no RLS)

### 2. UUID v7 Primary Keys
- Timestamp-based UUIDs for better index locality
- All tables use `uuid_generate_v7()` default

### 3. Money as BIGINT
- All monetary values stored in smallest unit (cents/xu)
- Fields: `unit_cost`, `line_total`, `total_value`, `shipping_cost`
- Currency code stored separately (default: VND)

### 4. Soft Delete Pattern
- All tables have `deleted_at TIMESTAMPTZ` column
- Indexes use `WHERE deleted_at IS NULL` for active records
- Preserves audit trail

### 5. Auto-generated Document Numbers
- Sequence-based generation: `XXX-YYYY-XXXXX`
- Year-based partitioning
- Format examples:
  - `GRN-2025-00001`
  - `DO-2025-00001`
  - `ST-2025-00001`
  - `STK-2025-00001`

### 6. Status Workflows
- **GRN:** draft → confirmed → partially_received → received | cancelled
- **DO:** draft → confirmed → partially_picked → picked → partially_shipped → shipped | cancelled
- **ST:** draft → confirmed → picked → shipped → received | cancelled
- **STK:** Draft → Scheduled → InProgress → Completed | Cancelled

### 7. Audit Fields
- `created_at`, `updated_at` (auto-managed via triggers)
- `created_by`, `updated_by` (user references)
- `approved_by`, `approved_at` (approval workflows)
- `deleted_at` (soft delete)

---

## Relationships Summary

### Parent-Child Relationships
```
goods_receipts (1) → (N) goods_receipt_items
delivery_orders (1) → (N) delivery_order_items
stock_transfers (1) → (N) stock_transfer_items
stock_takes (1) → (N) stock_take_lines
```

### Warehouse References
```
goods_receipts → warehouses (receiving warehouse)
delivery_orders → warehouses (shipping warehouse)
stock_transfers → warehouses (source + destination)
stock_takes → warehouses (counting warehouse)
```

### Product References
```
All line item tables → products (product being transacted)
All line item tables → unit_of_measures (quantity UOM)
```

### User References
```
All header tables → users (created_by)
stock_transfers → users (created_by, updated_by, approved_by)
stock_takes → users (created_by, assigned_to)
stock_take_lines → users (counted_by)
```

---

## How to Use the ERD

### Viewing the Diagram
1. Copy the entire content of `docs/database-erd.dbml`
2. Go to https://dbdiagram.io/d
3. Paste the code into the editor
4. View interactive database diagram with relationships
5. Export as PNG/PDF/SQL if needed

### DBML Features Used
- Table definitions with full column specs
- Index definitions with partial indexes
- Foreign key relationships (Ref syntax)
- Comments and notes
- Project-level documentation

---

## Validation Checklist

- [x] All 8 Stock Operations tables included
- [x] All supporting tables (products, warehouses, UOM) included
- [x] Column definitions match actual migrations
- [x] Foreign key relationships documented
- [x] Indexes documented with WHERE clauses
- [x] Soft delete patterns shown
- [x] Multi-tenancy composite keys shown
- [x] Comments and notes added
- [x] Auto-generated number patterns documented
- [x] Money-as-BIGINT convention shown
- [x] UUID v7 defaults documented

---

## Migration Files Referenced

- `20250110000017_create_products_table.sql`
- `20250110000018_create_unit_of_measures_table.sql`
- `20250110000023_create_warehouse_tables.sql`
- `20250110000028_create_goods_receipts_table.sql`
- `20250110000029_create_goods_receipt_items_table.sql`
- `20250110000030_create_delivery_orders_table.sql`
- `20250110000031_create_delivery_order_items_table.sql`
- `20250121000001_create_stock_transfers_table.sql`
- `20250121000002_create_stock_transfer_items_table.sql`
- `20250121000006_create_stock_takes_table.sql`
- `20251123000007_create_stock_take_lines_table.sql`

---

## Next Steps

### Immediate (High Priority)
- [ ] Complete RMA tables (Task 04.04.17 - currently Todo)
  - `rma_requests` table
  - `rma_items` table
  - Add to ERD when completed

### Documentation Updates
- [x] ERD synchronized with migrations ✅
- [ ] Update ARCHITECTURE.md with Stock Operations flow
- [ ] Create sequence diagrams for each operation type
- [ ] Document API endpoints for each operation

### Future Enhancements (Per INVENTORY_IMPROVE.md)
- [ ] Add Warehouse Management advanced features to ERD:
  - Putaway Rules table
  - Picking Methods table
  - Removal Strategies table
  - Cycle Count Schedules table
  - Scrap Management table
- [ ] Consider module restructuring (merge 4.3 + 4.4)
- [ ] Add Quality Management tables (separate module)

---

## Notes

- ERD now serves as the single source of truth for database schema
- All future table additions should update the ERD immediately
- Visualization at dbdiagram.io helps with understanding relationships
- ERD file is version-controlled alongside migrations

---

**Prepared by:** Claude (AI Agent)  
**Related Task:** 04.04 Stock Operations  
**Document Version:** 1.0
