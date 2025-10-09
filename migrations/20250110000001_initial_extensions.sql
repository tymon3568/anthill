-- Migration: Initial Extensions and Functions
-- Description: Enable required PostgreSQL extensions and create UUID v7 generation function
-- Author: System
-- Date: 2025-01-10

-- =============================================================================
-- EXTENSIONS
-- =============================================================================

-- UUID generation (for UUID v4 as fallback)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Cryptographic functions for password hashing and encryption
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- =============================================================================
-- UUID v7 GENERATION FUNCTION
-- =============================================================================

-- UUID v7 format (RFC 9562): Timestamp-based UUID for better index locality
-- Format: unix_ts_ms (48 bits) | ver (4 bits) | rand (12 bits) | var (2 bits) | rand (62 bits)
-- Benefits:
--   - Chronologically sortable
--   - Better database index performance (no random page splits)
--   - Preserves creation time in the UUID itself

CREATE OR REPLACE FUNCTION uuid_generate_v7()
RETURNS UUID
AS $$
DECLARE
  unix_ts_ms BIGINT;
  uuid_bytes BYTEA;
BEGIN
  -- Get current Unix timestamp in milliseconds
  unix_ts_ms := (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::BIGINT;
  
  -- Generate UUID v7
  uuid_bytes := 
    -- Timestamp (48 bits = 6 bytes)
    substring(int8send(unix_ts_ms) from 3 for 6) ||
    -- Version (4 bits = 0x7) + random (12 bits)
    set_byte('\x0000'::BYTEA, 0, (b'0111' || substring(get_byte(gen_random_bytes(1)::TEXT::BYTEA, 0)::BIT(8) from 5 for 4))::BIT(8)::INT) ||
    -- Variant (2 bits = 0b10) + random (62 bits = 7.75 bytes, use 8 bytes)
    set_byte(gen_random_bytes(8), 0, (b'10' || substring(get_byte(gen_random_bytes(1)::TEXT::BYTEA, 0)::BIT(8) from 3 for 6))::BIT(8)::INT);
  
  RETURN encode(uuid_bytes, 'hex')::UUID;
END;
$$ LANGUAGE plpgsql VOLATILE;

-- =============================================================================
-- HELPER FUNCTIONS FOR TIMESTAMPS
-- =============================================================================

-- Function to automatically update `updated_at` timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- COMMENTS
-- =============================================================================

COMMENT ON EXTENSION "uuid-ossp" IS 'UUID generation functions (v4 fallback)';
COMMENT ON EXTENSION "pgcrypto" IS 'Cryptographic functions for passwords and encryption';
COMMENT ON FUNCTION uuid_generate_v7() IS 'Generate timestamp-based UUID v7 for better index performance';
COMMENT ON FUNCTION update_updated_at_column() IS 'Trigger function to automatically update updated_at column';
