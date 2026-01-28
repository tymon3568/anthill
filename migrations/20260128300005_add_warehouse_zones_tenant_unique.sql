-- Migration: Add tenant unique constraint to warehouse_zones
-- Description: Add unique constraint on (tenant_id, zone_id) for FK references
-- Dependencies: 20250110000023_create_warehouse_tables.sql
-- Part of: Module 4.5 - Location Architecture Fix
-- Created: 2026-01-28

-- ==================================
-- ADD UNIQUE CONSTRAINT FOR FK REFERENCES
-- ==================================

-- Add unique constraint on (tenant_id, zone_id) to allow composite FK references
-- This enables other tables to reference warehouse_zones with (tenant_id, zone_id)
-- for proper multi-tenant isolation

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'warehouse_zones_tenant_zone_unique'
    ) THEN
        ALTER TABLE warehouse_zones
        ADD CONSTRAINT warehouse_zones_tenant_zone_unique
        UNIQUE (tenant_id, zone_id);
    END IF;
END $$;

-- ==================================
-- ADD FK CONSTRAINTS FOR TRANSFER ITEMS
-- ==================================

-- Now add the zone FK constraints that failed in migration 20260128300004

-- Source zone must belong to tenant
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_transfer_items_source_zone') THEN
        ALTER TABLE stock_transfer_items
        ADD CONSTRAINT fk_transfer_items_source_zone
        FOREIGN KEY (tenant_id, source_zone_id)
        REFERENCES warehouse_zones(tenant_id, zone_id);
    END IF;
END $$;

-- Destination zone must belong to tenant
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_transfer_items_dest_zone') THEN
        ALTER TABLE stock_transfer_items
        ADD CONSTRAINT fk_transfer_items_dest_zone
        FOREIGN KEY (tenant_id, destination_zone_id)
        REFERENCES warehouse_zones(tenant_id, zone_id);
    END IF;
END $$;

-- ==================================
-- COMMENTS
-- ==================================

COMMENT ON CONSTRAINT warehouse_zones_tenant_zone_unique ON warehouse_zones IS 'Enables composite FK references with (tenant_id, zone_id)';

-- ==================================
-- MIGRATION METADATA
-- ==================================
-- This migration:
-- 1. Adds unique constraint on warehouse_zones(tenant_id, zone_id)
-- 2. Completes FK constraints for stock_transfer_items zone columns
--    that failed in migration 20260128300004
