# Phase 4: Database Migration Plan

**Status**: 🚧 In Progress  
**Goal**: Migrate existing users from password-based auth to Kanidm OAuth2  
**Started**: 2025-11-03

---

## 📊 Overview

### Current State (Before Migration)
- ✅ Users table has `password_hash` column (bcrypt)
- ✅ Users table has `kanidm_user_id` column (added in Phase 3)
- ✅ `kanidm_tenant_groups` table exists
- ✅ OAuth2 flow working for NEW users
- ⚠️ OLD users still use password authentication

### Target State (After Migration)
- ✅ All users have `kanidm_user_id` (linked to Kanidm)
- ✅ `password_hash` column nullable (legacy support)
- ✅ All tenants mapped to Kanidm groups
- ✅ Dual authentication working (OAuth2 + password fallback)
- 🎯 Production ready for gradual rollout

---

## 🎯 Migration Strategy

### Strategy: **Gradual Migration (Dual Auth)**

**Why NOT "Big Bang"?**
- ❌ Risky: All users forced to Kanidm at once
- ❌ No rollback: Breaking change
- ❌ Support nightmare: Mass password resets

**Why Gradual?**
- ✅ Zero downtime
- ✅ Users migrate at their own pace
- ✅ Easy rollback (keep password auth)
- ✅ Support both auth methods simultaneously

### Migration Phases

```
Phase 4.1: Database Schema Updates (SAFE)
  ├─ Make password_hash nullable
  ├─ Add migration tracking columns
  └─ Create admin tools for bulk migration

Phase 4.2: Kanidm User Creation (MANUAL/AUTOMATED)
  ├─ Export existing users from PostgreSQL
  ├─ Create users in Kanidm (via API/CLI)
  ├─ Create tenant groups in Kanidm
  ├─ Assign users to groups
  └─ Record kanidm_user_id in PostgreSQL

Phase 4.3: Testing & Validation
  ├─ Test dual authentication
  ├─ Verify tenant isolation
  ├─ Test edge cases (missing groups, etc.)
  └─ Performance testing

Phase 4.4: Production Rollout (GRADUAL)
  ├─ Week 1: Internal users + admins
  ├─ Week 2-3: Pilot tenants (opt-in)
  ├─ Week 4+: All tenants (notify via email)
  └─ Month 2+: Deprecate password auth

Phase 4.5: Cleanup (AFTER 100% MIGRATION)
  ├─ Monitor auth method usage
  ├─ Drop password_hash column (AFTER confirmation)
  ├─ Remove password endpoints
  └─ Remove bcrypt dependencies
```

---

## 📝 Phase 4.1: Database Schema Updates

### Goal
Make database ready for dual authentication (OAuth2 + password fallback).

### Tasks

#### 1.1. Make password_hash Nullable
**Migration**: `20250110000014_password_hash_nullable.sql`

```sql
-- Allow password_hash to be NULL (for Kanidm-only users)
ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;

-- Add comment
COMMENT ON COLUMN users.password_hash IS 
  'DEPRECATED: Bcrypt password hash. NULL for Kanidm-only users. Will be removed after full migration.';
```

**Why nullable?**
- NEW users from Kanidm won't have password
- OLD users keep password (fallback)
- Gradual migration support

**Impact**: ✅ No breaking changes (existing data unchanged)

---

#### 1.2. Add Migration Tracking
**Migration**: `20250110000015_add_migration_tracking.sql`

```sql
-- Track user migration status
ALTER TABLE users ADD COLUMN auth_method VARCHAR(50) NOT NULL DEFAULT 'password';
ALTER TABLE users ADD COLUMN migration_invited_at TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN migration_completed_at TIMESTAMPTZ;

-- Constraints
ALTER TABLE users ADD CONSTRAINT users_auth_method_check 
  CHECK (auth_method IN ('password', 'kanidm', 'dual'));

-- Update existing Kanidm users
UPDATE users 
SET auth_method = 'kanidm', 
    migration_completed_at = kanidm_synced_at
WHERE kanidm_user_id IS NOT NULL;

-- Index for analytics
CREATE INDEX idx_users_auth_method ON users(auth_method);
CREATE INDEX idx_users_migration_status 
  ON users(tenant_id, auth_method, migration_completed_at);

-- Comments
COMMENT ON COLUMN users.auth_method IS 
  'Authentication method: password (legacy), kanidm (OAuth2 only), dual (both)';
COMMENT ON COLUMN users.migration_invited_at IS 
  'When user was invited to migrate to Kanidm (email sent)';
COMMENT ON COLUMN users.migration_completed_at IS 
  'When user completed Kanidm migration (first OAuth2 login)';
```

**Analytics Queries**:
```sql
-- Migration progress by tenant
SELECT 
  t.name AS tenant_name,
  COUNT(*) FILTER (WHERE u.auth_method = 'password') AS password_users,
  COUNT(*) FILTER (WHERE u.auth_method = 'kanidm') AS kanidm_users,
  COUNT(*) FILTER (WHERE u.auth_method = 'dual') AS dual_users,
  ROUND(100.0 * COUNT(*) FILTER (WHERE u.auth_method IN ('kanidm', 'dual')) / COUNT(*), 2) AS migration_percent
FROM users u
JOIN tenants t ON u.tenant_id = t.tenant_id
WHERE u.deleted_at IS NULL
GROUP BY t.tenant_id, t.name
ORDER BY migration_percent DESC;

-- Users not yet migrated
SELECT email, full_name, last_login_at, migration_invited_at
FROM users
WHERE auth_method = 'password' 
  AND deleted_at IS NULL
ORDER BY last_login_at DESC NULLS LAST;
```

---

#### 1.3. Sessions Table Update
**Migration**: `20250110000016_sessions_kanidm_support.sql`

```sql
-- Make token hashes nullable (Kanidm handles tokens)
ALTER TABLE sessions ALTER COLUMN access_token_hash DROP NOT NULL;
ALTER TABLE sessions ALTER COLUMN refresh_token_hash DROP NOT NULL;

-- Add Kanidm session tracking
ALTER TABLE sessions ADD COLUMN kanidm_session_id UUID;
ALTER TABLE sessions ADD COLUMN auth_method VARCHAR(50) NOT NULL DEFAULT 'jwt';

-- Constraint
ALTER TABLE sessions ADD CONSTRAINT sessions_auth_method_check
  CHECK (auth_method IN ('jwt', 'kanidm', 'dual'));

-- Index
CREATE INDEX idx_sessions_kanidm_session ON sessions(kanidm_session_id) WHERE kanidm_session_id IS NOT NULL;

-- Comments
COMMENT ON COLUMN sessions.kanidm_session_id IS 'Kanidm session UUID for session tracking';
COMMENT ON COLUMN sessions.auth_method IS 'Authentication method used: jwt (legacy), kanidm (OAuth2), dual (both)';
```

**Why?**
- Kanidm manages its own sessions
- Need to track which sessions are Kanidm-based
- Optional: Can revoke Kanidm sessions via API

---

## 📝 Phase 4.2: Kanidm User Creation

### Goal
Create all existing users in Kanidm and link them to PostgreSQL.

### Approach: Two Options

#### Option A: Manual (Small Deployments)
**When**: < 100 users, single tenant

**Steps**:
1. Export users: `scripts/export-users-for-kanidm.sh`
2. Manual Kanidm CLI commands
3. Update PostgreSQL with kanidm_user_id

#### Option B: Automated (Production)
**When**: 100+ users, multiple tenants

**Tools**:
- Migration script: `scripts/migrate-users-to-kanidm.sh`
- Kanidm API integration
- Batch processing with rollback

---

### 4.2.1. Export Existing Users

**Script**: `scripts/export-users-for-kanidm.sh`

```bash
#!/bin/bash
# Export users from PostgreSQL for Kanidm migration

set -e

DATABASE_URL=${DATABASE_URL:-"postgresql://anthill:anthill@localhost:5432/anthill"}
OUTPUT_FILE=${1:-"users_export_$(date +%Y%m%d_%H%M%S).json"}

echo "Exporting users from PostgreSQL..."

psql "$DATABASE_URL" -t -A -F"," --output="$OUTPUT_FILE" <<'SQL'
SELECT 
  json_agg(
    json_build_object(
      'user_id', u.user_id::text,
      'tenant_id', u.tenant_id::text,
      'tenant_slug', t.slug,
      'email', u.email,
      'full_name', u.full_name,
      'role', u.role,
      'status', u.status,
      'has_kanidm', (u.kanidm_user_id IS NOT NULL)
    )
  )
FROM users u
JOIN tenants t ON u.tenant_id = t.tenant_id
WHERE u.deleted_at IS NULL
  AND u.status = 'active'
  AND u.kanidm_user_id IS NULL -- Only users NOT yet in Kanidm
ORDER BY t.slug, u.email;
SQL

echo "✅ Exported to: $OUTPUT_FILE"
echo ""
echo "Next steps:"
echo "1. Review the file"
echo "2. Run: ./scripts/migrate-users-to-kanidm.sh $OUTPUT_FILE"
```

---

### 4.2.2. Automated Migration Script

**Script**: `scripts/migrate-users-to-kanidm.sh`

```bash
#!/bin/bash
# Migrate users from PostgreSQL to Kanidm
# Creates users in Kanidm, assigns to groups, updates PostgreSQL

set -e

# Configuration
KANIDM_URL=${KANIDM_URL:-"https://idm.example.com"}
KANIDM_ADMIN_TOKEN=${KANIDM_ADMIN_TOKEN:-""}
DATABASE_URL=${DATABASE_URL:-"postgresql://anthill:anthill@localhost:5432/anthill"}
INPUT_FILE=$1
DRY_RUN=${DRY_RUN:-"false"}

if [ -z "$INPUT_FILE" ]; then
  echo "Usage: $0 <users_export.json>"
  echo "Example: $0 users_export_20251103.json"
  exit 1
fi

if [ -z "$KANIDM_ADMIN_TOKEN" ]; then
  echo "❌ Error: KANIDM_ADMIN_TOKEN not set"
  echo "Get token with: kanidm login admin"
  echo "Then: export KANIDM_ADMIN_TOKEN=<token>"
  exit 1
fi

echo "🔧 Migration Configuration:"
echo "  Kanidm URL: $KANIDM_URL"
echo "  Database: $DATABASE_URL"
echo "  Input File: $INPUT_FILE"
echo "  Dry Run: $DRY_RUN"
echo ""

# Read users from JSON
USERS=$(cat "$INPUT_FILE")
USER_COUNT=$(echo "$USERS" | jq '. | length')

echo "📊 Found $USER_COUNT users to migrate"
echo ""

# Process each user
for i in $(seq 0 $((USER_COUNT - 1))); do
  USER=$(echo "$USERS" | jq -r ".[$i]")
  
  EMAIL=$(echo "$USER" | jq -r '.email')
  FULL_NAME=$(echo "$USER" | jq -r '.full_name')
  TENANT_SLUG=$(echo "$USER" | jq -r '.tenant_slug')
  ROLE=$(echo "$USER" | jq -r '.role')
  USER_ID=$(echo "$USER" | jq -r '.user_id')
  
  echo "[$((i+1))/$USER_COUNT] Migrating: $EMAIL (tenant: $TENANT_SLUG)"
  
  if [ "$DRY_RUN" == "true" ]; then
    echo "  [DRY RUN] Would create user in Kanidm"
    echo "  [DRY RUN] Would add to group: tenant_${TENANT_SLUG}_users"
    continue
  fi
  
  # Create user in Kanidm
  KANIDM_USER_ID=$(kanidm person create "$EMAIL" "$FULL_NAME" --output json | jq -r '.uuid')
  
  if [ -z "$KANIDM_USER_ID" ]; then
    echo "  ❌ Failed to create user in Kanidm"
    continue
  fi
  
  echo "  ✅ Created in Kanidm: $KANIDM_USER_ID"
  
  # Add to tenant group
  GROUP_NAME="tenant_${TENANT_SLUG}_users"
  kanidm group add-members "$GROUP_NAME" "$EMAIL" || {
    echo "  ⚠️  Group $GROUP_NAME not found, creating..."
    kanidm group create "$GROUP_NAME"
    kanidm group add-members "$GROUP_NAME" "$EMAIL"
  }
  
  # Add to role-specific group if admin
  if [ "$ROLE" == "admin" ] || [ "$ROLE" == "super_admin" ]; then
    ADMIN_GROUP="tenant_${TENANT_SLUG}_admins"
    kanidm group add-members "$ADMIN_GROUP" "$EMAIL" || {
      echo "  ⚠️  Group $ADMIN_GROUP not found, creating..."
      kanidm group create "$ADMIN_GROUP"
      kanidm group add-members "$ADMIN_GROUP" "$EMAIL"
    }
  fi
  
  # Update PostgreSQL with kanidm_user_id
  psql "$DATABASE_URL" -c "
    UPDATE users 
    SET kanidm_user_id = '$KANIDM_USER_ID',
        kanidm_synced_at = NOW(),
        auth_method = 'dual',
        migration_completed_at = NOW()
    WHERE user_id = '$USER_ID';
  "
  
  echo "  ✅ Updated PostgreSQL"
  echo ""
done

echo "🎉 Migration complete!"
echo ""
echo "Verification:"
echo "  psql $DATABASE_URL -c \"SELECT COUNT(*) FROM users WHERE kanidm_user_id IS NOT NULL;\""
```

---

### 4.2.3. Tenant Group Setup

**Script**: `scripts/setup-kanidm-tenant-groups.sh`

```bash
#!/bin/bash
# Create Kanidm groups for all tenants

set -e

DATABASE_URL=${DATABASE_URL:-"postgresql://anthill:anthill@localhost:5432/anthill"}

echo "📋 Fetching tenants from PostgreSQL..."

TENANTS=$(psql "$DATABASE_URL" -t -A -c "
  SELECT slug FROM tenants WHERE deleted_at IS NULL AND status = 'active';
")

for SLUG in $TENANTS; do
  echo "Creating groups for tenant: $SLUG"
  
  # Create users group
  kanidm group create "tenant_${SLUG}_users" || echo "  ℹ️  Group already exists"
  kanidm group set displayname "tenant_${SLUG}_users" "${SLUG} Users"
  
  # Create admins group
  kanidm group create "tenant_${SLUG}_admins" || echo "  ℹ️  Group already exists"
  kanidm group set displayname "tenant_${SLUG}_admins" "${SLUG} Admins"
  
  # Get group UUIDs
  USER_GROUP_UUID=$(kanidm group get "tenant_${SLUG}_users" --output json | jq -r '.uuid')
  ADMIN_GROUP_UUID=$(kanidm group get "tenant_${SLUG}_admins" --output json | jq -r '.uuid')
  
  # Get tenant_id
  TENANT_ID=$(psql "$DATABASE_URL" -t -A -c "SELECT tenant_id FROM tenants WHERE slug = '$SLUG';")
  
  # Insert into kanidm_tenant_groups
  psql "$DATABASE_URL" -c "
    INSERT INTO kanidm_tenant_groups (tenant_id, kanidm_group_uuid, kanidm_group_name, role)
    VALUES 
      ('$TENANT_ID', '$USER_GROUP_UUID', 'tenant_${SLUG}_users', 'member'),
      ('$TENANT_ID', '$ADMIN_GROUP_UUID', 'tenant_${SLUG}_admins', 'admin')
    ON CONFLICT (tenant_id, kanidm_group_uuid) DO NOTHING;
  " || echo "  ℹ️  Mapping already exists"
  
  echo "  ✅ Groups created and mapped"
done

echo ""
echo "🎉 All tenant groups set up!"
```

---

## 📝 Phase 4.3: Testing & Validation

### Test Checklist

#### 4.3.1. Dual Authentication Tests
- [ ] NEW user: OAuth2 login works
- [ ] OLD user: Password login still works
- [ ] MIGRATED user: Both OAuth2 AND password work
- [ ] User switches from password to OAuth2
- [ ] Invalid credentials rejected

#### 4.3.2. Tenant Isolation Tests
- [ ] User in tenant A cannot access tenant B data
- [ ] Groups correctly map to tenants
- [ ] Multiple groups handled correctly
- [ ] User without tenant group denied access

#### 4.3.3. Edge Cases
- [ ] User with no kanidm_user_id (legacy)
- [ ] User with kanidm_user_id but invalid UUID
- [ ] Kanidm server down (fallback to password)
- [ ] Expired Kanidm token (refresh works)
- [ ] Revoked Kanidm session

#### 4.3.4. Performance Tests
- [ ] JWT validation latency < 50ms
- [ ] JWKS cache working (not fetching every request)
- [ ] Database queries optimized (indexes used)
- [ ] No N+1 queries in OAuth2 callback

---

## 📝 Phase 4.4: Production Rollout

### Rollout Timeline

#### Week 1: Internal Testing
- Migrate internal users + admins
- Test all features with OAuth2
- Monitor error logs
- Fix any issues

#### Week 2-3: Pilot Program
- Select 2-3 friendly tenants
- Send invitation email with migration guide
- Provide dedicated support
- Collect feedback

#### Week 4+: General Availability
- Email all tenants about OAuth2 option
- Highlight benefits (Passkeys, TOTP, WebAuthn)
- Keep password auth as fallback
- Monitor adoption rate

#### Month 2+: Deprecation Notice
- Announce password auth deprecation timeline
- Require OAuth2 for NEW users
- Send reminders to non-migrated users

---

### Invitation Email Template

```markdown
Subject: 🔐 Introducing Secure Login with Kanidm

Hi [User Name],

We're excited to introduce a more secure way to access Anthill Inventory!

**What's changing?**
- New login option using Kanidm (our Identity Provider)
- Support for Passkeys, WebAuthn, and TOTP (2FA)
- More secure than traditional passwords

**What you need to do:**
1. Click "Login with Kanidm" on the login page
2. Create your Kanidm account (one-time setup)
3. Done! You can now use Passkeys or TOTP for login

**Your password still works**
- No rush - migrate when ready
- Password login available until [DATE]

**Questions?**
- Migration guide: https://docs.anthill.example/kanidm-migration
- Support: support@anthill.example

Thanks,
Anthill Team
```

---

## 📝 Phase 4.5: Cleanup (Future)

### Prerequisites
- ✅ 100% of active users migrated to Kanidm
- ✅ No password logins in last 30 days
- ✅ All tenants confirmed migrated

### Tasks

#### 5.1. Drop password_hash Column
**Migration**: `20250199999999_remove_password_auth.sql` (Future)

```sql
-- ⚠️ DESTRUCTIVE - Only run after 100% migration confirmed

-- Drop password column
ALTER TABLE users DROP COLUMN password_hash;
ALTER TABLE users DROP COLUMN password_changed_at;
ALTER TABLE users DROP COLUMN failed_login_attempts;
ALTER TABLE users DROP COLUMN locked_until;

-- Remove auth_method (all users on Kanidm)
ALTER TABLE users DROP COLUMN auth_method;
ALTER TABLE users DROP COLUMN migration_invited_at;
ALTER TABLE users DROP COLUMN migration_completed_at;

-- Update constraints
ALTER TABLE users ALTER COLUMN kanidm_user_id SET NOT NULL;

-- Update comments
COMMENT ON TABLE users IS 'User accounts authenticated via Kanidm OAuth2';
```

#### 5.2. Remove Password Endpoints
- Delete `POST /api/v1/auth/register`
- Delete `POST /api/v1/auth/login`
- Delete `POST /api/v1/auth/reset-password`
- Delete `POST /api/v1/auth/change-password`
- Update API documentation

#### 5.3. Remove Dependencies
```toml
# Remove from Cargo.toml
bcrypt = "0.15"  # No longer needed
```

---

## 📊 Success Metrics

### Key Performance Indicators

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Migration Rate | 95% in 60 days | `SELECT ... FROM users WHERE auth_method IN ('kanidm', 'dual')` |
| Auth Latency | < 100ms p99 | Monitor JWT validation time |
| Error Rate | < 0.1% | Count failed OAuth2 callbacks |
| User Satisfaction | 4.5/5 | Post-migration survey |
| Support Tickets | < 10 tickets | Track migration-related issues |

### Dashboard Queries

```sql
-- Overall migration status
WITH stats AS (
  SELECT 
    COUNT(*) AS total_users,
    COUNT(*) FILTER (WHERE auth_method = 'kanidm') AS kanidm_only,
    COUNT(*) FILTER (WHERE auth_method = 'dual') AS dual_auth,
    COUNT(*) FILTER (WHERE auth_method = 'password') AS password_only
  FROM users 
  WHERE deleted_at IS NULL
)
SELECT 
  *,
  ROUND(100.0 * (kanidm_only + dual_auth) / total_users, 2) AS migration_percent
FROM stats;

-- Daily migration progress
SELECT 
  DATE(migration_completed_at) AS migration_date,
  COUNT(*) AS users_migrated
FROM users
WHERE migration_completed_at >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY DATE(migration_completed_at)
ORDER BY migration_date DESC;
```

---

## ✅ Implementation Checklist

### Phase 4.1: Schema Updates
- [ ] Run migration: 20250110000014_password_hash_nullable.sql
- [ ] Run migration: 20250110000015_add_migration_tracking.sql
- [ ] Run migration: 20250110000016_sessions_kanidm_support.sql
- [ ] Verify migrations applied
- [ ] Test application still works

### Phase 4.2: Kanidm Setup
- [ ] Create scripts/export-users-for-kanidm.sh
- [ ] Create scripts/migrate-users-to-kanidm.sh
- [ ] Create scripts/setup-kanidm-tenant-groups.sh
- [ ] Test scripts in development
- [ ] Export production users
- [ ] Create Kanidm tenant groups
- [ ] Migrate users (dry run first)
- [ ] Verify kanidm_user_id populated

### Phase 4.3: Testing
- [ ] Write dual auth integration tests
- [ ] Test all scenarios (NEW, OLD, MIGRATED users)
- [ ] Performance testing
- [ ] Security testing
- [ ] Load testing with Kanidm

### Phase 4.4: Rollout
- [ ] Migrate internal users (Week 1)
- [ ] Monitor and fix issues
- [ ] Select pilot tenants (Week 2)
- [ ] Send invitation emails
- [ ] Collect feedback
- [ ] General rollout (Week 4+)
- [ ] Track adoption metrics

### Phase 4.5: Cleanup (Future)
- [ ] Confirm 100% migration
- [ ] Drop password_hash column
- [ ] Remove password endpoints
- [ ] Remove bcrypt dependency
- [ ] Update documentation

---

## 🚨 Rollback Plan

### If Migration Fails

#### Immediate Rollback (< 24 hours)
```sql
-- Revert users to password-only
UPDATE users 
SET auth_method = 'password',
    migration_completed_at = NULL
WHERE auth_method = 'dual';

-- Clear kanidm_user_id (optional)
UPDATE users SET kanidm_user_id = NULL;
```

#### Partial Rollback (Specific Tenant)
```sql
UPDATE users
SET auth_method = 'password'
WHERE tenant_id = '<tenant_id>' AND auth_method = 'dual';
```

#### Code Rollback
```bash
# Revert OAuth2 endpoints
git revert <commit_hash>

# Redeploy
./deploy.sh
```

---

## 📚 References

- Phase 3 Summary: [PHASE_3_SUMMARY.md](./PHASE_3_SUMMARY.md)
- Kanidm API Docs: https://kanidm.com/
- Migration Guide: [docs/KANIDM_OAUTH2_TESTING.md](./KANIDM_OAUTH2_TESTING.md)
- Database Schema: [migrations/README.md](../migrations/README.md)

---

**Next Action**: Run Phase 4.1 schema migrations
