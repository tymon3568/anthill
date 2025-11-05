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
- `config`: Environment config loader (`Config::from_env()`)
- `db`: `init_pool()` for PostgreSQL connection pooling
- `auth`: Casbin enforcer, Kanidm JWT validation, auth extractors (`AuthUser`, `RequireAdmin`)
- `kanidm_client`: OAuth2/OIDC client for Kanidm integration

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

Extract `tenant_id` from Kanidm JWT groups claim in middleware, map to PostgreSQL tenant via `kanidm_tenant_groups` table, inject into request context.

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

### Kanidm Integration (OAuth2/OIDC)
Kanidm is the Identity Provider handling all authentication:
1. **User registration/login** → Handled by Kanidm UI or API
2. **OAuth2 flow** → Authorization Code Grant + PKCE
3. **JWT issuance** → Kanidm signs JWTs with standard OIDC claims
4. **Token validation** → Services validate JWT using Kanidm public key
5. **Group management** → Kanidm groups map to Anthill tenants

JWT claims from Kanidm:
```rust
{
  "sub": "uuid-of-user-in-kanidm",
  "email": "user@example.com",
  "preferred_username": "username",
  "groups": ["tenant_acme_users", "tenant_acme_admins"]
}
```

### Tenant Mapping
```sql
-- Map Kanidm groups to tenants
CREATE TABLE kanidm_tenant_groups (
  tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
  kanidm_group_uuid UUID NOT NULL,
  kanidm_group_name TEXT NOT NULL,
  PRIMARY KEY (tenant_id, kanidm_group_uuid)
);

-- Link users to Kanidm
ALTER TABLE users 
  ADD COLUMN kanidm_user_id UUID UNIQUE,
  DROP COLUMN password_hash;
```

### OAuth2 Endpoints (User Service)
```rust
// Initiate OAuth2 flow
GET /api/v1/auth/oauth/authorize
  → Redirect to Kanidm: https://idm.example.com/ui/oauth2?client_id=...

// Handle OAuth2 callback
POST /api/v1/auth/oauth/callback { code, state }
  → Exchange code for tokens
  → Map Kanidm user to tenant
  → Return access_token

// Refresh token
POST /api/v1/auth/oauth/refresh { refresh_token }
  → Get new access_token from Kanidm
```

### Casbin RBAC
Multi-tenant model: `(subject, tenant, resource, action)`
- Policies stored in `casbin_rule` table
- Enforcer in `shared/auth` with PostgreSQL adapter
- Middleware: `shared_auth::casbin_middleware`
- Works with Kanidm JWT: extract `sub` + `groups`, map to tenant, enforce policies

### Auth Extractors
From `shared/auth/extractors.rs`:
```rust
// Validate Kanidm JWT and extract claims
async fn handler(user: AuthUser) -> String { 
    // user.kanidm_user_id: UUID from "sub" claim
    // user.tenant_id: mapped from groups claim
    // user.email: from "email" claim
}

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

// Kanidm errors
.map_err(|e| AppError::AuthenticationError(format!("Kanidm: {}", e)))?

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

## Documentation Lookup with Context7

Before implementing any code related to frameworks, libraries, installations, or configurations, always consult up-to-date documentation using Context7 tools:

1. **Resolve Library ID**: Use `mcp_upstash_conte_resolve-library-id` to find the correct Context7-compatible library ID for the framework or library in question.
2. **Fetch Documentation**: Use `mcp_upstash_conte_get-library-docs` with the resolved ID to retrieve current documentation, examples, and best practices.
3. **Reference Implementation**: Base your code on the retrieved documentation to ensure accuracy and adherence to official guidelines.

This ensures all implementations follow the latest standards and avoid deprecated patterns.

## Critical Files Reference

- `ARCHITECTURE.md` - System design, multi-tenancy strategy, technology decisions
- `STRUCTURE.md` - Directory layout, 3-crate pattern details
- `migrations/README.md` - Database schema conventions
- `Cargo.toml` - Workspace config, all crate versions centralized
- `shared/auth/src/lib.rs` - Auth exports (enforcer, middleware, extractors)
- `shared/error/src/lib.rs` - Error types and HTTP status mappings

# Svelte 5 Development Guidelines

## 🚨 CRITICAL: Always Use Svelte MCP and Svelte 5 Runes

**ALL Svelte/SvelteKit development MUST follow these rules:**

### 🔍 **Before Any Svelte Code Changes:**
1. **ALWAYS** call `mcp_svelte_list-sections` first to see available documentation
2. **ALWAYS** call `mcp_svelte_get-documentation` with relevant sections before implementing
3. **NEVER** implement Svelte code without consulting current Svelte 5 documentation

### 🏗️ **Svelte 5 Runes Usage (MANDATORY):**

#### State Management
```typescript
// ✅ CORRECT: Use $state rune
export const userState = $state<User | null>(null);
export const isLoading = $state(false);

// ❌ WRONG: Never use legacy stores
// const userStore = writable<User | null>(null);
```

#### Reactive Statements
```svelte
<script>
  let count = $state(0);
  let doubled = $derived(count * 2);

  // ✅ CORRECT: Use $effect for side effects
  $effect(() => {
    console.log('Count changed:', count);
  });

  // ❌ WRONG: Never use reactive statements
  // $: console.log('Count changed:', count);
  // $: doubled = count * 2;
</script>
```

#### Component Props
```svelte
<script>
  // ✅ CORRECT: Use $props rune
  let { title, items = $bindable() } = $props<{
    title: string;
    items?: string[];
  }>();

  // ❌ WRONG: Never use export let
  // export let title: string;
  // export let items: string[] = [];
</script>
```

#### Template Syntax
```svelte
<!-- ✅ CORRECT: Use {@render ...} -->
{@render children()}

<!-- ❌ WRONG: Never use <slot /> -->
<!-- <slot /> -->
```

### 📚 **Documentation Lookup Process:**

1. **Identify the task** (e.g., "implement form validation")
2. **Call `mcp_svelte_list-sections`** to see all available docs
3. **Analyze use_cases** to find relevant sections
4. **Call `mcp_svelte_get-documentation`** with ALL relevant sections
5. **Read and understand** the documentation thoroughly
6. **Implement using** the documented patterns

### 🔄 **Migration Checklist:**

When working with existing code:
- [ ] Replace `writable()` stores with `$state`
- [ ] Replace `$store` syntax with direct state access
- [ ] Replace `$:` reactive statements with `$effect` or `$derived`
- [ ] Replace `export let` with `$props`
- [ ] Replace `<slot />` with `{@render children()}`
- [ ] Remove `import { writable } from 'svelte/store'`

### 🛠️ **Tools to Use:**

- `mcp_svelte_list-sections` - List all available documentation
- `mcp_svelte_get-documentation` - Get specific documentation sections
- `mcp_svelte_svelte-autofixer` - Fix Svelte code issues
- `mcp_svelte_playground-link` - Create playground links for testing

### ⚠️ **Red Flags (DO NOT DO):**

- Using `writable()`, `readable()`, `derived()` from `svelte/store`
- Using `$storeName` syntax in templates
- Using `$:` reactive statements
- Using `export let` for props
- Using `<slot />` in templates
- Implementing without checking documentation first

### 📋 **Implementation Steps:**

1. **Read the task** and understand requirements
2. **Call `mcp_svelte_list-sections`** to explore docs
3. **Identify relevant sections** from use_cases
4. **Call `mcp_svelte_get-documentation`** with selected sections
5. **Study the documentation** carefully
6. **Implement using Svelte 5 patterns**
7. **Run `mcp_svelte_svelte-autofixer`** to validate code
8. **Test the implementation**

### 🎯 **Quality Assurance:**

After implementation:
- Run `bun run build` to ensure compilation
- Run `bun run lint` for code quality
- Run `bun run test` for functionality
- Use `mcp_svelte_svelte-autofixer` to check for issues

---

**REMEMBER: Svelte 5 runes are the ONLY acceptable way to write Svelte code in this project. Always consult MCP documentation first!**
