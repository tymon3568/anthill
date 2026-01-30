-- Migration: Create product_images table
-- Date: 2026-01-30
-- Description: Stores image metadata for products, images stored in RustFS
-- TaskID: 08.10.06.02

-- ============================================================================
-- UP: Create product_images table
-- ============================================================================

CREATE TABLE IF NOT EXISTS product_images (
    -- Primary key using UUID v7
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign key to products table
    product_id UUID NOT NULL REFERENCES products(product_id) ON DELETE CASCADE,

    -- Multi-tenancy isolation
    tenant_id UUID NOT NULL,

    -- Image URL (public access URL from RustFS)
    url TEXT NOT NULL,

    -- Alternative text for accessibility
    alt_text VARCHAR(255),

    -- Display position (0-based, lower = first)
    position INTEGER NOT NULL DEFAULT 0,

    -- Primary image flag (only one per product should be true)
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,

    -- File metadata
    file_size INTEGER,  -- Size in bytes
    mime_type VARCHAR(50),  -- image/jpeg, image/png, image/webp
    width INTEGER,  -- Image width in pixels
    height INTEGER,  -- Image height in pixels

    -- S3 object key for RustFS (used for deletion)
    object_key TEXT NOT NULL,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- Indexes
-- ============================================================================

-- Index for listing images by product (most common query)
CREATE INDEX IF NOT EXISTS idx_product_images_product_tenant
ON product_images(product_id, tenant_id);

-- Index for finding primary image quickly
CREATE INDEX IF NOT EXISTS idx_product_images_primary
ON product_images(product_id, is_primary)
WHERE is_primary = TRUE;

-- Index for position ordering
CREATE INDEX IF NOT EXISTS idx_product_images_position
ON product_images(product_id, position);

-- ============================================================================
-- Constraints
-- ============================================================================

-- Ensure position is non-negative
ALTER TABLE product_images ADD CONSTRAINT chk_product_images_position_positive
CHECK (position >= 0);

-- Ensure file_size is positive when set
ALTER TABLE product_images ADD CONSTRAINT chk_product_images_file_size_positive
CHECK (file_size IS NULL OR file_size > 0);

-- Ensure mime_type is a valid image type when set
ALTER TABLE product_images ADD CONSTRAINT chk_product_images_mime_type
CHECK (mime_type IS NULL OR mime_type IN ('image/jpeg', 'image/png', 'image/webp', 'image/gif'));

-- ============================================================================
-- Trigger for updated_at
-- ============================================================================

-- Reuse the update_updated_at_column function if it exists, otherwise create it
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_product_images_updated_at
    BEFORE UPDATE ON product_images
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- Comments
-- ============================================================================

COMMENT ON TABLE product_images IS 'Stores metadata for product images, actual files stored in RustFS';
COMMENT ON COLUMN product_images.object_key IS 'S3/RustFS object key for file operations (upload/delete)';
COMMENT ON COLUMN product_images.is_primary IS 'Only one image per product should be marked as primary';
COMMENT ON COLUMN product_images.position IS 'Display order, 0 is first';

-- ============================================================================
-- DOWN (for rollback): Drop product_images table
-- ============================================================================
-- To rollback, run these commands manually:
--
-- DROP TRIGGER IF EXISTS trigger_product_images_updated_at ON product_images;
-- DROP TABLE IF EXISTS product_images;
