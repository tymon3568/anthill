#!/bin/bash

# Anthill System Stop Script
# This script stops all Anthill services

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

PROJECT_DIR="/home/arch/Project/test/anthill-windsurf"
DOCKER_COMPOSE_DIR="$PROJECT_DIR/infra/docker_compose"

# Stop background services
stop_service() {
    local pid_file=$1
    local name=$2

    if [ -f "$pid_file" ]; then
        PID=$(cat "$pid_file")
        if kill -0 "$PID" 2>/dev/null; then
            log_info "Stopping $name (PID: $PID)..."
            kill "$PID" 2>/dev/null || true
        fi
        rm -f "$pid_file"
    fi
}

log_info "Stopping Anthill System..."

stop_service /tmp/frontend.pid "frontend"
stop_service /tmp/inventory-service.pid "inventory-service"
stop_service /tmp/user-service.pid "user-service"

# Stop Docker Compose
log_info "Stopping Docker Compose services..."
cd "$DOCKER_COMPOSE_DIR"
docker compose down

log_info "All services stopped"
