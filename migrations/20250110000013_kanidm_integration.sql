-- Migration: Kanidm Integration
-- Add kanidm_user_id to users and create kanidm_tenant_groups mapping table

-- Add kanidm_user_id to users table
ALTER TABLE users
  ADD COLUMN kanidm_user_id UUID UNIQUE,
  ADD COLUMN kanidm_synced_at TIMESTAMPTZ;

-- Create index for faster lookups
CREATE INDEX idx_users_kanidm_id ON users(kanidm_user_id) WHERE kanidm_user_id IS NOT NULL;

-- Create kanidm_tenant_groups mapping table
-- Maps Kanidm groups to Anthill tenants
CREATE TABLE kanidm_tenant_groups (
  tenant_id UUID NOT NULL REFERENCES tenants(tenant_id) ON DELETE CASCADE,
  kanidm_group_uuid UUID NOT NULL,
  kanidm_group_name TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('admin', 'member', 'viewer')),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  PRIMARY KEY (tenant_id, kanidm_group_uuid)
);

-- Index for reverse lookup (group -> tenant)
CREATE INDEX idx_kanidm_groups_name ON kanidm_tenant_groups(kanidm_group_name);

-- Comments for documentation
COMMENT ON TABLE kanidm_tenant_groups IS 'Maps Kanidm groups to Anthill tenants with role assignment';
COMMENT ON COLUMN users.kanidm_user_id IS 'UUID of user in Kanidm (from sub claim in JWT)';
COMMENT ON COLUMN users.kanidm_synced_at IS 'Last time user data was synced from Kanidm';
COMMENT ON COLUMN kanidm_tenant_groups.kanidm_group_uuid IS 'UUID of the group in Kanidm';
COMMENT ON COLUMN kanidm_tenant_groups.kanidm_group_name IS 'Name of the group in Kanidm (e.g., tenant_acme_admins)';
COMMENT ON COLUMN kanidm_tenant_groups.role IS 'User role in this tenant: admin, member, or viewer';
