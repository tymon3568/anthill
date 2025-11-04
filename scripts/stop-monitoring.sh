#!/usr/bin/env bash
# Stop Monitoring Stack Script
# Stops all monitoring services and optionally removes containers/volumes
# Usage: ./scripts/stop-monitoring.sh [options]

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default options
REMOVE_CONTAINERS=false
REMOVE_VOLUMES=false
REMOVE_IMAGES=false
FORCE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --remove|-r)
            REMOVE_CONTAINERS=true
            shift
            ;;
        --volumes|-v)
            REMOVE_VOLUMES=true
            shift
            ;;
        --images|-i)
            REMOVE_IMAGES=true
            shift
            ;;
        --all|-a)
            REMOVE_CONTAINERS=true
            REMOVE_VOLUMES=true
            REMOVE_IMAGES=true
            shift
            ;;
        --force|-f)
            FORCE=true
            shift
            ;;
        --help|-h)
            echo "Stop Monitoring Stack"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --remove, -r       Remove containers after stopping"
            echo "  --volumes, -v      Remove volumes (WARNING: destroys data)"
            echo "  --images, -i       Remove images after stopping"
            echo "  --all, -a          Remove containers, volumes, and images"
            echo "  --force, -f        Force stop (don't ask for confirmation)"
            echo "  --help, -h         Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                    # Stop services (keep containers)"
            echo "  $0 --remove          # Stop and remove containers"
            echo "  $0 --all             # Stop and remove everything"
            echo "  $0 --volumes --force # Remove volumes without confirmation"
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
    echo -e "${RED}❌ Docker not found.${NC}"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}❌ docker-compose not found.${NC}"
    exit 1
fi

# Check if monitoring docker-compose file exists
COMPOSE_FILE="infra/docker_compose/monitoring/docker-compose.yml"
if [ ! -f "$COMPOSE_FILE" ]; then
    echo -e "${RED}❌ Monitoring docker-compose file not found: $COMPOSE_FILE${NC}"
    exit 1
fi

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Stopping Monitoring Stack           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Show current status
echo -e "${BLUE}Current status:${NC}"
docker-compose -f "$COMPOSE_FILE" ps
echo ""

# Confirm destructive operations
if [[ "$REMOVE_VOLUMES" == true && "$FORCE" == false ]]; then
    echo -e "${RED}⚠ WARNING: Removing volumes will destroy all monitoring data!${NC}"
    read -p "Are you sure? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Cancelled.${NC}"
        exit 0
    fi
fi

if [[ "$REMOVE_IMAGES" == true && "$FORCE" == false ]]; then
    echo -e "${RED}⚠ WARNING: Removing images will require re-building!${NC}"
    read -p "Are you sure? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Cancelled.${NC}"
        exit 0
    fi
fi

# Stop services
echo -e "${BLUE}[1/3] Stopping services...${NC}"
if docker-compose -f "$COMPOSE_FILE" down; then
    echo -e "${GREEN}✓ Services stopped${NC}"
else
    echo -e "${YELLOW}⚠ Some services may not have been running${NC}"
fi

echo ""

# Remove containers if requested
if [[ "$REMOVE_CONTAINERS" == true ]]; then
    echo -e "${BLUE}[2/3] Removing containers...${NC}"

    # Get container names
    local containers=$(docker-compose -f "$COMPOSE_FILE" ps -q)

    if [[ -n "$containers" ]]; then
        docker-compose -f "$COMPOSE_FILE" down --remove-orphans
        echo -e "${GREEN}✓ Containers removed${NC}"
    else
        echo -e "${YELLOW}⚠ No containers to remove${NC}"
    fi

    echo ""
fi

# Remove volumes if requested
if [[ "$REMOVE_VOLUMES" == true ]]; then
    echo -e "${BLUE}[2/3] Removing volumes...${NC}"

    # Get volume names from docker-compose config
    local volumes=$(docker-compose -f "$COMPOSE_FILE" config --volumes 2>/dev/null | grep -v '^volumes:' | sed 's/.*://' | tr -d ' ')

    if [[ -n "$volumes" ]]; then
        for volume in $volumes; do
            if docker volume ls -q | grep -q "^${volume}$"; then
                echo -e "${YELLOW}Removing volume: $volume${NC}"
                docker volume rm "$volume" || true
            fi
        done
        echo -e "${GREEN}✓ Volumes removed${NC}"
    else
        echo -e "${YELLOW}⚠ No volumes to remove${NC}"
    fi

    echo ""
fi

# Remove images if requested
if [[ "$REMOVE_IMAGES" == true ]]; then
    echo -e "${BLUE}[3/3] Removing images...${NC}"

    # Get image names from docker-compose config
    local images=$(docker-compose -f "$COMPOSE_FILE" config | grep 'image:' | sed 's/.*image: //' | tr -d '"' | sort | uniq)

    if [[ -n "$images" ]]; then
        for image in $images; do
            if docker images -q "$image" &>/dev/null; then
                echo -e "${YELLOW}Removing image: $image${NC}"
                docker rmi "$image" || true
            fi
        done
        echo -e "${GREEN}✓ Images removed${NC}"
    else
        echo -e "${YELLOW}⚠ No images to remove${NC}"
    fi

    echo ""
fi

# Final status
echo -e "${BLUE}Final status:${NC}"
echo ""

# Check if any monitoring containers are still running
local running_containers=$(docker ps --filter "label=com.docker.compose.project=monitoring" --format "{{.Names}}" | wc -l)

if [[ $running_containers -gt 0 ]]; then
    echo -e "${YELLOW}⚠ Some monitoring containers may still be running:${NC}"
    docker ps --filter "label=com.docker.compose.project=monitoring" --format "table {{.Names}}\t{{.Status}}"
    echo ""
else
    echo -e "${GREEN}✓ No monitoring containers running${NC}"
fi

# Check volumes
local volumes_count=$(docker volume ls --filter "label=com.docker.compose.project=monitoring" --format "{{.Name}}" | wc -l)

if [[ $volumes_count -gt 0 ]]; then
    echo -e "${BLUE}Remaining volumes: $volumes_count${NC}"
    docker volume ls --filter "label=com.docker.compose.project=monitoring" --format "table {{.Name}}"
else
    echo -e "${GREEN}✓ No monitoring volumes${NC}"
fi

echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   Monitoring Stack Stopped!           ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"

# Show next steps
echo ""
echo -e "${BLUE}Next steps:${NC}"
if [[ "$REMOVE_CONTAINERS" == false && "$REMOVE_VOLUMES" == false && "$REMOVE_IMAGES" == false ]]; then
    echo -e "${YELLOW}• Start again: ./scripts/start-monitoring.sh${NC}"
    echo -e "${YELLOW}• View logs: docker-compose -f $COMPOSE_FILE logs [service]${NC}"
fi

if [[ "$REMOVE_VOLUMES" == true ]]; then
    echo -e "${RED}• Data was destroyed. Fresh start required.${NC}"
    echo -e "${YELLOW}• Reinitialize: ./scripts/start-monitoring.sh --build${NC}"
fi

if [[ "$REMOVE_IMAGES" == true ]]; then
    echo -e "${YELLOW}• Images removed. Next start will pull/build images.${NC}"
fi
