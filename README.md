# 🐜 Anthill - Inventory SaaS Platform

**Anthill** is a modern multi-tenant inventory management SaaS platform built with **Rust** (backend microservices) and **SvelteKit 5** (frontend), optimized for e-commerce businesses.

> 🐜 Just like an anthill works efficiently and organized, Anthill helps you manage inventory intelligently and automatically.

[![codecov](https://codecov.io/gh/tymon3568/anthill/branch/master/graph/badge.svg)](https://codecov.io/gh/tymon3568/anthill)

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
- **Frontend**: SvelteKit 5 + TypeScript + Tailwind CSS + shadcn-svelte
- **Authentication**: Kanidm (OAuth2/OIDC Identity Provider)
- **Authorization**: Casbin-rs (RBAC)
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
├── frontend/                    # SvelteKit 5 application
│   ├── src/
│   │   ├── app.html            # Main HTML template
│   │   ├── app.d.ts            # TypeScript declarations
│   │   ├── lib/                # Shared libraries
│   │   │   ├── components/     # Reusable UI components (shadcn-svelte)
│   │   │   ├── stores/         # Svelte 5 runes state management
│   │   │   ├── utils/          # Utility functions
│   │   │   ├── auth/           # Authentication helpers
│   │   │   └── api/            # API client functions
│   │   └── routes/             # Page routes
│   ├── static/                 # Static assets
│   ├── package.json            # Dependencies and scripts
│   ├── svelte.config.js        # SvelteKit configuration
│   ├── vite.config.ts          # Vite build configuration
│   ├── tsconfig.json           # TypeScript configuration
│   ├── tailwind.config.js      # Tailwind CSS configuration
│   └── playwright.config.ts    # E2E testing configuration
├── shared/                      # Shared libraries
│   ├── auth/                   # Casbin RBAC, Kanidm integration
│   ├── config/                 # Environment config loader
│   ├── db/                     # Database utilities
│   ├── error/                  # Error types and HTTP responses
│   ├── jwt/                    # JWT encoding/decoding
│   ├── kanidm_client/          # Kanidm OAuth2/OIDC client
│   └── openapi/                # OpenAPI spec generation
├── infra/                       # Infrastructure config
│   ├── docker_compose/         # Local dev environment
│   ├── nginx/                  # API Gateway configuration
│   └── monitoring/             # Prometheus, Grafana, Loki setup
├── migrations/                  # Database migrations
├── scripts/                     # Utility scripts
├── docs/                        # Documentation
├── Cargo.toml                   # Rust workspace
├── ARCHITECTURE.md              # Architecture documentation
├── STRUCTURE.md                 # Code structure guide
├── TODO.md                      # Task list and progress
└── README.md                    # This file
```

## 🚀 Quick Start

### Prerequisites

- **Rust** (stable + nightly): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Bun**: `curl -fsSL https://bun.sh/install | bash` (fast JavaScript runtime and package manager for SvelteKit frontend - replaces Node.js)
- **Docker & Docker Compose**: For running local environment
- **PostgreSQL Client**: `psql` (optional, for debugging)

### 1. Install Development Tools

```bash
# Install Rust toolchain
rustup default stable
rustup toolchain add nightly
rustup component add clippy rustfmt

# Install cargo tools
cargo install cargo-watch        # Auto-reload
cargo install sqlx-cli --features postgres  # DB migrations

# Install Bun (if not already installed)
curl -fsSL https://bun.sh/install | bash
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

### 4. Setup Frontend (SvelteKit)

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
bun install

# Start development server
bun dev

# In another terminal, run backend services...
```

### 4. Build & Run Backend Services

```bash
# Return to root directory
cd ..

# Build all services
cargo build --workspace

# Run user-service (default port 3000, configurable via PORT env var)
cargo run --bin user-service

# In another terminal, run inventory-service (port 8001 as per nginx config)
PORT=8001 cargo run --bin inventory-service

# Or run multiple services with different ports:
PORT=8000 cargo run --bin user-service &
PORT=8001 cargo run --bin inventory-service &
PORT=8002 cargo run --bin order-service &

# And continue with other services...
```

### 5. Setup Database

```bash
# Run database migrations
sqlx migrate run --database-url postgres://inventory_user:inventory_pass@localhost:5432/inventory_saas

# Verify schema
psql postgres://inventory_user:inventory_pass@localhost:5432/inventory_saas -c "\dt"
```

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

### Frontend (SvelteKit)

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
bun install

# Start development server
bun dev

# Build for production
bun run build

# Preview production build
bun run preview

# Run unit tests (Vitest)
bun run test:unit

# Run E2E tests (Playwright)
bun run test:e2e

# Format code
bun run format

# Lint code
bun run lint
```

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

## 📊 Database Schema

See details in `migrations/`. Main tables:

- `tenants`: Tenant information
- `users`: Users within each tenant
- `products`: Products
- `inventory_levels`: Inventory levels
- `orders`: Orders
- `integrations`: Marketplace integrations
- `payments`: Payment transactions
- `casbin_rule`: RBAC policies

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

See details in `docs/production-deployment.md`

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

See details in `docs/monitoring-setup.md`

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
- Follow existing patterns in the codebase
- Write tests for new features
- Update documentation as needed
- Commit messages: Use conventional commits (e.g., `feat:`, `fix:`, `chore:`)

## 📝 Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Detailed system architecture
- [STRUCTURE.md](./STRUCTURE.md) - Code structure and patterns
- [TODO.md](./TODO.md) - Task list and progress tracking
- [docs/production-deployment.md](./docs/production-deployment.md) - Production deployment guide
- [docs/monitoring-setup.md](./docs/monitoring-setup.md) - Monitoring setup guide
- [docs/troubleshooting.md](./docs/troubleshooting.md) - Troubleshooting guide
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

**Status**: 🚧 In Development - Phase 3 (User Service Production Integration)

**MVP Target**: 2-3 months

See [TODO.md](./TODO.md) for detailed progress tracking.
