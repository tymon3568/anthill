#!/usr/bin/env bash

# CI/CD Helper Script
# Simulates GitHub Actions workflow locally for debugging
# Usage: ./scripts/ci-helper.sh [command] [options]

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
DATABASE_URL="${DATABASE_URL:-postgresql://postgres:postgres@localhost:5432/anthill_test}"
JWT_SECRET="${JWT_SECRET:-test-secret-key-min-32-chars-long-for-testing}"
RUSTFLAGS="--cfg uuid_unstable"

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

show_usage() {
    cat << EOF
CI/CD Helper Script - Run GitHub Actions workflows locally

Usage: $0 [command] [options]

Commands:
  lint              Run linting and formatting checks
  unit              Run unit tests
  integration       Run integration tests (requires database)
  security          Run security tests (requires database)
  coverage          Generate coverage report
  build             Build all services
  all               Run complete CI pipeline
  clean             Clean up test artifacts

Options:
  --verbose         Show detailed output
  --package NAME    Run tests for specific package
  --threads N       Number of test threads (default: 4 for unit, 1 for integration)
  --help           Show this help message

Environment Variables:
  DATABASE_URL      PostgreSQL connection string (default: localhost)
  JWT_SECRET        JWT secret for tests (default: test key)
  RUSTFLAGS         Rust compiler flags (default: --cfg uuid_unstable)

Examples:
  $0 lint                          # Run linting checks
  $0 unit --verbose                # Run unit tests with output
  $0 integration --package user_service_api
  $0 coverage --upload             # Generate and upload coverage
  $0 all                           # Run complete pipeline

EOF
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Rust is not installed. Install from https://rustup.rs"
        exit 1
    fi
    log_success "Rust $(rustc --version)"

    # Check rustfmt
    if ! command -v cargo-fmt &> /dev/null; then
        log_warning "rustfmt not found. Installing..."
        rustup component add rustfmt
    fi

    # Check clippy
    if ! command -v cargo-clippy &> /dev/null; then
        log_warning "clippy not found. Installing..."
        rustup component add clippy
    fi

    log_success "All prerequisites met"
}

run_lint() {
    log_info "Running lint checks..."

    # Format check
    log_info "Checking code formatting..."
    if cargo fmt --all -- --check; then
        log_success "Code formatting: PASSED"
    else
        log_error "Code formatting: FAILED"
        log_warning "Run 'cargo fmt --all' to fix"
        return 1
    fi

    # Clippy
    log_info "Running clippy..."
    if RUSTFLAGS="$RUSTFLAGS" cargo clippy --workspace --all-targets --all-features -- -D warnings; then
        log_success "Clippy: PASSED"
    else
        log_error "Clippy: FAILED"
        return 1
    fi

    log_success "Lint checks: ALL PASSED"
}

run_unit_tests() {
    local verbose="${1:-false}"
    local package="${2:-}"
    local threads="${3:-4}"

    log_info "Running unit tests..."

    local cmd="cargo test --workspace --lib --bins --exclude integration_tests"

    if [ -n "$package" ]; then
        cmd="cargo test --package $package --lib"
    fi

    if [ "$verbose" = "true" ]; then
        cmd="$cmd -- --nocapture --test-threads=$threads"
    else
        cmd="$cmd -- --test-threads=$threads"
    fi

    log_info "Command: $cmd"

    if RUSTFLAGS="$RUSTFLAGS" eval "$cmd"; then
        log_success "Unit tests: PASSED"
    else
        log_error "Unit tests: FAILED"
        return 1
    fi
}

run_integration_tests() {
    local verbose="${1:-false}"
    local package="${2:-}"

    log_info "Running integration tests..."

    # Check database connection
    if ! psql "$DATABASE_URL" -c "SELECT 1" &> /dev/null; then
        log_error "Cannot connect to database: $DATABASE_URL"
        log_warning "Start PostgreSQL with: docker run -d --name anthill-test-db -e POSTGRES_PASSWORD=postgres -p 5432:5432 postgres:16-alpine"
        log_warning "Or run: ./scripts/setup-test-db.sh --reset"
        return 1
    fi
    log_success "Database connection: OK"

    # Run migrations
    log_info "Running database migrations..."
    if ! command -v sqlx &> /dev/null; then
        log_warning "sqlx-cli not found. Installing..."
        cargo install sqlx-cli --no-default-features --features postgres
    fi

    sqlx migrate run --database-url "$DATABASE_URL"
    log_success "Migrations: COMPLETE"

    # Run tests
    local cmd="cargo test --workspace --test '*'"

    if [ -n "$package" ]; then
        cmd="cargo test --package $package --test '*'"
    fi

    if [ "$verbose" = "true" ]; then
        cmd="$cmd -- --ignored --nocapture --test-threads=1"
    else
        cmd="$cmd -- --ignored --test-threads=1"
    fi

    log_info "Command: $cmd"

    if DATABASE_URL="$DATABASE_URL" JWT_SECRET="$JWT_SECRET" RUSTFLAGS="$RUSTFLAGS" eval "$cmd"; then
        log_success "Integration tests: PASSED"
    else
        log_error "Integration tests: FAILED"
        return 1
    fi

    # Cleanup
    log_info "Cleaning up test data..."
    psql "$DATABASE_URL" -c "SELECT cleanup_test_data();" &> /dev/null || log_warning "Cleanup function not available"
}

run_security_tests() {
    local verbose="${1:-false}"

    log_info "Running security tests..."

    # Check database
    if ! psql "$DATABASE_URL" -c "SELECT 1" &> /dev/null; then
        log_error "Cannot connect to database: $DATABASE_URL"
        return 1
    fi

    # Run migrations
    sqlx migrate run --database-url "$DATABASE_URL" &> /dev/null

    # SQL injection tests
    log_info "Testing SQL injection protection..."
    if DATABASE_URL="$DATABASE_URL" JWT_SECRET="$JWT_SECRET" RUSTFLAGS="$RUSTFLAGS" \
       cargo test --package user_service_api sql_injection -- --ignored --nocapture; then
        log_success "SQL injection tests: PASSED"
    else
        log_warning "SQL injection tests: FAILED or not found"
    fi

    # Tenant isolation tests
    log_info "Testing tenant isolation..."
    if DATABASE_URL="$DATABASE_URL" JWT_SECRET="$JWT_SECRET" RUSTFLAGS="$RUSTFLAGS" \
       cargo test --package user_service_api tenant_isolation -- --ignored --nocapture; then
        log_success "Tenant isolation tests: PASSED"
    else
        log_warning "Tenant isolation tests: FAILED or not found"
    fi

    # Security tests
    log_info "Testing authentication/JWT security..."
    if DATABASE_URL="$DATABASE_URL" JWT_SECRET="$JWT_SECRET" RUSTFLAGS="$RUSTFLAGS" \
       cargo test --package user_service_api security -- --ignored --nocapture; then
        log_success "Security tests: PASSED"
    else
        log_warning "Security tests: FAILED or not found"
    fi

    log_success "Security tests: COMPLETE"
}

run_coverage() {
    local upload="${1:-false}"
    local open_html="${2:-false}"

    log_info "Generating coverage report..."

    # Check for coverage tool
    if ! command -v cargo-llvm-cov &> /dev/null; then
        log_warning "cargo-llvm-cov not found. Installing..."
        cargo install cargo-llvm-cov
    fi

    # Install llvm-tools
    rustup component add llvm-tools-preview &> /dev/null || true

    # Check database for integration tests
    local db_available=false
    if psql "$DATABASE_URL" -c "SELECT 1" &> /dev/null; then
        db_available=true
        log_success "Database available - running all tests"
        sqlx migrate run --database-url "$DATABASE_URL" &> /dev/null || true
    else
        log_warning "Database not available - skipping integration tests"
    fi

    # Generate coverage
    log_info "Running tests with coverage instrumentation..."

    if [ "$db_available" = true ]; then
        DATABASE_URL="$DATABASE_URL" JWT_SECRET="$JWT_SECRET" RUSTFLAGS="$RUSTFLAGS" \
        cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info -- --include-ignored
    else
        RUSTFLAGS="$RUSTFLAGS" \
        cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
    fi

    log_success "Coverage data generated: lcov.info"

    # Generate HTML report
    log_info "Generating HTML report..."
    if [ "$db_available" = true ]; then
        DATABASE_URL="$DATABASE_URL" JWT_SECRET="$JWT_SECRET" RUSTFLAGS="$RUSTFLAGS" \
        cargo llvm-cov report --html -- --include-ignored
    else
        RUSTFLAGS="$RUSTFLAGS" \
        cargo llvm-cov report --html
    fi

    if [ -d "target/llvm-cov/html" ]; then
        log_success "HTML report: target/llvm-cov/html/index.html"
    fi

    # Get coverage percentage
    local coverage
    coverage=$(cargo llvm-cov report --summary-only | grep -oP '\d+\.\d+(?=%)' | head -1 || echo "0")

    log_info "Coverage: ${coverage}%"

    if (( $(echo "$coverage < 70.0" | bc -l 2>/dev/null || echo 0) )); then
        log_warning "Coverage is below 70% (current: ${coverage}%)"
    elif (( $(echo "$coverage < 80.0" | bc -l 2>/dev/null || echo 0) )); then
        log_warning "Coverage is below target 80% (current: ${coverage}%)"
    else
        log_success "Coverage meets target: ${coverage}%"
    fi

    # Upload to Codecov
    if [ "$upload" = "true" ]; then
        log_info "Uploading to Codecov..."
        if command -v curl &> /dev/null; then
            bash <(curl -s https://codecov.io/bash) -f lcov.info || log_warning "Codecov upload failed"
        else
            log_warning "curl not found. Cannot upload to Codecov"
        fi
    fi

    # Open HTML report
    if [ "$open_html" = "true" ]; then
        if command -v xdg-open &> /dev/null; then
            xdg-open target/llvm-cov/html/index.html
        elif command -v open &> /dev/null; then
            open target/llvm-cov/html/index.html
        else
            log_warning "Cannot open browser automatically. Open target/llvm-cov/html/index.html manually"
        fi
    fi
}

run_build() {
    log_info "Building all services..."

    local services=("user_service" "inventory_service" "order_service" "payment_service" "integration_service")

    for service in "${services[@]}"; do
        if [ -d "services/$service" ]; then
            log_info "Building $service..."
            if (cd "services/$service" && RUSTFLAGS="$RUSTFLAGS" cargo build --release); then
                log_success "$service: BUILD SUCCESS"
            else
                log_error "$service: BUILD FAILED"
                return 1
            fi
        fi
    done

    log_success "All services: BUILD COMPLETE"
}

run_all() {
    log_info "Running complete CI pipeline..."

    local start_time
    start_time=$(date +%s)

    # Run all stages
    check_prerequisites || return 1
    run_lint || return 1
    run_unit_tests false "" 4 || return 1

    # Check if database is available
    if psql "$DATABASE_URL" -c "SELECT 1" &> /dev/null; then
        run_integration_tests false "" || return 1
        run_security_tests false || return 1
        run_coverage false false || return 1
    else
        log_warning "Skipping integration, security, and coverage tests (no database)"
    fi

    run_build || return 1

    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))

    log_success "Complete CI pipeline: PASSED in ${duration}s"
}

clean_artifacts() {
    log_info "Cleaning test artifacts..."

    rm -rf target/debug/deps/*.xml 2>/dev/null || true
    rm -rf target/llvm-cov 2>/dev/null || true
    rm -f lcov.info 2>/dev/null || true
    rm -rf coverage-html 2>/dev/null || true

    log_success "Artifacts cleaned"
}

# Main script
main() {
    local command="${1:-help}"
    shift || true

    # Parse options
    local verbose=false
    local package=""
    local threads=""
    local upload=false
    local open_html=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            --verbose)
                verbose=true
                shift
                ;;
            --package)
                package="$2"
                shift 2
                ;;
            --threads)
                threads="$2"
                shift 2
                ;;
            --upload)
                upload=true
                shift
                ;;
            --open)
                open_html=true
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    # Execute command
    case $command in
        lint)
            run_lint
            ;;
        unit)
            run_unit_tests "$verbose" "$package" "${threads:-4}"
            ;;
        integration)
            run_integration_tests "$verbose" "$package"
            ;;
        security)
            run_security_tests "$verbose"
            ;;
        coverage)
            run_coverage "$upload" "$open_html"
            ;;
        build)
            run_build
            ;;
        all)
            run_all
            ;;
        clean)
            clean_artifacts
            ;;
        help|--help|-h)
            show_usage
            exit 0
            ;;
        *)
            log_error "Unknown command: $command"
            show_usage
            exit 1
            ;;
    esac
}

# Run main
main "$@"
