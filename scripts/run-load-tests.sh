#!/usr/bin/env bash
# Load Testing Runner Script
# Runs k6 load tests against the user service
# Usage: ./scripts/run-load-tests.sh [test-name] [options]

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000}"
TENANT_ID="${TENANT_ID:-00000000-0000-0000-0000-000000000001}"
K6_SCRIPTS_DIR="tests/load/k6-scripts"
OUTPUT_DIR="tests/load/results"

# Default test
TEST_NAME="${1:-auth}"
shift || true

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║       Load Testing Runner              ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Check if k6 is installed
if ! command -v k6 &> /dev/null; then
    echo -e "${RED}k6 is not installed.${NC}"
    echo ""
    echo "Install k6:"
    echo "  - macOS: brew install k6"
    echo "  - Linux: sudo snap install k6"
    echo "  - Docker: docker run --rm -i grafana/k6 run -"
    echo ""
    echo "More info: https://k6.io/docs/getting-started/installation/"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Map test name to script
case "$TEST_NAME" in
    auth|authentication)
        SCRIPT="auth-load-test.js"
        echo -e "${BLUE}Running Authentication Load Test${NC}"
        ;;
    api|endpoints)
        SCRIPT="api-load-test.js"
        echo -e "${BLUE}Running API Endpoint Load Test${NC}"
        ;;
    sessions|concurrent)
        SCRIPT="concurrent-sessions-test.js"
        echo -e "${BLUE}Running Concurrent Sessions Test${NC}"
        ;;
    all)
        echo -e "${BLUE}Running all load tests sequentially${NC}"
        for script in auth-load-test.js api-load-test.js concurrent-sessions-test.js; do
            echo ""
            echo -e "${YELLOW}Running: $script${NC}"
            k6 run \
                --env BASE_URL="$BASE_URL" \
                --env TENANT_ID="$TENANT_ID" \
                --out json="$OUTPUT_DIR/${script%.js}-$(date +%Y%m%d-%H%M%S).json" \
                "$K6_SCRIPTS_DIR/$script" "$@"
        done
        exit 0
        ;;
    *)
        echo -e "${RED}Unknown test: $TEST_NAME${NC}"
        echo ""
        echo "Available tests:"
        echo "  auth       - Authentication flow load test"
        echo "  api        - API endpoint load test"
        echo "  sessions   - Concurrent sessions test"
        echo "  all        - Run all tests"
        exit 1
        ;;
esac

echo -e "${YELLOW}Target: $BASE_URL${NC}"
echo -e "${YELLOW}Tenant: $TENANT_ID${NC}"
echo ""

# Run the selected test
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
OUTPUT_FILE="$OUTPUT_DIR/${SCRIPT%.js}-$TIMESTAMP.json"
SUMMARY_FILE="$OUTPUT_DIR/${SCRIPT%.js}-$TIMESTAMP-summary.html"

echo -e "${BLUE}Starting load test...${NC}"
echo ""

k6 run \
    --env BASE_URL="$BASE_URL" \
    --env TENANT_ID="$TENANT_ID" \
    --out json="$OUTPUT_FILE" \
    "$K6_SCRIPTS_DIR/$SCRIPT" "$@"

EXIT_CODE=$?

echo ""
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}║   ✓ Load test completed successfully  ║${NC}"
else
    echo -e "${RED}║   ✗ Load test failed                  ║${NC}"
fi

echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

echo -e "${BLUE}Results saved to:${NC}"
echo "  JSON: $OUTPUT_FILE"
echo ""

# Show next steps
echo -e "${BLUE}Next steps:${NC}"
echo "  • View results: cat $OUTPUT_FILE | jq '.'"
echo "  • Generate HTML report: k6-reporter --input $OUTPUT_FILE --output $SUMMARY_FILE"
echo "  • Compare with baseline: ./scripts/compare-load-results.sh"
echo ""

exit $EXIT_CODE
