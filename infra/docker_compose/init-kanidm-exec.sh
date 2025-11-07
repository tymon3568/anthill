#!/bin/sh
# Kanidm Initialization Script (Using Docker Exec Approach)
# This script runs kanidm CLI from tools container with proper CA certificate

set -e

echo "========================================="
echo "Kanidm Initialization for Anthill"
echo "========================================="

KANIDM_URL="${KANIDM_URL:-https://localhost:8300}"
ADMIN_PASSWORD="${KANIDM_ADMIN_PASSWORD}"

if [ -z "$ADMIN_PASSWORD" ]; then
  echo "❌ Error: KANIDM_ADMIN_PASSWORD environment variable is required"
  exit 1
fi

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

# Helper function to run kanidm commands
kanidm_cli() {
  docker run --rm \
    --network container:kanidm_idm \
    -v kanidm_data:/data \
    -e KANIDM_CA_PATH=/data/chain.pem \
    kanidm/tools:1.4.3 \
    kanidm "$@" \
    -H "${KANIDM_URL}" \
    -D admin \
    --skip-hostname-verification
}

# Login as admin
echo ""
echo "Logging in as admin..."
echo "${ADMIN_PASSWORD}" | kanidm_cli login || {
  echo "⚠ Admin login failed - you may need to recover the admin account first:"
  echo "  See Kanidm documentation for recover-account procedure"
  exit 1
}

# Create OAuth2 client for Anthill
echo ""
echo "Creating OAuth2 client: anthill..."
kanidm_cli system oauth2 create \
  anthill \
  "Anthill Inventory Management" \
  "http://localhost:5173/oauth/callback" || {
  echo "  ⚠ OAuth2 client 'anthill' may already exist"
}

# Add additional redirect URLs
echo ""
echo "Configuring redirect URLs..."
kanidm_cli system oauth2 add-redirect-url \
  anthill \
  "http://localhost:8000/oauth/callback" || echo "  (may already exist)"

kanidm_cli system oauth2 add-redirect-url \
  anthill \
  "https://app.example.com/oauth/callback" || echo "  (may already exist)"

# Enable PKCE (required for SPAs)
echo ""
echo "Enabling PKCE..."
kanidm_cli system oauth2 enable-pkce anthill || echo "  (may already be enabled)"

# Configure scopes
echo ""
echo "Configuring OAuth2 scopes..."
kanidm_cli system oauth2 update-scope-map \
  anthill \
  anthill_users \
  email openid profile groups || echo "  (may already be configured)"

# Get client secret
echo ""
echo "OAuth2 Client Secret:"
echo "-----------------------------------"
kanidm_cli system oauth2 show-basic-secret anthill || echo "⚠ Could not retrieve client secret"
echo "-----------------------------------"

# Create default tenant groups
echo ""
echo "Creating default tenant groups..."

kanidm_cli group create tenant_acme_users || echo "  tenant_acme_users may already exist"
kanidm_cli group create tenant_acme_admins || echo "  tenant_acme_admins may already exist"
kanidm_cli group create tenant_globex_users || echo "  tenant_globex_users may already exist"

# Create test users
echo ""
echo "Creating test users..."

# Alice (Acme admin)
kanidm_cli person create alice "Alice Admin" || echo "  alice may already exist"
kanidm_cli person update --mail alice@acme.example.com alice || true
kanidm_cli group add-members tenant_acme_users alice || echo "  alice already in tenant_acme_users"
kanidm_cli group add-members tenant_acme_admins alice || echo "  alice already in tenant_acme_admins"

# Bob (Acme user)
kanidm_cli person create bob "Bob User" || echo "  bob may already exist"
kanidm_cli person update --mail bob@acme.example.com bob || true
kanidm_cli group add-members tenant_acme_users bob || echo "  bob already in tenant_acme_users"

# Charlie (Globex user)
kanidm_cli person create charlie "Charlie Globex" || echo "  charlie may already exist"
kanidm_cli person update --mail charlie@globex.example.com charlie || true
kanidm_cli group add-members tenant_globex_users charlie || echo "  charlie already in tenant_globex_users"

echo ""
echo "========================================="
echo "✓ Kanidm initialization complete!"
echo "========================================="
echo ""
echo "📋 Summary:"
echo "  - OAuth2 Client: anthill"
echo "  - Redirect URLs: http://localhost:5173/oauth/callback"
echo "                   http://localhost:8000/oauth/callback"
echo "  - PKCE: Enabled"
echo "  - Groups Created:"
echo "    * tenant_acme_users (alice, bob)"
echo "    * tenant_acme_admins (alice)"
echo "    * tenant_globex_users (charlie)"
echo "  - Test Users:"
echo "    * alice@acme.example.com (admin)"
echo "    * bob@acme.example.com (user)"
echo "    * charlie@globex.example.com (user)"
echo ""
echo "⚠️  IMPORTANT: User passwords must be set via credential update tokens"
echo "  Create reset tokens for each user:"
echo '  docker run --rm --network container:kanidm_idm kanidm/tools:1.4.3 \'
echo '    kanidm person credential create-reset-token -H https://localhost:8300 -D admin --skip-hostname-verification alice'
echo ""
echo "🔗 Access Kanidm:"
echo "  - Web UI: https://localhost:8300/ui"
echo "  - API: https://localhost:8300"
echo ""
