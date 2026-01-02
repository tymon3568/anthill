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
- `auth`: Casbin enforcer, JWT validation, auth extractors (`AuthUser`, `RequireAdmin`)

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

Extract `tenant_id` from JWT claims in middleware, inject into request context.

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

### Email/Password Authentication
User Service handles all authentication internally:
1. **User registration** → Email/password with tenant context
2. **User login** → Validate credentials, return JWT tokens
3. **JWT issuance** → User Service generates and signs JWTs
4. **Token validation** → Services validate JWT using shared secret
5. **Session management** → Sessions stored in database

JWT claims structure:
```rust
{
  "sub": "uuid-of-user",
  "tenant_id": "uuid-of-tenant",
  "email": "user@example.com",
  "role": "admin",
  "exp": 1234567890,
  "token_type": "access"
}
```

### Auth Endpoints (User Service)
```rust
// Register new user + create/join tenant
POST /api/v1/auth/register { email, password, full_name, tenant_name }
  → Create tenant (if new) or join existing
  → Hash password with bcrypt
  → Return JWT tokens

// Login
POST /api/v1/auth/login { email, password }
  Headers: X-Tenant-ID: tenant-slug (or from subdomain)
  → Validate credentials
  → Return JWT tokens

// Refresh token
POST /api/v1/auth/refresh { refresh_token }
  → Validate refresh token
  → Return new access token

// Logout
POST /api/v1/auth/logout { refresh_token }
  → Revoke session
```

### Casbin RBAC
Multi-tenant model: `(subject, tenant, resource, action)`
- Policies stored in `casbin_rule` table
- Enforcer in `shared/auth` with PostgreSQL adapter
- Middleware: `shared_auth::casbin_middleware`
- Works with JWT: extract `user_id`, `tenant_id`, `role` to enforce policies

### Auth Extractors
From `shared/auth/extractors.rs`:
```rust
// Validate JWT and extract claims
async fn handler(user: AuthUser) -> String { 
    // user.user_id: UUID from "sub" claim
    // user.tenant_id: from JWT claim
    // user.email: from "email" claim
    // user.role: from "role" claim
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

# Run user service (port 8000)
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

## PR Review Auto-Fix Workflow Rules

### Core Principles
- **ALWAYS** follow this workflow when handling PR reviews to ensure consistent, automated fixes for unresolved issues.
- **NEVER** fix issues that are marked as resolved, informational, or outside the scope of code changes.
- **PRIORITY**: Focus on critical issues (security, breaking changes) first, then warnings, then style/lint issues.
- **TRANSPARENCY**: Log all actions in the PR comments and task files for auditability.
- **CONSENSUS**: If unsure about a fix, request human approval before proceeding.

### Pre-Fix Checklist (MANDATORY)
Before attempting any auto-fix:
1. ✅ **Fetch PR Details**: Use the `fetch` tool to retrieve the PR page content (e.g., `https://github.com/user/repo/pull/123`).
2. ✅ **Extract Unresolved Comments**: Parse the PR content to list all review comments that are:
   - Not marked as "resolved"
   - Related to code (ignore discussions, approvals, etc.)
   - From authorized reviewers (e.g., CodeRabbit, Greptile, or assigned reviewers).
3. ✅ **Categorize Issues**: Classify each unresolved comment by type:
   - **Critical**: Security vulnerabilities, breaking changes, data integrity issues.
   - **Warning**: Logic errors, performance issues, missing tests.
   - **Style**: Linting, formatting, documentation inconsistencies.
   - **Nitpick**: Minor suggestions, optional improvements.
4. ✅ **Check Task Status**: Ensure the related task is in `InProgress_By_[Agent]` and dependencies are satisfied.
5. ✅ **Run Local Validation**: Execute `cargo check --workspace` or equivalent to confirm current state.

### Auto-Fix Process Flow
```
Fetch PR → List Unresolved Issues → Generate Fix Prompt → Evaluate Fixability → Apply Fix → Test & Commit → Update PR
```

### Step 1: Fetch PR URL
- Use the `fetch` tool with the PR URL provided by the user or extracted from context.
- Example: `fetch(url="https://github.com/tymon3568/anthill/pull/123")`
- Parse the returned Markdown for review threads, comments, and status.

### Step 2: List Unresolved Review Errors
- Scan the fetched content for unresolved review comments.
- Format as a bulleted list:
  - **Comment ID/Line**: Brief description of the issue.
  - **Reviewer**: Who flagged it (e.g., CodeRabbit).
  - **Severity**: Critical/Warning/Style/Nitpick.
  - **Suggested Fix**: If provided in the comment.
- Ignore resolved threads or non-code feedback.

### Step 3: Prompt for AI Agents
- Generate a structured prompt for the AI agent to fix the issues.
- Prompt Template:
  ```
  ## PR Review Auto-Fix Prompt

  **PR URL**: [Insert PR URL]
  **Unresolved Issues**:
  - [Issue 1]: [Description] (Severity: [Level])
  - [Issue 2]: [Description] (Severity: [Level])
  - ...

  **Task Context**: [Brief summary from task file, e.g., "Creating stock_adjustments migration for multi-tenancy"]

  **Instructions**:
  1. Analyze each issue for fixability: Can it be resolved with code changes? (Yes/No/Needs Clarification)
  2. For fixable issues, propose specific code edits with file paths and line numbers.
  3. Ensure fixes align with project rules (e.g., Anthill architecture, multi-tenancy patterns).
  4. Prioritize fixes: Critical > Warning > Style > Nitpick.
  5. If multiple issues conflict, resolve conservatively.
  6. Output format: List fixes with before/after code blocks using `path/to/file#Lstart-end` syntax.

  **Constraints**:
  - Do not introduce new dependencies without approval.
  - Maintain backward compatibility.
  - Follow existing code style and patterns.
  - Test fixes locally before committing.
  ```

### Step 4: Evaluate Fixability
- **Criteria for Auto-Fix**:
  - **Yes**: Clear, actionable issues (e.g., missing DEFERRABLE, syntax errors, index optimizations).
  - **No**: Ambiguous issues, architectural changes, or those requiring human judgment (e.g., business rule decisions).
  - **Needs Clarification**: Request user input if the fix impacts scope or requires external data.
- If any issue is "No" or "Needs Clarification", stop and notify the user with a summary.
- If all are "Yes", proceed to fix.

### Step 5: Apply Fixes (Referencing Prompt)
- Use the AI agent's response from Step 3 to apply changes.
- For each fix:
  - Edit files using the provided code blocks.
  - Run `cargo check --workspace` after each edit to validate.
  - If errors occur, revert and flag as unfixable.
- Commit fixes with descriptive messages: `fix(pr_review): resolve [issue summary] [TaskID: XX.YY.ZZ]`
- Push to the feature branch.

### Step 6: Post-Fix Actions
- Update PR: Comment on resolved threads with "Auto-fixed by [Agent]: [Brief fix description]".
- Update Task File: Add log entry, mark sub-tasks as done, set status to `NeedsReview` if all issues resolved.
- Notify User: Summarize actions taken and any remaining issues.

### Success Criteria
- All auto-fixable issues are resolved without introducing new errors.
- PR comments are updated for transparency.
- Task file reflects progress accurately.
- Local tests pass (`cargo check`, etc.).

### Common Pitfalls to Avoid
❌ Fixing issues outside the agent's expertise (e.g., complex refactors).
❌ Ignoring severity levels – always prioritize critical issues.
❌ Committing without testing – validate every change.
❌ Overwriting human decisions – defer to user for ambiguous cases.
❌ Failing to update PR/task – maintain full traceability.

### Collaboration Rules
- If multiple agents are involved, coordinate via task file logs.
- For cross-task issues, check dependencies before fixing.
- Escalate to user if fixes require policy changes (e.g., altering business rules).

## Error Handling

Use `AppError` from `shared/error`:
```rust
// Business errors
Err(AppError::UserNotFound)
Err(AppError::ValidationError("Invalid email".to_string()))

// Database errors auto-convert
let user = query.fetch_one(&pool).await?; // sqlx::Error → AppError

// Authentication errors
.map_err(|e| AppError::AuthenticationError(format!("Auth: {}", e)))?

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

Stateful services (PostgreSQL, Redis, NATS, MinIO) deployed as One-Click Apps.

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

# Frontend Development

For Svelte 5 development guidelines, see:
- `frontend/.svelte-instructions.md` - Complete Svelte 5 guidelines
- `frontend/.cursor-instructions.md` - Cursor AI specific instructions
