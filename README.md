# 🐜 Anthill - Inventory SaaS Platform

**Anthill** là một nền tảng SaaS quản lý tồn kho đa người dùng (multi-tenant) hiện đại, được xây dựng bằng **Rust** (backend microservices) và **SvelteKit 5** (frontend), tối ưu cho các doanh nghiệp bán hàng online.

> 🐜 Giống như đàn kiến làm việc hiệu quả và có tổ chức, Anthill giúp bạn quản lý tồn kho một cách thông minh và tự động.

## 🎯 Tính Năng Chính

- ✅ **Multi-tenant Architecture**: Hỗ trợ nhiều khách hàng trên cùng một hạ tầng
- ✅ **Real-time Inventory Tracking**: Cập nhật tồn kho thời gian thực
- ✅ **Marketplace Integration**: Kết nối với Shopee, Lazada, Tiki, WooCommerce, Shopify
- ✅ **Order Management**: Quản lý đơn hàng từ nhiều kênh
- ✅ **Payment Gateway**: Tích hợp VNPay, Stripe, MoMo, ZaloPay
- ✅ **Analytics & Reporting**: Dashboard phân tích với Cube
- ✅ **Zero Downtime Deployment**: Triển khai liên tục không gián đoạn

## 🏗️ Kiến Trúc

Dự án sử dụng kiến trúc **Event-Driven Microservices** với các công nghệ:

- **Backend**: Rust + Axum + Tokio + SQLx
- **Frontend**: SvelteKit 5 + TypeScript + Tailwind CSS
- **Database**: PostgreSQL
- **Cache**: Redis
- **Message Queue**: NATS
- **Analytics**: Cube
- **Deployment**: CapRover (Docker Swarm)
- **Gateway**: NGINX (managed by CapRover)

Chi tiết kiến trúc xem tại [ARCHITECTURE.md](./ARCHITECTURE.md)

## 📁 Cấu Trúc Dự Án

```
anthill/
├── services/                    # Các microservices Rust
│   ├── user-service/           # Authentication & tenancy
│   ├── inventory-service/      # Quản lý tồn kho
│   ├── order-service/          # Quản lý đơn hàng
│   ├── integration-service/    # Tích hợp marketplace
│   └── payment-service/        # Xử lý thanh toán
├── shared/                      # Thư viện chung
│   ├── common/                 # Error types, config, tracing
│   ├── db/                     # Database utilities
│   ├── auth/                   # JWT, Casbin middleware
│   └── events/                 # Event definitions, NATS client
├── frontend/                    # SvelteKit application
├── infra/                       # Infrastructure config
│   ├── docker-compose/         # Local dev environment
│   └── sql-migrations/         # Database migrations
├── Cargo.toml                   # Rust workspace
├── ARCHITECTURE.md              # Tài liệu kiến trúc
├── TODO.md                      # Danh sách công việc
└── README.md                    # File này
```

## 🚀 Quick Start

### Prerequisites

- **Rust** (stable + nightly): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Docker & Docker Compose**: Để chạy môi trường local
- **Node.js** (>= 20) & **pnpm**: Cho frontend
- **PostgreSQL Client**: `psql` (optional, for debugging)

### 1. Cài đặt Rust Tools

```bash
# Cài đặt Rust toolchain
rustup default stable
rustup toolchain add nightly
rustup component add clippy rustfmt

# Cài đặt cargo tools
cargo install cargo-watch        # Auto-reload
cargo install sqlx-cli --features postgres  # DB migrations
cargo install cargo-make         # Task runner
```

### 2. Khởi Động Môi Trường Local

```bash
# Clone repository
git clone <your-repo-url>
cd anthill

# Khởi động PostgreSQL, Redis, NATS
cd infra/docker-compose
docker-compose up -d

# Quay lại thư mục gốc
cd ../..
```

### 3. Build & Run Backend Services

```bash
# Build tất cả services
cargo build --workspace

# Chạy user-service (port 3000)
cargo run -p user-service

# Trong terminal khác, chạy inventory-service (port 3001)
cargo run -p inventory-service

# Và cứ thế tiếp tục với các service khác...
```

### 4. Setup Frontend (SvelteKit)

```bash
cd frontend

# Cài đặt dependencies
pnpm install

# Chạy dev server
pnpm dev
```

Truy cập: `http://localhost:5173`

## 🛠️ Development Commands

### Backend (Rust)

```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --all -- -D warnings

# Run tests
cargo test --workspace

# Run specific service với auto-reload
cargo watch -x 'run -p user-service'

# Check code without building
cargo check --workspace
```

### Database Migrations

```bash
# Chạy migrations
sqlx migrate run --database-url postgres://user:password@localhost:5432/inventory_db

# Tạo migration mới
sqlx migrate add <migration_name>

# Revert migration
sqlx migrate revert --database-url postgres://user:password@localhost:5432/inventory_db
```

### Frontend

```bash
cd frontend

# Development
pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview

# Lint & format
pnpm lint
pnpm format
```

## 📊 Database Schema

Xem chi tiết trong `infra/sql-migrations/`. Các bảng chính:

- `tenants`: Thông tin khách hàng (tenant)
- `users`: Người dùng trong từng tenant
- `products`: Sản phẩm
- `inventory_levels`: Mức tồn kho
- `orders`: Đơn hàng
- `integrations`: Kết nối marketplace
- `payments`: Giao dịch thanh toán

## 🔐 Authentication & Authorization

- **Authentication**: JWT tokens (access + refresh)
- **Authorization**: Casbin-rs với multi-tenant RBAC
- **Tenant Isolation**: Tự động filter queries bằng `tenant_id`

## 🌐 API Documentation

Mỗi service expose OpenAPI spec tại endpoint `/api/docs`.

Ví dụ: `http://localhost:3000/api/docs` cho user-service.

## 📦 Deployment (CapRover)

### Local → CapRover

1. Cài đặt CapRover trên VPS của bạn: https://caprover.com/docs/get-started.html
2. Deploy các stateful services (PostgreSQL, Redis, NATS) qua One-Click Apps
3. Tạo `Dockerfile` cho mỗi microservice
4. Tạo app trong CapRover và kết nối với GitHub
5. Push code → CapRover tự động build & deploy

Chi tiết xem trong `TODO.md` - Phase 10.

## 🧪 Testing Strategy

- **Unit Tests**: `cargo test` - Coverage > 70%
- **Integration Tests**: Test API endpoints với test database
- **E2E Tests**: Playwright cho frontend
- **Load Tests**: K6 cho stress testing

## 📈 Monitoring & Observability

- **Logging**: `tracing` crate + OpenTelemetry
- **Metrics**: Prometheus + Grafana
- **Tracing**: Distributed tracing với Jaeger (optional)
- **Health Checks**: `/health` endpoint cho mỗi service

## 🤝 Contributing

1. Fork repository
2. Tạo feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Tạo Pull Request

### Code Style

- Rust: Chạy `cargo fmt` và `cargo clippy` trước khi commit
- TypeScript: Chạy `pnpm lint` và `pnpm format`
- Commit messages: Sử dụng conventional commits

## 📝 Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Chi tiết kiến trúc hệ thống
- [TODO.md](./TODO.md) - Danh sách công việc và tiến độ
- API Docs - OpenAPI spec tại mỗi service endpoint

## 📄 License

MIT License - Xem file `LICENSE` để biết thêm chi tiết.

## 👥 Team

- **Your Name** - Initial work

## 🙏 Acknowledgments

- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [SvelteKit](https://kit.svelte.dev/) - Frontend framework
- [CapRover](https://caprover.com/) - PaaS platform
- [Casbin](https://casbin.org/) - Authorization library
- [Cube](https://cube.dev/) - Analytics platform

---

**Trạng thái**: 🚧 Đang phát triển - Phase 1 (Thiết lập cơ sở hạ tầng)

**Mục tiêu MVP**: 2-3 tháng

Xem [TODO.md](./TODO.md) để theo dõi tiến độ chi tiết.
