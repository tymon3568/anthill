#!/usr/bin/env bash
# Coverage Report Generator using Tarpaulin + Codecov
# Usage: ./scripts/coverage.sh [options]
#
# Options:
#   --upload    Upload coverage to Codecov (requires CODECOV_TOKEN in CI)
#   --html      Generate HTML report (default: true)
#   --open      Open HTML report in browser after generation
#   --package   Run coverage for specific package (e.g., user_service_core)
#   --help      Show this help message

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default options
UPLOAD=false
GENERATE_HTML=true
OPEN_HTML=false
PACKAGE=""
CODECOV_TOKEN="${CODECOV_TOKEN:-}"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --upload)
            UPLOAD=true
            shift
            ;;
        --html)
            GENERATE_HTML=true
            shift
            ;;
        --open)
            OPEN_HTML=true
            shift
            ;;
        --package)
            PACKAGE="$2"
            shift 2
            ;;
        --help)
            echo "Coverage Report Generator"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --upload     Upload coverage to Codecov"
            echo "  --html       Generate HTML report (default)"
            echo "  --open       Open HTML report in browser"
            echo "  --package    Run for specific package (e.g., user_service_core)"
            echo "  --help       Show this help"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}Anthill Coverage Report Generator${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""

# Check if tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${YELLOW}cargo-tarpaulin not found. Installing...${NC}"
    cargo install cargo-tarpaulin
fi

# Check tarpaulin version
TARPAULIN_VERSION=$(cargo tarpaulin --version | head -n1)
echo -e "${GREEN}Using: $TARPAULIN_VERSION${NC}"
echo ""

# Create coverage directory
mkdir -p target/coverage

# Build tarpaulin command
TARPAULIN_CMD="cargo tarpaulin --config tarpaulin.toml"

if [[ -n "$PACKAGE" ]]; then
    echo -e "${BLUE}Running coverage for package: $PACKAGE${NC}"
    TARPAULIN_CMD="$TARPAULIN_CMD --package $PACKAGE"
else
    echo -e "${BLUE}Running coverage for entire workspace${NC}"
    TARPAULIN_CMD="$TARPAULIN_CMD --workspace"
fi

# Add output formats
OUTPUT_FORMATS="Lcov"
if [[ "$GENERATE_HTML" == true ]]; then
    OUTPUT_FORMATS="$OUTPUT_FORMATS,Html"
fi

TARPAULIN_CMD="$TARPAULIN_CMD --out $OUTPUT_FORMATS"

echo -e "${YELLOW}Executing: $TARPAULIN_CMD${NC}"
echo ""

# Run tarpaulin
eval "$TARPAULIN_CMD"

echo ""
echo -e "${GREEN}✓ Coverage generation complete!${NC}"
echo ""

# Display coverage summary
if [[ -f "target/coverage/lcov.info" ]]; then
    TOTAL_LINES=$(grep -c "^DA:" target/coverage/lcov.info || echo "0")
    COVERED_LINES=$(grep "^DA:" target/coverage/lcov.info | grep -v ",0$" | wc -l || echo "0")

    if [[ "$TOTAL_LINES" -gt 0 ]]; then
        COVERAGE_PCT=$(awk "BEGIN {printf \"%.2f\", ($COVERED_LINES / $TOTAL_LINES) * 100}")
        echo -e "${BLUE}Coverage Summary:${NC}"
        echo -e "  Total lines:   $TOTAL_LINES"
        echo -e "  Covered lines: $COVERED_LINES"
        echo -e "  Coverage:      ${GREEN}${COVERAGE_PCT}%${NC}"
        echo ""
    fi
fi

# Upload to Codecov if requested
if [[ "$UPLOAD" == true ]]; then
    echo -e "${BLUE}Uploading coverage to Codecov...${NC}"

    if [[ -z "$CODECOV_TOKEN" ]]; then
        echo -e "${YELLOW}Warning: CODECOV_TOKEN not set. Upload may fail for private repos.${NC}"
    fi

    # Use bash uploader (no installation needed, more secure)
    echo -e "${BLUE}Uploading to Codecov...${NC}"

    # Upload to codecov using bash uploader
    if [[ -n "$CODECOV_TOKEN" ]]; then
        bash <(curl -s https://codecov.io/bash) -t "$CODECOV_TOKEN" -f target/coverage/lcov.info -F unittests
    else
        bash <(curl -s https://codecov.io/bash) -f target/coverage/lcov.info -F unittests
    fi

    echo -e "${GREEN}✓ Coverage uploaded to Codecov${NC}"
    echo -e "${BLUE}View at: https://codecov.io/gh/tymon3568/anthill${NC}"
    echo ""
fi

# Open HTML report
if [[ "$OPEN_HTML" == true ]] && [[ -f "target/coverage/index.html" ]]; then
    echo -e "${BLUE}Opening HTML report...${NC}"

    if command -v xdg-open &> /dev/null; then
        xdg-open target/coverage/index.html
    elif command -v open &> /dev/null; then
        open target/coverage/index.html
    else
        echo -e "${YELLOW}Could not open browser automatically${NC}"
        echo -e "Open manually: file://$(pwd)/target/coverage/index.html"
    fi
fi

echo ""
echo -e "${GREEN}======================================${NC}"
echo -e "${GREEN}Done!${NC}"
echo -e "${GREEN}======================================${NC}"

if [[ "$GENERATE_HTML" == true ]]; then
    echo -e "${BLUE}HTML Report: file://$(pwd)/target/coverage/index.html${NC}"
fi
echo -e "${BLUE}LCOV Report: $(pwd)/target/coverage/lcov.info${NC}"
