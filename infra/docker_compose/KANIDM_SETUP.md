# Kanidm Setup Guide for Anthill

This guide explains how to set up and use Kanidm as the Identity Provider for Anthill.

## Quick Start

### 1. Start Services

```bash
cd infra/docker_compose
docker-compose up -d kanidm
```

### 2. Initialize Admin Account

On first run, you need to recover the admin account:

```bash
# Recover admin account and set password
docker exec -it kanidm_idm kanidm recover-account admin

# Follow the prompts to set admin password
# Recommended for dev: dev_admin_password_change_in_prod
```

### 3. Run Initialization Script

```bash
# Make script executable
chmod +x init-kanidm.sh

# Run initialization
docker exec -it kanidm_idm sh /init-kanidm.sh
```

This will:
- Create OAuth2 client `anthill`
- Configure redirect URLs
- Enable PKCE
- Create tenant groups (`tenant_acme_users`, `tenant_acme_admins`, etc.)
- Create test users (alice, bob, charlie)

### 4. Get OAuth2 Client Secret

```bash
docker exec kanidm_idm kanidm system oauth2 show-basic-secret \
  -H http://localhost:8300 \
  -D admin \
  anthill
```

**Save this secret** - you'll need it for the backend service.

## Access Kanidm

- **Web UI**: http://localhost:8300/ui
- **API**: http://localhost:8300

## Test Users

| Username | Email | Password | Groups | Tenant |
|----------|-------|----------|--------|--------|
| alice | alice@acme.example.com | Test123!@# | tenant_acme_users, tenant_acme_admins | Acme Corp |
| bob | bob@acme.example.com | Test123!@# | tenant_acme_users | Acme Corp |
| charlie | charlie@globex.example.com | Test123!@# | tenant_globex_users | Globex Inc |

## OAuth2 Configuration

### Client Details
- **Client ID**: `anthill`
- **Client Secret**: Use `show-basic-secret` command above
- **Redirect URLs**:
  - `http://localhost:5173/oauth/callback` (Frontend dev)
  - `http://localhost:8000/oauth/callback` (Backend dev)
  - `https://app.example.com/oauth/callback` (Production)

### OAuth2 Endpoints
- **Authorization**: `http://localhost:8300/ui/oauth2?client_id=anthill&...`
- **Token**: `http://localhost:8300/oauth2/token`
- **UserInfo**: `http://localhost:8300/oauth2/openid/userinfo`
- **JWKS**: `http://localhost:8300/oauth2/openid/.well-known/jwks.json`

### Scopes
- `openid` - Required for OIDC
- `profile` - User profile information
- `email` - User email address
- `groups` - User group memberships (for tenant mapping)

## Common Operations

### Create New Tenant Group

```bash
# Create group
docker exec kanidm_idm kanidm group create \
  -H http://localhost:8300 \
  -D admin \
  tenant_newcompany_users

# Set display name
docker exec kanidm_idm kanidm group set displayname \
  -H http://localhost:8300 \
  -D admin \
  tenant_newcompany_users \
  "New Company - Users"
```

### Create New User

```bash
# Create user
docker exec kanidm_idm kanidm person create \
  -H http://localhost:8300 \
  -D admin \
  johndoe \
  "John Doe"

# Set email
docker exec kanidm_idm kanidm person set mail \
  -H http://localhost:8300 \
  -D admin \
  johndoe \
  john@example.com

# Set password
docker exec kanidm_idm kanidm person set password \
  -H http://localhost:8300 \
  -D admin \
  johndoe \
  "SecurePassword123!"

# Add to group
docker exec kanidm_idm kanidm group add-members \
  -H http://localhost:8300 \
  -D admin \
  tenant_acme_users \
  johndoe
```

### Test OAuth2 Flow

```bash
# 1. Get authorization code (open in browser)
open "http://localhost:8300/ui/oauth2?client_id=anthill&redirect_uri=http://localhost:5173/oauth/callback&response_type=code&scope=openid%20profile%20email%20groups&state=random_state_string"

# 2. After login, you'll be redirected with a code
# http://localhost:5173/oauth/callback?code=xyz&state=random_state_string

# 3. Exchange code for token
curl -X POST http://localhost:8300/oauth2/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -u "anthill:YOUR_CLIENT_SECRET" \
  -d "grant_type=authorization_code" \
  -d "code=xyz" \
  -d "redirect_uri=http://localhost:5173/oauth/callback"

# Response:
# {
#   "access_token": "JWT_TOKEN_HERE",
#   "token_type": "Bearer",
#   "expires_in": 3600,
#   "refresh_token": "REFRESH_TOKEN_HERE",
#   "id_token": "ID_TOKEN_HERE"
# }
```

### Decode JWT to See Claims

```bash
# Install jq if needed: sudo apt install jq

# Decode access_token (paste your token)
echo "YOUR_JWT_TOKEN" | cut -d. -f2 | base64 -d | jq

# You should see claims like:
# {
#   "sub": "uuid-of-user",
#   "email": "alice@acme.example.com",
#   "preferred_username": "alice",
#   "groups": ["tenant_acme_users", "tenant_acme_admins"],
#   "exp": 1234567890,
#   "iat": 1234567890
# }
```

## Environment Variables for Backend

Add these to your `.env` or export:

```env
KANIDM_URL=http://localhost:8300
KANIDM_OAUTH2_CLIENT_ID=anthill
KANIDM_OAUTH2_CLIENT_SECRET=<from show-basic-secret command>
OAUTH2_REDIRECT_URI=http://localhost:8000/oauth/callback
OAUTH2_SCOPES=openid,profile,email,groups
```

## Troubleshooting

### Kanidm not starting

```bash
# Check logs
docker logs kanidm_idm

# Common issues:
# - Port 8300 already in use
# - Volume permission issues
# - Config file syntax errors
```

### OAuth2 redirect not working

1. Check redirect URL is registered:
```bash
docker exec kanidm_idm kanidm system oauth2 get \
  -H http://localhost:8300 \
  -D admin \
  anthill
```

2. Add missing redirect URL:
```bash
docker exec kanidm_idm kanidm system oauth2 add-redirect-url \
  -H http://localhost:8300 \
  -D admin \
  anthill \
  "http://your-url/callback"
```

### Invalid client credentials

Re-check client secret:
```bash
docker exec kanidm_idm kanidm system oauth2 show-basic-secret \
  -H http://localhost:8300 \
  -D admin \
  anthill
```

### User can't login

1. Check user exists:
```bash
docker exec kanidm_idm kanidm person get \
  -H http://localhost:8300 \
  -D admin \
  username
```

2. Reset password:
```bash
docker exec kanidm_idm kanidm person set password \
  -H http://localhost:8300 \
  -D admin \
  username \
  "NewPassword123!"
```

## Production Considerations

### Security

1. **Use HTTPS**: Enable TLS in `kanidm-server.toml`
2. **Strong passwords**: Change all default passwords
3. **Firewall**: Restrict access to Kanidm ports
4. **Backup**: Enable automatic backups in config
5. **PostgreSQL**: Switch from SQLite to PostgreSQL

### High Availability

1. Use Kanidm in Read-Write-Replica mode
2. Setup load balancer for multiple replicas
3. Use shared PostgreSQL database
4. Configure Redis for session storage

### Monitoring

1. Check `/status` endpoint for health
2. Monitor logs for errors
3. Track OAuth2 token issuance rate
4. Set up alerts for failed login attempts

## References

- **Kanidm Documentation**: https://kanidm.github.io/kanidm/
- **OAuth2 Integration**: https://kanidm.github.io/kanidm/master/integrations/oauth2.html
- **CLI Reference**: https://kanidm.github.io/kanidm/master/cli.html
- **Migration Plan**: `docs/KANIDM_MIGRATION_PLAN.md`
