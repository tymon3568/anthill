-- Migration: Create email_verification_tokens table for email verification flow
-- Created: 2026-01-15
-- Description: Stores secure verification tokens for user email verification

-- Create email_verification_tokens table
CREATE TABLE IF NOT EXISTS email_verification_tokens (
    -- Primary key
    token_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- User reference (who needs to verify)
    user_id UUID NOT NULL,
    tenant_id UUID NOT NULL,

    -- Verification token (SHA-256 hashed, never plaintext)
    -- The actual token sent to user is random 32-byte URL-safe string
    -- We store only the hash for security (same pattern as session tokens)
    token_hash VARCHAR(64) NOT NULL UNIQUE,

    -- Email address being verified
    email VARCHAR(255) NOT NULL,

    -- Token metadata
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Verification status
    verified_at TIMESTAMPTZ,
    verified_from_ip INET,
    verified_from_user_agent TEXT,

    -- Attempt tracking
    verification_attempts INT NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,

    -- Constraints
    CONSTRAINT fk_email_verification_tokens_user
        FOREIGN KEY (user_id, tenant_id)
        REFERENCES users(user_id, tenant_id)
        ON DELETE CASCADE,

    CONSTRAINT fk_email_verification_tokens_tenant
        FOREIGN KEY (tenant_id)
        REFERENCES tenants(tenant_id)
        ON DELETE CASCADE,

    -- Ensure token is valid
    CONSTRAINT chk_email_verification_expires_future
        CHECK (expires_at > created_at)
);

-- Create indexes for performance
CREATE INDEX idx_email_verification_tokens_user_id
    ON email_verification_tokens(user_id);

CREATE INDEX idx_email_verification_tokens_tenant_id
    ON email_verification_tokens(tenant_id);

CREATE INDEX idx_email_verification_tokens_token_hash
    ON email_verification_tokens(token_hash)
    WHERE verified_at IS NULL; -- Only index unverified tokens for faster lookup

CREATE INDEX idx_email_verification_tokens_expires_at
    ON email_verification_tokens(expires_at)
    WHERE verified_at IS NULL; -- For cleanup of expired tokens

CREATE INDEX idx_email_verification_tokens_email
    ON email_verification_tokens(email, tenant_id)
    WHERE verified_at IS NULL; -- For resend rate limiting

-- Comments for documentation
COMMENT ON TABLE email_verification_tokens IS 'Stores email verification tokens for user registration flow. Tokens are hashed (SHA-256) and expire after 24 hours.';
COMMENT ON COLUMN email_verification_tokens.token_hash IS 'SHA-256 hash of the verification token (never store plaintext)';
COMMENT ON COLUMN email_verification_tokens.expires_at IS 'Token expiration timestamp (default: 24 hours from creation)';
COMMENT ON COLUMN email_verification_tokens.verified_at IS 'Timestamp when token was successfully verified (NULL = not yet verified)';
COMMENT ON COLUMN email_verification_tokens.verification_attempts IS 'Counter for failed verification attempts (for security monitoring)';
