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

## Cấu Trúc Dự Án (Clean Architecture)

**Reference Template**: https://github.com/sukjaelee/clean_axum_demo

```
inventory-saas-platform/
├── services/                          # Microservices (Cargo workspace)
│   ├── user-service/
│   │   ├── src/
│   │   │   ├── main.rs                # Entry point
│   │   │   ├── app.rs                 # Router + middleware setup
│   │   │   ├── lib.rs
│   │   │   ├── common/                # Shared utilities
│   │   │   │   ├── app_state.rs      # AppState (dependency injection)
│   │   │   │   ├── bootstrap.rs      # Service initialization
│   │   │   │   ├── config.rs         # Env config loader
│   │   │   │   ├── error.rs          # AppError + IntoResponse
│   │   │   │   ├── hash_util.rs      # Argon2 password hashing
│   │   │   │   ├── jwt.rs            # JWT encode/decode
│   │   │   │   └── opentelemetry.rs  # Tracing setup
│   │   │   └── domains/               # Feature modules
│   │   │       ├── auth/
│   │   │       │   ├── api/          
│   │   │       │   │   ├── handlers.rs  # Login, register, refresh
│   │   │       │   │   └── routes.rs    # Route definitions
│   │   │       │   ├── domain/          # Business logic
│   │   │       │   │   ├── model.rs     # Auth entities
│   │   │       │   │   ├── repository.rs
│   │   │       │   │   └── service.rs
│   │   │       │   ├── dto/             # Data Transfer Objects
│   │   │       │   │   └── auth_dto.rs
│   │   │       │   ├── infra/           # Infrastructure impl
│   │   │       │   │   ├── impl_repository.rs
│   │   │       │   │   └── impl_service.rs
│   │   │       │   └── auth.rs          # Module entry
│   │   │       ├── user/                # User management
│   │   │       └── tenant/              # Tenant management
│   │   ├── tests/
│   │   │   ├── test_helpers.rs
│   │   │   └── test_auth_routes.rs
│   │   ├── .env.example
│   │   └── Cargo.toml
│   │
│   ├── inventory-service/             # ⭐ MAIN INVENTORY SERVICE
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── app.rs
│   │   │   ├── common/                # Same as user-service
│   │   │   └── domains/
│   │   │       ├── product/           # 📦 Product Master Data
│   │   │       │   ├── api/
│   │   │       │   │   ├── handlers.rs   # CRUD, list, search
│   │   │       │   │   └── routes.rs     # /api/v1/inventory/products
│   │   │       │   ├── domain/
│   │   │       │   │   ├── model.rs      # Product, ItemGroup, UoM
│   │   │       │   │   ├── repository.rs
│   │   │       │   │   └── service.rs
│   │   │       │   ├── dto/
│   │   │       │   │   └── product_dto.rs # CreateProduct, UpdateProduct
│   │   │       │   ├── infra/
│   │   │       │   └── product.rs
│   │   │       │
│   │   │       ├── warehouse/         # 🏭 Warehouse & Storage
│   │   │       │   ├── api/
│   │   │       │   │   └── handlers.rs   # Warehouse CRUD, locations
│   │   │       │   ├── domain/
│   │   │       │   │   └── model.rs      # Warehouse, StorageLocation
│   │   │       │   └── warehouse.rs
│   │   │       │
│   │   │       ├── stock/             # 📊 Stock Tracking
│   │   │       │   ├── api/
│   │   │       │   │   └── handlers.rs   # Get stock, movements, ledger
│   │   │       │   ├── domain/
│   │   │       │   │   ├── model.rs      # InventoryLevel, StockMove
│   │   │       │   │   └── service.rs    # Stock mutation logic
│   │   │       │   └── stock.rs
│   │   │       │
│   │   │       ├── receipt/           # 📥 Goods Receipt Note (GRN)
│   │   │       │   ├── api/
│   │   │       │   │   └── handlers.rs   # Create, validate, complete
│   │   │       │   ├── domain/
│   │   │       │   │   ├── model.rs      # GoodsReceipt, GoodsReceiptItem
│   │   │       │   │   └── service.rs    # Receipt workflow
│   │   │       │   └── receipt.rs
│   │   │       │
│   │   │       ├── delivery/          # 📤 Delivery Order (DO)
│   │   │       │   ├── api/
│   │   │       │   │   └── handlers.rs   # Reserve, pick, pack, ship
│   │   │       │   ├── domain/
│   │   │       │   │   ├── model.rs      # DeliveryOrder, items
│   │   │       │   │   └── service.rs    # Delivery workflow
│   │   │       │   └── delivery.rs
│   │   │       │
│   │   │       ├── transfer/          # 🔄 Stock Transfer
│   │   │       │   ├── api/
│   │   │       │   │   └── handlers.rs   # Create, confirm, receive
│   │   │       │   ├── domain/
│   │   │       │   │   └── service.rs    # Transfer workflow
│   │   │       │   └── transfer.rs
│   │   │       │
│   │   │       ├── stocktake/         # 📋 Physical Count
│   │   │       │   ├── api/
│   │   │       │   │   └── handlers.rs   # Create, count, finalize
│   │   │       │   ├── domain/
│   │   │       │   │   ├── model.rs      # StockTake, StockTakeLine
│   │   │       │   │   └── service.rs    # Reconciliation logic
│   │   │       │   └── stocktake.rs
│   │   │       │
│   │   │       ├── traceability/      # 🔍 Lot & Serial Number
│   │   │       │   ├── api/
│   │   │       │   │   └── handlers.rs   # Assign, track, FEFO
│   │   │       │   ├── domain/
│   │   │       │   │   ├── model.rs      # LotSerialNumber, moves
│   │   │       │   │   └── service.rs    # Traceability logic
│   │   │       │   └── traceability.rs
│   │   │       │
│   │   │       ├── valuation/         # 💰 Inventory Valuation
│   │   │       │   ├── api/
│   │   │       │   │   └── handlers.rs   # Get valuation, revalue
│   │   │       │   ├── domain/
│   │   │       │   │   ├── model.rs      # InventoryValuation, layers
│   │   │       │   │   └── service.rs    # FIFO/AVCO/Standard cost
│   │   │       │   └── valuation.rs
│   │   │       │
│   │   │       ├── quality/           # ✅ Quality Control
│   │   │       │   ├── api/
│   │   │       │   │   └── handlers.rs   # QC checks, pass/fail
│   │   │       │   └── quality.rs
│   │   │       │
│   │   │       ├── replenishment/     # 🔔 Reorder & Material Requests
│   │   │       │   ├── api/
│   │   │       │   │   └── handlers.rs   # Reorder rules, MR
│   │   │       │   ├── domain/
│   │   │       │   │   ├── model.rs      # ReorderRule, MaterialRequest
│   │   │       │   │   └── service.rs    # ROP calculation
│   │   │       │   └── replenishment.rs
│   │   │       │
│   │   │       ├── picking/           # 📦 Pick/Pack/Putaway
│   │   │       │   ├── api/
│   │   │       │   │   └── handlers.rs   # Generate pick lists, optimize
│   │   │       │   ├── domain/
│   │   │       │   │   ├── model.rs      # PickList, PickListItem
│   │   │       │   │   └── service.rs    # Batch/wave/cluster picking
│   │   │       │   └── picking.rs
│   │   │       │
│   │   │       └── reports/           # 📈 Inventory Reports
│   │   │           ├── api/
│   │   │           │   └── handlers.rs   # Ledger, aging, turnover
│   │   │           ├── domain/
│   │   │           │   └── service.rs    # Report generation
│   │   │           └── reports.rs
│   │   │
│   │   ├── tests/
│   │   │   ├── test_product_routes.rs
│   │   │   ├── test_receipt_workflow.rs
│   │   │   ├── test_delivery_workflow.rs
│   │   │   └── test_helpers.rs
│   │   └── Cargo.toml
│   │
│   ├── order-service/
│   │   └── src/domains/
│   │       ├── order/                 # Order management
│   │       ├── fulfillment/           # Order fulfillment
│   │       └── rma/                   # Return merchandise
│   │
│   ├── integration-service/
│   │   └── src/domains/
│   │       ├── marketplace/           # Marketplace adapters
│   │       │   ├── shopee/
│   │       │   ├── lazada/
│   │       │   └── tiki/
│   │       ├── webhook/               # Webhook handlers
│   │       └── sync/                  # Sync orchestration
│   │
│   └── payment-service/
│       └── src/domains/
│           ├── payment/               # Payment processing
│           ├── gateway/               # Gateway adapters (VNPay, Stripe)
│           └── refund/                # Refund handling
│
├── shared/                            # Shared Rust libraries
│   ├── common/                        # Common utilities
│   ├── db/                            # Database utilities
│   ├── auth/                          # Auth middleware
│   └── events/                        # NATS event definitions
├── frontend/                          # SvelteKit application
├── infra/
│   ├── docker-compose/
│   └── sql-migrations/
├── .env.global.example
├── rust-toolchain.toml
├── Makefile.toml
└── Cargo.toml
```

**Key Principles**:
1. **Clean Architecture**: API → Domain → Infrastructure layers
2. **Domain-Driven**: Each feature is self-contained (product, receipt, delivery, etc.)
3. **Type Safety**: Strong typing with compile-time SQLx checks
4. **Dependency Injection**: Use `AppState` pattern

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

## ⚡ Axum Production Best Practices

### 1. State Management (⚠️ CRITICAL)

**✅ DO**: Use `Arc<AppState>` with `with_state()`

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub jwt_secret: String,
}

// In main.rs
let app_state = Arc::new(AppState { db: pool, /* ... */ });

let app = Router::new()
    .route("/products", get(list_products))
    .with_state(app_state);  // ✅ Type-safe!

// In handler
async fn list_products(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Product>>, AppError> {
    // Use state.db
}
```

**❌ DON'T**: Use `Extension` for complex state

---

### 2. Error Handling (⚠️ CRITICAL)

**✅ DO**: Create `AppError` implementing `IntoResponse`

```rust
use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Validation: {0}")]
    Validation(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DB error"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, &msg),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}
```

**❌ DON'T**: Use `unwrap()` or `expect()` in production

---

### 3. Database & Connection Pool

**✅ DO**: SQLx with compile-time checks & offline mode

```rust
let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect(&database_url)
    .await?;

// Compile-time checked query
let products = sqlx::query_as!(
    Product,
    "SELECT * FROM products WHERE tenant_id = $1",
    tenant_id
).fetch_all(&pool).await?;

// Run: cargo sqlx prepare --workspace
```

**❌ DON'T**: Create pool per request

---

### 4. OpenAPI với Utoipa (⚠️ IMPORTANT)

**✅ DO**: Auto-generate Swagger UI

```rust
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(list_products, create_product),
    components(schemas(Product, CreateProductDto)),
    tags((name = "products"))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/products",
    responses(
        (status = 200, body = Vec<Product>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
async fn list_products(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Product>>, AppError> {
    // Implementation
}

// Mount Swagger
let app = Router::new()
    .merge(SwaggerUi::new("/docs")
        .url("/api-docs/openapi.json", ApiDoc::openapi()))
    .route("/api/v1/products", get(list_products));
```

Access: `http://localhost:3000/docs`

---

### 5. Validation với Custom Extractors

```rust
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateProductDto {
    #[validate(length(min = 1, max = 100))]
    pub sku: String,
    
    #[validate(range(min = 0.01))]
    pub price: f64,
}

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(data) = Json::<T>::from_request(req, state).await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        data.validate()
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        Ok(ValidatedJson(data))
    }
}

// Use in handler
async fn create_product(
    ValidatedJson(dto): ValidatedJson<CreateProductDto>,
) -> Result<Json<Product>, AppError> {
    // dto is validated!
}
```

---

### 6. Middleware & Tracing

```rust
use tower_http::{trace::TraceLayer, cors::CorsLayer, compression::CompressionLayer};

let app = Router::new()
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive())
    .layer(CompressionLayer::new());

// Structured logging
tracing::info!(
    tenant_id = %tenant_id,
    count = products.len(),
    "Fetched products"
);
```

**❌ DON'T**: Use `println!`

---

### 7. JWT Authentication

```rust
#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,        // user_id
    pub tenant_id: Uuid,
    pub role: String,
    pub exp: u64,
}

pub async fn jwt_middleware(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let claims = decode_jwt(auth.token(), &state.jwt_secret)?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

// Protected routes
let protected = Router::new()
    .route("/products", get(list_products))
    .layer(middleware::from_fn_with_state(state.clone(), jwt_middleware));
```

---

### 8. Testing without Server

```rust
use tower::ServiceExt;

#[tokio::test]
async fn test_list_products() {
    let pool = setup_test_db().await;
    let app_state = Arc::new(AppState { db: pool });
    
    let app = Router::new()
        .route("/products", get(list_products))
        .with_state(app_state);
    
    let request = Request::builder()
        .uri("/products")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

---

## 🎯 AI Coding Guidelines

### When AI Generates Code:

**✅ DO**:
1. **Use the template**: Start from `clean_axum_demo` structure
2. **Be specific**: "Create GRN handler following clean architecture with domain/repository/service separation"
3. **Request OpenAPI docs**: "Add utoipa macros for Swagger"
4. **Check tenant isolation**: Every query must filter by `tenant_id`
5. **Review lifetimes**: AI often gets `Arc<Mutex<>>`, `Send + Sync` wrong

**❌ DON'T**:
1. Trust AI 100% on async/await, lifetimes
2. Let AI use `unwrap()` in handlers
3. Accept runtime string queries (use sqlx compile-time checks)
4. Skip validation on DTOs

### Example Prompt Template:

```
I need a handler for creating goods receipts in the inventory-service.

Context:
- Project structure: Clean Architecture (api/domain/dto/infra layers)
- Framework: Axum 0.7 with sqlx
- State: Arc<AppState> with PgPool
- Error: AppError enum implementing IntoResponse
- Requirements: 
  - Multi-tenant (filter by tenant_id)
  - OpenAPI docs (utoipa)
  - Validated DTO
  - Idempotency key support

Please create:
1. GoodsReceiptDto in dto/receipt_dto.rs
2. Handler in api/handlers.rs
3. Service trait method signature
4. Repository trait method signature
```

---

## Multi-Tenant Database Patterns

```rust
// ✅ CORRECT: Always include tenant_id
let products = sqlx::query_as!(
    Product,
    "SELECT * FROM products WHERE tenant_id = $1 AND sku = $2",
    tenant_id,
    sku
).fetch_all(&pool).await?;

// ❌ WRONG: Missing tenant isolation
let products = sqlx::query_as!(
    Product,
    "SELECT * FROM products WHERE sku = $1",
    sku
).fetch_all(&pool).await?;
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

## 📄 OpenAPI Specification Workflow

### Auto-Export OpenAPI Spec

Mỗi service tự động export OpenAPI spec sang `shared/openapi/<service>.yaml` khi build với feature `export-spec`:

```bash
# Export spec for single service
cd services/user-service
cargo build --features export-spec
cargo run --features export-spec  # Export + start server

# Export all services
for service in services/*/; do
  (cd "$service" && cargo build --features export-spec)
done
```

### File Structure

```
shared/openapi/
├── README.md           # ⚠️ DO NOT EDIT - Auto-generated warning
├── user.yaml          # User service API spec
├── inventory.yaml     # Inventory service API spec  
├── order.yaml         # Order service API spec
├── payment.yaml       # Payment service API spec
├── integration.yaml   # Integration service API spec
└── api.yaml           # Merged final spec (for frontend)
```

### Swagger UI Access

Mỗi service expose Swagger UI tại `/docs`:

```bash
# User Service
http://localhost:3000/docs

# Inventory Service  
http://localhost:3001/docs

# Order Service
http://localhost:3002/docs
```

### Adding New API Endpoint

**1. Define DTO in `models.rs`:**
```rust
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateProductDto {
    #[schema(example = "PROD-001")]
    pub sku: String,
    
    #[schema(example = "Laptop")]
    pub name: String,
}
```

**2. Create handler in `handlers.rs`:**
```rust
#[utoipa::path(
    post,
    path = "/api/v1/products",
    tag = "products",
    operation_id = "inventory_create_product",  // Prefix with service name!
    request_body = CreateProductDto,
    responses(
        (status = 201, description = "Product created", body = Product),
        (status = 400, description = "Invalid input", body = ErrorResp),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_product(
    Json(payload): Json<CreateProductDto>,
) -> Result<(StatusCode, Json<Product>), AppError> {
    // Implementation
}
```

**3. Register in `openapi.rs`:**
```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::create_product,  // Add here
    ),
    components(schemas(
        crate::models::CreateProductDto,  // Add DTO
    ))
)]
pub struct ApiDoc;
```

**4. Export và verify:**
```bash
cargo build --features export-spec
cat ../../shared/openapi/inventory.yaml  # Check output
```

### Frontend SDK Generation (Future)

Khi CI/CD merge tất cả specs vào `api.yaml`, frontend sẽ auto-generate SDK:

```bash
# In frontend directory
pnpm orval  # or @hey-api/openapi-ts
```

---

## Quick Reference Commands

```bash
# Start local dev
cd infra/docker-compose && docker-compose up -d && cd ../..

# Run service with auto-reload
cargo watch -x 'run -p inventory-service'

# Export OpenAPI specs
for service in services/*/; do
  (cd "$service" && cargo build --features export-spec)
done

# SQLx offline mode
cargo sqlx prepare --workspace

# Format & lint
cargo fmt --all && cargo clippy --all -- -D warnings

# Run tests
cargo test --workspace

# View Swagger UI
open http://localhost:3000/docs  # user-service
```

---

## Resources

- **Clean Axum Template**: https://github.com/sukjaelee/clean_axum_demo
- **Utoipa Docs**: https://docs.rs/utoipa/
- **Axum Examples**: https://github.com/tokio-rs/axum/tree/main/examples
- **SQLx Guide**: https://github.com/launchbadge/sqlx
- **ARCHITECTURE.md**: Detailed system architecture
- **TODO.md**: Development roadmap with priorities

---

## Project-Specific Notes

### Inventory Service Domains Map:

| Domain | Purpose | Key Models | Routes |
|--------|---------|------------|--------|
| product | Product master data, UoM, variants | Product, ItemGroup, UoM | /api/v1/inventory/products |
| warehouse | Warehouse & storage locations | Warehouse, StorageLocation | /api/v1/inventory/warehouses |
| stock | Inventory levels, stock moves | InventoryLevel, StockMove | /api/v1/inventory/stock |
| receipt | Goods receipt (GRN) | GoodsReceipt, GoodsReceiptItem | /api/v1/inventory/receipts |
| delivery | Delivery orders (DO) | DeliveryOrder, DeliveryOrderItem | /api/v1/inventory/deliveries |
| transfer | Inter-warehouse transfers | StockTransfer, StockTransferItem | /api/v1/inventory/transfers |
| stocktake | Physical inventory count | StockTake, StockTakeLine | /api/v1/inventory/stock-takes |
| traceability | Lot & serial numbers | LotSerialNumber | /api/v1/inventory/tracking |
| valuation | FIFO/AVCO/Standard cost | InventoryValuation, ValuationLayer | /api/v1/inventory/valuation |
| quality | QC checks | QualityCheck | /api/v1/inventory/quality-checks |
| replenishment | Reorder rules, ROP | ReorderRule, MaterialRequest | /api/v1/inventory/replenishment |
| picking | Pick/pack/putaway | PickList, PickListItem | /api/v1/inventory/pick-lists |
| reports | Stock ledger, aging, turnover | - | /api/v1/inventory/reports |

### Adding New Domain Module:

1. Create folder structure: `domains/<module>/{api,domain,dto,infra}/`
2. Register in `domains.rs` and `app.rs`
3. Add to `app_state.rs` if needs dependencies
4. Update OpenAPI spec in `main.rs`
5. Add integration tests in `tests/test_<module>_routes.rs`

**Example**: See `/services/user-service/src/domains/auth/` as reference
