#!/usr/bin/env bash
# Start Monitoring Stack Script
# Starts all monitoring services (Prometheus, Grafana, Loki, AlertManager)
# Usage: ./scripts/start-monitoring.sh [options]

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default options
DETACHED=true
BUILD=false
SERVICES=""
WAIT=true
TIMEOUT=120

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-detach)
            DETACHED=false
            shift
            ;;
        --build)
            BUILD=true
            shift
            ;;
        --services)
            SERVICES="$2"
            shift 2
            ;;
        --no-wait)
            WAIT=false
            shift
            ;;
        --timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --help|-h)
            echo "Start Monitoring Stack"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --no-detach       Run in foreground (don't detach)"
            echo "  --build           Build images before starting"
            echo "  --services LIST   Start specific services (comma-separated)"
            echo "  --no-wait         Don't wait for services to be ready"
            echo "  --timeout SEC     Timeout for waiting (default: 120)"
            echo "  --help, -h        Show this help"
            echo ""
            echo "Services:"
            echo "  prometheus, grafana, loki, alertmanager, node-exporter"
            echo ""
            echo "Examples:"
            echo "  $0                           # Start all services"
            echo "  $0 --services prometheus,grafana  # Start specific services"
            echo "  $0 --no-detach --no-wait     # Run in foreground, no waiting"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Check if docker and docker-compose are available
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker not found. Please install Docker.${NC}"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}❌ docker-compose not found. Please install docker-compose.${NC}"
    exit 1
fi

# Check if monitoring docker-compose file exists
COMPOSE_FILE="infra/docker_compose/monitoring/docker-compose.yml"
if [ ! -f "$COMPOSE_FILE" ]; then
    echo -e "${RED}❌ Monitoring docker-compose file not found: $COMPOSE_FILE${NC}"
    echo -e "${YELLOW}Please ensure the monitoring infrastructure is set up.${NC}"
    exit 1
fi

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Starting Monitoring Stack           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Build images if requested
if [[ "$BUILD" == true ]]; then
    echo -e "${BLUE}[1/4] Building monitoring images...${NC}"
    docker-compose -f "$COMPOSE_FILE" build
    echo -e "${GREEN}✓ Images built${NC}"
    echo ""
fi

# Prepare docker-compose command
COMPOSE_CMD="docker-compose -f $COMPOSE_FILE"

if [[ -n "$SERVICES" ]]; then
    COMPOSE_CMD="$COMPOSE_CMD up $SERVICES"
    echo -e "${BLUE}Starting services: $SERVICES${NC}"
else
    COMPOSE_CMD="$COMPOSE_CMD up"
    echo -e "${BLUE}Starting all monitoring services...${NC}"
fi

if [[ "$DETACHED" == true ]]; then
    COMPOSE_CMD="$COMPOSE_CMD -d"
fi

# Start services
echo -e "${BLUE}[2/4] Starting containers...${NC}"
if ! $COMPOSE_CMD; then
    echo -e "${RED}❌ Failed to start monitoring services${NC}"
    echo -e "${YELLOW}Check logs: docker-compose -f $COMPOSE_FILE logs${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Containers started${NC}"
echo ""

# Wait for services to be ready
if [[ "$WAIT" == true ]]; then
    echo -e "${BLUE}[3/4] Waiting for services to be ready...${NC}"

    # Services to check
    declare -A SERVICES_TO_CHECK=(
        ["prometheus"]="http://localhost:9090/-/ready"
        ["grafana"]="http://localhost:3000/api/health"
        ["loki"]="http://localhost:3100/ready"
        ["alertmanager"]="http://localhost:9093/-/ready"
    )

    local all_ready=true
    local start_time=$(date +%s)

    for service in "${!SERVICES_TO_CHECK[@]}"; do
        local url="${SERVICES_TO_CHECK[$service]}"
        local ready=false
        local attempts=0
        local max_attempts=$((TIMEOUT / 5))  # Check every 5 seconds

        echo -e "${BLUE}Waiting for $service...${NC}"

        while [[ $attempts -lt $max_attempts ]]; do
            if curl -s --max-time 5 "$url" &>/dev/null; then
                echo -e "${GREEN}✓ $service ready${NC}"
                ready=true
                break
            fi

            ((attempts++))
            sleep 5

            # Check timeout
            local current_time=$(date +%s)
            if [[ $((current_time - start_time)) -gt $TIMEOUT ]]; then
                echo -e "${RED}✗ $service timeout after ${TIMEOUT}s${NC}"
                all_ready=false
                break
            fi
        done

        if [[ "$ready" == false ]]; then
            echo -e "${RED}✗ $service failed to start${NC}"
            all_ready=false
        fi
    done

    if [[ "$all_ready" == true ]]; then
        echo -e "${GREEN}✓ All services ready${NC}"
    else
        echo -e "${YELLOW}⚠ Some services may not be fully ready${NC}"
    fi

    echo ""
fi

# Show status and access information
echo -e "${BLUE}[4/4] Monitoring stack status:${NC}"
echo ""

# Check running containers
echo -e "${BLUE}Running containers:${NC}"
docker-compose -f "$COMPOSE_FILE" ps

echo ""
echo -e "${BLUE}Service URLs:${NC}"
echo -e "${GREEN}• Prometheus:${NC}    http://localhost:9090"
echo -e "${GREEN}• Grafana:${NC}       http://localhost:3000 (admin/admin)"
echo -e "${GREEN}• Loki:${NC}          http://localhost:3100"
echo -e "${GREEN}• AlertManager:${NC}  http://localhost:9093"

echo ""
echo -e "${BLUE}Useful commands:${NC}"
echo -e "${YELLOW}• Check health:${NC}    ./scripts/monitoring-health-check.sh"
echo -e "${YELLOW}• View logs:${NC}       docker-compose -f $COMPOSE_FILE logs -f [service]"
echo -e "${YELLOW}• Stop services:${NC}   ./scripts/stop-monitoring.sh"
echo -e "${YELLOW}• Restart service:${NC} docker-compose -f $COMPOSE_FILE restart [service]"

echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   Monitoring Stack Started!           ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"

# Show final status if waiting was enabled
if [[ "$WAIT" == true ]]; then
    echo ""
    echo -e "${BLUE}Running health check...${NC}"
    ./scripts/monitoring-health-check.sh --timeout 10
fi
