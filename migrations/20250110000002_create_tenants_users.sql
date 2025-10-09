-- Migration: Create Core Tables (Tenants, Users, Sessions)
-- Description: Foundation tables for multi-tenant SaaS platform
-- Author: System
-- Date: 2025-01-10

-- =============================================================================
-- TABLE: tenants
-- =============================================================================

CREATE TABLE tenants (
    tenant_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    
    -- Basic info
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE, -- URL-friendly identifier (e.g., "acme-corp")
    
    -- Plan and billing
    plan VARCHAR(50) NOT NULL DEFAULT 'free', -- free, starter, professional, enterprise
    plan_expires_at TIMESTAMPTZ,
    
    -- Settings and configuration (flexible JSONB)
    settings JSONB NOT NULL DEFAULT '{}',
    -- Example settings:
    -- {
    --   "timezone": "Asia/Ho_Chi_Minh",
    --   "currency": "VND",
    --   "features": ["inventory", "orders", "integrations"],
    --   "limits": {"users": 5, "products": 1000}
    -- }
    
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'active', -- active, suspended, cancelled
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ, -- Soft delete
    
    -- Constraints
    CONSTRAINT tenants_plan_check CHECK (plan IN ('free', 'starter', 'professional', 'enterprise')),
    CONSTRAINT tenants_status_check CHECK (status IN ('active', 'suspended', 'cancelled'))
);

-- Indexes
CREATE INDEX idx_tenants_slug ON tenants(slug) WHERE deleted_at IS NULL;
CREATE INDEX idx_tenants_status ON tenants(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_tenants_active ON tenants(tenant_id) WHERE deleted_at IS NULL AND status = 'active';

-- Auto-update updated_at
CREATE TRIGGER update_tenants_updated_at
    BEFORE UPDATE ON tenants
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments
COMMENT ON TABLE tenants IS 'Tenant organizations in the multi-tenant system';
COMMENT ON COLUMN tenants.slug IS 'URL-friendly unique identifier for tenant';
COMMENT ON COLUMN tenants.settings IS 'Flexible tenant-specific configuration in JSONB format';

-- =============================================================================
-- TABLE: users
-- =============================================================================

CREATE TABLE users (
    user_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id) ON DELETE CASCADE,
    
    -- Authentication
    email VARCHAR(255) NOT NULL,
    password_hash TEXT NOT NULL, -- bcrypt hash
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    email_verified_at TIMESTAMPTZ,
    
    -- Profile
    full_name VARCHAR(255),
    avatar_url TEXT,
    phone VARCHAR(50),
    
    -- Role-based access control
    role VARCHAR(50) NOT NULL DEFAULT 'user', -- super_admin, admin, manager, user, viewer
    
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'active', -- active, inactive, suspended
    last_login_at TIMESTAMPTZ,
    
    -- Security
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ, -- Account lockout after failed attempts
    password_changed_at TIMESTAMPTZ,
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ, -- Soft delete
    
    -- Constraints
    CONSTRAINT users_email_tenant_unique UNIQUE (tenant_id, email),
    CONSTRAINT users_role_check CHECK (role IN ('super_admin', 'admin', 'manager', 'user', 'viewer')),
    CONSTRAINT users_status_check CHECK (status IN ('active', 'inactive', 'suspended'))
);

-- Indexes
CREATE INDEX idx_users_tenant ON users(tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_email ON users(tenant_id, email) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_role ON users(tenant_id, role) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_status ON users(tenant_id, status) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_active ON users(tenant_id, user_id) WHERE deleted_at IS NULL AND status = 'active';

-- Auto-update updated_at
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments
COMMENT ON TABLE users IS 'User accounts with multi-tenant isolation';
COMMENT ON COLUMN users.password_hash IS 'Bcrypt password hash (cost factor 12)';
COMMENT ON COLUMN users.role IS 'User role for RBAC (handled by Casbin)';
COMMENT ON COLUMN users.failed_login_attempts IS 'Counter for account lockout security';

-- =============================================================================
-- TABLE: sessions
-- =============================================================================

CREATE TABLE sessions (
    session_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id) ON DELETE CASCADE,
    
    -- Token management
    access_token_hash TEXT NOT NULL, -- SHA-256 hash of access token
    refresh_token_hash TEXT NOT NULL, -- SHA-256 hash of refresh token
    
    -- Session metadata
    ip_address INET,
    user_agent TEXT,
    device_info JSONB, -- Browser, OS, device type
    
    -- Expiration
    access_token_expires_at TIMESTAMPTZ NOT NULL,
    refresh_token_expires_at TIMESTAMPTZ NOT NULL,
    
    -- Session control
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMPTZ,
    revoked_reason VARCHAR(255),
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Note: sessions table uses individual foreign keys instead of composite FK
-- Both tenant_id and user_id have ON DELETE CASCADE

-- Indexes
CREATE INDEX idx_sessions_user ON sessions(user_id) WHERE NOT revoked;
CREATE INDEX idx_sessions_tenant ON sessions(tenant_id) WHERE NOT revoked;
CREATE INDEX idx_sessions_access_token ON sessions(access_token_hash) WHERE NOT revoked;
CREATE INDEX idx_sessions_refresh_token ON sessions(refresh_token_hash) WHERE NOT revoked;
CREATE INDEX idx_sessions_expires ON sessions(access_token_expires_at) WHERE NOT revoked;

-- Index for cleanup of expired sessions
-- Note: Cannot use NOW() in index predicate (not immutable)
-- Use this query instead: WHERE revoked = TRUE OR access_token_expires_at < NOW()
CREATE INDEX idx_sessions_cleanup ON sessions(access_token_expires_at, created_at) WHERE revoked = TRUE;

-- Comments
COMMENT ON TABLE sessions IS 'Active user sessions with JWT token management';
COMMENT ON COLUMN sessions.access_token_hash IS 'SHA-256 hash of JWT access token for validation';
COMMENT ON COLUMN sessions.refresh_token_hash IS 'SHA-256 hash of JWT refresh token';
COMMENT ON COLUMN sessions.revoked IS 'Manual session revocation (logout, security)';

-- =============================================================================
-- SEED DATA (Optional - for development)
-- =============================================================================

-- Create a default super admin tenant (for development only)
-- In production, this should be done through an admin CLI tool
-- 
-- INSERT INTO tenants (name, slug, plan, status)
-- VALUES ('System Admin', 'system', 'enterprise', 'active')
-- ON CONFLICT (slug) DO NOTHING;
