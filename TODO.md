# TODO - Inventory SaaS Platform

## 📊 Tổng Quan Tiến Độ

- **Giai đoạn hiện tại**: Phase 3 - User Service Production Integration (~30% complete)
- **Ngày bắt đầu**: 2025-10-08
- **Ngày cập nhật**: 2025-01-10
- **Mục tiêu**: MVP trong 2-3 tháng
- **Kiến trúc**: 3-Crate Pattern (Clean Architecture + DDD + Repository Pattern)
- **User Service**: ✅ Production-ready với authentication, JWT, Swagger UI

---

## Phase 1: Infrastructure & Workspace ✅ (95% Complete)

### ✅ 1.1 Basic Setup (COMPLETED)
- [x] ✅ Git repo initialized
- [x] ✅ ARCHITECTURE.md created
- [x] ✅ Microservices directory structure
- [x] ✅ Cargo workspace configured
- [x] ✅ Docker compose for local PostgreSQL
- [x] ✅ GitHub Actions CI/CD
- [x] ✅ Workspace compiles successfully

### ✅ 1.2 Microservices Skeleton (COMPLETED)
- [x] ✅ User service → **Refactored to 3-Crate Pattern** (production-ready)
- [x] ✅ Inventory service skeleton
- [x] ✅ Order service skeleton
- [x] ✅ Integration service skeleton
- [x] ✅ Payment service skeleton
- [ ] ⏳ **TODO**: Refactor other services to 3-crate pattern (when needed)

### ✅ 1.3 Shared Libraries (COMPLETED)
- [x] ✅ `shared/error` - AppError + IntoResponse
- [x] ✅ `shared/jwt` - JWT encode/decode + Claims
- [x] ✅ `shared/config` - Environment config loader
- [x] ✅ `shared/types` - Common types (Uuid, DateTime)
- [x] ✅ `shared/db` - SQLx PgPool initialization
- [x] ✅ `shared/openapi` - OpenAPI spec export
- [x] ✅ `shared/auth` - Casbin RBAC + Auth middleware & extractors

### ✅ 1.4 Auth & Authorization Library (COMPLETED)
- [x] ✅ `shared/auth` crate - **COMPLETED 2025-01-10**
  - ✅ Casbin enforcer setup with PostgreSQL adapter
  - ✅ RBAC model configuration (subject, tenant, resource, action)
  - ✅ Helper functions: add_policy, add_role_for_user, enforce
  - ✅ Casbin middleware for Axum (JWT + permission check)
  - ✅ Auth extractors:
    - `AuthUser` - Basic JWT extraction
    - `RequireAdmin` - Admin-only endpoints
    - `RequirePermission` - Casbin permission check
  - ✅ Upgraded to Axum 0.8, SQLx 0.8, Tower 0.5
  - ✅ Workspace dependency management
  - ✅ Unit tests for extractors and error handling
  
### ⏳ 1.5 Pending Shared Libraries
- [ ] 🟡 **P1** `shared/events` crate (when implementing event-driven)
  - Event definitions
  - NATS client wrapper

### 1.6 Development Tools & Automation (Optional - P2)

> **Note**: These are "nice to have" optimizations. Add them when they become painful to not have.

- [ ] 🔵 **P2** Task automation (cargo-make / justfile)
  - Add when manual commands become repetitive
- [ ] 🔵 **P2** Pre-commit hooks
  - Add when team size > 1 person
- [ ] 🔵 **P2** Dev containers (.devcontainer)
  - Add when onboarding new developers
- [ ] 🔵 **P2** Dependency updates (Renovate/Dependabot)
  - Add when maintaining security patches becomes burden

---

## Phase 2: Database & Migrations ✅ (100% COMPLETE)

### ✅ 2.1 Database Design & Strategy (COMPLETED)

#### ✅ 2.1.1 Multi-Tenancy Strategy - **COMPLETED 2025-01-10**
- [x] ✅ **Quyết định**: Application-level filtering (documented in ARCHITECTURE.md)
  - ✅ Shared schema với `tenant_id` trong mỗi bảng
  - ✅ No Postgres RLS (for simplicity and performance)
  - ✅ Repository pattern enforces tenant isolation
  - ✅ Type-safe tenant context in Rust

#### ✅ 2.1.2 Database Standards - **COMPLETED 2025-01-10**
- [x] ✅ UUID v7 for all primary keys (timestamp-based)
- [x] ✅ BIGINT for currency (smallest unit: cents/xu)
- [x] ✅ TIMESTAMPTZ for all timestamps
- [x] ✅ Soft delete with `deleted_at` column
- [x] ✅ Application-level encryption for sensitive data
- [x] ✅ All documented in ARCHITECTURE.md

#### ✅ 2.1.3 SQL Migrations - **COMPLETED 2025-01-10**
- [x] ✅ Migration directory structure (`migrations/`)
- [x] ✅ Migration 001: Extensions (uuid-ossp, pgcrypto) + uuid_generate_v7()
- [x] ✅ Migration 002: Core tables (tenants, users, sessions)
- [x] ✅ Migration 003: Casbin RBAC tables (casbin_rule)
- [x] ✅ Migration helper script (`scripts/migrate.sh`)
- [x] ✅ `.env.example` file with DATABASE_URL
- [x] ✅ Migration README with guidelines

#### ✅ 2.1.4 Database ERD - **COMPLETED 2025-01-10**
- [x] ✅ ERD documented in DBML format (`docs/database-erd.dbml`)
- [x] ✅ Can be visualized on https://dbdiagram.io/d
- [x] ✅ Includes all core tables with relationships
- [x] ✅ Future tables documented as comments

### ✅ 2.2 Migration Testing & Deployment - **COMPLETED 2025-01-10**

- [x] ✅ Setup local PostgreSQL (Docker container)
- [x] ✅ Install sqlx-cli with postgres feature
- [x] ✅ Create .env file with DATABASE_URL
- [x] ✅ Run migrations successfully (all 3 migrations applied)
- [x] ✅ Verify database schema:
  - ✅ Extensions installed (uuid-ossp, pgcrypto)
  - ✅ uuid_generate_v7() function working
  - ✅ tenants table created with proper indexes
  - ✅ users table created with multi-tenant isolation
  - ✅ sessions table for JWT token management
  - ✅ casbin_rule table for RBAC policies
  - ✅ All triggers, constraints, and indexes in place
- [x] ✅ Test UUID v7 generation (timestamp-based UUIDs working)
- [x] ✅ Test tenant insertion (data successfully inserted)

### 📝 Migration Files Summary

```
migrations/
├── 20250110000001_initial_extensions.sql      (✅ Applied)
│   ├── uuid-ossp extension
│   ├── pgcrypto extension
│   ├── uuid_generate_v7() function
│   └── update_updated_at_column() trigger function
├── 20250110000002_create_tenants_users.sql    (✅ Applied)
│   ├── tenants table (with soft delete, JSONB settings)
│   ├── users table (multi-tenant, bcrypt hash, role-based)
│   └── sessions table (JWT management, token hashing)
└── 20250110000003_create_casbin_tables.sql    (✅ Applied)
    ├── casbin_rule table (policies & role assignments)
    └── Helper views (casbin_policies, casbin_role_assignments)
```

### ⏳ 2.3 Future Business Tables (Phase 4+)

> Note: Core foundation complete. Business domain tables will be added in later phases.

- [ ] ⏳ **Phase 4**: Inventory tables (products, warehouses, inventory_levels, stock_moves)
- [ ] ⏳ **Phase 5**: Order tables (orders, order_items)
- [ ] ⏳ **Phase 6**: Integration tables (integrations, marketplace_sync)
- [ ] ⏳ **Phase 7**: Payment tables (payments, transactions)

### 2.3 Indexes & Optimization (P0/P1)

#### 2.3.1 Essential Indexes (P0)
- [ ] 🔴 **P0** Tạo composite indexes cho multi-tenant queries
  ```sql
  -- Products
  CREATE INDEX idx_products_tenant_sku ON products(tenant_id, sku);
  CREATE INDEX idx_products_tenant_group ON products(tenant_id, item_group_id);

  -- Orders
  CREATE INDEX idx_orders_tenant_status_date ON orders(tenant_id, status, created_at DESC);
  CREATE INDEX idx_orders_tenant_customer ON orders(tenant_id, customer_id);

  -- Inventory Levels
  CREATE INDEX idx_inventory_tenant_product_warehouse 
    ON inventory_levels(tenant_id, product_id, warehouse_id);

  -- Stock Moves (CRITICAL - heavily queried)
  CREATE INDEX idx_stock_moves_tenant_product_date 
    ON stock_moves(tenant_id, product_id, move_date DESC);
  CREATE INDEX idx_stock_moves_reference 
    ON stock_moves(reference_type, reference_id);
  ```

- [ ] 🔴 **P0** Tạo partial indexes cho performance
  ```sql
  -- Only index active/non-deleted records
  CREATE INDEX idx_integrations_active 
    ON integrations(tenant_id, platform) 
    WHERE status = 'active' AND deleted_at IS NULL;

  -- Only index pending/in-progress orders
  CREATE INDEX idx_orders_pending 
    ON orders(tenant_id, created_at DESC) 
    WHERE status IN ('pending', 'confirmed', 'processing');

  -- Only index available stock (exclude reserved)
  CREATE INDEX idx_stock_available 
    ON inventory_levels(tenant_id, product_id) 
    WHERE quantity_available > 0;
  ```

#### 2.3.2 Performance Tuning (P1)
- [ ] 🟡 **P1** Table Partitioning cho large tables
  - Partition `orders` và `order_items` by `tenant_id` + `created_at` (monthly)
  - Khi dự kiến >100M rows
  ```sql
  -- Example: Range partitioning by date
  CREATE TABLE orders_2025_01 PARTITION OF orders
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');
  ```

- [ ] 🟡 **P1** Vacuum & Autovacuum tuning
  - Bảng `stock_moves` có nhiều INSERT → aggressive vacuum
  ```sql
  ALTER TABLE stock_moves SET (
    autovacuum_vacuum_scale_factor = 0.05,
    autovacuum_analyze_scale_factor = 0.02
  );
  ```

- [ ] 🟡 **P1** Connection Pool Sizing
  - Formula: `(CPU_cores * 2) + effective_io_concurrency`
  - Example: 4 cores → pool size = 8-10
  - Configure in SQLx:
  ```rust
  let pool = PgPoolOptions::new()
      .max_connections(10)
      .min_connections(2)
      .acquire_timeout(Duration::from_secs(5))
      .connect(&database_url)
      .await?;
  ```

- [ ] 🟡 **P1** Query performance monitoring
  - Enable `pg_stat_statements` extension
  - Monitor slow queries (>1s)
  ```sql
  CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
  -- Query slowest queries
  SELECT query, calls, mean_exec_time, max_exec_time
  FROM pg_stat_statements
  ORDER BY mean_exec_time DESC
  LIMIT 20;
  ```

### 2.4 Chạy Migrations
- [ ] ⏳ Chạy migrations: `sqlx migrate run --database-url postgres://...`
- [ ] ⏳ Verify schema trong PostgreSQL

---

## Phase 3: User Service (Auth & Tenancy)

### ✅ 3.0 Architecture Implementation (COMPLETED)
- [x] ✅ **3-Crate Pattern** fully implemented
  - [x] ✅ `user_service_api` - HTTP handlers, routing, OpenAPI, main.rs
  - [x] ✅ `user_service_core` - Domain models, DTOs, service/repository traits
  - [x] ✅ `user_service_infra` - PostgreSQL repo impl, service impl, bcrypt
- [x] ✅ **Clean separation of concerns**
  - [x] ✅ Core has zero infrastructure dependencies
  - [x] ✅ API generic over service traits (testable!)
  - [x] ✅ Infra implements all business logic

### 3.1 Core Authentication

#### 3.1.1 User Registration (P0)
- [x] ✅ **P0** Implement user registration endpoint
  - [x] ✅ POST `/api/v1/auth/register`
  - [x] ✅ Tạo tenant mới cho user đầu tiên
  - [x] ✅ Hash password với **bcrypt** (using bcrypt crate)
  - [x] ✅ Validate email format (validator crate)
  - [x] ✅ Check email uniqueness
  - [x] ✅ OpenAPI documentation with utoipa
  - [ ] ⏳ TODO: Migrate to Argon2id for better security

#### 3.1.2 Password Security (P0)
- [ ] 🔴 **P0** Password Policy Enforcement
  - Minimum length: 8 characters
  - Minimum entropy: 50 bits (use crate `zxcvbn`)
  - Check against top 10,000 breached passwords
    - Use HaveIBeenPwned API: `https://api.pwnedpasswords.com/range/{hash_prefix}`
    - Hoặc offline list từ: https://github.com/danielmiessler/SecLists
  ```rust
  use zxcvbn::zxcvbn;
  
  fn validate_password(password: &str) -> Result<(), String> {
      if password.len() < 8 {
          return Err("Password must be at least 8 characters".to_string());
      }
      let entropy = zxcvbn(password, &[]);
      if entropy.score() < 3 {
          return Err("Password is too weak".to_string());
      }
      // Check HaveIBeenPwned
      Ok(())
  }
  ```

#### 3.1.3 Login & Session Management (P0)
- [x] ✅ **P0** Implement login endpoint
  - [x] ✅ POST `/api/v1/auth/login`
  - [x] ✅ Generate JWT access token (15 min expiry) + refresh token (7 days)
  - [x] ✅ Return tokens + user info
  - [x] ✅ Store session in database with token hashes (SHA-256)
  - [ ] ⏳ TODO: Extract `user_agent`, `ip_address` from HTTP request headers
  - [ ] ⏳ TODO: Implement tenant resolution (currently creates new tenant)

- [ ] 🔴 **P0** Rate Limiting & Brute-Force Protection
  - **Login rate limit**: 5 attempts per IP per 5 minutes
  - **Forgot password**: 3 attempts per email per day
  - Use Redis for rate limit counters
  - Implement sliding window algorithm
  ```rust
  // tower_governor crate hoặc custom middleware
  use tower_governor::{GovernorLayer, GovernorConfigBuilder};
  
  let governor_conf = GovernorConfigBuilder::default()
      .per_second(1)  // 1 request per second
      .burst_size(5)  // Allow burst of 5
      .finish()
      .unwrap();
  
  Router::new()
      .route("/auth/login", post(login_handler))
      .layer(GovernorLayer { config: governor_conf })
  ```

- [x] ✅ **P0** Implement refresh token endpoint
  - [x] ✅ POST `/api/v1/auth/refresh`
  - [x] ✅ Generate new access token from refresh token
  - [x] ✅ Validate refresh token from database (hash lookup)
  - [x] ✅ Rotate refresh token (revoke old, create new session)

- [x] ✅ **P0** Implement logout endpoint - **COMPLETED 2025-01-10**
  - [x] ✅ POST `/api/v1/auth/logout`
  - [x] ✅ Revoke refresh token in database
  - [x] ✅ Session management fully implemented
  - ℹ️ Note: Access token blacklisting in Redis not implemented (adds overhead, short expiry sufficient)

#### 3.1.4 Security Headers (P0)
- [ ] 🔴 **P0** Configure secure HTTP headers
  - Use `tower_http::set_header` middleware
  ```rust
  use tower_http::set_header::SetResponseHeaderLayer;
  use http::header;
  
  let app = Router::new()
      .layer(SetResponseHeaderLayer::if_not_present(
          header::STRICT_TRANSPORT_SECURITY,
          HeaderValue::from_static("max-age=31536000; includeSubDomains"),
      ))
      .layer(SetResponseHeaderLayer::if_not_present(
          header::X_CONTENT_TYPE_OPTIONS,
          HeaderValue::from_static("nosniff"),
      ))
      .layer(SetResponseHeaderLayer::if_not_present(
          header::X_FRAME_OPTIONS,
          HeaderValue::from_static("DENY"),
      ))
      .layer(SetResponseHeaderLayer::if_not_present(
          HeaderValue::from_name("Content-Security-Policy").unwrap(),
          HeaderValue::from_static("default-src 'self'"),
      ));
  ```

#### 3.1.5 Audit Logging (P0)
- [ ] 🔴 **P0** Bảng `audit_logs`
  ```sql
  CREATE TABLE audit_logs (
      audit_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
      tenant_id UUID NOT NULL,
      user_id UUID,
      action VARCHAR(100) NOT NULL,  -- login, logout, create, update, delete
      resource_type VARCHAR(100),    -- product, order, user
      resource_id UUID,
      old_value JSONB,
      new_value JSONB,
      ip_address INET,
      user_agent TEXT,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  
  CREATE INDEX idx_audit_logs_tenant_date ON audit_logs(tenant_id, created_at DESC);
  CREATE INDEX idx_audit_logs_user ON audit_logs(user_id, created_at DESC);
  CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
  ```

- [ ] 🔴 **P0** Log critical actions
  - Login attempts (success & failure)
  - Password changes
  - User creation/deletion
  - Permission changes
  - Data exports
  - Integration credentials access

### 3.2 Authorization với Casbin (P0 - Core Infrastructure)

> **Decision**: Using `casbin-rs` for RBAC from the start for scalable, flexible authorization

#### 3.2.1 Casbin Setup (P0)
- [ ] 🔴 **P0** Add dependencies to `shared/auth` crate
  - `casbin = "2.0"` (core casbin-rs)
  - `casbin-sqlx-adapter = "0.6"` (PostgreSQL adapter)
  - `async-trait = "0.1"` (for async traits)
  
- [ ] 🔴 **P0** Tạo Casbin model file (`shared/auth/model.conf`)
  - Multi-tenant RBAC: `sub, dom, obj, act`
  - Model definition:
  ```conf
  [request_definition]
  r = sub, dom, obj, act

  [policy_definition]
  p = sub, dom, obj, act

  [role_definition]
  g = _, _, _

  [policy_effect]
  e = some(where (p.eft == allow))

  [matchers]
  m = g(r.sub, p.sub, r.dom) && r.dom == p.dom && r.obj == p.obj && r.act == p.act
  ```
  - **Explanation**:
    - `sub`: user_id or role name
    - `dom`: tenant_id (domain for multi-tenant isolation)
    - `obj`: resource path (e.g., `/api/v1/products`, `/api/v1/orders`)
    - `act`: HTTP method (GET, POST, PUT, DELETE) or custom action (read, write)
  - **Multi-tenant isolation**: `r.dom == p.dom` ensures policies only apply within same tenant

#### 3.2.2 Database Schema (P0)
- [ ] 🔴 **P0** Create `casbin_rule` table migration
  - Store all Casbin policies in PostgreSQL
  ```sql
  CREATE TABLE casbin_rule (
      id SERIAL PRIMARY KEY,
      ptype VARCHAR(12) NOT NULL,  -- 'p' (policy) or 'g' (grouping/role)
      v0 VARCHAR(128),              -- sub: user_id or role name
      v1 VARCHAR(128),              -- dom: tenant_id
      v2 VARCHAR(128),              -- obj: resource path or name
      v3 VARCHAR(128),              -- act: action (read, write, delete)
      v4 VARCHAR(128),              -- extra param (optional)
      v5 VARCHAR(128)               -- extra param (optional)
  );
  
  -- Index for performance
  CREATE INDEX idx_casbin_rule_ptype ON casbin_rule(ptype);
  CREATE INDEX idx_casbin_rule_v1 ON casbin_rule(v1);  -- tenant_id
  ```

#### 3.2.3 Casbin Integration (P0)
- [ ] 🔴 **P0** Initialize Casbin enforcer in `shared/auth`
  ```rust
  use casbin::{Enforcer, CoreApi};
  use casbin_sqlx_adapter::SqlxAdapter;
  
  pub async fn create_enforcer(db_pool: PgPool) -> Result<Enforcer, Box<dyn std::error::Error>> {
      let adapter = SqlxAdapter::new(db_pool).await?;
      let enforcer = Enforcer::new("model.conf", adapter).await?;
      Ok(enforcer)
  }
  ```

- [ ] 🔴 **P0** Implement Axum middleware cho authorization
  - Extract JWT claims (user_id, tenant_id, role)
  - Check permissions with Casbin enforcer
  - Return 403 Forbidden if not allowed
  ```rust
  use casbin::{Enforcer, CoreApi};
  use axum::{middleware::Next, Extension};
  use std::sync::Arc;
  use tokio::sync::RwLock;
  
  pub async fn casbin_middleware(
      Extension(enforcer): Extension<Arc<RwLock<Enforcer>>>,
      claims: Claims,  // Extracted from JWT
      req: Request<Body>,
      next: Next<Body>,
  ) -> Result<Response, StatusCode> {
      let resource = req.uri().path();  // e.g., "/api/v1/products"
      let action = req.method().as_str();  // GET, POST, PUT, DELETE
      
      // Check permission with Casbin
      let mut e = enforcer.write().await;
      let allowed = e.enforce((
          &claims.user_id.to_string(),        // sub
          &claims.tenant_id.to_string(),      // dom
          resource,                            // obj
          action,                              // act
      ))?;
      
      if !allowed {
          return Err(StatusCode::FORBIDDEN);
      }
      
      Ok(next.run(req).await)
  }
  ```

- [ ] 🔴 **P0** Axum extractor for role-based checks
  - `RequireRole` extractor: `RequireRole("admin")`
  - `RequirePermission` extractor: `RequirePermission { resource: "products", action: "write" }`

#### 3.2.4 Default Policies & Roles (P0)
- [ ] 🔴 **P0** Seed default roles for each tenant
  - **Admin role**: Full access to all resources
  - **Manager role**: CRUD on products, orders (no user management)
  - **User role**: Read-only access
  ```sql
  -- Policies cho Admin role (tenant_id = example tenant)
  INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
  ('p', 'admin', '00000000-0000-0000-0000-000000000001', '/api/v1/products', 'GET'),
  ('p', 'admin', '00000000-0000-0000-0000-000000000001', '/api/v1/products', 'POST'),
  ('p', 'admin', '00000000-0000-0000-0000-000000000001', '/api/v1/products', 'PUT'),
  ('p', 'admin', '00000000-0000-0000-0000-000000000001', '/api/v1/products', 'DELETE'),
  ('p', 'admin', '00000000-0000-0000-0000-000000000001', '/api/v1/orders', 'GET'),
  ('p', 'admin', '00000000-0000-0000-0000-000000000001', '/api/v1/orders', 'POST'),
  ('p', 'admin', '00000000-0000-0000-0000-000000000001', '/api/v1/users', 'GET'),
  ('p', 'admin', '00000000-0000-0000-0000-000000000001', '/api/v1/users', 'POST');
  
  -- Policies cho Manager role
  INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
  ('p', 'manager', '00000000-0000-0000-0000-000000000001', '/api/v1/products', 'GET'),
  ('p', 'manager', '00000000-0000-0000-0000-000000000001', '/api/v1/products', 'POST'),
  ('p', 'manager', '00000000-0000-0000-0000-000000000001', '/api/v1/products', 'PUT'),
  ('p', 'manager', '00000000-0000-0000-0000-000000000001', '/api/v1/orders', 'GET'),
  ('p', 'manager', '00000000-0000-0000-0000-000000000001', '/api/v1/orders', 'POST');
  
  -- Policies cho User role (read-only)
  INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
  ('p', 'user', '00000000-0000-0000-0000-000000000001', '/api/v1/products', 'GET'),
  ('p', 'user', '00000000-0000-0000-0000-000000000001', '/api/v1/orders', 'GET');
  
  -- Assign user to role (grouping policy)
  -- Example: user_id 'abc123' has role 'admin' in tenant '00000000-0000-0000-0000-000000000001'
  INSERT INTO casbin_rule (ptype, v0, v1, v2) VALUES
  ('g', 'abc123-user-uuid', 'admin', '00000000-0000-0000-0000-000000000001');
  ```

- [ ] 🔴 **P0** API endpoints for role management (admin only)
  - POST `/api/v1/admin/roles` - Create custom role
  - POST `/api/v1/admin/policies` - Add policy to role
  - DELETE `/api/v1/admin/policies` - Remove policy
  - POST `/api/v1/admin/users/:user_id/roles` - Assign role to user
  - DELETE `/api/v1/admin/users/:user_id/roles` - Revoke role

#### 3.2.5 Testing (P0)
- [ ] 🔴 **P0** Unit tests for Casbin enforcer
  - Test role assignments
  - Test permission checks
  - Test multi-tenant isolation
- [ ] 🔴 **P0** Integration tests for authorization middleware
  - Test admin can access all endpoints
  - Test manager cannot delete users
  - Test user cannot write to products
  - Test tenant isolation (user A cannot access tenant B resources)

### 3.3 User Management
### 3.3 User Management

#### 3.3.1 Basic User CRUD (P0)
- [ ] 🔴 **P0** Endpoint: List users trong tenant
  - GET `/api/v1/users`
  - Filter by role, status
  - Pagination support

- [ ] 🔴 **P0** Tenant Isolation Testing
  - **Critical Security Test**
  - Tạo 2 tenants: tenant_a, tenant_b
  - User A login → get JWT với tenant_a
  - Cố gắng access resource của tenant_b bằng JWT của A
  - **Expected**: 403 Forbidden hoặc 404 Not Found
  - Test scenarios:
    - Modify JWT header manually (change tenant_id claim)
    - Use valid JWT but query với tenant_b's resource IDs
    - SQL injection attempts to bypass tenant_id filter
  ```rust
  #[tokio::test]
  async fn test_tenant_isolation() {
      // Create tenant A and user A
      let tenant_a = create_tenant("Tenant A").await;
      let user_a = create_user(&tenant_a, "user_a@example.com").await;
      let jwt_a = generate_jwt(&user_a, &tenant_a).await;
      
      // Create tenant B and resource
      let tenant_b = create_tenant("Tenant B").await;
      let product_b = create_product(&tenant_b, "Product B").await;
      
      // Try to access tenant B's resource with tenant A's JWT
      let response = client
          .get(&format!("/api/v1/products/{}", product_b.id))
          .bearer_auth(&jwt_a)
          .send()
          .await;
      
      assert_eq!(response.status(), StatusCode::NOT_FOUND);
  }
  ```

#### 3.3.2 User Invitation (P1)
- [ ] 🟡 **P1** Bảng `user_invitations`
  ```sql
  CREATE TABLE user_invitations (
      invitation_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
      tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
      email VARCHAR(255) NOT NULL,
      role VARCHAR(50) NOT NULL,
      token VARCHAR(255) UNIQUE NOT NULL,  -- Random secure token
      invited_by UUID NOT NULL REFERENCES users(user_id),
      expires_at TIMESTAMPTZ NOT NULL,     -- 24 hours from created_at
      accepted_at TIMESTAMPTZ,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  
  CREATE INDEX idx_invitations_token ON user_invitations(token) WHERE accepted_at IS NULL;
  ```

- [ ] 🟡 **P1** Endpoint: Invite user mới
  - POST `/api/v1/users/invite`
  - Generate secure token (32 bytes random)
  - Send email với link: `https://app.example.com/accept-invite?token={token}`
  - Token expires in 24 hours

- [ ] 🟡 **P1** Endpoint: Accept invitation
  - POST `/api/v1/users/accept-invite`
  - Validate token (not expired, not used)
  - Create user account
  - Mark invitation as accepted

- [ ] ⏳ Endpoint: Cập nhật user role
  - PATCH `/api/v1/users/:user_id/role`
  - Log trong audit_logs

#### 3.3.3 Advanced Features (P1)
- [ ] 🟡 **P1** Impersonate (Admin login as user)
  - POST `/api/v1/users/:user_id/impersonate`
  - Only for super admin role
  - Generate special JWT với flag `impersonated: true`
  - Show banner in UI: "You are viewing as {user_name}"
  - All actions logged với `impersonated_by: admin_id`
  ```rust
  #[derive(Serialize, Deserialize)]
  struct JWTClaims {
      sub: Uuid,           // user_id
      tenant_id: Uuid,
      role: String,
      impersonated: bool,  // Flag for impersonation
      impersonated_by: Option<Uuid>,
      exp: u64,
  }
  ```

- [ ] 🟡 **P1** SSO Integration (Enterprise feature)
  - **SAML 2.0** support
    - Use crate `samael`
    - IdP metadata upload
    - SP metadata generation
  - **OIDC (OpenID Connect)** support
    - Use crate `openidconnect`
    - Support Google, Microsoft Azure AD, Okta
  - Table: `sso_configurations`
  ```sql
  CREATE TABLE sso_configurations (
      sso_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
      tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
      provider VARCHAR(50) NOT NULL,  -- saml, oidc, google, azure
      config JSONB NOT NULL,          -- Provider-specific config
      is_enabled BOOLEAN NOT NULL DEFAULT true,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  ```

### 3.4 Testing
- [ ] ⏳ Viết unit tests cho authentication logic
- [ ] ⏳ Viết integration tests cho API endpoints
- [ ] 🔴 **P0** Test tenant isolation (CRITICAL SECURITY)
- [ ] ⏳ Test authorization với Casbin

### ✅ 3.5 Documentation & DevEx (COMPLETED)
- [x] ✅ OpenAPI 3.0 specification with utoipa
- [x] ✅ Swagger UI at `/docs`
- [x] ✅ Health check endpoint `/health`
- [x] ✅ Input validation with validator crate
- [x] ✅ Comprehensive error handling with AppError
- [x] ✅ Workspace compilation works perfectly
- [x] ✅ GitHub Actions workflows for CI
- [x] ✅ Snake_case naming convention enforced

---

## Phase 4: Inventory Service

### 4.1 Product Master Data (Item Master)
- [ ] 🔴 **P0** Bảng `products` (Item Master - Single Source of Truth)
  - product_id (UUID, PK)
  - tenant_id (FK)
  - sku (unique per tenant)
  - name, description
  - item_group_id (FK) - Phân loại sản phẩm theo nhóm
  - product_type (storable, consumable, service, digital)
  - track_inventory (boolean)
  - **default_uom_id (FK)** - Đơn vị tính mặc định (unit of measure)
  - **barcode, qr_code** - Mã vạch cho scanning
  - **has_variants (boolean)** - Có biến thể không (màu, size...)
  - **abc_classification** - Vận tốc tồn kho (A=fast, B=medium, C=slow)
  - **accounting_category_id (FK)** - Quy định tài khoản, thuế
  - created_at, updated_at
- [ ] 🔴 **P0** Bảng `unit_of_measures` (UoM)
  - uom_id (UUID, PK)
  - tenant_id (FK)
  - name ("Piece", "Box", "Carton", "Kg", "Liter"...)
  - uom_type (unit, reference, bigger, smaller)
  - category (weight, volume, length, unit)
  - rounding_precision (0.01 for 2 decimal places)
- [ ] 🔴 **P0** Bảng `uom_conversions` (Quy đổi UoM)
  - conversion_id (UUID, PK)
  - tenant_id, product_id (FK)
  - from_uom_id, to_uom_id (FK)
  - conversion_factor (e.g., 1 thùng = 12 chai)
  - Example: 1 Box = 12 Pieces, 1 Carton = 10 Boxes = 120 Pieces
- [ ] 🟡 **P1** Bảng `product_variants` (Biến thể sản phẩm)
  - variant_id (UUID, PK)
  - parent_product_id (FK to products)
  - tenant_id (FK)
  - variant_attributes (JSONB) - {"color": "red", "size": "L"}
  - sku (unique variant SKU)
  - barcode
  - price_difference (so với sản phẩm chính)
  - Has own inventory levels
- [ ] ⏳ Bảng `item_groups` (Product Categories/Item Groups)
  - item_group_id (UUID, PK)
  - tenant_id (FK)
  - name, parent_group_id (self-reference cho tree structure)
  - description
- [ ] ⏳ Endpoint: Create product
  - POST `/api/v1/inventory/products`
  - Validate SKU uniqueness per tenant
  - Support product variants (JSONB field)
- [ ] ⏳ Endpoint: List products với filtering
  - GET `/api/v1/inventory/products`
  - Support pagination, filtering by item_group, product_type
  - Full-text search trên name và description
- [ ] ⏳ Endpoint: Get product by ID/SKU
  - GET `/api/v1/inventory/products/:id`
  - Include stock levels across warehouses
- [ ] ⏳ Endpoint: Update product
  - PATCH `/api/v1/inventory/products/:id`
- [ ] ⏳ Endpoint: Delete/Archive product
  - DELETE `/api/v1/inventory/products/:id`
  - Soft delete với `archived_at` field

### 4.2 Warehouse & Storage Locations
- [ ] 🔴 **P0** Bảng `warehouses`
  - warehouse_id (UUID, PK)
  - tenant_id (FK)
  - name, code (unique per tenant)
  - warehouse_type (physical, virtual, transit, **dropship, quarantine**)
  - address (JSONB)
  - is_active (boolean)
  - parent_warehouse_id (FK) - Tree structure cho multi-level warehouses
  - **is_quarantine (boolean)** - Kho cách ly chờ QC
- [ ] ⏳ Bảng `storage_locations`
  - location_id (UUID, PK)
  - tenant_id, warehouse_id (FK)
  - name, code (e.g., "Shelf-A-01", "Bin-B-12")
  - location_type (zone, aisle, shelf, bin)
  - parent_location_id (self-reference)
  - capacity_info (JSONB) - dimensions, weight limits
- [ ] ⏳ Bảng `location_types` (Virtual Locations)
  - Internal (WH/Stock)
  - Customer (shipped items)
  - Supplier (items in transit from vendor)
  - Inventory Loss/Adjustment
  - Production/Manufacturing
  - Quality Control
  - Transit/Inter-warehouse
- [ ] ⏳ Endpoint: Manage warehouses
  - CRUD operations cho warehouses
  - GET `/api/v1/inventory/warehouses` - Tree view structure
- [ ] ⏳ Endpoint: Manage storage locations
  - CRUD operations cho locations within warehouse
  - GET `/api/v1/inventory/warehouses/:id/locations`

### 4.3 Stock Tracking & Inventory Levels
- [ ] ⏳ Bảng `inventory_levels`
  - tenant_id, product_id, warehouse_id, location_id
  - quantity_on_hand (tồn kho thực tế)
  - quantity_reserved (đã lock cho orders)
  - quantity_available (on_hand - reserved)
  - reorder_level, reorder_quantity (cho auto-replenishment)
  - min_stock_level, max_stock_level
  - last_counted_at (last physical count date)
  - Composite PK: (tenant_id, product_id, warehouse_id, location_id)
- [ ] 🔴 **P0** Bảng `stock_moves` (Stock Ledger - **IMMUTABLE** audit trail)
  - move_id (UUID, PK)
  - tenant_id, product_id
  - source_location_id, destination_location_id
  - move_type (receipt, delivery, transfer, adjustment, return)
  - quantity, **uom_id (FK)**
  - unit_cost (valuation at time of move)
  - **valuation_method** (fifo, avco, standard_cost)
  - **balance_qty, balance_value** (running totals for Stock Ledger)
  - reference_type (order_id, transfer_id, adjustment_id)
  - reference_id (UUID)
  - **idempotency_key** (prevent duplicate moves on retry)
  - move_date, created_by
  - status (draft, confirmed, done, cancelled)
  - **⚠️ IMMUTABLE**: No UPDATE allowed, only INSERT
- [ ] 🔴 **P0** Bảng `stock_adjustments` (Lý do điều chỉnh)
  - adjustment_id (UUID, PK)
  - move_id (FK to stock_moves)
  - tenant_id, product_id, warehouse_id
  - reason_code (damaged, expired, stolen, counting_error, loss, found)
  - notes (TEXT)
  - approved_by (user_id)
  - approved_at
- [ ] ⏳ Endpoint: Get stock levels by warehouse
  - GET `/api/v1/inventory/stock`
  - Filter by warehouse_id, location_id, product_id
  - Show available vs reserved quantities
- [ ] ⏳ Endpoint: Stock movement history
  - GET `/api/v1/inventory/stock/movements`
  - Audit trail của tất cả stock moves
  - Filter by product, date range, move_type

### 4.4 Stock Operations (Quy trình nhập-xuất-chuyển-kiểm kê)

#### 4.4.1 Goods Receipt Note (GRN) - Nhập kho
- [ ] 🔴 **P0** Bảng `goods_receipts`
  - receipt_id (UUID, PK)
  - receipt_number (auto-generated: GRN-2025-00001)
  - tenant_id, warehouse_id (FK)
  - supplier_id (FK), purchase_order_id (FK optional)
  - receipt_date
  - status (draft, **waiting_qc**, qc_passed, qc_rejected, completed, cancelled)
  - total_items, total_quantity
  - notes
  - created_by, approved_by
- [ ] 🔴 **P0** Bảng `goods_receipt_items`
  - receipt_item_id (UUID, PK)
  - receipt_id (FK)
  - product_id, variant_id (FK)
  - expected_quantity, received_quantity
  - uom_id (FK)
  - unit_cost
  - lot_number, serial_number (if tracked)
  - qc_status (pending, passed, rejected)
  - storage_location_id (FK)
- [ ] 🔴 **P0** Endpoint: Create GRN
  - POST `/api/v1/inventory/receipts`
  - Generate receipt_number
  - **Idempotency key** trong header
  - Create stock move từ Supplier → Warehouse (or Quarantine)
  - Update inventory_levels
  - Publish event: `inventory.receipt.created`
- [ ] 🔴 **P0** Endpoint: Complete/Validate GRN
  - POST `/api/v1/inventory/receipts/:id/validate`
  - Create immutable stock_moves
  - Update valuation (FIFO/AVCO layers)
  - Publish event: `inventory.receipt.completed`
#### 4.4.2 Delivery Order (DO) - Xuất kho
- [ ] 🔴 **P0** Bảng `delivery_orders`
  - delivery_id (UUID, PK)
  - delivery_number (auto-generated: DO-2025-00001)
  - tenant_id, warehouse_id (FK)
  - order_id (FK from order-service)
  - customer_id (FK)
  - delivery_date, scheduled_date
  - status (draft, **reserved**, **picked**, **packed**, **shipped**, delivered, cancelled)
  - pick_list_id (FK optional)
  - shipping_carrier, tracking_number
  - notes
- [ ] 🔴 **P0** Bảng `delivery_order_items`
  - delivery_item_id (UUID, PK)
  - delivery_id (FK)
  - product_id, variant_id (FK)
  - ordered_quantity, picked_quantity, delivered_quantity
  - uom_id (FK)
  - lot_number, serial_number (if tracked)
  - source_location_id (FK)
- [ ] 🔴 **P0** Endpoint: Create DO from Order
  - POST `/api/v1/inventory/deliveries`
  - Subscribe to `order.confirmed` event
  - Auto-reserve stock
  - Status → "reserved"
- [ ] 🔴 **P0** Endpoint: Pick items for DO
  - POST `/api/v1/inventory/deliveries/:id/pick`
  - Generate pick list (if not batched)
  - Update picked_quantity
  - Status → "picked"
- [ ] 🔴 **P0** Endpoint: Pack items
  - POST `/api/v1/inventory/deliveries/:id/pack`
  - Generate packing slip
  - Status → "packed"
- [ ] 🔴 **P0** Endpoint: Ship/Validate DO
  - POST `/api/v1/inventory/deliveries/:id/ship`
  - Create immutable stock_moves (Warehouse → Customer)
  - Update inventory_levels (decrement)
  - Update valuation (COGS calculation)
  - Publish event: `inventory.delivery.completed`
  - Status → "shipped"
#### 4.4.3 Stock Transfer - Chuyển kho nội bộ
- [ ] 🔴 **P0** Bảng `stock_transfers`
  - transfer_id (UUID, PK)
  - transfer_number (auto-generated: ST-2025-00001)
  - tenant_id
  - source_warehouse_id, destination_warehouse_id (FK)
  - transfer_date
  - status (**draft, waiting, in_transit, received, validated**, cancelled)
  - shipment_tracking_number
  - expected_delivery_date
  - notes
- [ ] 🔴 **P0** Bảng `stock_transfer_items`
  - transfer_item_id (UUID, PK)
  - transfer_id (FK)
  - product_id, variant_id (FK)
  - quantity, uom_id (FK)
  - lot_number, serial_number
  - source_location_id, dest_location_id
- [ ] 🔴 **P0** Endpoint: Create Transfer
  - POST `/api/v1/inventory/transfers`
  - Status → "draft"
- [ ] 🔴 **P0** Endpoint: Confirm Transfer
  - POST `/api/v1/inventory/transfers/:id/confirm`
  - Deduct from source warehouse
  - Add to "Inter-warehouse Transit" virtual location
  - Status → "in_transit"
- [ ] 🔴 **P0** Endpoint: Receive Transfer
  - POST `/api/v1/inventory/transfers/:id/receive`
  - Remove from Transit location
  - Add to destination warehouse
  - Create immutable stock_moves
  - Status → "received" → "validated"
  - Publish event: `inventory.transfer.completed`
#### 4.4.4 Stock Take / Physical Inventory Count - Kiểm kê
- [ ] 🔴 **P0** Bảng `stock_takes`
  - stock_take_id (UUID, PK)
  - stock_take_number (auto-generated: ST-2025-00001)
  - tenant_id, warehouse_id (FK)
  - count_date, scheduled_date
  - status (draft, **in_progress**, completed, cancelled)
  - count_type (full, partial, cycle_count)
  - assigned_to (user_id)
  - notes
- [ ] 🔴 **P0** Bảng `stock_take_lines`
  - line_id (UUID, PK)
  - stock_take_id (FK)
  - product_id, variant_id (FK)
  - location_id (FK)
  - **expected_quantity** (snapshot from inventory_levels)
  - **actual_quantity** (counted via barcode scan)
  - difference_quantity (actual - expected)
  - uom_id (FK)
  - lot_number, serial_number
  - counted_by (user_id), counted_at
- [ ] 🔴 **P0** Endpoint: Create Stock Take
  - POST `/api/v1/inventory/stock-takes`
  - Snapshot current expected quantities
  - Generate stock_take_lines
  - Status → "draft"
- [ ] 🔴 **P0** Endpoint: Scan/Count items
  - POST `/api/v1/inventory/stock-takes/:id/count`
  - Scan barcode → update actual_quantity
  - Calculate difference_quantity
  - Status → "in_progress"
- [ ] 🔴 **P0** Endpoint: Finalize Stock Take
  - POST `/api/v1/inventory/stock-takes/:id/finalize`
  - Auto-generate adjustments for discrepancies
  - Create stock_moves với move_type="adjustment"
  - Use "Inventory Loss" location for negative adj
  - Update inventory_levels
  - Require approval if difference > threshold
  - Publish event: `inventory.stock_take.completed`
  - Status → "completed"
- [ ] ⏳ **Stock Reservation** (Đặt chỗ hàng)
  - POST `/api/v1/inventory/reservations`
  - Reserve stock cho specific orders (Make-to-Order, Purchase-to-Order)
  - Prevent double allocation
  - Auto-release nếu order cancelled
  - Bảng `stock_reservations`:
    - reservation_id, tenant_id, product_id, warehouse_id
    - order_id (FK), quantity_reserved
    - reserved_at, expires_at
    - status (active, fulfilled, cancelled, expired)

#### 4.4.5 Returned Merchandise Authorization (RMA)
- [ ] 🟡 **P1** Bảng `rma_requests`
  - rma_id (UUID, PK)
  - rma_number (auto-generated: RMA-2025-00001)
  - tenant_id, customer_id (FK)
  - original_delivery_id (FK)
  - rma_date, approved_date
  - status (requested, approved, rejected, received, refunded)
  - return_reason (defective, wrong_item, damaged, unwanted)
  - refund_method (credit_note, cash, exchange)
  - notes
- [ ] 🟡 **P1** Bảng `rma_items`
  - rma_item_id (UUID, PK)
  - rma_id (FK)
  - product_id, variant_id (FK)
  - delivery_item_id (FK)
  - quantity_returned
  - lot_number, serial_number
  - condition (new, used, damaged)
  - action (restock, scrap, return_to_supplier)
- [ ] 🟡 **P1** Endpoint: Create RMA
  - POST `/api/v1/inventory/rma`
  - Link to original delivery order
  - Status → "requested"
- [ ] 🟡 **P1** Endpoint: Approve RMA
  - POST `/api/v1/inventory/rma/:id/approve`
  - Status → "approved"
- [ ] 🟡 **P1** Endpoint: Receive returned goods
  - POST `/api/v1/inventory/rma/:id/receive`
  - Create stock move: Customer → Warehouse
  - Reverse delivery order
  - Update inventory_levels
  - Status → "received"

### 4.5 Lot & Serial Number Tracking (Traceability)
- [ ] 🔴 **P0** Bảng `lots_serial_numbers`
  - lot_serial_id (UUID, PK)
  - tenant_id, product_id
  - tracking_type (lot, serial)
  - lot_number / serial_number (unique)
  - manufacturing_date, expiry_date
  - **certificate_of_analysis (COA)** - Link to document
  - supplier_id (FK), **purchase_order_id (FK)** - Truy xuất nguồn gốc
  - supplier_info (JSONB)
  - quantity (for lots), always 1 for serial numbers
  - status (available, reserved, sold, returned, **quarantined**)
  - location_id (current location)
  - **qc_status (pending, passed, failed)**
  - created_at
- [ ] ⏳ Bảng `lot_serial_moves` (Lot/Serial traceability)
  - move_id (FK to stock_moves)
  - lot_serial_id (FK)
  - quantity
  - source_location, dest_location
- [ ] 🔴 **P0** Enable Lot/Serial Number tracking per product
  - Add field `tracking_method` in products table (none, lot, serial)
  - Serial numbers: unique per unit (1 serial = 1 product)
  - Lot numbers: batch tracking (1 lot = multiple units)
- [ ] 🟡 **P1** FEFO (First Expiry First Out) picking strategy
  - When creating delivery orders, pick lots with nearest expiry_date first
  - Alert if picking expired lots
  - Quarantine expired items automatically
- [ ] ⏳ Endpoint: Assign lot/serial numbers during receipt
  - POST `/api/v1/inventory/receipts/:id/assign-tracking`
  - Bulk generation of serial numbers
  - Import serial/lot numbers from CSV
- [ ] 🟡 **P1** Endpoint: Track lot/serial lifecycle
  - GET `/api/v1/inventory/tracking/:lot_serial_id`
  - Full traceability: serial → supplier → PO → COA → receipts → transfers → customer
  - Show all movements and current status
  - Link to quality check records
- [ ] ⏳ Display lot/serial numbers on delivery documents
  - Include in delivery API response
  - Required for RMA, warranty, product registration

### 4.6 Inventory Valuation (Định giá tồn kho)
- [ ] ⏳ Bảng `inventory_valuations`
  - valuation_id (UUID, PK)
  - tenant_id, product_id, warehouse_id
  - valuation_method (fifo, avco, standard_cost)
  - unit_cost (current average/FIFO cost)
  - total_value (quantity_on_hand * unit_cost)
  - last_updated_at
- [ ] ⏳ Bảng `stock_valuation_layers` (FIFO/AVCO tracking)
  - layer_id (UUID, PK)
  - tenant_id, product_id
  - move_id (FK to stock_moves)
  - quantity, unit_cost
  - remaining_quantity (for FIFO)
  - layer_date
- [ ] ⏳ Support 3 valuation methods:
  - **FIFO** (First In First Out): Oldest cost used first
  - **AVCO** (Average Cost): Dynamically recalculated
  - **Standard Cost**: Fixed cost per product
- [ ] ⏳ Endpoint: Inventory valuation report
  - GET `/api/v1/inventory/valuation`
  - Show total inventory value by product, warehouse
  - Historical valuation với date range
- [ ] ⏳ Endpoint: Revalue inventory manually
  - POST `/api/v1/inventory/valuation/revalue`
  - Update unit_cost for specific products
  - For standard costing or cost adjustments
- [ ] ⏳ Automatic valuation updates
  - Recalculate khi có receipts, deliveries
  - FIFO: track purchase order costs
  - AVCO: recalculate average on each incoming shipment

### 4.6.5 Quality Control Integration
- [ ] 🟡 **P1** Bảng `quality_checks`
  - qc_id (UUID, PK)
  - tenant_id, receipt_id (FK)
  - product_id, lot_serial_id (FK)
  - qc_date
  - inspector_id (user_id)
  - status (pending, **passed, rejected**)
  - defect_type (physical_damage, quality_issue, wrong_specification)
  - notes, photos (JSONB array of URLs)
  - approved_by, approved_at
- [ ] 🟡 **P1** QC Workflow for Receipts
  - When GRN created → Status "waiting_qc"
  - Items go to Quarantine warehouse
  - POST `/api/v1/inventory/quality-checks`
  - If QC passed → Move to main warehouse, receipt.status = "qc_passed"
  - If QC rejected → Keep in quarantine, create RMA to supplier

### 4.7 Stock Replenishment (Tự động đặt hàng bổ sung)
- [ ] 🟡 **P1** Bảng `reorder_rules`
  - rule_id (UUID, PK)
  - tenant_id, product_id, warehouse_id
  - **reorder_point (ROP)** = daily_usage * lead_time_days + safety_stock
  - min_quantity (min stock level)
  - max_quantity (max stock level)
  - reorder_quantity (economic order quantity)
  - **lead_time_days** (supplier lead time)
  - **safety_stock** (buffer stock)
  - **daily_usage** (average consumption)
  - is_active (boolean)
- [ ] 🟡 **P1** Automated reorder detection
  - Background job (cron) check inventory_levels.quantity_available
  - Calculate **projected_qty** = on_hand + incoming_po - reserved
  - If projected_qty < reorder_point → trigger reorder
  - Auto-generate Material Request or draft Purchase Order
  - Publish event: `inventory.reorder.triggered`
  - Email notification to procurement team
- [ ] ⏳ Material Requests (yêu cầu vật tư)
  - POST `/api/v1/inventory/material-requests`
  - Request stock from supplier hoặc other warehouses
  - Status: draft, submitted, ordered, received
  - Link to Purchase Orders

### 4.8 Batch/Wave/Cluster Picking (Tối ưu picking)
- [ ] ⏳ Bảng `pick_lists`
  - pick_list_id (UUID, PK)
  - tenant_id, warehouse_id
  - pick_type (single, batch, wave, cluster)
  - assigned_to (user_id)
  - status (draft, assigned, in_progress, completed)
  - created_at, completed_at
- [ ] ⏳ Bảng `pick_list_items`
  - pick_list_id (FK)
  - order_id (FK)
  - product_id, location_id
  - quantity_to_pick, quantity_picked
  - pick_sequence (optimization)
- [ ] ⏳ Generate pick lists
  - POST `/api/v1/inventory/pick-lists`
  - Batch multiple orders together
  - Optimize pick path by location
  - Cluster picking cho same products
- [ ] ⏳ Put-away strategies
  - POST `/api/v1/inventory/putaway`
  - Suggest optimal storage locations
  - Based on product velocity, dimensions, expiry date

### 4.9 Stock Reports & Analytics
- [ ] 🔴 **P0** Stock Ledger Report (ERPNext-style)
  - GET `/api/v1/inventory/reports/stock-ledger`
  - Mỗi dòng = 1 stock_move
  - Columns: Date, Move Type, Product, Qty, Valuation Rate, Balance Qty, Balance Value
  - Filter by product, warehouse, date range
  - Running balance calculation
- [ ] 🔴 **P0** Inventory Reconciliation Report (Cân đối kho)
  - GET `/api/v1/inventory/reports/reconciliation`
  - Tồn đầu kỳ + Nhập - Xuất = Tồn cuối kỳ
  - By warehouse, product category, accounting period
  - Compare book value vs physical count
- [ ] 🟡 **P1** Stock aging report (0-30, 31-60, 61-90, >90 days)
  - GET `/api/v1/inventory/reports/aging`
  - Identify slow-moving và dead stock (no movement >90 days)
  - By product, lot, warehouse
  - Suggest markdown or disposal
- [ ] 🟡 **P1** Stock movement report
  - GET `/api/v1/inventory/reports/movements`
  - Inbound vs outbound by period
  - By product, warehouse, item group
  - Graph: daily/weekly/monthly trends
- [ ] 🟡 **P1** Inventory turnover ratio
  - GET `/api/v1/inventory/reports/turnover`
  - Formula: COGS / Average Inventory Value
  - Higher = better (faster moving stock)
  - By product category
- [ ] 🟡 **P1** Low stock alerts
  - GET `/api/v1/inventory/reports/low-stock`
  - Products below reorder point
  - Projected stockout date
  - Suggested reorder quantity
- [ ] 🟡 **P1** Dead Stock Report
  - GET `/api/v1/inventory/reports/dead-stock`
  - Products with no transactions > 90 days
  - Total value locked in dead stock
  - Disposal recommendations
- [ ] 🔴 **P0** Inventory valuation report
  - GET `/api/v1/inventory/reports/valuation`
  - Total value by warehouse, product category
  - Historical valuation comparison
  - By valuation method (FIFO/AVCO/Standard)
- [ ] 🟡 **P1** Stock by Lot/Serial Report
  - GET `/api/v1/inventory/reports/lot-serial`
  - Track inventory by lot/serial number
  - Expiry date visibility
  - Supplier traceability

### 4.10 Real-time Updates & Events

- [ ] ⏳ Subscribe NATS events từ Integration Service
  - `integration.stock.synced` - Sync stock từ marketplace
  - `order.confirmed` - Reserve stock cho orders
  - `order.cancelled` - Release reserved stock
- [ ] ⏳ Publish NATS events khi stock thay đổi
  - `inventory.stock.updated` - Stock level changed
  - `inventory.stock.low_threshold` - Below reorder level
  - `inventory.receipt.completed` - Goods received
  - `inventory.delivery.completed` - Goods shipped
  - `inventory.transfer.completed` - Internal transfer done
  - `inventory.reorder.triggered` - Auto-reorder activated

### 4.11 Technical Implementation (P0 - Critical)

#### 4.11.1 Idempotency & Concurrency Control
- [ ] 🔴 **P0** Idempotency Key implementation
  - Require `X-Idempotency-Key` header on all POST requests
  - Store key in Redis with TTL (24 hours)
  - Return cached response if duplicate key detected
  - Prevent double GRN/DO creation on network retry
- [ ] 🔴 **P0** Distributed Locking (Redis Redlock)
  - Lock format: `inventory:lock:{tenant_id}:{warehouse_id}:{product_id}`
  - Acquire lock before any stock mutation (reserve, adjust, transfer)
  - Hold lock during transaction, release after commit
  - Timeout: 5 seconds max
  - Prevent race condition: 2 orders reserving same last item
- [ ] 🔴 **P0** Database Row-Level Locking
  - Use `SELECT ... FOR UPDATE` on inventory_levels
  - Wrap stock mutations in DB transactions
  - Implement optimistic locking với `version` column

#### 4.11.2 Event-Driven Architecture (Saga Pattern)
- [ ] 🔴 **P0** Outbox Pattern for reliable events
  - Bảng `event_outbox`:
    - event_id (UUID, PK)
    - aggregate_type (delivery_order, receipt, transfer)
    - aggregate_id (UUID)
    - event_type (inventory.stock.decreased, inventory.stock.increased)
    - payload (JSONB)
    - status (pending, published, failed)
    - created_at, published_at
  - Write to outbox in same transaction as business logic
  - Background worker polls outbox → publish to NATS
  - Mark as published after NATS confirm
- [ ] 🔴 **P0** Dead Letter Queue (DLQ) cho NATS
  - If event processing fails 3 times → move to DLQ
  - Alert operations team
  - Manual retry dashboard
- [ ] 🔴 **P0** Saga Orchestration for complex flows
  - Example: DO validation saga:
    1. Reserve stock (inventory-service)
    2. Create shipment (logistics-service)
    3. Generate invoice (accounting-service)
  - If any step fails → compensating transactions
  - Rollback: Cancel shipment, release stock, void invoice

#### 4.11.3 Performance Optimization
- [ ] 🟡 **P1** Snapshot stock cuối ngày (Materialized View)
  - Table `daily_stock_snapshots`:
    - snapshot_date (DATE, PK)
    - tenant_id, product_id, warehouse_id
    - opening_qty, closing_qty
    - total_receipts, total_deliveries
    - valuation_amount
  - Refresh daily via cron (incremental)
  - Speed up reports (no need to scan millions of stock_moves)
- [ ] 🟡 **P1** Indexing strategy
  - Composite index: (tenant_id, warehouse_id, product_id) on inventory_levels
  - Index on stock_moves: (tenant_id, product_id, move_date DESC)
  - Partial index: WHERE status = 'active' on reservations
- [ ] 🟡 **P1** Caching strategy (Redis)
  - Cache inventory_levels for hot products (TTL: 60s)
  - Cache-aside pattern
  - Invalidate on stock mutation

#### 4.11.4 Mobile/Barcode Integration
- [ ] 🟡 **P1** PWA for warehouse staff
  - Barcode scanner using device camera (ZXing library)
  - Offline-first: IndexedDB sync
  - Workflows: GRN receipt, stock take, picking
  - Push notifications for tasks

### 4.12 Multi-Echelon Inventory (P2 - Advanced)
- [ ] 🔵 **P2** Bảng `distribution_network`
  - Central warehouse → Regional hubs → Local stores
  - Define replenishment routes
  - Auto-transfer rules based on demand
- [ ] 🔵 **P2** Demand Forecasting
  - Simple moving average (last 30/60/90 days)
  - Seasonal adjustment
  - Integration with ML service (optional)
  - Forecast next 30 days demand
  - Adjust reorder_point dynamically

### 4.13 Testing & Quality Assurance
- [ ] ⏳ Unit tests cho business logic
  - Test FIFO/AVCO valuation calculations
  - Test stock reservation logic
  - Test reorder rules triggers
- [ ] ⏳ Integration tests cho API endpoints
  - Test full stock receipt → storage → delivery flow
  - Test lot/serial number tracking
  - Test inventory adjustments
- [ ] ⏳ Test concurrent stock updates (race conditions)
  - Multiple orders reserving same stock simultaneously
  - Concurrent transfers from same location
  - Use database transactions và row-level locking
- [ ] ⏳ Performance tests
  - Bulk import 10,000+ products
  - Concurrent stock moves (100+ operations/sec)
  - Query performance với millions of stock_moves records

---

## Phase 5: Order Service

### 5.1 Order Management
- [ ] ⏳ Endpoint: Create order
  - POST `/api/v1/orders`
  - Validate stock availability
  - Reserve stock (call Inventory Service)
- [ ] ⏳ Endpoint: List orders
  - GET `/api/v1/orders`
  - Support filtering by status, date
- [ ] ⏳ Endpoint: Get order by ID
  - GET `/api/v1/orders/:id`
- [ ] ⏳ Endpoint: Update order status
  - PATCH `/api/v1/orders/:id/status`

### 5.2 Order Processing với Event-Driven
- [ ] ⏳ Subscribe event: `order.placed` (từ Integration Service)
  - Validate stock
  - Reserve stock
  - Create order record
  - Publish `order.confirmed`
- [ ] ⏳ Subscribe event: `payment.completed`
  - Update order status → "paid"
  - Publish `order.ready_to_fulfill`
- [ ] ⏳ Subscribe event: `order.cancelled`
  - Release reserved stock
  - Update status

### 5.3 Fulfillment
- [ ] ⏳ Endpoint: Mark order as fulfilled
  - POST `/api/v1/orders/:id/fulfill`
  - Update stock (decrement)
  - Publish `order.fulfilled`

### 5.4 Testing
- [ ] ⏳ Unit tests
- [ ] ⏳ Integration tests
- [ ] ⏳ Test order flow end-to-end với events

---

## Phase 6: Integration Service (Marketplace)

### 6.1 Adapter Pattern Setup
- [ ] ⏳ Định nghĩa trait `MarketplaceAdapter`
  - `authenticate()`, `sync_products()`, `sync_orders()`, `update_inventory()`
- [ ] ⏳ Implement `ShopeeAdapter`
  - Sử dụng Shopee Open Platform API
  - Handle OAuth2 flow
- [ ] ⏳ Implement `LazadaAdapter`
- [ ] ⏳ Implement `TikiAdapter`

### 6.2 Integration Management
- [ ] ⏳ Endpoint: Connect marketplace
  - POST `/api/v1/integrations`
  - Store credentials (encrypted)
- [ ] ⏳ Endpoint: OAuth callback handler
  - GET `/api/v1/integrations/callback/:platform`
- [ ] ⏳ Endpoint: List integrations
  - GET `/api/v1/integrations`
- [ ] ⏳ Endpoint: Disconnect integration
  - DELETE `/api/v1/integrations/:id`

### 6.3 Sync Logic
- [ ] ⏳ Implement product sync (push inventory to marketplace)
  - Scheduled job hoặc manual trigger
  - Handle rate limiting
- [ ] ⏳ Implement order sync (pull orders from marketplace)
  - Polling strategy (fallback)
  - Publish `order.placed` event
- [ ] ⏳ Implement webhook receiver
  - POST `/api/v1/integrations/webhooks/:platform`
  - Verify signature
  - Publish events to NATS

### 6.4 Testing
- [ ] ⏳ Mock marketplace APIs cho testing
- [ ] ⏳ Test sync flow
- [ ] ⏳ Test webhook handling

---

## Phase 7: Payment Service

### 7.1 Payment Gateway Integration
- [ ] ⏳ Implement VNPay adapter
- [ ] ⏳ Implement Stripe adapter
- [ ] ⏳ (Optional) MoMo, ZaloPay adapters

### 7.2 Payment Processing
- [ ] ⏳ Endpoint: Create payment intent
  - POST `/api/v1/payments`
  - Return payment URL
- [ ] ⏳ Endpoint: Handle gateway callback/webhook
  - POST `/api/v1/payments/callback/:gateway`
  - Verify signature
  - Publish `payment.completed` or `payment.failed`
- [ ] ⏳ Endpoint: Get payment status
  - GET `/api/v1/payments/:id`

### 7.3 Refunds
- [ ] ⏳ Endpoint: Process refund
  - POST `/api/v1/payments/:id/refund`
  - Publish `payment.refunded`

### 7.4 Testing
- [ ] ⏳ Unit tests
- [ ] ⏳ Integration tests với mock gateways
- [ ] ⏳ Test idempotency

---

## Phase 8: Frontend (SvelteKit)

### 8.1 Thiết Lập Project
- [ ] ⏳ Init SvelteKit project trong `frontend/`
  - `pnpm create svelte@latest`
  - Enable TypeScript strict mode
- [ ] ⏳ Cài đặt dependencies
  - TailwindCSS / shadcn-svelte
  - TanStack Query (@tanstack/svelte-query)
  - Zod (validation)
  - Superforms (form handling)

### 8.2 Authentication UI
- [ ] ⏳ Trang `/login`
- [ ] ⏳ Trang `/register`
- [ ] ⏳ Implement session management (stores)
- [ ] ⏳ Protected routes middleware

### 8.3 Dashboard
- [ ] ⏳ Layout chính với sidebar navigation
- [ ] ⏳ Dashboard overview (metrics, charts)
- [ ] ⏳ Real-time updates (SSE/WebSocket)

### 8.4 Product Management UI
- [ ] ⏳ Trang danh sách sản phẩm
- [ ] ⏳ Form tạo/sửa sản phẩm
- [ ] ⏳ Trang quản lý tồn kho

### 8.5 Order Management UI
- [ ] ⏳ Trang danh sách đơn hàng
- [ ] ⏳ Chi tiết đơn hàng
- [ ] ⏳ Update order status

### 8.6 Integration UI
- [ ] ⏳ Trang kết nối marketplace
- [ ] ⏳ OAuth flow UI
- [ ] ⏳ Sync management

### 8.7 Settings
- [ ] ⏳ User profile
- [ ] ⏳ Tenant settings
- [ ] ⏳ Payment gateway configuration

---

## Phase 9: Analytics (Cube)

### 9.1 Cube Setup
- [ ] ⏳ Tạo Cube schema files trong `infra/cube/schema/`
- [ ] ⏳ Define measures: GMV, inventory value, order count
- [ ] ⏳ Define dimensions: time, product, warehouse, channel

### 9.2 Integration với Frontend
- [ ] ⏳ Gọi Cube REST API từ SvelteKit
- [ ] ⏳ Hiển thị dashboard charts (Chart.js / ECharts)

### 9.3 Pre-aggregations
- [ ] ⏳ Cấu hình pre-aggregations cho queries phổ biến
- [ ] ⏳ Cache trên Redis

---

## Phase 10: Deployment (CapRover)

### 10.1 Chuẩn Bị CapRover
- [ ] ⏳ Cài đặt CapRover trên server (VPS)
  - Follow: https://caprover.com/docs/get-started.html
- [ ] ⏳ Cấu hình domain cho CapRover

### 10.2 Deploy Stateful Services
- [ ] ⏳ Deploy PostgreSQL (One-Click App)
- [ ] ⏳ Deploy Redis (One-Click App)
- [ ] ⏳ Deploy NATS (One-Click App)
- [ ] ⏳ Deploy Cube (Custom Dockerfile)

### 10.3 Deploy Microservices
- [ ] ⏳ Tạo `Dockerfile` cho mỗi service (multi-stage build)
- [ ] ⏳ Tạo CapRover app cho từng service
  - user-service
  - inventory-service
  - order-service
  - integration-service
  - payment-service
- [ ] ⏳ Cấu hình environment variables cho mỗi app
- [ ] ⏳ Enable HTTP/HTTPS và wildcard SSL

### 10.4 Deploy Frontend
- [ ] ⏳ Build SvelteKit app
- [ ] ⏳ Tạo CapRover app cho frontend
- [ ] ⏳ Cấu hình domain

### 10.5 CI/CD
- [ ] ⏳ Tạo GitHub Actions workflow
  - Rust: fmt, clippy, test, build Docker
  - Frontend: lint, test, build
- [ ] ⏳ Trigger deploy trên CapRover khi merge to `main`

---

## Phase 11: Monitoring & Observability

### 11.1 Logging
- [ ] ⏳ Deploy Netdata (One-Click App)
- [ ] ⏳ Cấu hình tracing spans trong các service
- [ ] ⏳ OpenTelemetry export (optional)

### 11.2 Metrics
- [ ] ⏳ Expose Prometheus metrics từ mỗi service
- [ ] ⏳ Deploy Grafana
- [ ] ⏳ Tạo dashboards

### 11.3 Alerting
- [ ] ⏳ Cấu hình alerts (disk full, high CPU, service down)

---

## Phase 12: Testing & Quality Assurance

### 12.1 Unit Tests
- [ ] ⏳ Đạt coverage > 70% cho core business logic

### 12.2 Integration Tests
- [ ] ⏳ Test critical user journeys

### 12.3 E2E Tests
- [ ] ⏳ Playwright tests cho frontend

### 12.4 Load Testing
- [ ] ⏳ K6 scripts cho stress test
- [ ] ⏳ Test spike scenarios (Black Friday simulation)

### 12.5 Security
- [ ] ⏳ `cargo audit` cho Rust dependencies
- [ ] ⏳ `pnpm audit` cho frontend dependencies
- [ ] ⏳ Pentest OAuth flow
- [ ] ⏳ Pentest webhook endpoints

---

## 📝 Notes & Decisions Log

### 2025-01-10 - Phase 2 Complete! 🎉
- ✅ **DATABASE MIGRATIONS**: All foundation tables created and tested
  - 3 migrations applied successfully (extensions, core tables, Casbin)
  - PostgreSQL 16 running in Docker with proper schema
  - UUID v7 working correctly (timestamp-based for better indexing)
  - Multi-tenant isolation ready at database level
  - Casbin RBAC tables ready for authorization
- ✅ **TOOLS SETUP**: sqlx-cli installed, migration helper script created
- ✅ **DOCUMENTATION**: ERD in DBML format, ARCHITECTURE.md updated
- ⏳ **NEXT**: Integrate Casbin middleware into User Service, update repositories

### 2025-01-09
- ✅ **MAJOR REFACTOR**: User service migrated to production 3-crate pattern
  - Crate structure: `api` (binary) → `infra` (lib) → `core` (lib) → `shared/*` (libs)
  - Clean Architecture + DDD + Repository Pattern
  - Zero infrastructure dependencies in core domain logic
  - Generic handlers over service traits for testability
- ✅ Created 6 shared libraries: error, jwt, config, types, db, openapi
- ✅ Enforced snake_case naming: `user_service/` instead of `user-service/`
- ✅ Binary names still use kebab-case: `user-service` (Rust convention)
- ✅ Full OpenAPI 3.0 documentation with Swagger UI
- ✅ Authentication flow working: register → login → refresh
- ✅ JWT with tenant_id claim for multi-tenancy
- ✅ Password hashing with bcrypt (TODO: migrate to Argon2id)
- ✅ Comprehensive STRUCTURE.md and ARCHITECTURE.md documentation
- ⏳ **NEXT**: Database migrations, auth middleware, integration tests

### 2025-10-08
- ✅ Quyết định sử dụng CapRover thay vì Kubernetes để đơn giản hóa deployment
- ✅ Chọn NGINX (do CapRover quản lý) thay vì Traefik cho API Gateway
- ✅ Sử dụng Docker Swarm Overlay Network thay vì Service Mesh (Linkerd)
- ✅ Triết lý: "Sử dụng công cụ phổ biến, battle-tested" thay vì "viết mọi thứ bằng Rust"

---

## 🚀 Quick Commands

```bash
# Start local dev environment
cd infra/docker-compose && docker-compose up -d

# Build all services
cargo build --workspace

# Run a specific service
cargo run -p user-service

# Run database migrations
sqlx migrate run --database-url postgres://user:password@localhost:5432/inventory_db

# Format code
cargo fmt --all

# Lint code
cargo clippy --all -- -D warnings

# Run tests
cargo test --workspace
```

---

**Cập nhật lần cuối**: 2025-01-10  
**Tiến độ tổng thể**: ~30% (Database Foundation Complete)

### 📊 Progress Breakdown
- **Phase 1**: ✅ 95% complete (infrastructure, workspace, shared libs, auth crate)
- **Phase 2**: ✅ 100% complete (database migrations applied & tested)
- **Phase 3**: ⏳ 30% complete (user service needs migration to new schema & Casbin integration)
- **Phase 4-12**: ⏳ 0% complete (not started)

### 🎯 Immediate Next Steps (Priority Order)
1. ✅ ~~Update User Service repositories to use new database schema~~ (COMPLETED)
2. ✅ ~~Integrate Casbin middleware into User Service API~~ (COMPLETED)
3. ✅ ~~Implement session management (store in database, logout endpoint)~~ (COMPLETED 2025-01-10)
4. 🔴 **P0** Tenant isolation testing (CRITICAL SECURITY)
5. 🔴 **P0** Integration tests for auth endpoints
6. 🔴 **P0** Extract IP address & User-Agent in session management
7. 🟡 **P1** Implement tenant resolution in login
8. 🟡 **P1** Migrate password hashing to Argon2id
