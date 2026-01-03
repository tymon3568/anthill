# Anthill Project Structure

## 📁 Overview

Production-ready microservices architecture using Rust with clean 3-crate pattern.

## 🏗️ Directory Structure

```
anthill/
├── services/                         # Microservices
│   ├── user_service/                # User authentication & management (3-crate)
│   │   ├── api/                     # Binary - HTTP layer (Axum)
│   │   │   ├── src/
│   │   │   │   ├── main.rs         # Bootstrap + dependency injection
│   │   │   │   ├── handlers.rs     # HTTP request handlers
│   │   │   │   ├── admin_handlers.rs # Admin role/permission APIs
│   │   │   │   ├── profile_handlers.rs # User profile APIs
│   │   │   │   ├── extractors.rs   # Request extractors
│   │   │   │   └── openapi.rs      # OpenAPI documentation
│   │   │   └── Cargo.toml
│   │   ├── core/                    # Library - Business logic
│   │   │   ├── src/
│   │   │   │   ├── lib.rs
│   │   │   │   ├── domains.rs
│   │   │   │   └── domains/
│   │   │   │       └── auth/       # Auth feature domain
│   │   │   │           ├── api/    # (empty - handlers in api crate)
│   │   │   │           ├── domain/ # Entities + traits
│   │   │   │           │   ├── model.rs      # User, Tenant entities
│   │   │   │           │   ├── repository.rs # Repo traits
│   │   │   │           │   └── service.rs    # Service traits
│   │   │   │           ├── dto/    # Data transfer objects
│   │   │   │           │   ├── auth_dto.rs   # Auth API contracts
│   │   │   │           │   └── admin_dto.rs  # Admin API contracts
│   │   │   │           └── infra/  # (empty - impl in infra crate)
│   │   │   └── Cargo.toml
│   │   └── infra/                   # Library - Infrastructure
│   │       ├── src/
│   │       │   ├── lib.rs
│   │       │   └── auth/
│   │       │       ├── repository.rs         # PostgreSQL user repo
│   │       │       ├── service.rs            # Auth service impl
│   │       │       ├── session_repository.rs # Session management
│   │       │       ├── profile_repository.rs # Profile repo
│   │       │       └── profile_service.rs    # Profile service impl
│   │       └── Cargo.toml
│   │
│   ├── inventory_service/           # Inventory management (3-crate) - DONE
│   │   ├── api/                     # HTTP handlers, routes
│   │   ├── core/                    # Domain models, traits
│   │   └── infra/                   # Repository implementations
│   │
│   ├── order_service/               # TODO: Refactor to 3-crate
│   ├── payment_service/             # TODO: Refactor to 3-crate
│   └── integration_service/         # TODO: Refactor to 3-crate
│
├── shared/                          # Shared libraries (DRY)
│   ├── error/                       # Common error types
│   │   ├── src/lib.rs              # AppError + IntoResponse
│   │   └── Cargo.toml
│   ├── config/                      # Configuration loading
│   │   ├── src/lib.rs              # Config struct + from_env
│   │   └── Cargo.toml
│   ├── types/                       # Common types
│   │   ├── src/lib.rs              # Re-exports (Uuid, DateTime, etc.)
│   │   └── Cargo.toml
│   ├── db/                          # Database utilities
│   │   ├── src/lib.rs              # init_pool function
│   │   └── Cargo.toml
│   ├── jwt/                         # JWT utilities
│   │   ├── src/lib.rs              # Token generation & validation
│   │   └── Cargo.toml
│   ├── auth/                        # Authentication & Authorization
│   │   ├── src/
│   │   │   ├── lib.rs              # Re-exports
│   │   │   ├── casbin/             # Casbin RBAC
│   │   │   │   ├── mod.rs
│   │   │   │   ├── enforcer.rs     # Casbin enforcer setup
│   │   │   │   └── adapter.rs      # PostgreSQL adapter
│   │   │   ├── extractors.rs       # AuthUser, RequireAdmin extractors
│   │   │   └── middleware.rs       # Auth middleware
│   │   └── Cargo.toml
│   ├── events/                      # Event publishing (NATS)
│   │   ├── src/lib.rs
│   │   └── Cargo.toml
│   └── openapi/                     # OpenAPI specs (CI/CD exports)
│       ├── user.yaml               # User service spec
│       ├── inventory.yaml          # Inventory service spec
│       └── README.md
│
├── frontend/                        # SvelteKit frontend
│   ├── src/
│   │   ├── routes/                 # SvelteKit file-based routing
│   │   ├── lib/
│   │   │   ├── api/               # API client modules
│   │   │   ├── components/        # Reusable components
│   │   │   ├── stores/            # Svelte stores
│   │   │   └── types/             # TypeScript types
│   │   └── app.html
│   ├── package.json
│   └── svelte.config.js
│
├── infra/                           # Infrastructure
│   └── docker_compose/              # Docker compose configs
│       └── docker-compose.yml      # Dev environment
│
├── migrations/                      # Database migrations (sqlx)
│   ├── 20250110000001_initial.sql
│   ├── 20250110000002_users.sql
│   └── ...
│
├── scripts/                         # Utility scripts
│   ├── setup-integration-test.sh
│   └── test-tenant-context.sh
│
├── Cargo.toml                       # Workspace configuration
├── Cargo.lock
├── .env.example                     # Environment variables template
├── README.md                        # Project documentation
├── ARCHITECTURE.md                  # System architecture docs
└── STRUCTURE.md                     # This file
```

## 🎯 Design Principles

### 3-Crate Pattern (per service):

1. **API Crate** (binary):
   - HTTP handlers
   - Routing
   - Middleware
   - OpenAPI documentation
   - Application bootstrap

2. **Core Crate** (library):
   - Business logic traits
   - Domain entities
   - DTOs (API contracts)
   - Pure business rules
   - Zero dependencies on infrastructure

3. **Infra Crate** (library):
   - Repository implementations (Database)
   - Service implementations (Business logic)
   - External API clients
   - Infrastructure concerns

### Dependency Flow:
```
api ──> infra ──> core ──> shared/*
```

✅ **Benefits:**
- Clear separation of concerns
- Testable business logic (core)
- Reusable infrastructure (shared)
- Incremental compilation
- No circular dependencies

## 🚀 Running Services

### User Service:

```bash
# 1. Setup environment
export DATABASE_URL="postgresql://user:password@localhost:5432/inventory_db"
export JWT_SECRET="your-jwt-secret-min-32-chars"
export JWT_EXPIRATION=900           # 15 minutes
export JWT_REFRESH_EXPIRATION=604800 # 7 days
export HOST="0.0.0.0"
export PORT=8000

# 2. Start database
docker compose -f infra/docker_compose/docker-compose.yml up -d postgres

# 3. Run migrations
sqlx migrate run

# 4. Start service
cargo run --bin user-service

# 5. Access API
curl http://localhost:8000/health
open http://localhost:8000/docs  # Swagger UI
```

### Example: Register & Login

```bash
# Register new user + tenant
curl -X POST http://localhost:8000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@acme.com",
    "password": "SecureP@ss123",
    "full_name": "John Doe",
    "tenant_name": "ACME Corp"
  }'

# Login
curl -X POST http://localhost:8000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: acme-corp" \
  -d '{
    "email": "admin@acme.com",
    "password": "SecureP@ss123"
  }'
```

## 📦 Workspace Members

### Services:
- `services/user_service/api` (binary: user-service)
- `services/user_service/core`
- `services/user_service/infra`
- `services/inventory_service/api` (binary: inventory-service)
- `services/inventory_service/core`
- `services/inventory_service/infra`

### Shared Libraries:
- `shared/error` - Error handling (AppError)
- `shared/config` - Configuration loading
- `shared/types` - Common types (Uuid, DateTime)
- `shared/db` - Database utilities (connection pool)
- `shared/jwt` - JWT token generation & validation
- `shared/auth` - Casbin RBAC + auth extractors
- `shared/events` - NATS event publishing

## 🔧 Development Workflow

```bash
# Check all crates
cargo check --workspace

# Build all services
cargo build --workspace

# Run specific service
cargo run --bin user-service

# Test all crates
cargo test --workspace

# Lint with clippy
cargo clippy --workspace

# Export OpenAPI specs
cargo build --features export-spec
```

## 📚 Tech Stack

| Category | Technology |
|----------|------------|
| **Framework** | Axum 0.8 |
| **Database** | PostgreSQL 16 (via sqlx) |
| **Authentication** | Email/Password + bcrypt + JWT |
| **Authorization** | Casbin-rs (RBAC) |
| **Password Strength** | zxcvbn |
| **API Docs** | OpenAPI 3.0 (utoipa) |
| **Validation** | validator |
| **Logging** | tracing + tracing-subscriber |
| **Async Runtime** | tokio |
| **Frontend** | SvelteKit 2 + Svelte 5 |
| **Cache** | Redis |
| **Message Queue** | NATS |

## 🔐 Authentication

The platform uses **self-built email/password authentication**:

- **Password Hashing**: bcrypt with cost factor 12
- **Password Validation**: zxcvbn for strength scoring
- **Token Format**: JWT (access + refresh tokens)
- **Session Storage**: PostgreSQL sessions table
- **Token Expiration**: Access 15min, Refresh 7 days

**Auth Extractors** (from `shared/auth`):
- `AuthUser` - Extract authenticated user from JWT
- `RequireAdmin` - Require admin role
- `RequirePermission` - Casbin permission check

## 🎯 Next Steps

1. ✅ User service (3-crate) - **DONE**
2. ✅ Inventory service (3-crate) - **DONE**
3. ✅ Database migrations - **DONE**
4. ✅ Authentication middleware - **DONE**
5. ⏳ Email verification flow
6. ⏳ Password reset flow
7. ⏳ Rate limiting
8. ⏳ Refactor remaining services to 3-crate pattern
9. ⏳ Integration tests
10. ⏳ CI/CD pipeline
11. ⏳ Docker containerization

---

**Last Updated**: 2026-01-04
**Status**: User + Inventory services production-ready
