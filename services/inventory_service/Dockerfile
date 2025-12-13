# Multi-stage build for inventory_service
FROM rust:1.70-slim AS builder

WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./
COPY shared/ ./shared/
COPY services/inventory_service/core/Cargo.toml ./services/inventory_service/core/
COPY services/inventory_service/infra/Cargo.toml ./services/inventory_service/infra/
COPY services/inventory_service/api/Cargo.toml ./services/inventory_service/api/

# Create dummy source files for dependency caching
RUN mkdir -p services/inventory_service/core/src && echo "pub fn dummy() {}" > services/inventory_service/core/src/lib.rs
RUN mkdir -p services/inventory_service/infra/src && echo "pub fn dummy() {}" > services/inventory_service/infra/src/lib.rs
RUN mkdir -p services/inventory_service/api/src && echo "fn main() {}" > services/inventory_service/api/src/main.rs

# Build dependencies
RUN cargo build --release --package inventory_service_api --bin inventory-service

# Copy source code
COPY services/inventory_service/ ./services/inventory_service/

# Build the binary
RUN cargo build --release --package inventory_service_api --bin inventory-service

# Runtime stage
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/inventory-service /usr/local/bin/inventory-service

EXPOSE 8001

RUN adduser --disabled-password --gecos '' appuser && chown -R appuser:appuser /usr/local/bin
USER appuser
CMD ["inventory-service"]
