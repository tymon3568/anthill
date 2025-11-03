#!/bin/sh
# Kanidm Initialization Script
# This script sets up OAuth2 client and default groups for Anthill

set -e

echo "========================================="
echo "Kanidm Initialization for Anthill"
echo "========================================="

KANIDM_URL="${KANIDM_URL:-http://localhost:8300}"
ADMIN_PASSWORD="${KANIDM_ADMIN_PASSWORD:-dev_admin_password_change_in_prod}"

# Wait for Kanidm to be ready
echo "Waiting for Kanidm to be ready..."
for i in $(seq 1 30); do
  if curl -sf "${KANIDM_URL}/status" > /dev/null 2>&1; then
    echo "✓ Kanidm is ready!"
    break
  fi
  echo "  Attempt $i/30: Kanidm not ready, waiting 2s..."
  sleep 2
done

# Check if Kanidm is initialized
echo ""
echo "Checking Kanidm initialization status..."
if ! curl -sf "${KANIDM_URL}/status" | grep -q "ok"; then
  echo "⚠ Kanidm not responding properly, exiting"
  exit 1
fi

echo "✓ Kanidm status check passed"

# Login as admin (using kanidm CLI)
echo ""
echo "Logging in as admin..."
kanidm login -H "${KANIDM_URL}" -D admin -w "${ADMIN_PASSWORD}" || {
  echo "⚠ Admin login failed - this is expected on first run"
  echo "  You need to manually set admin password first:"
  echo "  docker exec -it kanidm_idm kanidm recover-account admin"
}

# Create OAuth2 client for Anthill
echo ""
echo "Creating OAuth2 client: anthill"
kanidm system oauth2 create \
  -H "${KANIDM_URL}" \
  -D admin \
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
  anthill \
  "http://localhost:3000/oauth/callback" || echo "  (may already exist)"

kanidm system oauth2 add-redirect-url \
  -H "${KANIDM_URL}" \
  -D admin \
  anthill \
  "https://app.example.com/oauth/callback" || echo "  (may already exist)"

# Enable PKCE (required for SPAs)
echo ""
echo "Enabling PKCE..."
kanidm system oauth2 enable-pkce \
  -H "${KANIDM_URL}" \
  -D admin \
  anthill || echo "  (may already be enabled)"

# Configure scopes
echo ""
echo "Configuring OAuth2 scopes..."
kanidm system oauth2 update-scope-map \
  -H "${KANIDM_URL}" \
  -D admin \
  anthill \
  anthill_users email openid profile groups || echo "  (may already be configured)"

# Get client secret
echo ""
echo "OAuth2 Client Secret:"
echo "-----------------------------------"
kanidm system oauth2 show-basic-secret \
  -H "${KANIDM_URL}" \
  -D admin \
  anthill || echo "⚠ Could not retrieve client secret"
echo "-----------------------------------"

# Create default tenant groups
echo ""
echo "Creating default tenant groups..."

# Test tenant: Acme Corp
kanidm group create \
  -H "${KANIDM_URL}" \
  -D admin \
  tenant_acme_users || echo "  tenant_acme_users may already exist"

kanidm group set displayname \
  -H "${KANIDM_URL}" \
  -D admin \
  tenant_acme_users \
  "Acme Corp - Users" || true

kanidm group create \
  -H "${KANIDM_URL}" \
  -D admin \
  tenant_acme_admins || echo "  tenant_acme_admins may already exist"

kanidm group set displayname \
  -H "${KANIDM_URL}" \
  -D admin \
  tenant_acme_admins \
  "Acme Corp - Administrators" || true

# Test tenant: Globex Inc
kanidm group create \
  -H "${KANIDM_URL}" \
  -D admin \
  tenant_globex_users || echo "  tenant_globex_users may already exist"

kanidm group set displayname \
  -H "${KANIDM_URL}" \
  -D admin \
  tenant_globex_users \
  "Globex Inc - Users" || true

# Create test users
echo ""
echo "Creating test users..."

# Alice (Acme admin)
kanidm person create \
  -H "${KANIDM_URL}" \
  -D admin \
  alice \
  "Alice Admin" || echo "  alice may already exist"

kanidm person set mail \
  -H "${KANIDM_URL}" \
  -D admin \
  alice \
  alice@acme.example.com || true

kanidm person set password \
  -H "${KANIDM_URL}" \
  -D admin \
  alice \
  "Test123!@#" || true

kanidm group add-members \
  -H "${KANIDM_URL}" \
  -D admin \
  tenant_acme_users \
  alice || echo "  alice already in tenant_acme_users"

kanidm group add-members \
  -H "${KANIDM_URL}" \
  -D admin \
  tenant_acme_admins \
  alice || echo "  alice already in tenant_acme_admins"

# Bob (Acme user)
kanidm person create \
  -H "${KANIDM_URL}" \
  -D admin \
  bob \
  "Bob User" || echo "  bob may already exist"

kanidm person set mail \
  -H "${KANIDM_URL}" \
  -D admin \
  bob \
  bob@acme.example.com || true

kanidm person set password \
  -H "${KANIDM_URL}" \
  -D admin \
  bob \
  "Test123!@#" || true

kanidm group add-members \
  -H "${KANIDM_URL}" \
  -D admin \
  tenant_acme_users \
  bob || echo "  bob already in tenant_acme_users"

# Charlie (Globex user)
kanidm person create \
  -H "${KANIDM_URL}" \
  -D admin \
  charlie \
  "Charlie Globex" || echo "  charlie may already exist"

kanidm person set mail \
  -H "${KANIDM_URL}" \
  -D admin \
  charlie \
  charlie@globex.example.com || true

kanidm person set password \
  -H "${KANIDM_URL}" \
  -D admin \
  charlie \
  "Test123!@#" || true

kanidm group add-members \
  -H "${KANIDM_URL}" \
  -D admin \
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
echo "  - Web UI: http://localhost:8300/ui"
echo "  - API: http://localhost:8300"
echo ""
echo "⚠️  Remember to retrieve client secret:"
echo "  docker exec kanidm_idm kanidm system oauth2 show-basic-secret -H http://localhost:8300 -D admin anthill"
echo ""
