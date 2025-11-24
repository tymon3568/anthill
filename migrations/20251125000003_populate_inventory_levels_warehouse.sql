-- Migration: Populate warehouse_id in inventory_levels for existing data
-- Description: Assigns default warehouse to existing inventory_levels rows that lack warehouse_id
-- Dependencies: 20251124000002_add_warehouse_to_inventory_levels.sql, 20250121000000_seed_system_records.sql
-- Created: 2025-11-25

-- ==================================
-- DATA MIGRATION: POPULATE WAREHOUSE_ID
-- ==================================

-- Verify system warehouse exists before proceeding
DO $$
DECLARE
    system_warehouse_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO system_warehouse_count
    FROM warehouses
    WHERE tenant_id = '00000000-0000-0000-0000-000000000000'::uuid
    AND warehouse_code = 'SYS-WH'
    AND deleted_at IS NULL;

    IF system_warehouse_count = 0 THEN
        RAISE EXCEPTION 'Migration failed: System warehouse (SYS-WH) not found for default tenant. Ensure seed migration has run.';
    END IF;
END $$;

-- Assign the system warehouse to all existing inventory_levels rows
-- Dynamically lookup the system warehouse to avoid hardcoded UUIDs
WITH system_warehouse AS (
    SELECT warehouse_id
    FROM warehouses
    WHERE tenant_id = '00000000-0000-0000-0000-000000000000'::uuid
    AND warehouse_code = 'SYS-WH'
    AND deleted_at IS NULL
    LIMIT 1
)
UPDATE inventory_levels
SET warehouse_id = (SELECT warehouse_id FROM system_warehouse)
WHERE warehouse_id IS NULL;

-- ==================================
-- VALIDATION
-- ==================================

-- Ensure the system warehouse exists and no rows remain without warehouse_id
DO $$
DECLARE
    system_warehouse_count INTEGER;
BEGIN
    -- Check if system warehouse exists
    SELECT COUNT(*) INTO system_warehouse_count
    FROM warehouses
    WHERE tenant_id = '00000000-0000-0000-0000-000000000000'::uuid
    AND warehouse_code = 'SYS-WH'
    AND deleted_at IS NULL;

    IF system_warehouse_count = 0 THEN
        RAISE EXCEPTION 'Data migration failed: System warehouse (SYS-WH) not found for default tenant';
    END IF;

    -- Check if any inventory_levels rows still lack warehouse_id
    IF EXISTS (SELECT 1 FROM inventory_levels WHERE warehouse_id IS NULL) THEN
        RAISE EXCEPTION 'Data migration failed: Some inventory_levels rows still lack warehouse_id after update';
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
