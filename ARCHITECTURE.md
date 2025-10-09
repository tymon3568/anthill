# Kiến Trúc Hệ Thống - Inventory SaaS Platform (CapRover Edition)

## 🎯 Triết Lý Kiến Trúc

Kiến trúc này được thiết kế dựa trên triết lý thực dụng: **"Sử dụng công cụ phù hợp nhất cho từng công việc"**. Chúng ta ưu tiên các công cụ hạ tầng phổ biến, hiệu suất cao và đã được chứng minh (`battle-tested`), đồng thời tập trung sức mạnh của **Rust** vào nơi nó tạo ra nhiều giá trị nhất: **core business logic**. Nền tảng triển khai là **CapRover**, một PaaS mạnh mẽ giúp đơn giản hóa tối đa việc vận hành.

- **Đơn giản & Hiệu quả**: Tận dụng tối đa các tính năng tự động của CapRover để giảm thiểu công sức quản lý hạ tầng.
- **Hiệu năng cao**: Sử dụng các công cụ tiêu chuẩn ngành (NGINX, Docker Swarm, PostgreSQL, Redis) kết hợp với các microservice viết bằng Rust.
- **An toàn & Bảo mật**: Tận dụng mạng nội bộ của Docker và các cơ chế bảo mật của CapRover, kết hợp với sự an toàn bộ nhớ của Rust.

## 🏗️ Kiến Trúc Tổng Thể trên CapRover

CapRover xây dựng trên Docker Swarm, cung cấp một môi trường PaaS tiện lợi. Kiến trúc của chúng ta sẽ xoay quanh các khái niệm "App" và "One-Click App" của CapRover.

```
                 Internet
                     │
┌────────────────────▼─────────────────────┐
│              CapRover Cluster            │
│          (1 hoặc nhiều server)           │
│ ┌──────────────────────────────────────┐ │
│ │     CapRover NGINX Ingress Proxy     │ │ (Gateway Tự Động)
│ │   (Load Balancing, SSL, Routing)     │ │
│ └──────────────────┬───────────────────┘ │
│                    │ (Route tới app qua Hostname)
│ ┌──────────────────┴───────────────────┐ │
│ │       Docker Swarm Overlay Network     │ │ (Mạng nội bộ an toàn)
│ │                                      │ │
│ │ ┌──────────────┐   ┌───────────────┐ │ │
│ │ │  Rust Service  │   │ Rust Service  │ │ │
│ │ │   (App 1)      ├─► │   (App 2)     │ │ │  (Core Logic)
│ │ │ inventory-svc  │   │  order-svc    │ │ │
│ │ └──────▲───────┘   └───────▲───────┘ │ │
│ │        │                   │         │ │
│ │ ┌──────┴─────────┐  ┌──────┴───────┐ │ │
│ │ │   PostgreSQL   │  │ NATS / Redis │ │ │  (Stateful Services)
│ │ │ (One-Click App)│  │(One-Click App)│ │ │
│ │ └────────────────┘  └──────────────┘ │ │
│ └──────────────────────────────────────┘ │
└──────────────────────────────────────────┘
```

## 🧩 Chi tiết các thành phần

### 1. Gateway & Routing: NGINX của CapRover

- **Công cụ**: NGINX được tích hợp sẵn và quản lý hoàn toàn bởi CapRover.
- **Vai trò**:
  - **Edge Gateway**: Là điểm vào duy nhất cho tất cả traffic từ internet.
  - **Load Balancer**: Tự động cân bằng tải giữa các instance của một app.
  - **SSL Termination**: Tự động cài đặt và gia hạn chứng chỉ Let's Encrypt.
  - **Routing**: Route traffic đến các service Rust dựa trên tên miền được cấu hình cho mỗi app (ví dụ: `api.yourdomain.com`, `inventory-api.yourdomain.com`).
- **Lợi ích**: Không cần quản lý gateway riêng. Cấu hình cực kỳ đơn giản qua giao diện CapRover.

### 2. Backend Microservices: Rust & Axum

- **Công nghệ**: Rust, Axum, Tokio, SQLx.
- **Vai trò**: Đây là nơi chứa đựng toàn bộ business logic của hệ thống (User, Inventory, Order, Integration...). Mỗi service là một project Rust riêng biệt.
- **Triển khai**:
  1.  Mỗi service có một `Dockerfile` để đóng gói thành một image.
  2.  Trong CapRover, mỗi service được định nghĩa là một "App".
  3.  Kết nối CapRover với GitHub/GitLab, và nó sẽ tự động build và deploy mỗi khi có `git push`.
  4.  Scaling (tăng/giảm số container) được thực hiện dễ dàng qua giao diện.

### 3. Giao tiếp giữa các Service: Docker Swarm Network

- **Công nghệ**: Docker Swarm Overlay Network.
- **Vai trò**: Tạo một mạng ảo riêng tư và an toàn cho tất cả các app trong CapRover.
- **Cách hoạt động**: Các service có thể gọi nhau qua tên app. CapRover tự động tạo một hostname là `srv-<app-name>`. Ví dụ, từ `order-service`, bạn có thể kết nối tới `inventory-service` qua địa chỉ `http://srv-inventory-svc:8000`.
- **Lợi ích**: Đơn giản, an toàn, không cần cấu hình service discovery phức tạp như Consul hay Etcd.

### 4. Database & Message Queue: CapRover One-Click Apps

- **Công nghệ**: Sử dụng kho ứng dụng có sẵn của CapRover.
- **Các lựa chọn**:
  - **Database**: **PostgreSQL** (đã được chứng minh, viết bằng C).
  - **Cache**: **Redis** (tiêu chuẩn ngành, viết bằng C).
  - **Message Queue**: **NATS** (hiệu năng cao, viết bằng Go).
  - **Analytics**: **Cube.js** có thể được triển khai như một app riêng.
- **Triển khai**:
  - Vào mục "One-Click Apps", tìm và triển khai các ứng dụng trên chỉ với vài cú click.
  - CapRover tự động quản lý việc lưu trữ dữ liệu bền vững (persistent storage) cho chúng.

### 5. Authorization: Casbin-rs

- **Công nghệ**: Crate `casbin-rs`.
- **Vai trò**:
  - Vẫn tích hợp trực tiếp vào các microservice Rust (đặc biệt là User Service và API Gateway nếu tự build).
  - Models và policies có thể được lưu trong PostgreSQL, sử dụng `casbin-sqlx-adapter`.
  - Một middleware trong Axum sẽ load enforcer và kiểm tra quyền hạn cho mỗi request.
  - Shared crate `shared/auth` cung cấp middleware và extractors cho tất cả services.

### 6. Multi-Tenancy Strategy

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

3. **Middleware**: Extract tenant_id từ JWT và inject vào request
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

### 7. Database Design Standards

#### 7.1 UUID Version: Use UUID v7

- **Lý do**: UUID v7 có timestamp prefix → better index locality, improved query performance
- **Implementation**: Sử dụng `uuid` crate với feature `v7`
  ```rust
  use uuid::Uuid;
  let id = Uuid::now_v7(); // Timestamp-based UUID
  ```

#### 7.2 Currency/Money: Use BIGINT (cents)

- **Quyết định**: Lưu tiền dưới dạng `BIGINT` (đơn vị nhỏ nhất - cents, xu)
- **Lý do**:
  - ✅ No floating-point rounding errors
  - ✅ Better performance than NUMERIC
  - ✅ Easy arithmetic operations
- **Example**: $10.50 → 1050 cents, 100.000 VND → 100000
- **Rust type**: `i64` hoặc custom `Money` type

#### 7.3 Soft Delete Strategy

- **Pattern**: Add `deleted_at TIMESTAMPTZ` column
- **Apply to**: Critical tables (products, orders, users)
- **Index**: Create partial index `WHERE deleted_at IS NULL` for active records
  ```sql
  ALTER TABLE products ADD COLUMN deleted_at TIMESTAMPTZ;
  CREATE INDEX idx_products_active ON products(tenant_id, sku) 
    WHERE deleted_at IS NULL;
  ```

#### 7.4 Timestamps Convention

- Use `TIMESTAMPTZ` (timezone-aware) cho tất cả timestamp columns
- Standard columns: `created_at`, `updated_at`, `deleted_at`
- Set defaults:
  ```sql
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  ```

#### 7.5 Sensitive Data: Application-level Encryption

- **Use case**: `credentials` field trong bảng `integrations`
- **Strategy**: Encrypt trong Rust trước khi lưu DB
- **Library**: `ring` hoặc `RustCrypto`
- **Key management**: Environment variable, không hard-code
- **Format**: Store as `BYTEA` hoặc `TEXT` (base64-encoded)

## 🔧 Technology Stack Summary (CapRover Edition)

### Core Application
- **Backend Language**: Rust (Stable)
- **Web Framework**: Axum
- **Async Runtime**: Tokio
- **Database Driver**: SQLx

### Infrastructure & Platform
- **PaaS**: CapRover
- **Container Orchestration**: Docker Swarm (do CapRover quản lý)
- **API Gateway**: NGINX (do CapRover quản lý)
- **Service Networking**: Docker Swarm Overlay Network

### Stateful Services & Middleware ( развернуто как One-Click Apps)
- **Database**: PostgreSQL
- **Cache**: Redis
- **Message Queue**: NATS
- **Analytics**: Cube

### DevOps
- **CI/CD**: Tích hợp sẵn trong CapRover (Webhook từ Git) hoặc dùng GitHub Actions để build Docker image và trigger deploy trên CapRover.
- **Monitoring**: Netdata (thường có sẵn trong CapRover One-Click Apps).

## 🚀 Quy trình phát triển & triển khai

1.  **Local Dev**: Sử dụng `docker_compose` để mô phỏng môi trường CapRover (các service Rust, Postgres, Redis, NATS).
2.  **Code**: Viết logic cho các microservice bằng Rust.
3.  **Push**: Đẩy code lên GitHub.
4.  **Deploy**: CapRover nhận webhook, tự động build image từ `Dockerfile` và triển khai phiên bản mới.
5.  **Scale/Manage**: Sử dụng giao diện CapRover để theo dõi logs, scaling, và quản lý các biến môi trường.

Kiến trúc này vừa hiện đại, hiệu năng cao, vừa cực kỳ thực tế và dễ vận hành cho đội ngũ nhỏ, cho phép bạn tập trung vào việc xây dựng sản phẩm.
