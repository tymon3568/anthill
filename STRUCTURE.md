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
│   │   │   │           │   └── auth_dto.rs   # API contracts
│   │   │   │           └── infra/  # (empty - impl in infra crate)
│   │   │   └── Cargo.toml
│   │   └── infra/                   # Library - Infrastructure
│   │       ├── src/
│   │       │   ├── lib.rs
│   │       │   └── auth/
│   │       │       ├── repository.rs # PostgreSQL implementations
│   │       │       └── service.rs    # Business logic implementations
│   │       └── Cargo.toml
│   │
│   ├── inventory_service/           # TODO: Refactor to 3-crate
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
│   ├── auth/                        # Authentication & Authorization
│   │   ├── src/
│   │   │   ├── lib.rs              # Re-exports
│   │   │   ├── casbin/             # Casbin RBAC (KEPT)
│   │   │   ├── extractors.rs      # AuthUser extractor (uses Kanidm JWT)
│   │   │   └── kanidm.rs           # Kanidm JWT validation (NEW)
│   │   └── Cargo.toml
│   ├── kanidm_client/               # Kanidm OAuth2/OIDC client (NEW)
│   │   ├── src/lib.rs              # OAuth2 flow, token validation
│   │   └── Cargo.toml
│   └── openapi/                     # OpenAPI specs (CI/CD exports)
│       ├── user.yaml               # User service spec
│       └── README.md
│
├── infra/                           # Infrastructure
│   ├── migrations/                  # Database migrations (sqlx)
│   └── docker_compose/              # Docker compose configs
│
├── Cargo.toml                       # Workspace configuration
├── Cargo.lock
├── .env.example                     # TODO: Environment variables template
├── README.md                        # Project documentation
├── WARP.md                          # Development rules & best practices
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
export DATABASE_URL="postgresql://localhost/anthill"
export KANIDM_URL="https://idm.example.com"
export KANIDM_OAUTH2_CLIENT_ID="anthill"
export KANIDM_OAUTH2_CLIENT_SECRET="your-client-secret"
export HOST="0.0.0.0"
export PORT=3000

# 2. Run migrations (TODO)
# sqlx migrate run

# 3. Start service
cargo run --bin user-service

# 4. Access API
curl http://localhost:3000/health
open http://localhost:3000/docs  # Swagger UI

# 5. OAuth2 login flow
# Redirect to: https://idm.example.com/ui/oauth2?client_id=anthill&...
```

## 📦 Workspace Members

### Services:
- `services/user_service/api` (binary: user-service)
- `services/user_service/core`
- `services/user_service/infra`

### Shared Libraries:
- `shared/error` - Error handling
- `shared/config` - Configuration
- `shared/types` - Common types
- `shared/db` - Database utilities
- `shared/auth` - Casbin RBAC + Kanidm JWT validation
- `shared/kanidm_client` - Kanidm OAuth2/OIDC integration

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

# Export OpenAPI specs
cargo build --features export-spec
```

## 📚 Tech Stack

- **Framework**: Axum 0.8
- **Database**: PostgreSQL (via sqlx)
- **Auth**: Kanidm (OAuth2/OIDC) + Casbin (RBAC)
- **Token Validation**: JWT (Kanidm-issued)
- **API Docs**: OpenAPI 3.0 (utoipa)
- **Validation**: validator
- **Logging**: tracing + tracing-subscriber
- **Async Runtime**: tokio

## 🎯 Next Steps

1. ✅ User service (3-crate) - **DONE**
2. ⏳ Database migrations for user service
3. ⏳ Refactor other services to 3-crate pattern
4. ⏳ Add authentication middleware
5. ⏳ Integration tests
6. ⏳ CI/CD pipeline
7. ⏳ Docker containerization

---

**Last Updated**: 2025-01-09
**Status**: User service production-ready, others pending refactor
