-- Integration Test Database Schema
-- This file defines additional tables/functions needed specifically for integration tests
-- Separate from main migrations to keep test concerns isolated

-- Test data cleanup function
CREATE OR REPLACE FUNCTION cleanup_test_data()
RETURNS void AS $$
BEGIN
    -- Delete in reverse dependency order to avoid FK violations
    DELETE FROM sessions WHERE tenant_id IN (SELECT tenant_id FROM tenants WHERE slug LIKE 'test-%');
    DELETE FROM user_profiles WHERE user_id IN (SELECT user_id FROM users WHERE tenant_id IN (SELECT tenant_id FROM tenants WHERE slug LIKE 'test-%'));
    DELETE FROM users WHERE tenant_id IN (SELECT tenant_id FROM tenants WHERE slug LIKE 'test-%');
    DELETE FROM tenants WHERE slug LIKE 'test-%';

    -- Reset sequences if needed
    -- ALTER SEQUENCE users_id_seq RESTART WITH 1;
END;
$$ LANGUAGE plpgsql;

-- Test data isolation function
-- Ensures test data doesn't interfere with production-like data
CREATE OR REPLACE FUNCTION is_test_tenant(tenant_id_param UUID)
RETURNS boolean AS $$
DECLARE
    tenant_slug TEXT;
BEGIN
    SELECT slug INTO tenant_slug FROM tenants WHERE tenant_id = tenant_id_param;
    RETURN tenant_slug LIKE 'test-%';
END;
$$ LANGUAGE plpgsql;

-- Snapshot function for test state verification
CREATE OR REPLACE FUNCTION snapshot_tenant_data(tenant_id_param UUID)
RETURNS TABLE(
    users_count BIGINT,
    sessions_count BIGINT,
    profiles_count BIGINT,
    tenant_status TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        (SELECT COUNT(*) FROM users WHERE tenant_id = tenant_id_param),
        (SELECT COUNT(*) FROM sessions WHERE tenant_id = tenant_id_param),
        (SELECT COUNT(*) FROM user_profiles WHERE user_id IN (SELECT user_id FROM users WHERE tenant_id = tenant_id_param)),
        (SELECT status FROM tenants WHERE tenant_id = tenant_id_param);
END;
$$ LANGUAGE plpgsql;

-- Test data generator for bulk testing
CREATE OR REPLACE FUNCTION generate_test_users(
    tenant_id_param UUID,
    count_param INTEGER DEFAULT 10
)
RETURNS void AS $$
DECLARE
    i INTEGER;
    user_id_var UUID;
BEGIN
    FOR i IN 1..count_param LOOP
        user_id_var := gen_random_uuid();

        INSERT INTO users (
            user_id, tenant_id, email, password_hash, role, status,
            email_verified, email_verified_at, full_name, created_at, updated_at
        ) VALUES (
            user_id_var,
            tenant_id_param,
            'test-user-' || i || '@test.com',
            '$argon2id$v=19$m=19456,t=2,p=1$test-salt$test-hash',
            'user',
            'active',
            true,
            NOW(),
            'Test User ' || i,
            NOW(),
            NOW()
        );
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Index for faster test cleanup
-- Partial index on tenants with test slugs
CREATE INDEX IF NOT EXISTS idx_tenants_test_slug ON tenants(slug) WHERE slug LIKE 'test-%';

-- Regular index on users.tenant_id for efficient cleanup joins
CREATE INDEX IF NOT EXISTS idx_users_tenant_test ON users(tenant_id);

-- Grant execute permissions (commented out for dev environment)
-- GRANT EXECUTE ON FUNCTION cleanup_test_data() TO anthill;
-- GRANT EXECUTE ON FUNCTION is_test_tenant(UUID) TO anthill;
-- GRANT EXECUTE ON FUNCTION snapshot_tenant_data(UUID) TO anthill;
-- GRANT EXECUTE ON FUNCTION generate_test_users(UUID, INTEGER) TO anthill;

-- Comments for documentation
COMMENT ON FUNCTION cleanup_test_data() IS 'Removes all test data (tenants with slug starting with "test-")';
COMMENT ON FUNCTION is_test_tenant(UUID) IS 'Returns true if tenant_id belongs to a test tenant';
COMMENT ON FUNCTION snapshot_tenant_data(UUID) IS 'Returns summary statistics for a tenant (for test verification)';
COMMENT ON FUNCTION generate_test_users(UUID, INTEGER) IS 'Generates N test users for a given tenant (for bulk testing)';
