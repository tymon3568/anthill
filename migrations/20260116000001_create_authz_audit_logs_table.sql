-- Migration: Create AuthZ Audit Logs Table
-- Task: task_03.02.15 - Implement Authorization Audit Logging System
-- Description: Immutable audit trail for authorization events, policy changes, and role assignments

-- Create authz_audit_logs table
CREATE TABLE IF NOT EXISTS authz_audit_logs (
    audit_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Context
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID,                          -- NULL for system/anonymous actions
    session_id UUID,                       -- Link to session if available

    -- Event details
    event_type VARCHAR(50) NOT NULL,       -- 'authorization', 'policy_change', 'role_assignment', 'security'
    event_action VARCHAR(100) NOT NULL,    -- 'enforce', 'create_role', 'assign_role', 'account_locked', etc.

    -- Authorization context (for event_type = 'authorization')
    resource VARCHAR(255),                 -- e.g., '/api/v1/users'
    action VARCHAR(50),                    -- e.g., 'POST', 'read', 'write'
    decision VARCHAR(10),                  -- 'allow', 'deny'
    policy_version BIGINT,                 -- tenant's authz_version at decision time

    -- Change context (for policy/role changes)
    target_entity_type VARCHAR(50),        -- 'role', 'policy', 'user_role'
    target_entity_id VARCHAR(255),         -- role name, policy key, or user_id
    old_value JSONB,                       -- Previous state (for updates)
    new_value JSONB,                       -- New state

    -- Request context
    ip_address TEXT,
    user_agent TEXT,
    request_id UUID,                       -- Correlation ID for distributed tracing

    -- Metadata
    metadata JSONB DEFAULT '{}',           -- Additional context
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT authz_audit_event_type_check CHECK (
        event_type IN ('authorization', 'policy_change', 'role_assignment', 'security')
    )
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_authz_audit_tenant_time
    ON authz_audit_logs(tenant_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_authz_audit_user_time
    ON authz_audit_logs(user_id, created_at DESC)
    WHERE user_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_authz_audit_event_type
    ON authz_audit_logs(tenant_id, event_type, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_authz_audit_decision_deny
    ON authz_audit_logs(tenant_id, created_at DESC)
    WHERE decision = 'deny';

CREATE INDEX IF NOT EXISTS idx_authz_audit_security
    ON authz_audit_logs(tenant_id, created_at DESC)
    WHERE event_type = 'security';

-- Comment for documentation
COMMENT ON TABLE authz_audit_logs IS 'Immutable audit trail for authorization events. No UPDATE or DELETE allowed via application.';
COMMENT ON COLUMN authz_audit_logs.event_type IS 'authorization=permission checks, policy_change=CRUD on roles/policies, role_assignment=user-role changes, security=lockouts/suspicious activity';
COMMENT ON COLUMN authz_audit_logs.policy_version IS 'Tenant authz_version at the time of the decision, for cache invalidation correlation';
