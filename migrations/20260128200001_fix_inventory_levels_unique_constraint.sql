-- Migration: Fix inventory_levels unique constraint to handle NULL location_id properly
--
-- Problem: The current unique index doesn't use NULLS NOT DISTINCT, allowing
-- duplicate rows with the same (tenant_id, warehouse_id, NULL, product_id).
-- This causes the ON CONFLICT clause in upsert queries to fail, inserting new rows
-- instead of updating existing ones.
--
-- Solution:
-- 1. Consolidate duplicate inventory records (merge quantities for same warehouse/product with NULL location)
-- 2. Drop the old unique index
-- 3. Create new unique index with NULLS NOT DISTINCT (PostgreSQL 15+)

-- Step 1: Create a temp table to hold consolidated inventory data
-- Use MIN(inventory_id::text)::uuid to work around PostgreSQL's lack of MIN for uuid
CREATE TEMP TABLE consolidated_inventory AS
SELECT
    tenant_id,
    warehouse_id,
    location_id,
    product_id,
    SUM(available_quantity) as total_available,
    SUM(reserved_quantity) as total_reserved,
    (MIN(inventory_id::text))::uuid as keep_inventory_id
FROM inventory_levels
WHERE deleted_at IS NULL
GROUP BY tenant_id, warehouse_id, location_id, product_id
HAVING COUNT(*) > 1;

-- Step 2: Update the record we're keeping with consolidated quantities
UPDATE inventory_levels il
SET
    available_quantity = ci.total_available,
    reserved_quantity = ci.total_reserved,
    updated_at = NOW()
FROM consolidated_inventory ci
WHERE il.inventory_id = ci.keep_inventory_id;

-- Step 3: Soft delete duplicate records (keep only the one with MIN inventory_id)
UPDATE inventory_levels il
SET
    deleted_at = NOW(),
    updated_at = NOW()
FROM consolidated_inventory ci
WHERE il.tenant_id = ci.tenant_id
    AND il.warehouse_id = ci.warehouse_id
    AND (il.location_id = ci.location_id OR (il.location_id IS NULL AND ci.location_id IS NULL))
    AND il.product_id = ci.product_id
    AND il.inventory_id != ci.keep_inventory_id
    AND il.deleted_at IS NULL;

-- Step 4: Drop the old unique index
DROP INDEX IF EXISTS idx_inventory_levels_tenant_warehouse_location_product_unique;

-- Step 5: Create new unique index with NULLS NOT DISTINCT
-- This ensures that NULL location_id values are treated as equal for uniqueness
CREATE UNIQUE INDEX idx_inventory_levels_tenant_warehouse_location_product_unique
ON inventory_levels (tenant_id, warehouse_id, location_id, product_id)
NULLS NOT DISTINCT
WHERE deleted_at IS NULL;

-- Step 6: Drop temp table
DROP TABLE IF EXISTS consolidated_inventory;

-- Verification query (for manual check, not executed in migration):
-- SELECT tenant_id, warehouse_id, location_id, product_id, COUNT(*)
-- FROM inventory_levels
-- WHERE deleted_at IS NULL
-- GROUP BY tenant_id, warehouse_id, location_id, product_id
-- HAVING COUNT(*) > 1;
