# Multi-stage build for inventory_service
FROM rust:1.70-slim AS builder

WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./
COPY shared/ ./shared/
COPY services/inventory_service/core/Cargo.toml ./services/inventory_service/core/
COPY services/inventory_service/infra/Cargo.toml ./services/inventory_service/infra/
COPY services/inventory_service/api/Cargo.toml ./services/inventory_service/api/

# Build dependencies
RUN cargo build --release --package inventory_service_api --bin inventory_service

# Copy source code
COPY services/inventory_service/ ./services/inventory_service/

# Build the binary
RUN cargo build --release --package inventory_service_api --bin inventory_service

# Runtime stage
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/inventory_service /usr/local/bin/inventory_service

EXPOSE 8000

CMD ["inventory_service"]
