-- Migration: Sessions table Kanidm support
-- Description: Update sessions table to support Kanidm OAuth2 sessions
-- Author: System
-- Date: 2025-01-10
-- Phase: 4.1 - Database Schema Updates

-- =============================================================================
-- STEP 1: Make token hashes nullable
-- =============================================================================

-- Kanidm manages tokens, so our sessions may not store them
ALTER TABLE sessions ALTER COLUMN access_token_hash DROP NOT NULL;
ALTER TABLE sessions ALTER COLUMN refresh_token_hash DROP NOT NULL;

COMMENT ON COLUMN sessions.access_token_hash IS 
  'SHA-256 hash of JWT access token (legacy auth). NULL for Kanidm sessions.';

COMMENT ON COLUMN sessions.refresh_token_hash IS 
  'SHA-256 hash of JWT refresh token (legacy auth). NULL for Kanidm sessions.';

-- =============================================================================
-- STEP 2: Add Kanidm session tracking
-- =============================================================================

-- Store Kanidm session ID for session management
ALTER TABLE sessions ADD COLUMN kanidm_session_id UUID;

-- Track authentication method used for this session
ALTER TABLE sessions ADD COLUMN auth_method VARCHAR(50) NOT NULL DEFAULT 'jwt';

-- Add constraint for valid auth methods
ALTER TABLE sessions ADD CONSTRAINT sessions_auth_method_check
  CHECK (auth_method IN ('jwt', 'kanidm', 'dual'));

COMMENT ON COLUMN sessions.kanidm_session_id IS 
  'Kanidm session UUID for OAuth2 sessions. NULL for legacy JWT sessions.';

COMMENT ON COLUMN sessions.auth_method IS 
  'Authentication method: jwt (legacy), kanidm (OAuth2), dual (hybrid session)';

-- =============================================================================
-- STEP 3: Update existing sessions
-- =============================================================================

-- Mark existing sessions as JWT-based (legacy)
UPDATE sessions 
SET auth_method = 'jwt'
WHERE kanidm_session_id IS NULL;

-- =============================================================================
-- STEP 4: Create indexes
-- =============================================================================

-- Index for Kanidm session lookup
CREATE INDEX idx_sessions_kanidm_session 
  ON sessions(kanidm_session_id) 
  WHERE kanidm_session_id IS NOT NULL AND NOT revoked;

-- Index for auth method analytics
CREATE INDEX idx_sessions_auth_method 
  ON sessions(auth_method, created_at) 
  WHERE NOT revoked;

-- Composite index for user sessions by auth method
CREATE INDEX idx_sessions_user_auth 
  ON sessions(user_id, auth_method, created_at) 
  WHERE NOT revoked;

-- =============================================================================
-- ANALYTICS VIEW
-- =============================================================================

-- Session distribution by auth method
CREATE OR REPLACE VIEW v_session_stats AS
SELECT 
  auth_method,
  COUNT(*) AS total_sessions,
  COUNT(*) FILTER (WHERE revoked = FALSE) AS active_sessions,
  COUNT(*) FILTER (WHERE access_token_expires_at > NOW()) AS valid_sessions,
  AVG(EXTRACT(EPOCH FROM (NOW() - created_at)) / 3600)::INTEGER AS avg_age_hours,
  MAX(last_used_at) AS most_recent_use
FROM sessions
WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'
GROUP BY auth_method
ORDER BY total_sessions DESC;

COMMENT ON VIEW v_session_stats IS 
  'Session statistics by authentication method (last 7 days)';

-- =============================================================================
-- CONSTRAINTS & VALIDATION
-- =============================================================================

-- Add check constraint: Kanidm sessions should have kanidm_session_id
-- Note: This is a soft constraint (warning, not enforced) because we want flexibility
-- ALTER TABLE sessions ADD CONSTRAINT sessions_kanidm_consistency_check
--   CHECK (
--     (auth_method = 'kanidm' AND kanidm_session_id IS NOT NULL) OR
--     (auth_method != 'kanidm')
--   );
-- Commented out - too strict for gradual migration

-- =============================================================================
-- CLEANUP HELPER
-- =============================================================================

-- Function to clean up expired sessions
CREATE OR REPLACE FUNCTION cleanup_expired_sessions(days_old INTEGER DEFAULT 30)
RETURNS TABLE (deleted_count BIGINT) AS $$
DECLARE
  deleted BIGINT;
BEGIN
  DELETE FROM sessions
  WHERE (
    -- Expired access tokens
    access_token_expires_at < NOW() - (days_old || ' days')::INTERVAL
    OR
    -- Revoked sessions older than threshold
    (revoked = TRUE AND revoked_at < NOW() - (days_old || ' days')::INTERVAL)
  );
  
  GET DIAGNOSTICS deleted = ROW_COUNT;
  RETURN QUERY SELECT deleted;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_expired_sessions IS 
  'Delete expired and old revoked sessions. Usage: SELECT * FROM cleanup_expired_sessions(30);';

-- =============================================================================
-- VERIFICATION QUERIES
-- =============================================================================

-- Check session distribution by auth method
-- SELECT * FROM v_session_stats;

-- Find Kanidm sessions
-- SELECT 
--   s.session_id,
--   s.auth_method,
--   s.kanidm_session_id,
--   u.email,
--   s.created_at,
--   s.last_used_at
-- FROM sessions s
-- JOIN users u ON s.user_id = u.user_id
-- WHERE s.auth_method = 'kanidm'
--   AND s.revoked = FALSE
-- ORDER BY s.created_at DESC
-- LIMIT 20;

-- Cleanup old sessions (dry run)
-- SELECT * FROM cleanup_expired_sessions(30);
