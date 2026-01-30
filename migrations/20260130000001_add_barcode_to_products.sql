-- Migration: Add barcode fields to products table
-- Date: 2026-01-30
-- Description: Adds barcode and barcode_type columns to support product scanning
-- TaskID: 08.10.06.01

-- ============================================================================
-- UP: Add barcode columns
-- ============================================================================

-- Add barcode column (the actual barcode value)
ALTER TABLE products ADD COLUMN IF NOT EXISTS barcode VARCHAR(50);

-- Add barcode_type column (EAN13, UPC-A, ISBN, CUSTOM)
ALTER TABLE products ADD COLUMN IF NOT EXISTS barcode_type VARCHAR(20);

-- Create index for barcode lookup (filtered to exclude deleted and null barcodes)
CREATE INDEX IF NOT EXISTS idx_products_tenant_barcode
ON products(tenant_id, barcode)
WHERE deleted_at IS NULL AND barcode IS NOT NULL;

-- Create unique constraint on barcode per tenant (only for non-null, non-deleted)
-- Using partial unique index instead of constraint for flexibility
CREATE UNIQUE INDEX IF NOT EXISTS uq_products_tenant_barcode
ON products(tenant_id, barcode)
WHERE deleted_at IS NULL AND barcode IS NOT NULL;

-- Add check constraint for barcode_type values
ALTER TABLE products ADD CONSTRAINT chk_barcode_type
CHECK (barcode_type IS NULL OR barcode_type IN ('ean13', 'upc_a', 'isbn', 'custom'));

-- ============================================================================
-- DOWN (for rollback): Remove barcode columns
-- ============================================================================
-- To rollback, run these commands manually:
--
-- ALTER TABLE products DROP CONSTRAINT IF EXISTS chk_barcode_type;
-- DROP INDEX IF EXISTS uq_products_tenant_barcode;
-- DROP INDEX IF EXISTS idx_products_tenant_barcode;
-- ALTER TABLE products DROP COLUMN IF EXISTS barcode_type;
-- ALTER TABLE products DROP COLUMN IF EXISTS barcode;
