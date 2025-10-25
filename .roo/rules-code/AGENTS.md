# .roo/rules-code/AGENTS.md

## 💻 Project Coding Rules (Non-Obvious Only)

### 🏗️ Architecture Patterns (Critical)

**3-Crate Structure** (services/user_service/ only):
- ✅ `api/` crate: HTTP handlers, routing, OpenAPI, main.rs bootstrap
- ✅ `core/` crate: Domain entities, DTOs, service/repository traits only
- ✅ `infra/` crate: PostgreSQL implementations, business logic implementations
- ❌ Other services: Must refactor to 3-crate pattern before implementing

**Dependency Flow** (enforce strictly):
```rust
// CORRECT: api → infra → core → shared/*
api/handlers.rs → infra/service.rs → core/domains/ → shared/error
// AVOID: Skip layers or create circular dependencies
```

### 💰 Data Handling (Financial & Critical)

**Currency Pattern**:
- Always use `i64` cents, never `f64` for money
- Import: `use shared_types::Money` (custom type)
- Pattern: `Money::from_dollars(10.50)` → stores 1050 cents

**UUID Generation**:
- Always use `Uuid::now_v7()` (timestamp-based)
- Never use `Uuid::new_v4()` for primary keys
- Import: `use uuid::Uuid`

**Soft Delete Implementation**:
- Add `deleted_at TIMESTAMPTZ` to all entities
- Repository methods must check `WHERE deleted_at IS NULL`
- Use partial indexes: `CREATE INDEX ... WHERE deleted_at IS NULL`

### 🔐 Security Patterns (Authentication & Authorization)

**Multi-Tenancy Implementation**:
- EVERY query must include `WHERE tenant_id = $1`
- RLS handles database-level filtering automatically
- Casbin handles application-level authorization
- Manual tenant_id checks as defense in depth

**JWT Handling**:
- Use `shared/jwt` crate for token operations
- Extractors from `shared/auth`: `AuthUser`, `RequireAdmin`, `RequirePermission`
- Token claims must include: `sub` (user_id), `tenant_id`, `role`

**Password Security**:
- Use `bcrypt` crate (not Argon2id yet, but TODO)
- Password strength: zxcvbn Score 3+ required
- Context-aware validation (check against email, name, tenant)

### 🛠️ Development Patterns (Project-Specific)

**Error Handling**:
- Use `shared/error::AppError` everywhere
- Implements `IntoResponse` for Axum automatically
- Never `unwrap()` or `panic!()` in handlers
- Proper error variants: `Unauthorized`, `ValidationError`, `DatabaseError`

**Database Operations**:
- Use SQLx for compile-time checked queries
- Repository pattern: Define traits in core, implement in infra
- Connection pooling: `shared/db` provides `init_pool()`
- Transactions for multi-table operations

**OpenAPI Documentation**:
- Use `utoipa` crate for API documentation
- DTOs must have `#[derive(ToSchema)]`
- Export specs with `cargo build --features export-spec`

## 🚨 Critical Coding Gotchas

**Service Implementation Order**:
1. Start with `core/` crate (traits and models)
2. Implement `infra/` crate (database and logic)
3. Create `api/` crate (handlers and routing)
4. Test each layer independently

**Testing Requirements**:
- Unit tests in `core/` (no database needed)
- Integration tests in `api/tests/` (require DATABASE_URL)
- Mock services for API testing without external dependencies

**Import Organization**:
```rust
// 1. Standard library
use std::collections::HashMap;

// 2. External crates (alphabetical)
use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// 3. Workspace crates (alphabetical)
use shared_db::init_pool;
use shared_error::AppError;
use shared_types::{Money, TenantId};

// 4. Local modules (alphabetical)
use crate::domains::auth;