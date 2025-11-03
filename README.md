# 🐜 Anthill - Inventory SaaS Platform

**Anthill** is a modern multi-tenant inventory management SaaS platform built with **Rust** (backend microservices) and **SvelteKit 5** (frontend), optimized for e-commerce businesses.

> 🐜 Just like an anthill works efficiently and organized, Anthill helps you manage inventory intelligently and automatically.

## 🎯 Key Features

- ✅ **Multi-tenant Architecture**: Support multiple tenants on the same infrastructure
- ✅ **Real-time Inventory Tracking**: Update inventory in real-time
- ✅ **Marketplace Integration**: Connect with Shopee, Lazada, Tiki, WooCommerce, Shopify
- ✅ **Order Management**: Manage orders from multiple channels
- ✅ **Payment Gateway**: Integrate with VNPay, Stripe, MoMo, ZaloPay
- ✅ **Analytics & Reporting**: Analytics dashboard with Cube
- ✅ **Zero Downtime Deployment**: Continuous deployment without interruption

## 🏗️ Architecture

The project uses **Event-Driven Microservices** architecture with the following technologies:

- **Backend**: Rust + Axum + Tokio + SQLx
- **Authentication**: Kanidm (OAuth2/OIDC Identity Provider)
- **Authorization**: Casbin-rs (RBAC)
- **Frontend**: SvelteKit 5 + TypeScript + Tailwind CSS
- **Database**: PostgreSQL
- **Cache**: Redis
- **Message Queue**: NATS
- **Analytics**: Cube
- **Deployment**: CapRover (Docker Swarm)
- **Gateway**: NGINX (managed by CapRover)

See architecture details at [ARCHITECTURE.md](./ARCHITECTURE.md)

## 📁 Project Structure

```bash
anthill/
├── services/                    # Rust microservices
│   ├── user-service/           # Authentication & tenancy
│   ├── inventory-service/      # Inventory management
│   ├── order-service/          # Order management
│   ├── integration-service/    # Marketplace integration
│   └── payment-service/        # Payment processing
├── shared/                      # Shared libraries
│   ├── common/                 # Error types, config, tracing
│   ├── db/                     # Database utilities
│   ├── auth/                   # Casbin RBAC, Kanidm integration
│   ├── kanidm_client/          # Kanidm OAuth2/OIDC client
│   └── events/                 # Event definitions, NATS client
├── frontend/                    # SvelteKit application
├── infra/                       # Infrastructure config
│   ├── docker-compose/         # Local dev environment
│   └── sql-migrations/         # Database migrations
├── Cargo.toml                   # Rust workspace
├── ARCHITECTURE.md              # Architecture documentation
├── TODO.md                      # Task list
└── README.md                    # This file
```

## 🚀 Quick Start

### Prerequisites

- **Rust** (stable + nightly): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Docker & Docker Compose**: For running local environment
- **Node.js** (>= 20) & **pnpm**: For frontend
- **PostgreSQL Client**: `psql` (optional, for debugging)

### 1. Install Rust Tools

```bash
# Install Rust toolchain
rustup default stable
rustup toolchain add nightly
rustup component add clippy rustfmt

# Install cargo tools
cargo install cargo-watch        # Auto-reload
cargo install sqlx-cli --features postgres  # DB migrations
cargo install cargo-make         # Task runner
```

### 2. Start Local Environment

```bash
# Clone repository
git clone <your-repo-url>
cd anthill

# Start PostgreSQL, Redis, NATS
docker-compose up -d

# Return to root directory
```

### 3. Build & Run Backend Services

```bash
# Build all services
cargo build --workspace

# Run user-service (port 3000)
cargo run -p user-service

# In another terminal, run inventory-service (port 3001)
cargo run -p inventory-service

# And continue with other services...
```

### 4. Setup Frontend (SvelteKit)

```bash
cd frontend

# Install dependencies
pnpm install

# Run dev server
pnpm dev
```

Access: `http://localhost:5173`

## 🛠️ Development Commands

### Pre-commit Hooks (Recommended)

This project uses pre-commit hooks to automatically check code quality before commits.

```bash
# Install pre-commit (one-time setup)
pipx install pre-commit

# Install git hooks (run in project root)
pre-commit install

# Run hooks manually on all files
pre-commit run --all-files

# Skip hooks for a specific commit (use sparingly)
git commit --no-verify
```

**What the hooks do:**
- ✅ Format Rust code with `cargo fmt`
- ✅ Lint Rust code with `cargo clippy` (blocks commits with warnings)
- ✅ Check YAML syntax
- ✅ Trim trailing whitespace
- ✅ Fix end-of-file issues
- ✅ Format TOML files
- ✅ Prevent large files from being committed

### Backend (Rust)

```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --all -- -D warnings

# Run tests
cargo test --workspace

# Run specific service with auto-reload
cargo watch -x 'run -p user-service'

# Check code without building
cargo check --workspace
```

### Database Migrations

```bash
# Run migrations
sqlx migrate run --database-url postgres://user:password@localhost:5432/inventory_db

# Create new migration
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

See details in `infra/sql-migrations/`. Main tables:

- `tenants`: Tenant information
- `users`: Users within each tenant
- `products`: Products
- `inventory_levels`: Inventory levels
- `orders`: Orders
- `integrations`: Marketplace integrations
- `payments`: Payment transactions

## 🔐 Authentication & Authorization

- **Authentication**: Kanidm (OAuth2/OIDC Provider)
  - User registration, login, password management
  - Multi-factor authentication (Passkeys, WebAuthn, TOTP)
  - JWT token issuance and validation
  - Session management
- **Authorization**: Casbin-rs with multi-tenant RBAC
  - Policy-based access control
  - Group-based role mapping from Kanidm
- **Tenant Isolation**: Automatically filter queries by `tenant_id` from Kanidm groups

## 🌐 API Documentation

Each service exposes OpenAPI spec at `/api/docs` endpoint.

Example: `http://localhost:3000/api/docs` for user-service.

## 📦 Deployment (CapRover)

### Local → CapRover

1. Install CapRover on your VPS: https://caprover.com/docs/get-started.html
2. Deploy stateful services (PostgreSQL, Redis, NATS) via One-Click Apps
3. Create `Dockerfile` for each microservice
4. Create app in CapRover and connect with GitHub
5. Push code → CapRover automatically builds & deploys

See details in `TODO.md` - Phase 10.

## 🧪 Testing Strategy

- **Unit Tests**: `cargo test` - Coverage > 70%
- **Integration Tests**: Test API endpoints with test database
- **E2E Tests**: Playwright for frontend
- **Load Tests**: K6 for stress testing

## 📈 Monitoring & Observability

- **Logging**: `tracing` crate + OpenTelemetry
- **Metrics**: Prometheus + Grafana
- **Tracing**: Distributed tracing with Jaeger (optional)
- **Health Checks**: `/health` endpoint for each service

## 🤝 Contributing

1. Fork repository
2. Create feature branch: `git checkout -b feature/amazing-feature`
3. Install pre-commit hooks: `pre-commit install`
4. Make your changes (hooks will run automatically on commit)
5. Commit changes: `git commit -m 'Add amazing feature'`
6. Push to branch: `git push origin feature/amazing-feature`
7. Create Pull Request

### Code Style

- Rust: Pre-commit hooks will automatically run `cargo fmt` and `cargo clippy`
- TypeScript/Svelte: Run `pnpm lint` before committing
- Follow existing patterns in the codebase
- Write tests for new features
- Update documentation as needed
- Commit messages: Use conventional commits (e.g., `feat:`, `fix:`, `chore:`)

## 📝 Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Detailed system architecture
- [TODO.md](./TODO.md) - Task list and progress
- API Docs - OpenAPI spec at each service endpoint

## 📄 License

MIT License - See `LICENSE` file for more details.

## 👥 Team

- **Your Name** - Initial work

## 🙏 Acknowledgments

- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [SvelteKit](https://kit.svelte.dev/) - Frontend framework
- [Kanidm](https://kanidm.com/) - Identity management platform
- [CapRover](https://caprover.com/) - PaaS platform
- [Casbin](https://casbin.org/) - Authorization library
- [Cube](https://cube.dev/) - Analytics platform

---

**Status**: 🚧 In Development - Phase 1 (Infrastructure Setup)

**MVP Target**: 2-3 months

See [TODO.md](./TODO.md) for detailed progress tracking.
