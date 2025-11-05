#!/usr/bin/env bash
# Monitoring Health Check Script
# Checks the health of all monitoring components
# Usage: ./scripts/monitoring-health-check.sh [options]

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default options
VERBOSE=false
JSON_OUTPUT=false
TIMEOUT=30

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --json)
            JSON_OUTPUT=true
            shift
            ;;
        --timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --help|-h)
            echo "Monitoring Health Check"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --verbose, -v    Show detailed output"
            echo "  --json           Output in JSON format"
            echo "  --timeout SEC    Timeout for health checks (default: 30)"
            echo "  --help, -h       Show this help"
            echo ""
            echo "Environment Variables:"
            echo "  PROMETHEUS_URL   Prometheus URL (default: http://localhost:9090)"
            echo "  GRAFANA_URL      Grafana URL (default: http://localhost:3000)"
            echo "  LOKI_URL         Loki URL (default: http://localhost:3100)"
            echo "  ALERTMANAGER_URL AlertManager URL (default: http://localhost:9093)"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Configuration
PROMETHEUS_URL="${PROMETHEUS_URL:-http://localhost:9090}"
GRAFANA_URL="${GRAFANA_URL:-http://localhost:3000}"
LOKI_URL="${LOKI_URL:-http://localhost:3100}"
ALERTMANAGER_URL="${ALERTMANAGER_URL:-http://localhost:9093}"

# Health check results
declare -A RESULTS
declare -A DETAILS

# Function to check HTTP endpoint
check_http() {
    local url=$1
    local name=$2
    local expected_code=${3:-200}

    if [[ "$VERBOSE" == true ]]; then
        echo -e "${BLUE}Checking $name: $url${NC}"
    fi

    local response
    if ! response=$(curl -s -w "%{http_code}" -o /dev/null --max-time "$TIMEOUT" "$url" 2>/dev/null); then
        RESULTS[$name]="DOWN"
        DETAILS[$name]="Connection failed"
        return 1
    fi

    if [[ "$response" == "$expected_code" ]]; then
        RESULTS[$name]="UP"
        DETAILS[$name]="HTTP $response"
        return 0
    else
        RESULTS[$name]="DOWN"
        DETAILS[$name]="HTTP $response (expected $expected_code)"
        return 1
    fi
}

# Function to check Prometheus
check_prometheus() {
    if ! check_http "$PROMETHEUS_URL/-/ready" "Prometheus" 200; then
        return 1
    fi

    # Check if Prometheus can query itself
    if ! check_http "$PROMETHEUS_URL/api/v1/query?query=up" "Prometheus_Query" 200; then
        RESULTS["Prometheus"]="DEGRADED"
        DETAILS["Prometheus"]="Ready but query failed"
        return 1
    fi

    return 0
}

# Function to check Grafana
check_grafana() {
    if ! check_http "$GRAFANA_URL/api/health" "Grafana" 200; then
        return 1
    fi

    # Check if Grafana can access datasources
    if ! check_http "$GRAFANA_URL/api/datasources" "Grafana_Datasources" 200; then
        RESULTS["Grafana"]="DEGRADED"
        DETAILS["Grafana"]="Health OK but datasources inaccessible"
        return 1
    fi

    return 0
}

# Function to check Loki
check_loki() {
    check_http "$LOKI_URL/ready" "Loki" 200
}

# Function to check AlertManager
check_alertmanager() {
    check_http "$ALERTMANAGER_URL/-/ready" "AlertManager" 200
}

# Function to check Docker containers
check_containers() {
    local containers=("prometheus" "grafana" "loki" "alertmanager")
    local all_up=true

    for container in "${containers[@]}"; do
        if docker ps --format "table {{.Names}}" | grep -q "^${container}$"; then
            RESULTS["Container_${container}"]="UP"
            DETAILS["Container_${container}"]="Running"
        else
            RESULTS["Container_${container}"]="DOWN"
            DETAILS["Container_${container}"]="Not running"
            all_up=false
        fi
    done

    if [ "$all_up" == true ]; then
        return 0
    else
        return 1
    fi
}

# Function to check metrics collection
check_metrics() {
    # Check if Prometheus is scraping targets
    local targets_url="$PROMETHEUS_URL/api/v1/targets"
    local response

    if ! response=$(curl -s --max-time "$TIMEOUT" "$targets_url" 2>/dev/null); then
        RESULTS["Metrics_Collection"]="DOWN"
        DETAILS["Metrics_Collection"]="Cannot access targets API"
        return 1
    fi

    # Parse JSON response to check for healthy targets
    local healthy_targets=$(echo "$response" | grep -o '"health":"up"' | wc -l)
    local total_targets=$(echo "$response" | grep -o '"health":' | wc -l)

    if [[ $total_targets -eq 0 ]]; then
        RESULTS["Metrics_Collection"]="DOWN"
        DETAILS["Metrics_Collection"]="No targets configured"
        return 1
    fi

    if [[ $healthy_targets -eq $total_targets ]]; then
        RESULTS["Metrics_Collection"]="UP"
        DETAILS["Metrics_Collection"]="$healthy_targets/$total_targets targets healthy"
        return 0
    else
        RESULTS["Metrics_Collection"]="DEGRADED"
        DETAILS["Metrics_Collection"]="$healthy_targets/$total_targets targets healthy"
        return 1
    fi
}

# Function to check alerting rules
check_alerting() {
    local rules_url="$PROMETHEUS_URL/api/v1/rules"
    local response

    if ! response=$(curl -s --max-time "$TIMEOUT" "$rules_url" 2>/dev/null); then
        RESULTS["Alerting_Rules"]="DOWN"
        DETAILS["Alerting_Rules"]="Cannot access rules API"
        return 1
    fi

    local rule_count=$(echo "$response" | grep -o '"name":' | wc -l)

    if [[ $rule_count -gt 0 ]]; then
        RESULTS["Alerting_Rules"]="UP"
        DETAILS["Alerting_Rules"]="$rule_count rules loaded"
        return 0
    else
        RESULTS["Alerting_Rules"]="WARNING"
        DETAILS["Alerting_Rules"]="No alerting rules configured"
        return 0
    fi
}

# Function to check logs ingestion
check_logs() {
    # Check if Loki can query logs (simple test)
    local query_url="$LOKI_URL/loki/api/v1/query?query={job=\".*\"}&limit=1"
    local response

    if ! response=$(curl -s --max-time "$TIMEOUT" "$query_url" 2>/dev/null); then
        RESULTS["Log_Ingestion"]="DOWN"
        DETAILS["Log_Ingestion"]="Cannot query logs"
        return 1
    fi

    # Check if there are any log entries
    local log_count=$(echo "$response" | grep -o '"values":\[' | wc -l)

    if [[ $log_count -gt 0 ]]; then
        RESULTS["Log_Ingestion"]="UP"
        DETAILS["Log_Ingestion"]="Logs available"
        return 0
    else
        RESULTS["Log_Ingestion"]="WARNING"
        DETAILS["Log_Ingestion"]="No logs ingested yet"
        return 0
    fi
}

# Main health check execution
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Monitoring Health Check             ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Run all checks
echo -e "${BLUE}Running health checks...${NC}"
echo ""

# Core services
check_prometheus
check_grafana
check_loki
check_alertmanager

# Infrastructure
check_containers

# Functionality
check_metrics
check_alerting
check_logs

# Calculate overall status
OVERALL_STATUS="UP"
FAILED_COUNT=0
DEGRADED_COUNT=0

for service in "${!RESULTS[@]}"; do
    case "${RESULTS[$service]}" in
        "DOWN")
            OVERALL_STATUS="DOWN"
            ((FAILED_COUNT++))
            ;;
        "DEGRADED")
            if [[ "$OVERALL_STATUS" != "DOWN" ]]; then
                OVERALL_STATUS="DEGRADED"
            fi
            ((DEGRADED_COUNT++))
            ;;
    esac
done

# Output results
if [[ "$JSON_OUTPUT" == true ]]; then
    # JSON output
    echo "{"
    echo "  \"timestamp\": \"$(date -Iseconds)\","
    echo "  \"overall_status\": \"$OVERALL_STATUS\","
    echo "  \"services\": {"

    first=true
    for service in "${!RESULTS[@]}"; do
        if [[ "$first" == false ]]; then
            echo ","
        fi
        echo -n "    \"$service\": {"
        echo -n "\"status\": \"${RESULTS[$service]}\", "
        echo "\"details\": \"${DETAILS[$service]}\""
        echo -n "}"
        first=false
    done
    echo ""
    echo "  },"
    echo "  \"summary\": {"
    echo "    \"total_services\": ${#RESULTS[@]},"
    echo "    \"failed\": $FAILED_COUNT,"
    echo "    \"degraded\": $DEGRADED_COUNT"
    echo "  }"
    echo "}"
else
    # Human-readable output
    echo -e "${BLUE}Service Status:${NC}"
    echo -e "${BLUE}═══════════════${NC}"

    for service in "${!RESULTS[@]}"; do
        case "${RESULTS[$service]}" in
            "UP")
                echo -e "${GREEN}✓${NC} $service: ${RESULTS[$service]}"
                ;;
            "DEGRADED")
                echo -e "${YELLOW}⚠${NC} $service: ${RESULTS[$service]}"
                ;;
            "DOWN")
                echo -e "${RED}✗${NC} $service: ${RESULTS[$service]}"
                ;;
            "WARNING")
                echo -e "${YELLOW}⚠${NC} $service: ${RESULTS[$service]}"
                ;;
            *)
                echo -e "${BLUE}?$NC} $service: ${RESULTS[$service]}"
                ;;
        esac

        if [[ "$VERBOSE" == true ]]; then
            echo -e "  ${BLUE}Details:${NC} ${DETAILS[$service]}"
        fi
    done

    echo ""
    echo -e "${BLUE}Summary:${NC}"
    echo -e "${BLUE}═════════${NC}"

    case "$OVERALL_STATUS" in
        "UP")
            echo -e "${GREEN}✓ Overall Status: HEALTHY${NC}"
            ;;
        "DEGRADED")
            echo -e "${YELLOW}⚠ Overall Status: DEGRADED${NC}"
            ;;
        "DOWN")
            echo -e "${RED}✗ Overall Status: UNHEALTHY${NC}"
            ;;
    esac

    echo -e "${BLUE}Total Services:${NC} ${#RESULTS[@]}"
    echo -e "${BLUE}Failed:${NC} $FAILED_COUNT"
    echo -e "${BLUE}Degraded:${NC} $DEGRADED_COUNT"

    # Show next steps
    echo ""
    echo -e "${BLUE}Next Steps:${NC}"

    if [[ "$OVERALL_STATUS" == "DOWN" ]]; then
        echo -e "${RED}• Check failed services and restart containers${NC}"
        echo -e "${RED}• Review logs: docker-compose logs monitoring${NC}"
        echo -e "${RED}• Verify configuration files${NC}"
    elif [[ "$OVERALL_STATUS" == "DEGRADED" ]]; then
        echo -e "${YELLOW}• Review degraded services for potential issues${NC}"
        echo -e "${YELLOW}• Check alerting rules and metrics collection${NC}"
    else
        echo -e "${GREEN}• All monitoring services are healthy${NC}"
        echo -e "${GREEN}• Consider reviewing dashboards for insights${NC}"
    fi
fi

# Exit with appropriate code
case "$OVERALL_STATUS" in
    "UP")
        exit 0
        ;;
    "DEGRADED")
        exit 1
        ;;
    "DOWN")
        exit 2
        ;;
esac
