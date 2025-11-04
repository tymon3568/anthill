#!/bin/bash
set -euo pipefail

# Kanidm Initialization Script
# This script uses kanidm CLI tools for reliable OAuth2 setup
# Rust binary is used for complex operations in future iterations

KANIDM_URL="${KANIDM_URL:-https://kanidm:8300}"
ADMIN_USER="${KANIDM_ADMIN_USER:-idm_admin}"
ADMIN_PASS="${KANIDM_ADMIN_PASSWORD:-}"

# Set CA bundle if available
if [ -f /data/chain.pem ]; then
    export CURL_CA_BUNDLE=/data/chain.pem
    export SSL_CERT_FILE=/data/chain.pem
    export REQUESTS_CA_BUNDLE=/data/chain.pem
fi

if [ -z "$ADMIN_PASS" ]; then
    echo "❌ KANIDM_ADMIN_PASSWORD environment variable required"
    exit 1
fi

echo "═══════════════════════════════════════"
echo "Kanidm Initialization for Anthill"
echo "═══════════════════════════════════════"
echo ""
echo "🔧 Configuration:"
echo "  - Kanidm URL: $KANIDM_URL"
echo "  - Admin User: $ADMIN_USER"
echo ""

# Create kanidm config for certificate handling
echo "🔧 Setting up kanidm CLI configuration..."
mkdir -p ~/.config
cat > ~/.config/kanidm << EOF
uri = "$KANIDM_URL"
verify_ca = false
EOF

# Wait for Kanidm to be ready
echo "⏳ Waiting for Kanidm to be ready..."
MAX_RETRIES=30
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -k -s "${KANIDM_URL}/status" > /dev/null 2>&1; then
        echo "✅ Kanidm is ready!"
        break
    fi
    RETRY_COUNT=$((RETRY_COUNT + 1))
    echo "   Attempt $RETRY_COUNT/$MAX_RETRIES..."
    sleep 2
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    echo "❌ Kanidm failed to become ready after $MAX_RETRIES attempts"
    exit 1
fi

echo ""
echo "═══════════════════════════════════════"
echo "Setting up OAuth2 Client"
echo "═══════════════════════════════════════"

# Login as admin
echo "🔐 Logging in as $ADMIN_USER..."
expect << EOF
spawn kanidm login -C /dev/null -D $ADMIN_USER
expect "Enter password:"
send "$ADMIN_PASS\r"
expect eof
EOF

# Create OAuth2 client
echo "🔧 Creating OAuth2 client: anthill"
kanidm system oauth2 create \
    -C /dev/null \
    anthill \
    "Anthill Inventory SaaS" \
    "https://localhost:5173" || echo "⚠️  OAuth2 client may already exist"

# Enable PKCE
echo "🔒 Enabling PKCE..."
kanidm system oauth2 enable-pkce -C /dev/null anthill || true

# Add redirect URLs
echo "🔗 Adding redirect URLs..."
kanidm system oauth2 add-redirect-url \
    -C /dev/null \
    anthill \
    "https://localhost:5173/auth/callback" || true

kanidm system oauth2 add-redirect-url \
    -C /dev/null \
    anthill \
    "http://localhost:5173/auth/callback" || true

echo ""
echo "═══════════════════════════════════════"
echo "Setting up Groups"
echo "═══════════════════════════════════════"

# Create tenant groups
GROUPS=(
    "tenant_acme_users:ACME Corp Users"
    "tenant_acme_admins:ACME Corp Admins"
    "tenant_globex_users:Globex Corp Users"
    "anthill_users:Anthill Platform Users"
)

for group_spec in "${GROUPS[@]}"; do
    group_name="${group_spec%%:*}"
    group_display="${group_spec#*:}"
    echo "👥 Creating group: $group_name"
    kanidm group create -C /dev/null "$group_name" || echo "⚠️  Group may already exist"
done

# Setup scope maps
echo ""
echo "🔐 Setting up scope maps..."

# ACME users get basic profile + email
kanidm system oauth2 update-scope-map \
    -C /dev/null \
    anthill \
    tenant_acme_users \
    openid profile email groups || true

# ACME admins get additional admin scope
kanidm system oauth2 update-scope-map \
    -C /dev/null \
    anthill \
    tenant_acme_admins \
    openid profile email groups admin || true

# Globex users
kanidm system oauth2 update-scope-map \
    -C /dev/null \
    anthill \
    tenant_globex_users \
    openid profile email groups || true

# Anthill platform users
kanidm system oauth2 update-scope-map \
    -C /dev/null \
    anthill \
    anthill_users \
    openid profile email groups || true

echo ""
echo "═══════════════════════════════════════"
echo "Setting up Test Users"
echo "═══════════════════════════════════════"

# Create test users
USERS=(
    "alice:Alice Anderson:alice@acme.example.com:tenant_acme_admins,tenant_acme_users,anthill_users"
    "bob:Bob Builder:bob@acme.example.com:tenant_acme_users,anthill_users"
    "charlie:Charlie Chen:charlie@globex.example.com:tenant_globex_users,anthill_users"
)

for user_spec in "${USERS[@]}"; do
    IFS=':' read -r username displayname email groups_str <<< "$user_spec"

    echo "👤 Creating user: $username ($displayname)"
    kanidm person create \
        -C /dev/null \
        "$username" \
        "$displayname" || echo "⚠️  User may already exist"

    # Set email
    kanidm person update \
        -C /dev/null \
        "$username" \
        mail "$email" || true

    # Set password (for testing)
    echo "🔑 Setting password for $username"
    kanidm person credential create-reset-token \
        -C /dev/null \
        "$username" || true

    # Add to groups
    IFS=',' read -ra user_groups <<< "$groups_str"
    for group in "${user_groups[@]}"; do
        echo "  ➕ Adding $username to $group"
        kanidm group add-members \
            -C /dev/null \
            "$group" \
            "$username" || true
    done
done

echo ""
echo "═══════════════════════════════════════"
echo "✅ Kanidm Initialization Complete!"
echo "═══════════════════════════════════════"
echo ""
echo "📋 Summary:"
echo "  - OAuth2 Client: anthill"
echo "  - Authorization Endpoint: ${KANIDM_URL}/ui/oauth2"
echo "  - Token Endpoint: ${KANIDM_URL}/oauth2/token"
echo "  - JWKS Endpoint: ${KANIDM_URL}/oauth2/openid/anthill/discovery"
echo ""
echo "  - Test Users Created:"
echo "    • alice (ACME Admin)"
echo "    • bob (ACME User)"
echo "    • charlie (Globex User)"
echo ""
echo "🔗 Next Steps:"
echo "  1. Get OAuth2 client secret: kanidm system oauth2 show-basic-secret anthill"
echo "  2. Update SvelteKit .env with client ID and secret"
echo "  3. Run integration tests: cargo test --package user_service_api oauth2"
echo ""
