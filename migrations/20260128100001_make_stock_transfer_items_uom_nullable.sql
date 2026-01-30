-- Migration: Make uom_id nullable in stock_transfer_items
-- Description: Allows transfers to be created without specifying a UoM when the product doesn't have a default
-- Created: 2026-01-28

-- Drop the NOT NULL constraint on uom_id
ALTER TABLE stock_transfer_items
    ALTER COLUMN uom_id DROP NOT NULL;

-- Drop the foreign key constraint (we'll recreate it to allow NULL)
ALTER TABLE stock_transfer_items
    DROP CONSTRAINT IF EXISTS stock_transfer_items_tenant_uom_fk;

-- Recreate the foreign key constraint (allowing NULL values)
ALTER TABLE stock_transfer_items
    ADD CONSTRAINT stock_transfer_items_tenant_uom_fk
    FOREIGN KEY (tenant_id, uom_id)
    REFERENCES unit_of_measures (tenant_id, uom_id)
    ON DELETE RESTRICT;

-- Update comments
COMMENT ON COLUMN stock_transfer_items.uom_id IS 'Unit of measure for quantities (optional - can be NULL if product has no default UoM)';
