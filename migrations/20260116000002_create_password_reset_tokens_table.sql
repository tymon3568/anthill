-- Migration: Create password_reset_tokens table for password reset flow
-- Created: 2026-01-16
-- Task: V1_MVP/03_User_Service/3.6_Self_Auth_Enhancements/task_03.06.02_password_reset_flow.md
-- Description: Stores secure password reset tokens with single-use enforcement

-- Create password_reset_tokens table
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    -- Primary key
    token_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- User reference (who requested reset)
    user_id UUID NOT NULL,
    tenant_id UUID NOT NULL,

    -- Password reset token (SHA-256 hashed, never plaintext)
    -- The actual token sent to user is random 32-byte URL-safe string
    -- We store only the hash for security (same pattern as verification tokens)
    token_hash VARCHAR(64) NOT NULL UNIQUE,

    -- Token metadata
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Token usage (single-use enforcement)
    used_at TIMESTAMPTZ,

    -- Request context for audit
    ip_address INET,
    user_agent TEXT,

    -- Completion context
    reset_from_ip INET,
    reset_from_user_agent TEXT,

    -- Constraints
    CONSTRAINT fk_password_reset_tokens_user
        FOREIGN KEY (user_id, tenant_id)
        REFERENCES users(user_id, tenant_id)
        ON DELETE CASCADE,

    CONSTRAINT fk_password_reset_tokens_tenant
        FOREIGN KEY (tenant_id)
        REFERENCES tenants(tenant_id)
        ON DELETE CASCADE,

    -- Ensure token is valid
    CONSTRAINT chk_password_reset_expires_future
        CHECK (expires_at > created_at)
);

-- Create indexes for performance

-- Index for finding unused tokens by hash (primary lookup path)
CREATE INDEX idx_password_reset_tokens_token_hash
    ON password_reset_tokens(token_hash)
    WHERE used_at IS NULL;

-- Index for cleanup of expired tokens
CREATE INDEX idx_password_reset_tokens_expires_at
    ON password_reset_tokens(expires_at)
    WHERE used_at IS NULL;

-- Index for rate limiting lookups (find recent tokens by user)
CREATE INDEX idx_password_reset_tokens_user_created
    ON password_reset_tokens(user_id, created_at DESC);

-- Index for tenant-scoped lookups
CREATE INDEX idx_password_reset_tokens_tenant_id
    ON password_reset_tokens(tenant_id);

-- Create password_reset_audit table for logging
CREATE TABLE IF NOT EXISTS password_reset_audit (
    -- Primary key
    audit_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- User reference (may be null if email not found)
    user_id UUID,
    tenant_id UUID,

    -- Email address (stored even if user doesn't exist, for audit)
    email VARCHAR(255) NOT NULL,

    -- Event details
    event_type VARCHAR(50) NOT NULL,  -- 'requested', 'completed', 'failed', 'expired', 'rate_limited'
    ip_address INET,
    user_agent TEXT,
    failure_reason TEXT,

    -- Timestamp
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Foreign key constraints (nullable refs)
    CONSTRAINT fk_password_reset_audit_user
        FOREIGN KEY (user_id, tenant_id)
        REFERENCES users(user_id, tenant_id)
        ON DELETE SET NULL,

    CONSTRAINT fk_password_reset_audit_tenant
        FOREIGN KEY (tenant_id)
        REFERENCES tenants(tenant_id)
        ON DELETE SET NULL,

    -- Valid event types
    CONSTRAINT chk_password_reset_audit_event_type
        CHECK (event_type IN ('requested', 'completed', 'failed', 'expired', 'rate_limited'))
);

-- Create indexes for audit queries
CREATE INDEX idx_password_reset_audit_email
    ON password_reset_audit(email, created_at DESC);

CREATE INDEX idx_password_reset_audit_user
    ON password_reset_audit(user_id, created_at DESC)
    WHERE user_id IS NOT NULL;

CREATE INDEX idx_password_reset_audit_tenant
    ON password_reset_audit(tenant_id, created_at DESC)
    WHERE tenant_id IS NOT NULL;

CREATE INDEX idx_password_reset_audit_event_type
    ON password_reset_audit(event_type, created_at DESC);

-- Comments for documentation
COMMENT ON TABLE password_reset_tokens IS 'Stores password reset tokens for forgot-password flow. Tokens are hashed (SHA-256) and expire after 1 hour.';
COMMENT ON COLUMN password_reset_tokens.token_hash IS 'SHA-256 hash of the reset token (never store plaintext)';
COMMENT ON COLUMN password_reset_tokens.expires_at IS 'Token expiration timestamp (default: 1 hour from creation)';
COMMENT ON COLUMN password_reset_tokens.used_at IS 'Timestamp when token was used (NULL = not yet used, single-use enforcement)';
COMMENT ON COLUMN password_reset_tokens.ip_address IS 'IP address of the client that requested the reset';
COMMENT ON COLUMN password_reset_tokens.reset_from_ip IS 'IP address of the client that completed the reset';

COMMENT ON TABLE password_reset_audit IS 'Audit log for password reset events. Records all attempts for security monitoring.';
COMMENT ON COLUMN password_reset_audit.event_type IS 'Type of event: requested, completed, failed, expired, rate_limited';
COMMENT ON COLUMN password_reset_audit.failure_reason IS 'Reason for failure (e.g., invalid_token, expired, rate_limited)';
