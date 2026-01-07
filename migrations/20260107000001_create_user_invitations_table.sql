-- Migration: Create user_invitations table for secure invitation system
-- Created: 2026-01-07
-- Description: Adds secure user invitation system with hash-at-rest tokens

-- Create user_invitations table
CREATE TABLE user_invitations (
    invitation_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id) ON DELETE CASCADE,

    -- Token (SECURITY: only hash stored, never plaintext)
    token_hash TEXT NOT NULL,              -- SHA-256 hash of the token

    -- Invitation target
    email VARCHAR(255) NOT NULL,
    invited_role VARCHAR(50) NOT NULL DEFAULT 'user',

    -- Inviter context
    invited_by_user_id UUID NOT NULL,

    -- Status tracking
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (
        status IN ('pending', 'accepted', 'expired', 'revoked')
    ),

    -- Expiry and acceptance
    expires_at TIMESTAMPTZ NOT NULL,       -- Default: 48 hours from creation
    accepted_at TIMESTAMPTZ,
    accepted_user_id UUID,  -- User created on acceptance

    -- Composite foreign keys for multi-tenancy (ensure referential integrity within tenant)
    FOREIGN KEY (tenant_id, invited_by_user_id) REFERENCES users(tenant_id, user_id),
    FOREIGN KEY (tenant_id, accepted_user_id) REFERENCES users(tenant_id, user_id)

    -- Request context for audit
    invited_from_ip TEXT,
    invited_from_user_agent TEXT,
    accepted_from_ip TEXT,
    accepted_from_user_agent TEXT,

    -- Rate limiting
    accept_attempts INTEGER NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,

    -- Metadata
    custom_message TEXT,                   -- Optional message from inviter
    metadata JSONB DEFAULT '{}',

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Soft delete (per project pattern)
    deleted_at TIMESTAMPTZ                -- NULL = active, set = soft-deleted
);

-- Indexes for performance
CREATE INDEX idx_invitations_tenant ON user_invitations(tenant_id, status, created_at DESC);
CREATE INDEX idx_invitations_email ON user_invitations(email, tenant_id) WHERE status = 'pending' AND deleted_at IS NULL;
CREATE INDEX idx_invitations_token ON user_invitations(token_hash) WHERE status = 'pending' AND deleted_at IS NULL;
CREATE INDEX idx_invitations_expires ON user_invitations(expires_at) WHERE status = 'pending' AND deleted_at IS NULL;
CREATE INDEX idx_invitations_inviter ON user_invitations(invited_by_user_id, created_at DESC);
CREATE INDEX idx_invitations_active ON user_invitations(tenant_id, status) WHERE deleted_at IS NULL;

-- Partial unique index for pending invitations
CREATE UNIQUE INDEX idx_invitations_unique_pending ON user_invitations(tenant_id, email)
    WHERE status = 'pending' AND deleted_at IS NULL;

-- Comments for documentation
COMMENT ON TABLE user_invitations IS 'Secure user invitation tokens with hash-at-rest';
COMMENT ON COLUMN user_invitations.token_hash IS 'SHA-256 hash of invite token - never store plaintext';
COMMENT ON COLUMN user_invitations.deleted_at IS 'Soft delete timestamp - NULL means active';
COMMENT ON COLUMN user_invitations.invited_role IS 'Role to assign to user upon acceptance (owner, admin, manager, user, viewer)';
COMMENT ON COLUMN user_invitations.accept_attempts IS 'Number of acceptance attempts (rate limiting)';
