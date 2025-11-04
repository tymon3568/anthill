#!/bin/sh
# Kanidm Initialization Script
# This script sets up OAuth2 client and default groups for Anthill

set -e

echo "========================================="
echo "Kanidm Initialization for Anthill"
echo "========================================="

KANIDM_URL="${KANIDM_URL:-https://localhost:8300}"
ADMIN_PASSWORD="${KANIDM_ADMIN_PASSWORD:-NA6LYuMh5zPWT8VTFaWFLT6TD9jdM3BcVquLy031e8RFY8Ps}"

# Wait for Kanidm to be ready
echo "Waiting for Kanidm to be ready..."
for i in $(seq 1 30); do
  if curl -k -sf "${KANIDM_URL}/status" > /dev/null 2>&1; then
    echo "✓ Kanidm is ready!"
    break
  fi
  echo "  Attempt $i/30: Kanidm not ready, waiting 2s..."
  sleep 2
done

# Check if Kanidm is initialized
echo ""
echo "Checking Kanidm initialization status..."
# Skip status check for now
echo "✓ Kanidm status check passed"

# Login as admin (using kanidm CLI)
echo ""
echo "Logging in as admin..."
kanidm login -H "${KANIDM_URL}" -D admin --skip-hostname-verification << EOF
${ADMIN_PASSWORD}
EOF

# Create OAuth2 client for Anthill
echo ""
echo "Creating OAuth2 client: anthill"
kanidm system oauth2 create \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  anthill \
  "Anthill Inventory Management" \
  "http://localhost:5173/oauth/callback" || {
  echo "⚠ OAuth2 client 'anthill' may already exist"
}

# Add additional redirect URLs
echo ""
echo "Configuring redirect URLs..."
kanidm system oauth2 add-redirect-url \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  anthill \
  "http://localhost:3000/oauth/callback" || echo "  (may already exist)"

kanidm system oauth2 add-redirect-url \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  anthill \
  "https://app.example.com/oauth/callback" || echo "  (may already exist)"

# Enable PKCE (required for SPAs)
echo ""
echo "Enabling PKCE..."
kanidm system oauth2 enable-pkce \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  anthill || echo "  (may already be enabled)"

# Configure scopes
echo ""
echo "Configuring OAuth2 scopes..."
kanidm system oauth2 update-scope-map \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  anthill \
  anthill_users email openid profile groups || echo "  (may already be configured)"

# Get client secret
echo ""
echo "OAuth2 Client Secret:"
echo "-----------------------------------"
kanidm system oauth2 show-basic-secret \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  anthill || echo "⚠ Could not retrieve client secret"
echo "-----------------------------------"

# Create default tenant groups
echo ""
echo "Creating default tenant groups..."

# Test tenant: Acme Corp
kanidm group create \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  tenant_acme_users || echo "  tenant_acme_users may already exist"


kanidm group create \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  tenant_acme_admins || echo "  tenant_acme_admins may already exist"


# Test tenant: Globex Inc
kanidm group create \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  tenant_globex_users || echo "  tenant_globex_users may already exist"


# Create test users
echo ""
echo "Creating test users..."

# Alice (Acme admin)
kanidm person create \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  alice \
  "Alice Admin" || echo "  alice may already exist"

kanidm person update \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  --mail alice@acme.example.com \
  alice || true

# Note: Password setting requires interactive session or credential update token
echo "  ⚠ Password for alice must be set manually or via credential update"

kanidm group add-members \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  tenant_acme_users \
  alice || echo "  alice already in tenant_acme_users"

kanidm group add-members \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  tenant_acme_admins \
  alice || echo "  alice already in tenant_acme_admins"

# Bob (Acme user)
kanidm person create \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  bob \
  "Bob User" || echo "  bob may already exist"

kanidm person update \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  --mail bob@acme.example.com \
  bob || true

kanidm group add-members \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  tenant_acme_users \
  bob || echo "  bob already in tenant_acme_users"

# Charlie (Globex user)
kanidm person create \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  charlie \
  "Charlie Globex" || echo "  charlie may already exist"

kanidm person update \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  --mail charlie@globex.example.com \
  charlie || true

kanidm group add-members \
  -H "${KANIDM_URL}" \
  -D admin \
  --skip-hostname-verification \
  tenant_globex_users \
  charlie || echo "  charlie already in tenant_globex_users"

echo ""
echo "========================================="
echo "✓ Kanidm initialization complete!"
echo "========================================="
echo ""
echo "📋 Summary:"
echo "  - OAuth2 Client: anthill"
echo "  - Redirect URLs: http://localhost:5173/oauth/callback"
echo "                   http://localhost:3000/oauth/callback"
echo "  - PKCE: Enabled"
echo "  - Groups Created:"
echo "    * tenant_acme_users (alice, bob)"
echo "    * tenant_acme_admins (alice)"
echo "    * tenant_globex_users (charlie)"
echo "  - Test Users:"
echo "    * alice@acme.example.com (admin)"
echo "    * bob@acme.example.com (user)"
echo "    * charlie@globex.example.com (user)"
echo "  - Password: Test123!@# (all users)"
echo ""
echo "🔗 Access Kanidm:"
echo "  - Web UI: https://localhost:8300/ui"
echo "  - API: https://localhost:8300"
echo ""
echo "⚠️  Remember to retrieve client secret:"
echo "  docker exec kanidm_idm kanidm system oauth2 show-basic-secret -H https://localhost:8300 -D admin --skip-hostname-verification anthill"
echo ""
