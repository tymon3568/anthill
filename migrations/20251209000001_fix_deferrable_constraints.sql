-- Migration: Fix DEFERRABLE constraints in stock operations tables
-- Description: Makes foreign key and unique constraints DEFERRABLE INITIALLY DEFERRED for bulk operations
-- Dependencies: All stock operations tables must exist
-- Created: 2025-12-09

-- ==================================
-- FIX DEFERRABLE CONSTRAINTS FOR STOCK OPERATIONS TABLES
-- ==================================

-- This migration addresses the deferrability issue with foreign key and unique constraints
-- in stock operations tables. Making them DEFERRABLE INITIALLY DEFERRED allows constraint
-- checking to be deferred until the end of the transaction, enabling bulk operations that
-- might temporarily violate the constraints during data loading or migrations.

-- ==================================
-- GOODS_RECEIPTS TABLE
-- ==================================

-- Drop existing constraints
ALTER TABLE goods_receipts DROP CONSTRAINT goods_receipts_tenant_warehouse_fk;
ALTER TABLE goods_receipts DROP CONSTRAINT goods_receipts_tenant_created_by_fk;
ALTER TABLE goods_receipts DROP CONSTRAINT goods_receipts_tenant_receipt_unique;

-- Recreate as deferrable
ALTER TABLE goods_receipts ADD CONSTRAINT goods_receipts_tenant_warehouse_fk
    FOREIGN KEY (tenant_id, warehouse_id)
    REFERENCES warehouses (tenant_id, warehouse_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE goods_receipts ADD CONSTRAINT goods_receipts_tenant_created_by_fk
    FOREIGN KEY (tenant_id, created_by)
    REFERENCES users (tenant_id, user_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE goods_receipts ADD CONSTRAINT goods_receipts_tenant_receipt_unique
    UNIQUE (tenant_id, receipt_id) DEFERRABLE INITIALLY DEFERRED;

-- ==================================
-- DELIVERY_ORDERS TABLE
-- ==================================

-- Drop existing constraints
ALTER TABLE delivery_orders DROP CONSTRAINT delivery_orders_tenant_warehouse_fk;
ALTER TABLE delivery_orders DROP CONSTRAINT delivery_orders_tenant_created_by_fk;
ALTER TABLE delivery_orders DROP CONSTRAINT delivery_orders_tenant_delivery_unique;

-- Recreate as deferrable
ALTER TABLE delivery_orders ADD CONSTRAINT delivery_orders_tenant_warehouse_fk
    FOREIGN KEY (tenant_id, warehouse_id)
    REFERENCES warehouses (tenant_id, warehouse_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE delivery_orders ADD CONSTRAINT delivery_orders_tenant_created_by_fk
    FOREIGN KEY (tenant_id, created_by)
    REFERENCES users (tenant_id, user_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE delivery_orders ADD CONSTRAINT delivery_orders_tenant_delivery_unique
    UNIQUE (tenant_id, delivery_id) DEFERRABLE INITIALLY DEFERRED;

-- ==================================
-- STOCK_TRANSFERS TABLE
-- ==================================

-- Drop existing constraints
ALTER TABLE stock_transfers DROP CONSTRAINT stock_transfers_tenant_source_warehouse_fk;
ALTER TABLE stock_transfers DROP CONSTRAINT stock_transfers_tenant_destination_warehouse_fk;
ALTER TABLE stock_transfers DROP CONSTRAINT stock_transfers_tenant_created_by_fk;
ALTER TABLE stock_transfers DROP CONSTRAINT stock_transfers_tenant_transfer_unique;

-- Recreate as deferrable
ALTER TABLE stock_transfers ADD CONSTRAINT stock_transfers_tenant_source_warehouse_fk
    FOREIGN KEY (tenant_id, source_warehouse_id)
    REFERENCES warehouses (tenant_id, warehouse_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE stock_transfers ADD CONSTRAINT stock_transfers_tenant_destination_warehouse_fk
    FOREIGN KEY (tenant_id, destination_warehouse_id)
    REFERENCES warehouses (tenant_id, warehouse_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE stock_transfers ADD CONSTRAINT stock_transfers_tenant_created_by_fk
    FOREIGN KEY (tenant_id, created_by)
    REFERENCES users (tenant_id, user_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE stock_transfers ADD CONSTRAINT stock_transfers_tenant_transfer_unique
    UNIQUE (tenant_id, transfer_id) DEFERRABLE INITIALLY DEFERRED;

-- ==================================
-- STOCK_TAKES TABLE
-- ==================================

-- Drop existing constraints
ALTER TABLE stock_takes DROP CONSTRAINT stock_takes_number_unique_per_tenant;
ALTER TABLE stock_takes DROP CONSTRAINT stock_takes_tenant_id_unique;
ALTER TABLE stock_takes DROP CONSTRAINT stock_takes_tenant_warehouse_fk;
ALTER TABLE stock_takes DROP CONSTRAINT stock_takes_tenant_created_by_fk;
ALTER TABLE stock_takes DROP CONSTRAINT stock_takes_tenant_assigned_to_fk;

-- Recreate as deferrable
ALTER TABLE stock_takes ADD CONSTRAINT stock_takes_number_unique_per_tenant
    UNIQUE (tenant_id, stock_take_number) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE stock_takes ADD CONSTRAINT stock_takes_tenant_id_unique
    UNIQUE (tenant_id, stock_take_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE stock_takes ADD CONSTRAINT stock_takes_tenant_warehouse_fk
    FOREIGN KEY (tenant_id, warehouse_id)
    REFERENCES warehouses (tenant_id, warehouse_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE stock_takes ADD CONSTRAINT stock_takes_tenant_created_by_fk
    FOREIGN KEY (tenant_id, created_by)
    REFERENCES users (tenant_id, user_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE stock_takes ADD CONSTRAINT stock_takes_tenant_assigned_to_fk
    FOREIGN KEY (tenant_id, assigned_to)
    REFERENCES users (tenant_id, user_id) DEFERRABLE INITIALLY DEFERRED;

-- ==================================
-- STOCK_TAKE_LINES TABLE
-- ==================================

-- Drop existing constraints
ALTER TABLE stock_take_lines DROP CONSTRAINT stock_take_lines_tenant_stock_take_fk;
ALTER TABLE stock_take_lines DROP CONSTRAINT stock_take_lines_tenant_product_fk;
ALTER TABLE stock_take_lines DROP CONSTRAINT stock_take_lines_tenant_counted_by_fk;

-- Recreate as deferrable
ALTER TABLE stock_take_lines ADD CONSTRAINT stock_take_lines_tenant_stock_take_fk
    FOREIGN KEY (tenant_id, stock_take_id)
    REFERENCES stock_takes (tenant_id, stock_take_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE stock_take_lines ADD CONSTRAINT stock_take_lines_tenant_product_fk
    FOREIGN KEY (tenant_id, product_id)
    REFERENCES products (tenant_id, product_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE stock_take_lines ADD CONSTRAINT stock_take_lines_tenant_counted_by_fk
    FOREIGN KEY (tenant_id, counted_by)
    REFERENCES users (tenant_id, user_id) DEFERRABLE INITIALLY DEFERRED;

-- ==================================
-- STOCK_ADJUSTMENTS TABLE
-- ==================================

-- Drop existing constraints
ALTER TABLE stock_adjustments DROP CONSTRAINT stock_adjustments_unique_move;
ALTER TABLE stock_adjustments DROP CONSTRAINT stock_adjustments_tenant_move_fk;
ALTER TABLE stock_adjustments DROP CONSTRAINT stock_adjustments_tenant_warehouse_fk;

-- Recreate as deferrable
ALTER TABLE stock_adjustments ADD CONSTRAINT stock_adjustments_unique_move
    UNIQUE (tenant_id, move_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE stock_adjustments ADD CONSTRAINT stock_adjustments_tenant_move_fk
    FOREIGN KEY (tenant_id, move_id)
    REFERENCES stock_moves (tenant_id, move_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE stock_adjustments ADD CONSTRAINT stock_adjustments_tenant_warehouse_fk
    FOREIGN KEY (tenant_id, warehouse_id)
    REFERENCES warehouse_locations (tenant_id, location_id) DEFERRABLE INITIALLY DEFERRED;

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration fixes deferrability issues in stock operations tables:
--
-- Foreign key and unique constraints were not deferrable, preventing bulk operations
-- that temporarily create duplicate or invalid references during data loading or migrations.
--
-- Making them DEFERRABLE INITIALLY DEFERRED allows constraint checking to be deferred
-- until the end of the transaction, enabling bulk operations while maintaining data integrity.
--
-- Affected tables:
-- - goods_receipts
-- - delivery_orders
-- - stock_transfers
-- - stock_takes
-- - stock_take_lines
-- - stock_adjustments
