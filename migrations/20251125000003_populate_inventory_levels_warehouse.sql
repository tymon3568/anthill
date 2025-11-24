-- Migration: Populate warehouse_id in inventory_levels for existing data
-- Description: Assigns default warehouse to existing inventory_levels rows that lack warehouse_id
-- Dependencies: 20251124000002_add_warehouse_to_inventory_levels.sql, 20250121000000_seed_system_records.sql
-- Created: 2025-11-25

-- ==================================
-- DATA MIGRATION: POPULATE WAREHOUSE_ID
-- ==================================

-- Assign the system warehouse to all existing inventory_levels rows
-- This assumes the system warehouse exists (created in seed migration)
UPDATE inventory_levels
SET warehouse_id = '00000000-0000-0000-0000-000000000001'::uuid
WHERE warehouse_id IS NULL;

-- ==================================
-- VALIDATION
-- ==================================

-- Ensure no rows remain without warehouse_id
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM inventory_levels WHERE warehouse_id IS NULL) THEN
        RAISE EXCEPTION 'Data migration failed: Some inventory_levels rows still lack warehouse_id';
    END IF;
END $$;

-- ==================================
-- MIGRATION NOTES
-- ==================================

-- This migration assigns the system warehouse (ID: 00000000-0000-0000-0000-000000000001)
-- to all existing inventory_levels rows that were created before warehouse support.
--
-- In a multi-tenant system, you may want to assign different default warehouses
-- per tenant. If so, modify the UPDATE statement accordingly:
--
-- UPDATE inventory_levels
-- SET warehouse_id = CASE
--     WHEN tenant_id = 'tenant-1-uuid' THEN 'warehouse-1-uuid'
--     WHEN tenant_id = 'tenant-2-uuid' THEN 'warehouse-2-uuid'
--     ELSE '00000000-0000-0000-0000-000000000001'::uuid -- system warehouse
-- END
-- WHERE warehouse_id IS NULL;

COMMENT ON TABLE inventory_levels IS 'Inventory levels per tenant, warehouse, and product - supports multi-warehouse stock tracking';
