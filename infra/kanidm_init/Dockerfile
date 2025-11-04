# Multi-stage build: Rust binary + minimal runtime
FROM rust:1.83-alpine AS builder

WORKDIR /build

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

# Copy manifests
COPY Cargo.toml ./

# Create dummy main to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source
COPY src ./src

# Build for release
RUN touch src/main.rs && cargo build --release

# Runtime: Ubuntu base with kanidm tools
FROM kanidm/tools:1.4.3 AS kanidm-tools

FROM ubuntu:24.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates curl bash expect && rm -rf /var/lib/apt/lists/*

# Copy kanidm CLI from tools image
COPY --from=kanidm-tools /usr/sbin/kanidm /usr/local/bin/kanidm

# Copy Rust binary
COPY --from=builder /build/target/release/kanidm-init /usr/local/bin/kanidm-init

# Copy entrypoint script
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
COPY setup-kanidm.sh /usr/local/bin/setup-kanidm.sh
RUN chmod +x /usr/local/bin/entrypoint.sh /usr/local/bin/kanidm-init /usr/local/bin/setup-kanidm.sh

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
