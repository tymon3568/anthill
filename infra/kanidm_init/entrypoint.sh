#!/bin/bash
set -euo pipefail

# Kanidm Initialization - Hybrid Approach
# Uses Rust binary for API operations (can skip cert verify for dev)

KANIDM_URL="${KANIDM_URL:-https://localhost:8300}"
ADMIN_USER="${KANIDM_ADMIN_USER:-idm_admin}"
ADMIN_PASS="${KANIDM_ADMIN_PASSWORD:-}"

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

# Wait for Kanidm
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
    echo "❌ Kanidm failed to become ready"
    exit 1
fi

# Run Rust binary for initialization
echo ""
echo "🦀 Running Rust binary for OAuth2 setup..."
export KANIDM_SKIP_TLS_VERIFY=true
if /usr/local/bin/kanidm-init; then
    echo "✅ Rust binary setup completed successfully"
else
    echo "⚠️  Rust binary failed, falling back to kanidm CLI..."
    # Copy setup script and run it
    cp /usr/local/bin/setup-kanidm.sh /tmp/setup-kanidm.sh
    chmod +x /tmp/setup-kanidm.sh
    /tmp/setup-kanidm.sh
fi

echo ""
echo "═══════════════════════════════════════"
echo "✅ Kanidm Initialization Complete!"
echo "═══════════════════════════════════════"
