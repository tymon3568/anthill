#!/bin/bash
# Test script for kanidm-init binary

set -e

echo "================================================"
echo "Testing Kanidm Init Binary"
echo "================================================"
echo ""

# Check if Kanidm is running
if ! docker ps | grep -q kanidm_idm; then
    echo "❌ Kanidm container is not running!"
    echo "   Start it with: cd infra/docker_compose && docker compose up -d kanidm"
    exit 1
fi

echo "✅ Kanidm container is running"
echo ""

# Build the binary
echo "📦 Building kanidm-init binary..."
cd infra/kanidm_init
cargo build --release
echo "✅ Build complete"
echo ""

# Set environment variables
export KANIDM_URL="https://localhost:8300"
export KANIDM_ADMIN_USER="admin"
export KANIDM_ADMIN_PASSWORD="${KANIDM_ADMIN_PASSWORD:-NA6LYuMh5zPWT8VTFaWFLT6TD9jdM3BcVquLy031e8RFY8Ps}"
export RUST_LOG="info"

# Run the binary
echo "🚀 Running kanidm-init..."
echo ""
./target/release/kanidm-init

echo ""
echo "================================================"
echo "✅ Test Complete!"
echo "================================================"
