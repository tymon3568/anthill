-- Migration: Fix users soft delete unique constraint
-- Description: Replaces unconditional unique constraint on (tenant_id, email) with partial unique index
--              to support email reuse after soft delete
-- Issue: When a user is soft-deleted, their email cannot be reused because the
--        unique constraint still blocks it at the database level, even though
--        email_exists() check correctly excludes soft-deleted users
-- Created: 2026-01-19

-- ==================================
-- USERS
-- ==================================
-- Drop unconditional constraint created in 20250110000002_create_tenants_users.sql
-- Original constraint: CONSTRAINT users_email_tenant_unique UNIQUE (tenant_id, email)
ALTER TABLE users
DROP CONSTRAINT IF EXISTS users_email_tenant_unique;

-- Create partial UNIQUE index filtering out deleted records
-- This allows email reuse after account deletion while still preventing
-- duplicate active accounts with the same email
CREATE UNIQUE INDEX idx_users_email_unique_active
ON users(tenant_id, email)
WHERE deleted_at IS NULL;

-- ==================================
-- MIGRATION NOTES
-- ==================================
-- This migration fixes the issue where soft-deleted users prevented reuse of
-- their email address for new account creation.
--
-- By replacing the unconditional UNIQUE constraint with a partial UNIQUE index
-- (WHERE deleted_at IS NULL), we enforce uniqueness only among active users.
--
-- This aligns with the behavior already implemented in:
-- - email_exists() in repository.rs: "Soft-deleted users are excluded, allowing
--   email re-registration after account deletion"
-- - Similar fix applied to other tables in 20251216000002_fix_soft_delete_constraints.sql
