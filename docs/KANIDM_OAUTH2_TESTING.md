# Kanidm OAuth2 Flow - Testing Guide

## Prerequisites

1. **Start Services**:
   ```bash
   cd infra/docker_compose
   docker compose up -d postgres kanidm
   ```

2. **Run Database Migrations**:
   ```bash
   # Set test database URL
   export DATABASE_URL="postgres://anthill_test:anthill_test_password@localhost:5433/anthill_test"
   
   # Run migrations
   sqlx migrate run
   ```

3. **Verify Migration**:
   ```bash
   # Check if kanidm_integration migration was applied
   psql $DATABASE_URL -c "\d users" | grep kanidm
   psql $DATABASE_URL -c "\d kanidm_tenant_groups"
   ```

## Setup Test Data

### 1. Create Test Tenant

```sql
-- Connect to database
psql postgres://anthill_test:anthill_test_password@localhost:5433/anthill_test

-- Create test tenant
INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
VALUES (
  '018c3f1e-1234-7890-abcd-000000000001'::uuid,
  'ACME Corporation',
  'acme',
  'enterprise',
  'active',
  '{}'::jsonb,
  NOW(),
  NOW()
);
```

### 2. Initialize Kanidm

```bash
# Access Kanidm container
docker exec -it kanidm bash

# Setup admin password
kanidmd recover-account -c /data/server.toml admin

# Create OAuth2 client
kanidm login -H https://localhost:8300 -D anonymous
kanidm system oauth2 create anthill "Anthill Inventory" https://localhost:3000

# Configure OAuth2 client
kanidm system oauth2 add-redirect-url anthill https://localhost:3000/oauth/callback
kanidm system oauth2 add-redirect-url anthill http://localhost:3000/oauth/callback
kanidm system oauth2 enable-pkce anthill

# Configure scope map
kanidm system oauth2 update-scope-map anthill anthill_users email openid profile groups

# Get client secret
kanidm system oauth2 show-basic-secret anthill
# Save this secret for .env
```

### 3. Create Test User in Kanidm

```bash
# Still in Kanidm container or use kanidm CLI

# Create user
kanidm person create testuser "Test User" --mail testuser@example.com

# Set password
kanidm person update testuser --legalname "Test User"
kanidm person credential create-reset-token testuser
# Use web UI to complete password setup: https://localhost:8300

# Create tenant groups
kanidm group create tenant_acme_admins
kanidm group create tenant_acme_users

# Add user to group
kanidm group add-members tenant_acme_admins testuser

# Grant OAuth2 access
kanidm group add-members anthill_users testuser
```

### 4. Map Kanidm Groups to Tenant

```sql
-- Connect to database
psql $DATABASE_URL

-- Map tenant_acme_admins group to ACME tenant
INSERT INTO kanidm_tenant_groups (tenant_id, kanidm_group_uuid, kanidm_group_name, role, created_at, updated_at)
VALUES (
  '018c3f1e-1234-7890-abcd-000000000001'::uuid,
  '00000000-0000-0000-0000-000000000001'::uuid, -- Placeholder UUID
  'tenant_acme_admins',
  'admin',
  NOW(),
  NOW()
);

-- Map tenant_acme_users group
INSERT INTO kanidm_tenant_groups (tenant_id, kanidm_group_uuid, kanidm_group_name, role, created_at, updated_at)
VALUES (
  '018c3f1e-1234-7890-abcd-000000000001'::uuid,
  '00000000-0000-0000-0000-000000000002'::uuid,
  'tenant_acme_users',
  'member',
  NOW(),
  NOW()
);
```

## Test OAuth2 Flow

### 1. Start User Service

```bash
# Set environment variables
export DATABASE_URL="postgres://anthill_test:anthill_test_password@localhost:5433/anthill_test"
export JWT_SECRET="your-super-secret-jwt-key-min-32-chars-long"
export JWT_EXPIRATION=3600
export JWT_REFRESH_EXPIRATION=604800
export KANIDM_URL="https://localhost:8300"
export KANIDM_CLIENT_ID="anthill"
export KANIDM_CLIENT_SECRET="<secret-from-step-2>"
export KANIDM_REDIRECT_URL="http://localhost:3000/oauth/callback"

# Run user service
cargo run --bin user-service
```

### 2. Test Authorize Endpoint

```bash
curl -X POST http://localhost:3000/api/v1/auth/oauth/authorize \
  -H "Content-Type: application/json" \
  -d '{"state": "random-state-123"}' | jq

# Expected response:
# {
#   "authorization_url": "https://localhost:8300/ui/oauth2?client_id=anthill&...",
#   "state": "random-state-123",
#   "code_verifier": "<pkce-verifier>"
# }

# SAVE the code_verifier for next step!
```

### 3. Complete Authentication in Browser

1. Copy `authorization_url` from response
2. Open in browser (accept SSL warning for localhost)
3. Login with testuser credentials
4. Approve OAuth2 consent
5. Browser redirects to: `http://localhost:3000/oauth/callback?code=<auth-code>&state=random-state-123`
6. Extract the `code` parameter from URL

### 4. Test Callback Endpoint

```bash
# Replace with actual values from previous steps
AUTH_CODE="<code-from-browser-redirect>"
CODE_VERIFIER="<verifier-from-authorize-response>"
STATE="random-state-123"

curl -X POST http://localhost:3000/api/v1/auth/oauth/callback \
  -H "Content-Type: application/json" \
  -d "{
    \"code\": \"$AUTH_CODE\",
    \"state\": \"$STATE\",
    \"code_verifier\": \"$CODE_VERIFIER\"
  }" | jq

# Expected response:
# {
#   "access_token": "<kanidm-jwt>",
#   "refresh_token": "<refresh-token>",
#   "token_type": "Bearer",
#   "expires_in": 3600,
#   "user": {
#     "kanidm_user_id": "<uuid>",
#     "email": "testuser@example.com",
#     "preferred_username": "testuser",
#     "groups": ["tenant_acme_admins", "anthill_users"]
#   },
#   "tenant": {
#     "tenant_id": "018c3f1e-1234-7890-abcd-000000000001",
#     "name": "ACME Corporation",
#     "slug": "acme",
#     "role": "admin"
#   }
# }
```

### 5. Verify User Was Created

```sql
-- Check users table
psql $DATABASE_URL -c "SELECT user_id, email, kanidm_user_id, kanidm_synced_at, tenant_id FROM users;"

-- Should see new user with:
-- - kanidm_user_id populated
-- - tenant_id = ACME tenant
-- - kanidm_synced_at = recent timestamp
```

### 6. Test Refresh Token

```bash
REFRESH_TOKEN="<refresh-token-from-callback>"

curl -X POST http://localhost:3000/api/v1/auth/oauth/refresh \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\": \"$REFRESH_TOKEN\"}" | jq

# Expected: new access_token
```

## Expected Logs

When testing, user service should log:

```
✅ Kanidm client initialized
OAuth2 callback received with code
Token exchange successful
Token validated - user: <uuid>, email: Some("testuser@example.com")
Checking group: tenant_acme_admins
Found tenant: ACME Corporation with role: admin
Mapping user to tenant: ACME Corporation with role: admin
Created new user from Kanidm authentication: <user-uuid>
```

## Troubleshooting

### Issue: "No matching tenant found"
- **Check**: Kanidm groups claim in JWT
  ```bash
  # Decode JWT to see groups
  echo "<access-token>" | cut -d'.' -f2 | base64 -d | jq
  ```
- **Verify**: kanidm_tenant_groups table has matching group names
- **Fix**: Ensure group names match exactly (case-sensitive)

### Issue: "Connection refused" to Kanidm
- **Check**: Kanidm is running: `docker ps | grep kanidm`
- **Check**: SSL certificate accepted in browser first
- **Try**: Use KANIDM_URL with http:// instead of https:// for dev

### Issue: "Invalid code" in callback
- **Cause**: PKCE code_verifier doesn't match
- **Fix**: Ensure code_verifier from authorize response is used
- **Note**: Authorization codes expire quickly (60 seconds)

### Issue: User created with wrong tenant
- **Check**: JWT groups claim includes expected tenant group
- **Check**: kanidm_tenant_groups mapping exists
- **Debug**: Add logging in map_tenant_from_groups()

## Next Steps

After successful testing:
1. ✅ Verify OAuth2 flow works end-to-end
2. ✅ User auto-creation working
3. ✅ Tenant mapping from groups working
4. Move to Phase 4: Database migration for production
5. Phase 5: Integration tests
6. Phase 6: Documentation & cleanup
