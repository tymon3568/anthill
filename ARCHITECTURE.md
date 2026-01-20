# Kiến Trúc Hệ Thống - Inventory SaaS Platform (Docker Compose Edition)

## 🎯 Triết Lý Kiến Trúc

### SQLx Standard (Enterprise): Compile-time Macros + Offline Mode (Mandatory)
Để đảm bảo **an toàn schema** (bắt lỗi SQL/type sớm) và **CI ổn định** (không phụ thuộc DB live trong bước build), dự án chuẩn hóa như sau:

#### 1) Quy ước bắt buộc
- **Ưu tiên dùng `sqlx::query!` / `sqlx::query_as!` / `sqlx::query_scalar!`** (compile-time checked) thay vì `sqlx::query(...)` + `.bind(...)` khi SQL là static.
- **Bật SQLx Offline Mode** với thư mục **`.sqlx/`** được commit vào git.
- Mọi thay đổi schema (migrations) hoặc thay đổi query macro phải đi kèm cập nhật `.sqlx/`.

> Lý do: compile-time macros bắt lỗi sai tên cột, sai type, sai số lượng tham số ngay lúc compile; offline mode giúp CI/build không cần DB live mà vẫn giữ compile-time validation.

#### 2) Cách vận hành Offline Mode
- Khi cần cập nhật metadata:
  1. Chạy PostgreSQL local/test (đúng schema).
  2. Set `DATABASE_URL` trỏ vào DB đã migrate.
  3. Chạy: `cargo sqlx prepare` để sinh/refresh `.sqlx/`.
  4. Commit thay đổi trong `.sqlx/` cùng với code/migrations.

- Trong CI:
  - **Không cần DB live cho bước compile** nếu `.sqlx/` đã đúng.
  - Nên chạy `cargo sqlx prepare --check` để đảm bảo `.sqlx/` luôn đồng bộ với code/schema.

#### 3) Phạm vi áp dụng
- **Bắt buộc** cho code production (infra repositories, shared DB code).
- Khuyến nghị mạnh cho integration tests/helpers (đặc biệt các câu `SELECT/INSERT/DELETE` cố định).
- Chỉ dùng runtime `sqlx::query(...)` khi:
  - SQL phải dynamic (không thể là string literal), hoặc
  - thật sự cần builder/phức tạp; khi đó phải có test coverage đủ tốt.


Kiến trúc này được thiết kế dựa trên triết lý thực dụng: **"Sử dụng công cụ phù hợp nhất cho từng công việc"**. Chúng ta ưu tiên các công cụ hạ tầng phổ biến, hiệu suất cao và đã được chứng minh (`battle-tested`), đồng thời tập trung sức mạnh của **Rust** vào nơi nó tạo ra nhiều giá trị nhất: **core business logic**. Nền tảng triển khai là **Docker Compose**, một giải pháp container orchestration đơn giản và mạnh mẽ, phù hợp cho việc deploy lên VPS với các công cụ như **Dokploy** hoặc **Komodo**.

- **Đơn giản & Hiệu quả**: Sử dụng Docker Compose để quản lý toàn bộ stack, dễ dàng triển khai và bảo trì.
- **Hiệu năng cao**: Sử dụng các công cụ tiêu chuẩn ngành (Apache APISIX, Docker, PostgreSQL, KeyDB, RustFS) kết hợp với các microservice viết bằng Rust.
- **An toàn & Bảo mật**: Tận dụng mạng nội bộ của Docker và các cơ chế bảo mật, kết hợp với sự an toàn bộ nhớ của Rust.
- **Authentication nội bộ**: Sử dụng **Email/Password authentication** do User Service quản lý, đơn giản và phù hợp cho MVP.

## 🏗️ Kiến Trúc Tổng Thể với Docker Compose

Docker Compose cung cấp một cách đơn giản để định nghĩa và chạy multi-container Docker applications. Kiến trúc của chúng ta xoay quanh các service containers.

```
                 Internet
                     │
┌────────────────────▼─────────────────────────────────────┐
│              Docker Compose Stack                        │
│          (Single VPS hoặc multi-node)                   │
│ ┌────────────────────────────────────────────────────┐   │
│ │     Apache APISIX API Gateway                      │   │ (Gateway)
│ │   (Load Balancing, SSL, Routing, Plugins)          │   │
│ └──────────────────┬─────────────────────────────────┘   │
│                    │ (Route tới service qua hostname)    │
│ ┌──────────────────┴─────────────────────────────────┐   │
│ │       Docker Bridge Network                        │   │ (Mạng nội bộ an toàn)
│ │                                                    │   │
│ │ ┌───────────────┐   ┌───────────────┐              │   │
│ │ │  Rust Service │   │  Rust Service │              │   │
│ │ │   User Svc    │   │ inventory-svc ├─► ...        │   │  (Auth & Business Logic)
│ │ │  + Casbin     │   └───────▲───────┘              │   │
│ │ │  + JWT Auth   │           │                       │   │
│ │ └───────▲───────┘           │                       │   │
│ │         │                    │                       │   │
│ │ ┌───────┴───────────────────┴───────┐             │   │
│ │ │  PostgreSQL  │  │ NATS / KeyDB   │             │   │  (Stateful Services)
│ │ │              │  │                │             │   │
│ │ └──────────────┘  └────────────────┘             │   │
│ │ ┌──────────────────────────────────┐             │   │
│ │ │         RustFS (S3-compatible)   │             │   │  (Object Storage)
│ │ └──────────────────────────────────┘             │   │
│ └────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘

Legend:
  User Svc: Authentication (Email/Password), User/Tenant management, Casbin authorization
  Other Services: Inventory, Order, Payment, Integration
  KeyDB: High-performance Redis-compatible cache (multi-threaded)
  RustFS: High-performance S3-compatible object storage (Rust-native)
```

## 🧩 Chi tiết các thành phần

### 1. Gateway & Routing: Apache APISIX

- **Công cụ**: Apache APISIX được cấu hình qua Docker Compose.
- **Vai trò**:
  - **Edge Gateway**: Là điểm vào duy nhất cho tất cả traffic từ internet.
  - **Load Balancer**: Cân bằng tải giữa các instance của một service.
  - **SSL Termination**: Xử lý SSL/TLS certificates (Let's Encrypt hoặc custom).
  - **Routing**: Route traffic đến các service Rust dựa trên path patterns.
  - **Plugin System**: Hỗ trợ nhiều plugins cho rate limiting, authentication, logging, etc.
- **Lợi ích**: 
  - High performance (dựa trên OpenResty/NGINX + Lua).
  - Dynamic configuration qua Admin API (không cần restart).
  - Plugin ecosystem phong phú.
  - Dashboard UI để quản lý routes và plugins.

### 2. Backend Microservices: Rust & Axum

- **Công nghệ**: Rust, Axum, Tokio, SQLx.
- **Vai trò**: Đây là nơi chứa đựng toàn bộ business logic của hệ thống (User, Inventory, Order, Integration...). Mỗi service là một project Rust riêng biệt.
- **Triển khai**:
  1.  Mỗi service có một `Dockerfile` để đóng gói thành một image.
  2.  Trong Docker Compose, mỗi service được định nghĩa như một container.
  3.  Kết nối với CI/CD (GitHub Actions) để tự động build và deploy.
  4.  Scaling được thực hiện qua Docker Compose `scale` hoặc Docker Swarm.

### 3. Frontend Application: SvelteKit 2 with Svelte 5

- **Công nghệ**: SvelteKit 2, Svelte 5, TypeScript, Tailwind CSS, shadcn-svelte.
- **Vai trò**: Giao diện người dùng chính cho hệ thống, bao gồm dashboard, quản lý sản phẩm, đơn hàng, và cài đặt.
- **Tính năng chính**:
  - **State Management**: Sử dụng Svelte 5 runes cho reactive state.
  - **Form Validation**: Valibot cho client-side validation.
  - **UI Components**: shadcn-svelte theo chuẩn thiết kế Frappe UI.
  - **Authentication**: JWT tokens từ User Service, handle refresh tokens.
  - **API Client**: Native fetch API để call backend APIs.
  - **Testing**: Vitest cho unit tests, Playwright cho E2E tests.
- **Triển khai**:
  - Deployed như một container riêng biệt trong Docker Compose.
  - Build thành static assets hoặc SSR dựa trên nhu cầu.
  - Kết nối với backend services qua internal Docker network.

### 4. Giao tiếp giữa các Service: Docker Network

- **Công nghệ**: Docker Bridge Network (hoặc Overlay Network cho multi-node).
- **Vai trò**: Tạo một mạng ảo riêng tư và an toàn cho tất cả các containers.
- **Cách hoạt động**: Các service có thể gọi nhau qua container name. Ví dụ, từ `order-service`, bạn có thể kết nối tới `inventory-service` qua địa chỉ `http://inventory-service:8001`.
- **Lợi ích**: Đơn giản, an toàn, không cần cấu hình service discovery phức tạp.

### 5. Database & Message Queue

- **Công nghệ**: Containers được định nghĩa trong Docker Compose.
- **Các lựa chọn**:
  - **Database**: **PostgreSQL 16** (đã được chứng minh, viết bằng C).
  - **Cache**: **KeyDB** (Redis-compatible, multi-threaded, high-performance).
  - **Message Queue**: **NATS 2.10** (hiệu năng cao, viết bằng Go).
  - **Object Storage**: **RustFS** (S3-compatible, high-performance, viết bằng Rust).
  - **Analytics**: **Cube.js** có thể được triển khai như một container riêng.
- **Triển khai**:
  - Định nghĩa trong `docker-compose.yml` với persistent volumes.
  - Health checks để đảm bảo service availability.

### 6. Object Storage: MinIO

- **Công nghệ**: MinIO - High-performance S3-compatible object storage.
- **Vai trò**:
  - **S3-compatible**: 100% tương thích với S3 API.
  - **High Performance**: Được tối ưu cho cloud-native workloads.
  - **File Storage**: Lưu trữ avatars, documents, và các files ứng dụng.
- **Cấu hình**:
  ```yaml
  minio:
    image: minio/minio:latest
    environment:
      MINIO_ROOT_USER: ${MINIO_ROOT_USER:-minioadmin}
      MINIO_ROOT_PASSWORD: ${MINIO_ROOT_PASSWORD:-minioadmin}
    volumes:
      - minio_data:/data
    command: server /data --console-address ":9001"
  ```
- **Lợi ích**:
  - ✅ Production-ready và được sử dụng rộng rãi.
  - ✅ Có sẵn Docker image chính thức.
  - ✅ Ecosystem phong phú với mc CLI.
  - ✅ Fully S3-compatible.

### 7. Cache Layer: KeyDB

- **Công nghệ**: KeyDB - High-performance, multi-threaded Redis fork.
- **Vai trò**:
  - **Session Storage**: Lưu trữ user sessions.
  - **Caching**: Cache frequently accessed data.
  - **Rate Limiting**: Store rate limit counters.
  - **Pub/Sub**: Real-time messaging giữa services.
- **Cấu hình**:
  ```yaml
  keydb:
    image: eqalpha/keydb:latest
    command: keydb-server --server-threads 2 --appendonly yes
    volumes:
      - keydb_data:/data
  ```
- **Lợi ích so với Redis**:
  - ✅ **Multi-threaded**: Tận dụng multiple CPU cores (Redis là single-threaded).
  - ✅ **Higher throughput**: 2-5x performance improvement trong nhiều workloads.
  - ✅ **Active-Active replication**: Built-in multi-master support.
  - ✅ **100% Redis compatible**: Drop-in replacement, không cần thay đổi code.

### 8. Authentication: Email/Password (User Service)

- **Công nghệ**: User Service (Rust) với bcrypt password hashing, JWT tokens.
- **Vai trò**:
  - **User Authentication**: Xử lý login, registration, password management.
  - **JWT Token Issuance**: Tạo và ký JWT tokens (access + refresh).
  - **Session Management**: Quản lý user sessions trong database.
  - **Tenant Context**: Extract tenant từ subdomain hoặc X-Tenant-ID header.
- **API Endpoints**:
  ```
  POST /api/v1/auth/register    - Đăng ký user mới + tạo/join tenant
  POST /api/v1/auth/login       - Đăng nhập, trả về JWT tokens
  POST /api/v1/auth/refresh     - Refresh access token
  POST /api/v1/auth/logout      - Logout, revoke session
  ```
- **Security Features**:
  - ✅ Password hashing với bcrypt (cost factor 12)
  - ✅ Password strength validation (zxcvbn)
  - ✅ JWT với expiration (access: 15min, refresh: 7 days)
  - ✅ Session tracking trong database
  - ✅ Rate limiting cho login attempts
- **Lợi ích**:
  - ✅ Đơn giản, không cần external IdP.
  - ✅ Full control over authentication flow.
  - ✅ Phù hợp cho MVP và small-to-medium teams.

### 9. Authorization: Casbin-rs

- **Công nghệ**: Crate `casbin-rs`.
- **Vai trò**:
  - Tích hợp trực tiếp vào các microservice Rust (đặc biệt là User Service).
  - Models và policies được lưu trong PostgreSQL, sử dụng `casbin-sqlx-adapter`.
  - Một middleware trong Axum sẽ load enforcer và kiểm tra quyền hạn cho mỗi request.
  - Shared crate `shared/auth` cung cấp middleware và extractors cho tất cả services.
  - **Làm việc với JWT**: Extracts user_id, tenant_id, và role từ JWT tokens để enforce policies.

**Policy Format**: `(role, tenant_id, resource, action)`

```rust
// Example policies
("admin", "tenant-uuid-123", "users", "manage")
("manager", "tenant-uuid-123", "products", "write")
("user", "tenant-uuid-123", "products", "read")
```

### 10. Multi-Tenancy Strategy

**Quyết định kiến trúc**: Sử dụng **Shared Database với Tenant Isolation bằng tenant_id**

#### Lý do chọn Shared Schema:
- **Đơn giản**: Chỉ một database, dễ quản lý migrations và backups
- **Tiết kiệm chi phí**: Không cần nhiều database instances
- **Performance tốt**: Có thể optimize indexes cho multi-tenant queries
- **Scalable**: Có thể shard theo tenant_id khi cần

#### Tenant Isolation Strategy:

**Quyết định**: **Application-level filtering** (không dùng Postgres RLS)

**Lý do**:
- ✅ **Đơn giản hơn**: Dễ debug, dễ hiểu flow
- ✅ **Performance**: Không có overhead của RLS
- ✅ **Flexibility**: Dễ implement cross-tenant queries (cho admin/super-admin)
- ✅ **Testing**: Dễ test hơn, không cần setup RLS policies
- ⚠️ **Trade-off**: Cần cẩn thận thêm `WHERE tenant_id = $1` trong mọi query

#### Tenant Context Flow:

```
┌─────────────────────────────────────────────────────────────┐
│  Request arrives                                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 1. Check X-Tenant-ID header                         │   │
│  │ 2. Or parse subdomain: acme.anthill.com → "acme"   │   │
│  │ 3. Lookup tenant by slug/id in database            │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│                          ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ TenantContext { tenant_id: UUID }                   │   │
│  │ Injected into request extensions                    │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│                          ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ All repository queries include tenant_id           │   │
│  │ SELECT * FROM products WHERE tenant_id = $1        │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**Implementation Guidelines**:

1. **Repository Pattern**: Tất cả queries qua Repository layer
2. **Type Safety**: Sử dụng Rust type system để enforce tenant_id
   ```rust
   // Example
   pub struct TenantContext {
       pub tenant_id: Uuid,
   }
   
   impl Repository {
       pub async fn find_by_id(&self, ctx: &TenantContext, id: Uuid) -> Result<Product> {
           sqlx::query_as!(Product,
               "SELECT * FROM products WHERE tenant_id = $1 AND product_id = $2",
               ctx.tenant_id, id
           )
           .fetch_one(&self.pool)
           .await
       }
   }
   ```

3. **Middleware**: Extract tenant_id từ JWT claims và inject vào request
4. **Testing**: Unit tests verify tenant isolation
5. **Audit**: Log tất cả queries với tenant_id

#### Database Schema Convention:

- Mọi bảng có dữ liệu tenant-specific **PHẢI** có cột `tenant_id UUID NOT NULL`
- Composite indexes: `(tenant_id, <other_columns>)` để optimize multi-tenant queries
- Foreign keys: Include tenant_id trong composite keys khi cần
  ```sql
  -- Example: Order Items reference Products
  FOREIGN KEY (tenant_id, product_id) REFERENCES products(tenant_id, product_id)
  ```

### 11. Database Design Standards

#### 11.1 UUID Version: Use UUID v7

- **Lý do**: UUID v7 có timestamp prefix → better index locality, improved query performance
- **Implementation**: Sử dụng `uuid` crate với feature `v7`
  ```rust
  use uuid::Uuid;
  let id = Uuid::now_v7(); // Timestamp-based UUID
  ```

#### 11.2 Currency/Money: Use BIGINT (cents)

- **Quyết định**: Lưu tiền dưới dạng `BIGINT` (đơn vị nhỏ nhất - cents, xu)
- **Lý do**:
  - ✅ No floating-point rounding errors
  - ✅ Better performance than NUMERIC
  - ✅ Easy arithmetic operations
- **Example**: $10.50 → 1050 cents, 100.000 VND → 100000
- **Rust type**: `i64` hoặc custom `Money` type

#### 11.3 Soft Delete Strategy

- **Pattern**: Add `deleted_at TIMESTAMPTZ` column
- **Apply to**: Critical tables (products, orders, users)
- **Index**: Create partial index `WHERE deleted_at IS NULL` for active records
  ```sql
  ALTER TABLE products ADD COLUMN deleted_at TIMESTAMPTZ;
  CREATE INDEX idx_products_active ON products(tenant_id, sku) 
    WHERE deleted_at IS NULL;
  ```

#### 11.4 Timestamps Convention

- Use `TIMESTAMPTZ` (timezone-aware) cho tất cả timestamp columns
- Standard columns: `created_at`, `updated_at`, `deleted_at`
- Set defaults:
  ```sql
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  ```

#### 11.5 Sensitive Data: Application-level Encryption

- **Use case**: `credentials` field trong bảng `integrations`
- **Strategy**: Encrypt trong Rust trước khi lưu DB
- **Library**: `ring` hoặc `RustCrypto`
- **Key management**: Environment variable, không hard-code
- **Format**: Store as `BYTEA` hoặc `TEXT` (base64-encoded)

## 🔧 Technology Stack Summary (Docker Compose Edition)

### Service Port Assignments

All services use standardized ports for consistency across development and production environments:

- **User Service**: Port 8000 (Authentication, User Management, Casbin RBAC)
- **Inventory Service**: Port 8001 (Product Management, Stock Tracking)
- **Order Service**: Port 8002 (Order Processing, Fulfillment)
- **Integration Service**: Port 8003 (Marketplace Integrations, Sync Operations)
- **Payment Service**: Port 8004 (Payment Processing, Gateway Integration)
- **Frontend**: Port 5173 (Development) / Port 3000 (Production)

**Port Override Mechanism**: Each service can override the default port via `PORT` environment variable for flexibility in different deployment scenarios.

### Core Application
- **Backend Language**: Rust (Stable)
- **Web Framework**: Axum
- **Async Runtime**: Tokio
- **Database Driver**: SQLx

### Frontend Application
- **Framework**: SvelteKit 2 with Svelte 5
- **Language**: TypeScript (strict mode)
- **State Management**: Svelte 5 runes
- **Form Validation**: Valibot
- **UI Components**: shadcn-svelte + Tailwind CSS
- **Design System**: Frappe UI standards
- **API Client**: Native fetch API
- **Testing**: Vitest (unit) + Playwright (E2E)

### Authentication & Authorization
- **Authentication**: Email/Password (User Service managed)
- **Password Hashing**: bcrypt (cost factor 12)
- **Password Validation**: zxcvbn (strength scoring)
- **Token Format**: JWT (access + refresh tokens)
- **Authorization**: Casbin-rs (RBAC with tenant context)

### Infrastructure & Platform
- **Container Orchestration**: Docker Compose
- **Deployment Platforms**: Dokploy, Komodo (VPS deployment)
- **API Gateway**: Apache APISIX
- **Service Networking**: Docker Bridge/Overlay Network

### Stateful Services & Middleware
- **Database**: PostgreSQL 16
- **Cache**: KeyDB (Redis-compatible, multi-threaded)
- **Message Queue**: NATS 2.10
- **Object Storage**: RustFS (S3-compatible, Rust-native)
- **Analytics**: Cube (optional)

### DevOps
- **CI/CD**: GitHub Actions để build Docker images và deploy.
- **Monitoring**: Prometheus + Grafana, hoặc Netdata.

### Observability & Distributed Tracing

Microservices architecture yêu cầu observability toàn diện để debug và monitor request flows. Stack này sử dụng **OpenTelemetry** làm chuẩn instrumentation và **SigNoz** làm backend phân tích.

#### Observability Stack

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Observability Architecture                       │
│                                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                   │
│  │ User Service │  │Inventory Svc │  │ Order Service│  ... (Rust Apps)  │
│  │  + OTel SDK  │  │  + OTel SDK  │  │  + OTel SDK  │                   │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                   │
│         │                  │                  │                          │
│         └──────────────────┼──────────────────┘                          │
│                            │ OTLP (gRPC/HTTP)                            │
│                            ▼                                             │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │              OpenTelemetry Collector                             │    │
│  │  ┌─────────┐  ┌──────────────┐  ┌─────────────────────────┐     │    │
│  │  │Receivers│→ │  Processors  │→ │       Exporters          │     │    │
│  │  │ (OTLP)  │  │(Batch, Filter)│  │(SigNoz, Prometheus, etc)│     │    │
│  │  └─────────┘  └──────────────┘  └─────────────────────────┘     │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                            │                                             │
│                            ▼                                             │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                        SigNoz                                    │    │
│  │  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐        │    │
│  │  │    Traces     │  │    Metrics    │  │     Logs      │        │    │
│  │  │  (Jaeger UI)  │  │  (Dashboards) │  │  (Search)     │        │    │
│  │  └───────────────┘  └───────────────┘  └───────────────┘        │    │
│  │                            │                                     │    │
│  │                            ▼                                     │    │
│  │  ┌─────────────────────────────────────────────────────────┐    │    │
│  │  │                    ClickHouse                            │    │    │
│  │  │  (High-performance columnar storage for traces/metrics)  │    │    │
│  │  └─────────────────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
```

#### Components

| Component | Role | Technology |
|-----------|------|------------|
| **OpenTelemetry SDK** | Instrumentation trong Rust services | `tracing`, `tracing-opentelemetry`, `opentelemetry-otlp` |
| **OTel Collector** | Nhận, xử lý và export telemetry data | OpenTelemetry Collector Contrib |
| **SigNoz** | Unified observability platform (traces, metrics, logs) | SigNoz (open-source APM) |
| **ClickHouse** | High-performance storage cho telemetry data | ClickHouse (columnar DB) |

#### Rust Service Instrumentation

```rust
// Cargo.toml dependencies
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-opentelemetry = "0.22"
opentelemetry = { version = "0.21", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.14", features = ["tonic"] }

// Example: Initialize OpenTelemetry tracing
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_telemetry(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://otel-collector:4317"),
        )
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_resource(opentelemetry::sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", service_name.to_string()),
                ])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(telemetry_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

// Example: Instrument async functions
#[tracing::instrument(skip(pool))]
pub async fn get_product(pool: &PgPool, product_id: Uuid) -> Result<Product> {
    tracing::info!("Fetching product");
    // ... database query
}

// Example: Add custom spans for NATS messaging
#[tracing::instrument(skip(nats_client, payload))]
pub async fn publish_order_event(nats_client: &Client, payload: &OrderEvent) -> Result<()> {
    tracing::info!(event_type = %payload.event_type, "Publishing to NATS");
    // ... publish to NATS
}
```

#### Key Tracing Scenarios

1. **HTTP Request Flow**: Trace requests từ Gateway → Service → Database
2. **NATS Message Flow**: Trace async messages giữa services qua NATS
3. **Database Queries**: Trace tất cả SQL queries với timing
4. **External API Calls**: Trace calls đến marketplace APIs (Shopee, Lazada, etc.)
5. **Background Jobs**: Trace scheduled tasks và async workers

#### Trace Context Propagation

```rust
// Propagate trace context qua NATS headers
use opentelemetry::global;
use opentelemetry::propagation::TextMapPropagator;

pub fn inject_trace_context(headers: &mut HashMap<String, String>) {
    let propagator = global::get_text_map_propagator();
    let cx = tracing::Span::current().context();
    propagator.inject_context(&cx, &mut HeaderInjector(headers));
}

pub fn extract_trace_context(headers: &HashMap<String, String>) -> Context {
    let propagator = global::get_text_map_propagator();
    propagator.extract(&HeaderExtractor(headers))
}
```

#### Benefits

- ✅ **End-to-end visibility**: Xem toàn bộ request flow qua multiple services
- ✅ **Debug async flows**: Trace NATS messages và async events
- ✅ **Performance insights**: Identify bottlenecks và slow queries
- ✅ **Error correlation**: Link errors với specific traces
- ✅ **Unified platform**: Traces, metrics, và logs trong một UI

## 🚀 Quy trình phát triển & triển khai

1.  **Local Dev**: Sử dụng `docker-compose.yml` trong `infra/docker_compose/` để chạy môi trường development (PostgreSQL, KeyDB, NATS, RustFS).
2.  **Code**: Viết logic cho các microservice bằng Rust.
3.  **Push**: Đẩy code lên GitHub.
4.  **Build**: GitHub Actions build Docker images và push to registry.
5.  **Deploy**: Sử dụng Dokploy hoặc Komodo để deploy lên VPS với Docker Compose.
6.  **Scale/Manage**: Sử dụng Docker Compose scaling hoặc Docker Swarm cho high availability.

## 🔐 Authentication Flow

### Registration Flow
```
┌─────────────────────────────────────────────────────────────┐
│  User fills registration form                               │
│  - Email, Password, Full Name, Organization Name           │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  POST /api/v1/auth/register                                 │
│  {                                                          │
│    "email": "user@example.com",                            │
│    "password": "SecureP@ss123",                            │
│    "full_name": "John Doe",                                │
│    "tenant_name": "ACME Corp"                              │
│  }                                                          │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  User Service:                                              │
│  1. Validate password strength (zxcvbn score >= 3)         │
│  2. Check/Create tenant by slug                            │
│  3. Check email uniqueness within tenant                   │
│  4. Hash password with bcrypt                              │
│  5. Create user record                                     │
│  6. Generate JWT tokens (access + refresh)                 │
│  7. Create session record                                  │
│  8. Return tokens + user info                              │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  Response:                                                  │
│  {                                                          │
│    "access_token": "eyJhbGc...",                           │
│    "refresh_token": "eyJhbGc...",                          │
│    "token_type": "Bearer",                                 │
│    "expires_in": 900,                                      │
│    "user": { "id": "...", "email": "...", ... }           │
│  }                                                          │
└─────────────────────────────────────────────────────────────┘
```

### Login Flow
```
┌─────────────────────────────────────────────────────────────┐
│  User enters credentials                                    │
│  - Tenant context from subdomain or manual input           │
│  - Email + Password                                         │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  POST /api/v1/auth/login                                    │
│  Headers: X-Tenant-ID: acme (or from subdomain)            │
│  Body: { "email": "...", "password": "..." }               │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  User Service:                                              │
│  1. Resolve tenant from header/subdomain                   │
│  2. Find user by email + tenant_id                         │
│  3. Verify password with bcrypt                            │
│  4. Check account status (active, not locked)              │
│  5. Generate JWT tokens                                    │
│  6. Create session record                                  │
│  7. Return tokens + user info                              │
└─────────────────────────────────────────────────────────────┘
```

Kiến trúc này vừa hiện đại, hiệu năng cao, vừa cực kỳ thực tế và dễ vận hành cho đội ngũ nhỏ, cho phép bạn tập trung vào việc xây dựng sản phẩm.
