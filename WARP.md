# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Tổng Quan Dự Án

Đây là một nền tảng **SaaS quản lý tồn kho đa người dùng (multi-tenant)** được xây dựng với:
- **Backend**: Rust microservices (Axum + Tokio + SQLx)
- **Frontend**: SvelteKit 5 + TypeScript + Tailwind CSS
- **Database**: PostgreSQL với multi-tenant isolation
- **Cache**: Redis
- **Message Queue**: NATS (event-driven architecture)
- **Analytics**: Cube.js
- **Deployment**: CapRover (Docker Swarm based PaaS)

**Mục tiêu**: Quản lý tồn kho thời gian thực, tích hợp marketplace (Shopee, Lazada, Tiki, WooCommerce, Shopify), xử lý đơn hàng, và thanh toán (VNPay, Stripe, MoMo, ZaloPay).

**Trạng thái hiện tại**: Phase 1 - Thiết lập cơ sở hạ tầng (~10% hoàn thành)

## Cấu Trúc Dự Án

```
inventory-saas-platform/
├── services/                    # Rust microservices (Cargo workspace)
│   ├── user-service/           # Authentication, tenancy, authorization
│   ├── inventory-service/      # Product & stock management
│   ├── order-service/          # Order processing & fulfillment
│   ├── integration-service/    # Marketplace adapters (Shopee, Lazada...)
│   └── payment-service/        # Payment gateway integration
├── shared/                      # Shared Rust libraries (chưa triển khai)
│   ├── common/                 # Error types, config, tracing utilities
│   ├── db/                     # Database utilities, tenant context
│   ├── auth/                   # JWT + Casbin middleware
│   └── events/                 # NATS event definitions
├── frontend/                    # SvelteKit application (chưa khởi tạo)
├── infra/
│   ├── docker-compose/         # Local dev environment
│   │   └── docker-compose.yml
│   └── sql-migrations/         # SQLx migrations (sẽ được tạo)
├── Cargo.toml                   # Rust workspace root
├── ARCHITECTURE.md              # Kiến trúc chi tiết
├── TODO.md                      # Danh sách công việc theo phase
└── README.md                    # Documentation chính
```

## Kiến Trúc Microservices

### Event-Driven Communication

Các service giao tiếp qua **NATS** message queue với pattern pub/sub:

**Ví dụ luồng xử lý đơn hàng:**
1. `integration-service` publish `order.placed` (từ marketplace webhook)
2. `order-service` subscribe, validate stock → publish `order.confirmed`
3. `payment-service` subscribe, xử lý thanh toán → publish `payment.completed`
4. `order-service` subscribe, cập nhật trạng thái → publish `order.ready_to_fulfill`
5. `inventory-service` subscribe, giảm tồn kho → publish `inventory.stock.updated`

### Multi-Tenant Isolation

- **Strategy**: Shared schema với `tenant_id` trong mỗi bảng
- **Row-Level Security**: Tự động filter queries bằng `tenant_id` ở database layer
- **Authorization**: Casbin-rs với multi-tenant RBAC model `(sub, dom, obj, act)`
- **JWT Tokens**: Chứa `tenant_id` + `user_id`, được validate ở mỗi request

### Service Ports (Local Development)

- `user-service`: 3000
- `inventory-service`: 3001
- `order-service`: 3002
- `integration-service`: 3003
- `payment-service`: 3004
- `frontend`: 5173 (Vite dev server)
- PostgreSQL: 5432
- Redis: 6379
- NATS: 4222 (client), 8222 (monitoring)
- Cube.js: 4000 (API), 9009 (Playground)

## Lệnh Phát Triển Thường Dùng

### Khởi Động Môi Trường Local

```bash
# 1. Khởi động các stateful services (PostgreSQL, Redis, NATS, Cube)
cd infra/docker-compose
docker-compose up -d

# Quay lại thư mục gốc
cd ../..

# 2. Kiểm tra services đang chạy
docker ps

# 3. Xem logs của một service
docker logs -f postgres_db
docker logs -f nats_server
```

### Backend (Rust Microservices)

```bash
# Build tất cả services trong workspace
cargo build --workspace

# Build release (tối ưu)
cargo build --workspace --release

# Chạy một service cụ thể
cargo run -p user-service
cargo run -p inventory-service
cargo run -p order-service

# Chạy service với auto-reload (requires cargo-watch)
cargo watch -x 'run -p user-service'

# Check code mà không build (nhanh hơn)
cargo check --workspace

# Format code
cargo fmt --all

# Lint với clippy (fail on warnings)
cargo clippy --all -- -D warnings

# Chạy tests
cargo test --workspace

# Chạy test của một service cụ thể
cargo test -p user-service

# Chạy một test cụ thể
cargo test -p user-service test_name

# Chạy tests với output chi tiết
cargo test --workspace -- --nocapture

# Check for security vulnerabilities
cargo audit
```

### Database Migrations (SQLx)

**Lưu ý**: `DATABASE_URL` phải được set trong environment hoặc `.env` file:
```bash
export DATABASE_URL="postgres://user:password@localhost:5432/inventory_db"
```

```bash
# Chạy tất cả migrations
sqlx migrate run

# Tạo migration mới
sqlx migrate add create_tenants_table
sqlx migrate add create_users_table

# Revert migration gần nhất
sqlx migrate revert

# Kiểm tra migration status
sqlx migrate info

# Build-time verification (compile-time checked queries)
cargo sqlx prepare
```

### Frontend (SvelteKit) - Khi đã được khởi tạo

```bash
cd frontend

# Cài đặt dependencies
pnpm install

# Development server
pnpm dev

# Build production
pnpm build

# Preview production build
pnpm preview

# Lint
pnpm lint

# Format code
pnpm format

# Type checking
pnpm check
```

## Workflow Phát Triển

### 1. Tạo Microservice Mới

Các service mới phải được thêm vào Cargo workspace:

```bash
# Tạo thư mục service
mkdir -p services/new-service/src

# Tạo Cargo.toml
cat > services/new-service/Cargo.toml << 'EOF'
[package]
name = "new-service"
version = { workspace = true }
edition = { workspace = true }
authors = { workspace = true }

[dependencies]
tokio = { workspace = true }
axum = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
sqlx = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
config = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
EOF

# Tạo main.rs template
cat > services/new-service/src/main.rs << 'EOF'
use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(health_check));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3005));
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}
EOF

# Build để verify
cargo build -p new-service
```

### 2. Tạo Database Migration

```bash
# Tạo migration file
sqlx migrate add <descriptive_name>

# Ví dụ:
sqlx migrate add create_tenants_table
sqlx migrate add add_tenant_id_to_products

# File sẽ được tạo trong infra/sql-migrations/
# Format: <timestamp>_<name>.sql

# Viết SQL migration:
cat > infra/sql-migrations/<timestamp>_create_tenants_table.sql << 'EOF'
CREATE TABLE tenants (
    tenant_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    plan VARCHAR(50) NOT NULL DEFAULT 'free',
    settings JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tenants_name ON tenants(name);
EOF

# Apply migration
sqlx migrate run
```

### 3. Thêm Shared Library

```bash
# Tạo shared crate
mkdir -p shared/common/src
cd shared/common

# Tạo Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "common"
version = { workspace = true }
edition = { workspace = true }

[dependencies]
thiserror = { workspace = true }
anyhow = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }
EOF

# Tạo lib.rs
echo "pub mod error;" > src/lib.rs

# Sử dụng trong service
# Thêm vào services/*/Cargo.toml:
# common = { path = "../../shared/common" }
```

### 4. Testing Strategy

**Unit Tests** (trong cùng file hoặc `tests/` module):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(1 + 1, 2);
    }
}
```

**Integration Tests** (file trong `tests/` directory):
```bash
# Tạo integration test
mkdir -p services/user-service/tests
cat > services/user-service/tests/api_test.rs << 'EOF'
#[tokio::test]
async fn test_health_endpoint() {
    // Test implementation
}
EOF

# Chạy
cargo test -p user-service --test api_test
```

### 5. Debugging

```bash
# Chạy với RUST_LOG để xem logs chi tiết
RUST_LOG=debug cargo run -p user-service

# Chỉ log của crate cụ thể
RUST_LOG=user_service=debug cargo run -p user-service

# Trace tất cả
RUST_LOG=trace cargo run -p user-service

# Sử dụng rust-gdb
rust-gdb --args target/debug/user-service

# Hoặc lldb trên macOS
rust-lldb target/debug/user-service
```

## Patterns và Best Practices

### Multi-Tenant Database Queries

Luôn include `tenant_id` trong WHERE clause:

```rust
// ✅ ĐÚNG
let products = sqlx::query_as!(
    Product,
    "SELECT * FROM products WHERE tenant_id = $1 AND sku = $2",
    tenant_id,
    sku
)
.fetch_all(&pool)
.await?;

// ❌ SAI - Thiếu tenant isolation
let products = sqlx::query_as!(
    Product,
    "SELECT * FROM products WHERE sku = $1",
    sku
)
.fetch_all(&pool)
.await?;
```

### Event Publishing với NATS

```rust
// Định nghĩa event
#[derive(Serialize, Deserialize)]
struct OrderPlacedEvent {
    order_id: Uuid,
    tenant_id: Uuid,
    items: Vec<OrderItem>,
    timestamp: DateTime<Utc>,
}

// Publish
let event = OrderPlacedEvent { /* ... */ };
let payload = serde_json::to_vec(&event)?;
nats_client.publish("order.placed", payload).await?;

// Subscribe
let sub = nats_client.subscribe("order.placed").await?;
while let Some(msg) = sub.next().await {
    let event: OrderPlacedEvent = serde_json::from_slice(&msg.data)?;
    // Process event
}
```

### Error Handling

Sử dụng `thiserror` cho custom errors và `anyhow` cho propagation:

```rust
use thiserror::Error;
use anyhow::Result;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Tenant not found: {0}")]
    TenantNotFound(Uuid),
    
    #[error("Unauthorized access")]
    Unauthorized,
}

// Trong handler
async fn create_product(payload: Json<CreateProduct>) -> Result<Json<Product>, ServiceError> {
    // Implementation
}
```

### Authorization với Casbin

```rust
// Middleware extract tenant_id từ JWT
// Load enforcer với policies của tenant
let enforcer = load_tenant_enforcer(tenant_id).await?;

// Check permission
let allowed = enforcer.enforce((
    &user_id.to_string(),      // subject
    &tenant_id.to_string(),     // domain
    "products",                 // object
    "write",                    // action
))?;

if !allowed {
    return Err(ServiceError::Unauthorized);
}
```

## CapRover Deployment (Production)

### Dockerfile Template cho Microservice

```dockerfile
# Multi-stage build
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p user-service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/user-service /usr/local/bin/
EXPOSE 3000
CMD ["user-service"]
```

### CapRover App Configuration

1. **Environment Variables** cần set trong CapRover UI:
   - `DATABASE_URL`
   - `REDIS_URL`
   - `NATS_URL`
   - `JWT_SECRET`
   - `CASBIN_MODEL_PATH`

2. **Internal Service URLs** (Docker Swarm overlay network):
   - `http://srv-user-service:3000`
   - `http://srv-inventory-service:3001`
   - `http://srv-postgres:5432`
   - `http://srv-redis:6379`

3. **Health Check Endpoint**: Mỗi service phải expose `/health`

## Troubleshooting

### PostgreSQL Connection Issues

```bash
# Test connection
psql -h localhost -U user -d inventory_db -c "SELECT 1"

# Check if postgres container is running
docker ps | grep postgres

# View logs
docker logs postgres_db

# Reset database (⚠️ CAUTION: Deletes all data)
docker-compose down -v
docker-compose up -d postgres
```

### NATS Not Receiving Events

```bash
# Check NATS connection
curl http://localhost:8222/varz

# Subscribe to topic from CLI (for testing)
nats sub "order.*"

# Publish test message
nats pub order.test "Hello World"
```

### Build Errors

```bash
# Bersihkan cache
cargo clean

# Update dependencies
cargo update

# Check for outdated dependencies
cargo outdated
```

### SQLx Compile-Time Verification Failures

```bash
# Ensure database is running and migrations are applied
sqlx migrate run

# Prepare query metadata (offline mode)
cargo sqlx prepare

# Sẽ tạo file .sqlx/query-*.json để verify queries tại compile time
# Commit các file này vào git
```

## Resources

- **Architecture**: Xem `ARCHITECTURE.md` cho kiến trúc chi tiết với diagrams
- **Development Phases**: Xem `TODO.md` để theo dõi tiến độ và roadmap
- **Rust Book**: https://doc.rust-lang.org/book/
- **Axum Docs**: https://docs.rs/axum/
- **SQLx Guide**: https://github.com/launchbadge/sqlx
- **CapRover Docs**: https://caprover.com/docs/
- **NATS Docs**: https://docs.nats.io/
- **Casbin-rs**: https://github.com/casbin/casbin-rs

## Quick Reference

```bash
# Startup sequence
cd infra/docker-compose && docker-compose up -d && cd ../..
cargo run -p user-service &
cargo run -p inventory-service &
cd frontend && pnpm dev

# Full rebuild
cargo clean && cargo build --workspace --release

# Run all tests with coverage
cargo test --workspace --all-features

# Database reset (dev only)
sqlx database drop && sqlx database create && sqlx migrate run

# Format & lint all code
cargo fmt --all && cargo clippy --all -- -D warnings

# Check project health
cargo check --workspace && cargo test --workspace && cargo clippy --all
```
