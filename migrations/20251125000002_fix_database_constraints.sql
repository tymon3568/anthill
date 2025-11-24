-- Migration: Fix database constraints for deferrability
-- Description: Makes uom_conversions constraint deferrable for bulk operations
-- Dependencies: uom_conversions table must exist
-- Created: 2025-11-25

-- ==================================
-- FIX UOM_CONVERSIONS CONSTRAINT DEFERRABILITY
-- ==================================

-- Drop the existing unique constraint
ALTER TABLE uom_conversions DROP CONSTRAINT uom_conversions_unique_per_product_uom_pair;

-- Recreate it as deferrable initially deferred
ALTER TABLE uom_conversions ADD CONSTRAINT uom_conversions_unique_per_product_uom_pair
    UNIQUE (tenant_id, product_id, from_uom_id, to_uom_id) DEFERRABLE INITIALLY DEFERRED;

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration addresses the deferrability issue with uom_conversions:
--
-- The unique constraint on (tenant_id, product_id, from_uom_id, to_uom_id)
-- was not deferrable, preventing bulk operations that temporarily create
-- duplicate conversions during data loading or migrations.
--
-- Making it DEFERRABLE INITIALLY DEFERRED allows constraint checking to be
-- deferred until the end of the transaction, enabling bulk operations that
-- might temporarily violate the constraint.
