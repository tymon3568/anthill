# WARP.md - Anthill Project Guide

## 🎯 Project Overview

**Anthill** is a multi-tenant inventory management SaaS platform built with:
- **Backend**: Rust microservices (Axum 0.8 + Tokio + SQLx)
- **Frontend**: SvelteKit 5 + TypeScript (TODO)
- **Database**: PostgreSQL with multi-tenant isolation  
- **Architecture**: Clean Architecture + 3-Crate Pattern
- **Status**: **Phase 3 - User Service Production Ready (~25% complete)**

## 📁 Project Structure (Production Pattern)

### ✅ **Current Implementation (User Service)**

```
anthill/
├── services/
│   └── user_service/              ✅ PRODUCTION READY (3-crate pattern)
│       ├── api/                   # Binary crate - HTTP layer
│       │   ├── src/
│       │   │   ├── main.rs       # Bootstrap + DI
│       │   │   ├── handlers.rs   # HTTP handlers (generic over service trait)
│       │   │   └── openapi.rs    # OpenAPI 3.0 spec
│       │   └── Cargo.toml
│       ├── core/                  # Library - Business logic
│       │   ├── src/
│       │   │   ├── lib.rs
│       │   │   └── domains/      
│       │   │       └── auth/     # Auth domain
│       │   │           ├── api/        (empty - in api crate)
│       │   │           ├── domain/     Entity models + traits
│       │   │           │   ├── model.rs          (User, Tenant)
│       │   │           │   ├── repository.rs     (trait)
│       │   │           │   └── service.rs        (trait)
│       │   │           ├── dto/        API contracts
│       │   │           │   └── auth_dto.rs       (DTOs with utoipa)
│       │   │           └── infra/      (empty - in infra crate)
│       │   └── Cargo.toml
│       └── infra/                 # Library - Infrastructure
│           ├── src/
│           │   └── auth/
│           │       ├── repository.rs   # PostgreSQL impl
│           │       └── service.rs      # Business logic impl
│           └── Cargo.toml
│
├── shared/                        ✅ SHARED LIBRARIES (DRY)
│   ├── error/      # AppError + IntoResponse
│   ├── jwt/        # JWT encode/decode + Claims
│   ├── config/     # Environment config loader
│   ├── types/      # Common types (Uuid, DateTime)
│   ├── db/         # DB pool initialization
│   └── openapi/    # OpenAPI specs (CI/CD exports)
│
├── infra/                         # Infrastructure
│   ├── docker_compose/           # Docker compose configs
│   └── migrations/               # Database migrations (TODO)
│
└── services/ (other services - TODO refactor to 3-crate):
    ├── inventory_service/
    ├── order_service/
    ├── payment_service/
    └── integration_service/
```

## 🏗️ Architecture Principles

### **3-Crate Pattern (per service)**

**Dependency Flow**: `api → infra → core → shared/*`

#### 1. **API Crate** (binary)
- HTTP handlers with Axum
- Routing & middleware
- OpenAPI documentation  
- Application bootstrap
- **Generic over service traits** (testable!)

#### 2. **Core Crate** (library)
- Domain entities & DTOs
- Business logic traits (repository, service)
- **Zero infrastructure dependencies**
- Pure business rules
- Multi-tenant domain models

#### 3. **Infra Crate** (library)
- Repository implementations (PostgreSQL via SQLx)
- Service implementations (business logic)
- External API clients
- JWT, password hashing
- Infrastructure concerns

### **Benefits**
✅ Clear separation of concerns  
✅ Testable business logic (core independent)  
✅ Reusable infrastructure (shared crates)  
✅ Incremental compilation  
✅ No circular dependencies  
✅ Easy to refactor & maintain

## 📚 Tech Stack

### **Backend**
- **Framework**: Axum 0.8 (async HTTP)
- **Runtime**: Tokio (async)
- **Database**: PostgreSQL + SQLx (compile-time checked queries)
- **Auth**: JWT (jsonwebtoken) + bcrypt
- **Validation**: validator crate
- **API Docs**: OpenAPI 3.0 (utoipa + Swagger UI)
- **Logging**: tracing + tracing-subscriber
- **Error Handling**: thiserror + custom AppError

### **Frontend** (TODO)
- SvelteKit 5 + TypeScript
- Tailwind CSS

### **Infrastructure**
- Docker + Docker Compose
- PostgreSQL 15+
- Redis (TODO - caching)
- NATS (TODO - event bus)

## 🚀 Quick Start

### **Prerequisites**
```bash
rustup default stable
rustup component add clippy rustfmt
cargo install sqlx-cli --features postgres
```

### **Environment Setup**
```bash
# Required environment variables
export DATABASE_URL="postgresql://localhost/anthill"
export JWT_SECRET="your-secret-key-here"
export JWT_EXPIRATION=900              # 15 minutes
export JWT_REFRESH_EXPIRATION=604800   # 7 days
export HOST="0.0.0.0"
export PORT=3000
```

### **Run User Service**
```bash
# Check compilation
cargo check --workspace

# Run service
cargo run --bin user-service

# Export OpenAPI spec
cargo build --bin user-service --features export-spec

# Access API
curl http://localhost:8000/health
open http://localhost:8000/docs    # Swagger UI
```

### **Development Commands**
```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --all -- -D warnings

# Run tests
cargo test --workspace

# Build all
cargo build --workspace --release
```

## 🎯 Current Status & Next Steps

### ✅ **Completed (Phase 1-3)**
1. ✅ Project structure with 3-crate pattern
2. ✅ Shared libraries (error, jwt, config, types, db)
3. ✅ User service fully implemented:
   - ✅ Registration (with tenant creation)
   - ✅ Login (TODO: tenant resolution)
   - ✅ Refresh token
   - ✅ JWT authentication
   - ✅ Password hashing (bcrypt)
   - ✅ Input validation
   - ✅ OpenAPI documentation
   - ✅ Swagger UI
4. ✅ Snake_case naming convention
5. ✅ GitHub Actions workflows
6. ✅ Comprehensive documentation

### 🔄 **In Progress (Phase 3-4)**
- [ ] Database migrations for user_service
- [ ] Auth middleware (JWT validation)
- [ ] Tenant resolution for login
- [ ] Integration tests

### ⏳ **TODO (Phase 4+)**
1. Refactor other services to 3-crate pattern:
   - inventory_service
   - order_service
   - payment_service
   - integration_service
2. Implement inventory service (core domain)
3. Event-driven architecture (NATS)
4. Frontend with SvelteKit 5
5. Deployment (Docker + CapRover)

## 📝 Development Guidelines

### **Naming Conventions**
- **Crates**: `snake_case` (e.g., `user_service_core`)
- **Directories**: `snake_case` (e.g., `services/user_service/`)
- **Binary names**: `kebab-case` OK (e.g., `user-service`)
- **Rust code**: Follow Rust RFC 430

### **Code Organization**
- **Domains** = Features (e.g., `auth`, `user`, `tenant`)
- **Each domain** has 4 layers:
  - `api/` - HTTP handlers & routes
  - `domain/` - Entities, traits, business rules
  - `dto/` - Data transfer objects (API contracts)
  - `infra/` - Implementations (DB, external APIs)

### **Error Handling**
- Use `shared_error::AppError` everywhere
- Implements `IntoResponse` for Axum
- Variants: Unauthorized, ValidationError, DatabaseError, etc.
- Never `unwrap()` or `panic!()` in handlers

### **Multi-Tenancy**
- Every query MUST filter by `tenant_id`
- Use Row-Level Security (RLS) in PostgreSQL
- JWT contains `tenant_id` claim
- All entities have `tenant_id` field

### **Testing**
- Unit tests in `core/` (business logic)
- Integration tests in `api/tests/`
- Test tenant isolation!
- Mock services for API tests

## 🔗 Important Links

- **GitHub**: https://github.com/tymon3568/anthill
- **Clean Axum Demo** (reference): https://github.com/sukjaelee/clean_axum_demo
- **Swagger UI** (local): http://localhost:8000/docs

## 📚 Additional Documentation

- `STRUCTURE.md` - Detailed project structure
- `ARCHITECTURE.md` - System architecture & deployment
- `TODO.md` - Detailed task breakdown
- `README.md` - Project overview
- `Cargo.toml` - Workspace configuration

---

**Last Updated**: 2025-01-09  
**Author**: tymon3568 <tymon3568@gmail.com>  
**License**: MIT
