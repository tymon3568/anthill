#!/bin/bash

# Anthill System Startup Script
# This script starts all required services for the Anthill platform

set -e

PROJECT_DIR="/home/arch/Project/test/anthill-windsurf"
DOCKER_COMPOSE_DIR="$PROJECT_DIR/infra/docker_compose"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Start Docker Compose services
start_docker() {
    log_info "Starting Docker Compose services..."
    cd "$DOCKER_COMPOSE_DIR"
    docker compose up -d
    log_info "Docker services started"
}

# Run database migrations
run_migrations() {
    log_info "Running database migrations..."
    cd "$PROJECT_DIR"
    sqlx migrate run
    log_info "Migrations completed"
}

# Start backend services in background
start_backend() {
    log_info "Starting user-service..."
    cd "$PROJECT_DIR"
    cargo run --bin user-service &
    USER_SERVICE_PID=$!
    echo $USER_SERVICE_PID > /tmp/user-service.pid
    log_info "user-service started (PID: $USER_SERVICE_PID)"
}

# Start inventory service
start_inventory() {
    log_info "Starting inventory-service..."
    cd "$PROJECT_DIR"
    PORT=8001 NATS_URL=nats://localhost:4222 cargo run --bin inventory-service &
    INVENTORY_SERVICE_PID=$!
    echo $INVENTORY_SERVICE_PID > /tmp/inventory-service.pid
    log_info "inventory-service started (PID: $INVENTORY_SERVICE_PID)"
}

# Start frontend
start_frontend() {
    log_info "Starting frontend..."
    cd "$PROJECT_DIR/frontend"
    bun run dev &
    FRONTEND_PID=$!
    echo $FRONTEND_PID > /tmp/frontend.pid
    log_info "frontend started (PID: $FRONTEND_PID)"
}

# Main execution
main() {
    log_info "Starting Anthill System..."

    start_docker
    sleep 5  # Wait for Docker services to be ready

    run_migrations

    start_backend
    sleep 3  # Wait for user-service to start

    start_inventory
    sleep 2

    start_frontend

    log_info "All services started successfully!"
    echo ""
    echo "Services running:"
    echo "  - Docker Compose: running"
    echo "  - user-service: http://localhost:8000"
    echo "  - inventory-service: http://localhost:8001"
    echo "  - frontend: http://localhost:5173"
    echo ""
    echo "To stop all services, run: ./stop-system.sh"

    # Wait for all background processes
    wait
}

main "$@"
