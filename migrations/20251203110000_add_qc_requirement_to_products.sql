-- Migration: Add QC Requirement to Products
-- Description: Adds quality control requirement flags to products table
-- Created: 2025-12-03

-- Add QC requirement columns to products table
ALTER TABLE products
ADD COLUMN qc_incoming_required BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN qc_outgoing_required BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN qc_internal_required BOOLEAN NOT NULL DEFAULT false;

-- Add comment for documentation
COMMENT ON COLUMN products.qc_incoming_required IS 'Whether incoming goods require quality control inspection';
COMMENT ON COLUMN products.qc_outgoing_required IS 'Whether outgoing goods require quality control inspection';
COMMENT ON COLUMN products.qc_internal_required IS 'Whether internal operations require quality control inspection';

-- Create index for QC-required products (optional, for performance)
CREATE INDEX idx_products_qc_required ON products(tenant_id, qc_incoming_required, qc_outgoing_required, qc_internal_required) WHERE qc_incoming_required = true OR qc_outgoing_required = true OR qc_internal_required = true;
