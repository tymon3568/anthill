# Anthill - Inventory SaaS Platform

Multi-tenant inventory management SaaS built with Rust microservices and event-driven architecture.

## Architecture Overview

### 3-Crate Service Pattern
Every service follows strict separation:
- **`api/`**: Axum HTTP handlers, routing, OpenAPI docs, binary entry point
- **`core/`**: Business logic traits, domain entities, DTOs - **zero infrastructure deps**
- **`infra/`**: PostgreSQL repositories, service implementations, external clients

**Dependency flow**: `api → infra → core → shared/*`

Example: `user_service_api` depends on `user_service_infra`, which depends on `user_service_core`.

### Shared Libraries
Located in `shared/`:
- `error`: `AppError` enum with `IntoResponse`, standardized error codes
- `jwt`: `encode_jwt()`, `decode_jwt()`, `Claims` struct
- `config`: Environment config loader (`Config::from_env()`)
- `db`: `init_pool()` for PostgreSQL connection pooling
- `auth`: Casbin enforcer, JWT middleware, auth extractors (`AuthUser`, `RequireAdmin`)

All services **must** use shared crates instead of duplicating code.

## Multi-Tenancy Implementation

### Database Schema Rules
Every tenant-scoped table:
1. **MUST** have `tenant_id UUID NOT NULL`
2. Use composite indexes: `(tenant_id, <other_columns>)` for performance
3. Include `tenant_id` in composite foreign keys:
   ```sql
   FOREIGN KEY (tenant_id, product_id) REFERENCES products(tenant_id, product_id)
   ```

### Application-Level Isolation
**We use application filtering, NOT Postgres RLS.**

All queries through Repository layer must include `tenant_id`:
```rust
pub async fn find_by_id(&self, ctx: &TenantContext, id: Uuid) -> Result<Product> {
    sqlx::query_as!(Product,
        "SELECT * FROM products WHERE tenant_id = $1 AND product_id = $2",
        ctx.tenant_id, id
    )
    .fetch_one(&self.pool)
    .await
}
```

Extract `tenant_id` from JWT in middleware, inject into request context.

## Database Standards

### Use UUID v7 (Not v4)
Timestamp-prefixed for better index locality:
```rust
use uuid::Uuid;
let id = Uuid::now_v7();
```

**Setup required:**
1. Add v7 feature to `Cargo.toml`:
   ```toml
   uuid = { version = "1.0", features = ["v7", "serde"] }
   ```
2. Compile with unstable flag (UUID v7 is currently unstable):
   ```bash
   RUSTFLAGS="--cfg uuid_unstable" cargo build
   ```

### Money as BIGINT (cents)
Never use floating-point for currency:
```rust
// Store: $10.50 → 1050 cents, 100.000 VND → 100000
pub struct Money(i64);
```

### Soft Delete Pattern
Add to critical tables:
```sql
ALTER TABLE products ADD COLUMN deleted_at TIMESTAMPTZ;
CREATE INDEX idx_products_active ON products(tenant_id, sku) WHERE deleted_at IS NULL;
```

### Timestamps
Always `TIMESTAMPTZ` with defaults:
```sql
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
```

## Authentication & Authorization

### JWT Flow
1. Login → returns `access_token` (15min) + `refresh_token` (7 days)
2. Extract with `AuthUser` extractor (validates JWT, extracts claims)
3. Authorize with `RequireAdmin` or Casbin middleware

### Casbin RBAC
Multi-tenant model: `(subject, tenant, resource, action)`
- Policies stored in `casbin_rule` table
- Enforcer in `shared/auth` with PostgreSQL adapter
- Middleware: `shared_auth::casbin_middleware` (currently disabled in main.rs)

### Auth Extractors
From `shared/auth/extractors.rs`:
```rust
// Basic JWT validation
async fn handler(user: AuthUser) -> String { ... }

// Admin-only endpoints
async fn admin_handler(RequireAdmin(user): RequireAdmin) -> String { ... }

// Casbin permission check
async fn protected(perm: RequirePermission) -> String { ... }
```

## Development Workflow

### Running Services
```bash
# Check entire workspace
cargo check --workspace

# Run user service (port 3000)
cargo run --bin user-service

# With auto-reload
cargo watch -x 'run --bin user-service'

# Export OpenAPI specs
cargo build --features export-spec
```

### Database Migrations
Using `sqlx-cli`:
```bash
# Run migrations
sqlx migrate run

# Create new migration
sqlx migrate add <name>

# Revert last
sqlx migrate revert
```

### Testing
```bash
# All tests
cargo test --workspace

# Specific package
cargo test --package user_service_core
```

## Error Handling

Use `AppError` from `shared/error`:
```rust
// Business errors
Err(AppError::UserNotFound)
Err(AppError::ValidationError("Invalid email".to_string()))

// Database errors auto-convert
let user = query.fetch_one(&pool).await?; // sqlx::Error → AppError

// Manual mapping
.map_err(|e| AppError::InternalError(format!("Casbin: {}", e)))?
```

Never use `unwrap()` or `expect()` in production code. Always propagate errors up.

## Common Patterns

### Service State (API crate)
```rust
pub struct AppState<S: AuthService> {
    pub auth_service: Arc<S>,
    pub enforcer: SharedEnforcer,
    pub jwt_secret: String,
}

impl<S: AuthService> JwtSecretProvider for AppState<S> {
    fn get_jwt_secret(&self) -> &str { &self.jwt_secret }
}
```

### Repository Traits (Core crate)
```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_email(&self, email: &str, tenant_id: Uuid) -> Result<Option<User>, AppError>;
    async fn create(&self, user: &User) -> Result<User, AppError>;
}
```

### OpenAPI Annotations
```rust
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    operation_id = "user_login", // Unique across ALL services
    request_body = LoginReq,
    responses(
        (status = 200, body = AuthResp),
        (status = 401, body = ErrorResp)
    )
)]
pub async fn login<S: AuthService>(...) -> Result<Json<AuthResp>, AppError> { ... }
```

## Performance & Best Practices

### Avoid N+1 Queries
Use joins or batch queries:
```rust
// Bad
for order in orders {
    let items = repo.get_items(order.id).await?;
}

// Good
let items = repo.get_items_batch(&order_ids).await?;
```

### Compile-Time SQL Checks
Use `sqlx::query_as!` for macro-checked queries:
```rust
sqlx::query_as!(User, "SELECT * FROM users WHERE tenant_id = $1", tenant_id)
```

### Sensitive Data Encryption
Encrypt in application before DB storage (e.g., `integrations.credentials`):
```rust
use ring::aead; // or RustCrypto
let encrypted = encrypt(&plaintext, &key)?;
sqlx::query!("INSERT INTO integrations (credentials) VALUES ($1)", encrypted)
```

## Deployment (CapRover)

Each service:
1. Has `Dockerfile` in service directory
2. Deployed as separate "App" in CapRover
3. Communicates via `srv-<app-name>` hostname on Docker overlay network
4. Auto-scaled via CapRover UI

Stateful services (PostgreSQL, Redis, NATS) deployed as One-Click Apps.

## Critical Files Reference

- `ARCHITECTURE.md` - System design, multi-tenancy strategy, technology decisions
- `STRUCTURE.md` - Directory layout, 3-crate pattern details
- `migrations/README.md` - Database schema conventions
- `Cargo.toml` - Workspace config, all crate versions centralized
- `shared/auth/src/lib.rs` - Auth exports (enforcer, middleware, extractors)
- `shared/error/src/lib.rs` - Error types and HTTP status mappings
