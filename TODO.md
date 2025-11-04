# TODO - Inventory SaaS Platform

## 📊 Tổng Quan Tiến Độ

- **Giai đoạn hiện tại**: Phase 3 - User Service Production Integration (~30% complete)
- **Ngày bắt đầu**: 2025-10-08
- **Ngày cập nhật**: 2025-11-04
- **Mục tiêu**: MVP trong 2-3 tháng
- **Kiến trúc**: 3-Crate Pattern (Clean Architecture + DDD + Repository Pattern)
- **User Service**: ✅ Production-ready với authentication, JWT, Swagger UI

----

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
  - ✅ Axum middleware for JWT + permission check
  - ✅ Auth extractors: AuthUser, RequireAdmin, RequirePermission
  - ✅ Upgraded to Axum 0.8, SQLx 0.8, Tower 0.5
  - ✅ Workspace dependency management
  - ✅ Unit tests for extractors and error handling

### ⏳ 1.5 Pending Shared Libraries
- [ ] 🟡 **P1** `shared/events` crate (when implementing event-driven)
  - Event definitions
  - NATS client wrapper

### 1.6 Development Tools & Automation (Optional - P2)
- [ ] 🔵 **P2** Task automation (cargo-make / justfile)
  - Add when manual commands become repetitive
- [ ] 🔵 **P2** Pre-commit hooks
  - Add when team size > 1 person
- [ ] 🔵 **P2** Dev containers (.devcontainer)
  - Add when onboarding new developers
- [ ] 🔵 **P2** Dependency updates (Renovate/Dependabot)
  - Add when maintaining security patches becomes burden

----

## Phase 2: Database & Migrations ✅ (100% COMPLETE)

### ✅ 2.1 Database Design & Strategy (COMPLETED)
- [x] ✅ **Quyết định**: Application-level filtering (documented in ARCHITECTURE.md)
- [x] ✅ Shared schema với `tenant_id` trong mỗi bảng
- [x] ✅ No Postgres RLS (for simplicity and performance)
- [x] ✅ Repository pattern enforces tenant isolation
- [x] ✅ Type-safe tenant context in Rust

### ✅ 2.2 Database Standards (COMPLETED)
- [x] ✅ UUID v7 for all primary keys (timestamp-based)
- [x] ✅ BIGINT for currency (smallest unit: cents/xu)
- [x] ✅ TIMESTAMPTZ for all timestamps
- [x] ✅ Soft delete with `deleted_at` column
- [x] ✅ Application-level encryption for sensitive data
- [x] ✅ All documented in ARCHITECTURE.md

### ✅ 2.3 SQL Migrations (COMPLETED)
- [x] ✅ Migration directory structure (`migrations/`)
- [x] ✅ Migration 001: Extensions (uuid-ossp, pgcrypto) + uuid_generate_v7()
- [x] ✅ Migration 002: Core tables (tenants, users, sessions)
- [x] ✅ Migration 003: Casbin RBAC tables (casbin_rule)
- [x] ✅ Migration helper script (`scripts/migrate.sh`)
- [x] ✅ `.env.example` file with DATABASE_URL
- [x] ✅ Migration README with guidelines

### ✅ 2.4 Migration Testing & Deployment (COMPLETED)
- [x] ✅ Setup local PostgreSQL (Docker container)
- [x] ✅ Install sqlx-cli with postgres feature
- [x] ✅ Create .env file with DATABASE_URL
- [x] ✅ Run migrations successfully (all 3 migrations applied)
- [x] ✅ Verify database schema
- [x] ✅ Test UUID v7 generation (working correctly)
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

### ⏳ 2.5 Future Business Tables (Phase 4+)
- [ ] ⏳ **Phase 4**: Inventory tables (products, warehouses, inventory_levels, stock_moves)
- [ ] ⏳ **Phase 5**: Order tables (orders, order_items)
- [ ] ⏳ **Phase 6**: Integration tables (integrations, marketplace_sync)
- [ ] ⏳ **Phase 7**: Payment tables (payments, transactions)

### 2.6 Indexes & Optimization (P0/P1)
- [ ] 🔴 **P0** Tạo composite indexes cho multi-tenant queries
- [ ] 🔴 **P0** Tạo partial indexes cho performance
- [ ] 🟡 **P1** Table Partitioning cho large tables
- [ ] 🟡 **P1** Vacuum & Autovacuum tuning
- [ ] 🟡 **P1** Connection Pool Sizing
- [ ] 🟡 **P1** Query performance monitoring

### 2.7 Chạy Migrations
- [ ] ⏳ Chạy migrations: `sqlx migrate run --database-url postgres://...`
- [ ] ⏳ Verify schema trong PostgreSQL

----

## Phase 3: User Service (Auth & Tenancy)

### ✅ 3.0 Architecture Implementation (COMPLETED)
- [x] ✅ **3-Crate Pattern** fully implemented
  - [x] ✅ `user_service_api` - HTTP handlers, routing, OpenAPI, main.rs
  - [x] ✅ `user_service_core` - Domain models, DTOs, service/repository traits
  - [x] ✅ `user_service_infra` - PostgreSQL repo impl, service impl, bcrypt
- [x] ✅ **Clean separation of concerns**
- [x] ✅ **Generic handlers over service traits (testable!)**

### 3.1 Core Authentication

#### ✅ 3.1.1 User Registration (P0) - COMPLETED
- [x] ✅ **P0** Implement user registration endpoint
- [x] ✅ POST `/api/v1/auth/register`
- [x] ✅ Tạo tenant mới cho user đầu tiên
- [x] ✅ Hash password với **bcrypt**
- [x] ✅ Validate email format (validator crate)
- [x] ✅ Check email uniqueness
- [x] ✅ OpenAPI documentation with utoipa
- [x] ✅ Password strength validation with zxcvbn (Score 3+)
- [ ] 🟡 **P1** TODO: Migrate to Argon2id for better security

#### ✅ 3.1.2 Password Security (P0) - COMPLETED
- [x] ✅ **P0** Password Policy Enforcement - **COMPLETED**
- [x] ✅ Minimum length: 8 characters
- [x] ✅ Entropy analysis: zxcvbn Score 3+ required (Strong)
- [x] ✅ Context-aware: Checks against user email, name, tenant
- [x] ✅ Pattern detection: Common passwords, keyboard patterns, dates
- [x] ✅ Detailed feedback with suggestions
- [x] ✅ 9 comprehensive unit tests

#### 3.1.3 Login & Session Management (P0)
- [x] ✅ **P0** Implement login endpoint
- [x] ✅ POST `/api/v1/auth/login`
- [x] ✅ Generate JWT access token (15 min expiry) + refresh token (7 days)
- [x] ✅ Return tokens + user info
- [x] ✅ Store session in database with token hashes (SHA-256)
- [x] ✅ Extract `user_agent`, `ip_address` from HTTP request (proxy-aware)
- [ ] 🟡 **P1** TODO: Implement tenant resolution (currently creates new tenant)

#### 3.1.4 Security Headers (P0) - COMPLETED
- [x] ✅ **P0** Configure secure HTTP headers - **COMPLETED**
- [x] ✅ HSTS: max-age=31536000; includeSubDomains; preload
- [x] ✅ X-Content-Type-Options: nosniff
- [x] ✅ X-Frame-Options: DENY
- [x] ✅ Content-Security-Policy: default-src 'self' (Swagger UI compatible)
- [x] ✅ Referrer-Policy: strict-origin-when-cross-origin
- [x] ✅ X-Permitted-Cross-Domain-Policies: none
- [x] ✅ Uses tower_http::SetResponseHeaderLayer
- [x] ✅ OWASP compliant, zero performance impact

#### 3.1.5 Rate Limiting & Brute-Force Protection
- [ ] 🔴 **P0** Rate Limiting Implementation
  - **Login rate limit**: 5 attempts per IP per 5 minutes
  - **Forgot password**: 3 attempts per email per day
  - Use Redis for rate limit counters
  - Implement sliding window algorithm

#### 3.1.6 Audit Logging (P1 - Deferred)
- [ ] 🟡 **P1** Bảng `audit_logs`
- [ ] 🟡 **P1** Log critical actions

### 3.2 Authorization với Casbin (P0 - Core Infrastructure)

#### 3.2.1 Casbin Setup (P0)
- [ ] 🔴 **P0** Add dependencies to `shared/auth` crate
- [ ] 🔴 **P0** Tạo Casbin model file (`shared/auth/model.conf`)
- [ ] 🔴 **P0** Database Schema: `casbin_rule` table migration
- [ ] 🔴 **P0** Initialize Casbin enforcer in `shared/auth`
- [ ] 🔴 **P0** Axum middleware cho authorization
- [ ] 🔴 **P0** Axum extractor for role-based checks
- [ ] 🔴 **P0** Default Policies & Roles (admin, manager, user)
- [ ] 🔴 **P0** API endpoints for role management (admin only)
- [ ] 🔴 **P0** Testing: Unit tests + Integration tests

### 3.3 User Management
#### 3.3.1 Basic User CRUD (P0)
- [ ] 🔴 **P0** Endpoint: List users trong tenant
- [ ] 🔴 **P0** Endpoint: Get user by ID
- [ ] 🔴 **P0** Endpoint: Update user profile
- [ ] 🔴 **P0** Endpoint: Delete user (soft delete)

#### 3.3.2 Tenant Isolation Testing
- [ ] 🔴 **P0** **Critical Security Test**
- [ ] 🔴 **P0** Test scenarios: JWT modification, cross-tenant access

#### 3.3.3 User Invitation (P1)
- [ ] 🟡 **P1** Bảng `user_invitations`
- [ ] 🟡 **P1** Endpoint: Invite user mới
- [ ] 🟡 **P1** Endpoint: Accept invitation

#### 3.3.4 Advanced Features (P1)
- [ ] 🟡 **P1** Impersonate (Admin login as user)
- [ ] 🟡 **P1** SSO Integration (Enterprise feature)

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

----

## Phase 4: Inventory Service

### 4.1 Product Master Data (Item Master)
- [ ] 🔴 **P0** Bảng `products` (Item Master - Single Source of Truth)
- [ ] 🔴 **P0** Bảng `unit_of_measures` (UoM)
- [ ] 🔴 **P0** Bảng `uom_conversions` (Quy đổi UoM)
- [ ] 🟡 **P1** Bảng `product_variants` (Biến thể sản phẩm)
- [ ] ⏳ Bảng `item_groups` (Product Categories/Item Groups)
- [ ] ⏳ Endpoint: Create product
- [ ] ⏳ Endpoint: List products với filtering
- [ ] ⏳ Endpoint: Get product by ID/SKU
- [ ] ⏳ Endpoint: Update product
- [ ] ⏳ Endpoint: Delete/Archive product

### 4.2 Warehouse & Storage Locations
- [ ] 🔴 **P0** Bảng `warehouses`
- [ ] ⏳ Bảng `storage_locations`
- [ ] ⏳ Bảng `location_types` (Virtual Locations)
- [ ] ⏳ Endpoint: Manage warehouses
- [ ] ⏳ Endpoint: Manage storage locations

### 4.3 Stock Tracking & Inventory Levels
- [ ] ⏳ Bảng `inventory_levels`
- [ ] 🔴 **P0** Bảng `stock_moves` (Stock Ledger - **IMMUTABLE** audit trail)
- [ ] 🔴 **P0** Bảng `stock_adjustments` (Lý do điều chỉnh)
- [ ] ⏳ Endpoint: Get stock levels by warehouse
- [ ] ⏳ Endpoint: Stock movement history

### 4.4 Stock Operations (Quy trình nhập-xuất-chuyển-kiểm kê)

#### 4.4.1 Goods Receipt Note (GRN) - Nhập kho
- [ ] 🔴 **P0** Bảng `goods_receipts`
- [ ] 🔴 **P0** Bảng `goods_receipt_items`
- [ ] 🔴 **P0** Endpoint: Create GRN
- [ ] 🔴 **P0** Endpoint: Complete/Validate GRN

#### 4.4.2 Delivery Order (DO) - Xuất kho
- [ ] 🔴 **P0** Bảng `delivery_orders`
- [ ] 🔴 **P0** Bảng `delivery_order_items`
- [ ] 🔴 **P0** Endpoint: Create DO from Order
- [ ] 🔴 **P0** Endpoint: Pick items for DO
- [ ] 🔴 **P0** Endpoint: Pack items
- [ ] 🔴 **P0** Endpoint: Ship/Validate DO

#### 4.4.3 Stock Transfer - Chuyển kho nội bộ
- [ ] 🔴 **P0** Bảng `stock_transfers`
- [ ] 🔴 **P0** Bảng `stock_transfer_items`
- [ ] 🔴 **P0** Endpoint: Create Transfer
- [ ] 🔴 **P0** Endpoint: Confirm Transfer
- [ ] 🔴 **P0** Endpoint: Receive Transfer

#### 4.4.4 Stock Take / Physical Inventory Count - Kiểm kê
- [ ] 🔴 **P0** Bảng `stock_takes`
- [ ] 🔴 **P0** Bảng `stock_take_lines`
- [ ] 🔴 **P0** Endpoint: Create Stock Take
- [ ] 🔴 **P0** Endpoint: Scan/Count items
- [ ] 🔴 **P0** Endpoint: Finalize Stock Take

#### 4.4.5 Returned Merchandise Authorization (RMA)
- [ ] 🟡 **P1** Bảng `rma_requests`
- [ ] 🟡 **P1** Bảng `rma_items`
- [ ] 🟡 **P1** Endpoint: Create RMA
- [ ] 🟡 **P1** Endpoint: Approve RMA
- [ ] 🟡 **P1** Endpoint: Receive returned goods

### 4.5 Lot & Serial Number Tracking (Traceability)
- [ ] 🔴 **P0** Bảng `lots_serial_numbers`
- [ ] ⏳ Bảng `lot_serial_moves` (Lot/Serial traceability)
- [ ] 🔴 **P0** Enable Lot/Serial Number tracking per product
- [ ] 🟡 **P1** FEFO (First Expiry First Out) picking strategy
- [ ] ⏳ Endpoint: Assign lot/serial numbers during receipt
- [ ] 🟡 **P1** Endpoint: Track lot/serial lifecycle
- [ ] ⏳ Display lot/serial numbers on delivery documents

### 4.6 Inventory Valuation (Định giá tồn kho)
- [ ] ⏳ Bảng `inventory_valuations`
- [ ] ⏳ Bảng `stock_valuation_layers` (FIFO/AVCO tracking)
- [ ] ⏳ Support 3 valuation methods: FIFO, AVCO, Standard Cost
- [ ] ⏳ Endpoint: Inventory valuation report
- [ ] ⏳ Endpoint: Revalue inventory manually

### 4.7 Stock Replenishment (Tự động đặt hàng bổ sung)
- [ ] 🟡 **P1** Bảng `reorder_rules`
- [ ] 🟡 **P1** Automated reorder detection
- [ ] ⏳ Material Requests (yêu cầu vật tư)

### 4.8 Batch/Wave/Cluster Picking (Tối ưu picking)
- [ ] ⏳ Bảng `pick_lists`
- [ ] ⏳ Bảng `pick_list_items`
- [ ] ⏳ Generate pick lists
- [ ] ⏳ Put-away strategies

### 4.9 Stock Reports & Analytics
- [ ] 🔴 **P0** Stock Ledger Report (ERPNext-style)
- [ ] 🔴 **P0** Inventory Reconciliation Report (Cân đối kho)
- [ ] 🟡 **P1** Stock aging report
- [ ] 🟡 **P1** Stock movement report
- [ ] 🟡 **P1** Inventory turnover ratio
- [ ] 🟡 **P1** Low stock alerts
- [ ] 🟡 **P1** Dead Stock Report
- [ ] 🔴 **P0** Inventory valuation report
- [ ] 🟡 **P1** Stock by Lot/Serial Report

### 4.10 Real-time Updates & Events
- [ ] ⏳ Subscribe NATS events từ Integration Service
- [ ] ⏳ Publish NATS events khi stock thay đổi

### 4.11 Technical Implementation (P0 - Critical)
- [ ] 🔴 **P0** Idempotency & Concurrency Control
- [ ] 🔴 **P0** Distributed Locking (Redis Redlock)
- [ ] 🔴 **P0** Database Row-Level Locking
- [ ] 🔴 **P0** Event-Driven Architecture (Saga Pattern)
- [ ] 🔴 **P0** Outbox Pattern for reliable events
- [ ] 🔴 **P0** Dead Letter Queue (DLQ) cho NATS
- [ ] 🔴 **P0** Saga Orchestration for complex flows
- [ ] 🟡 **P1** Performance Optimization
- [ ] 🟡 **P1** Mobile/Barcode Integration

### 4.12 Multi-Echelon Inventory (P2 - Advanced)
- [ ] 🔵 **P2** Bảng `distribution_network`
- [ ] 🔵 **P2** Demand Forecasting

### 4.13 Testing & Quality Assurance
- [ ] ⏳ Unit tests cho business logic
- [ ] ⏳ Integration tests cho API endpoints
- [ ] ⏳ Test concurrent stock updates (race conditions)
- [ ] ⏳ Performance tests

----

## Phase 5: Order Service

### 5.1 Order Management
- [ ] ⏳ Endpoint: Create order
- [ ] ⏳ Endpoint: List orders
- [ ] ⏳ Endpoint: Get order by ID
- [ ] ⏳ Endpoint: Update order status

### 5.2 Order Processing với Event-Driven
- [ ] ⏳ Subscribe event: `order.placed` (từ Integration Service)
- [ ] ⏳ Subscribe event: `payment.completed`
- [ ] ⏳ Subscribe event: `order.cancelled`

### 5.3 Fulfillment
- [ ] ⏳ Endpoint: Mark order as fulfilled

### 5.4 Testing
- [ ] ⏳ Unit tests
- [ ] ⏳ Integration tests
- [ ] ⏳ Test order flow end-to-end với events

----

## Phase 6: Integration Service (Marketplace)

### 6.1 Adapter Pattern Setup
- [ ] ⏳ Định nghĩa trait `MarketplaceAdapter`
- [ ] ⏳ Implement `ShopeeAdapter`
- [ ] ⏳ Implement `LazadaAdapter`
- [ ] ⏳ Implement `TikiAdapter`

### 6.2 Integration Management
- [ ] ⏳ Endpoint: Connect marketplace
- [ ] ⏳ Endpoint: OAuth callback handler
- [ ] ⏳ Endpoint: List integrations
- [ ] ⏳ Endpoint: Disconnect integration

### 6.3 Sync Logic
- [ ] ⏳ Implement product sync (push inventory to marketplace)
- [ ] ⏳ Implement order sync (pull orders from marketplace)
- [ ] ⏳ Implement webhook receiver

### 6.4 Testing
- [ ] ⏳ Mock marketplace APIs cho testing
- [ ] ⏳ Test sync flow
- [ ] ⏳ Test webhook handling

----

## Phase 7: Payment Service

### 7.1 Payment Gateway Integration
- [ ] ⏳ Implement VNPay adapter
- [ ] ⏳ Implement Stripe adapter
- [ ] ⏳ (Optional) MoMo, ZaloPay adapters

### 7.2 Payment Processing
- [ ] ⏳ Endpoint: Create payment intent
- [ ] ⏳ Endpoint: Handle gateway callback/webhook
- [ ] ⏳ Endpoint: Get payment status

### 7.3 Refunds
- [ ] ⏳ Endpoint: Process refund

### 7.4 Testing
- [ ] ⏳ Unit tests
- [ ] ⏳ Integration tests với mock gateways
- [ ] ⏳ Test idempotency

----

## Phase 8: Frontend (SvelteKit)

### 8.1 Thiết Lập Project
- [ ] ⏳ Init SvelteKit project trong `frontend/`
- [ ] ⏳ Cài đặt dependencies
- [ ] ⏳ Enable TypeScript strict mode

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

----

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

----

## Phase 10: Deployment (CapRover)

### 10.1 Chuẩn Bị CapRover
- [ ] ⏳ Cài đặt CapRover trên server (VPS)
- [ ] ⏳ Cấu hình domain cho CapRover

### 10.2 Deploy Stateful Services
- [ ] ⏳ Deploy PostgreSQL (One-Click App)
- [ ] ⏳ Deploy Redis (One-Click App)
- [ ] ⏳ Deploy NATS (One-Click App)

### 10.3 Deploy Microservices
- [ ] ⏳ Tạo `Dockerfile` cho mỗi service (multi-stage build)
- [ ] ⏳ Tạo CapRover app cho từng service
- [ ] ⏳ Cấu hình environment variables cho mỗi app
- [ ] ⏳ Enable HTTP/HTTPS và wildcard SSL

### 10.4 Deploy Frontend
- [ ] ⏳ Build SvelteKit app
- [ ] ⏳ Tạo CapRover app cho frontend
- [ ] ⏳ Cấu hình domain

### 10.5 CI/CD
- [ ] ⏳ Tạo GitHub Actions workflow
- [ ] ⏳ Trigger deploy trên CapRover khi merge to `main`

----

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

----

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
- [ ] ⏳ Pentest OAuth flow
- [ ] ⏳ Pentest webhook endpoints

----

## 📝 Notes & Decisions Log

### 2025-11-04 - Documentation Recovery Complete! 📚
- ✅ **DOCUMENTATION RECOVERY**: All production deployment docs restored
- ✅ **Production Deployment Guide**: Complete CapRover deployment instructions
- ✅ **Monitoring Setup Guide**: Prometheus, Grafana, Loki, AlertManager configuration
- ✅ **Troubleshooting Guide**: Comprehensive error resolution guide
- ✅ **README.md**: Updated with current architecture and quick start
- ✅ **TODO.md**: Current progress tracking restored

### 2025-01-10 - Security Hardening Complete! 🔒
- ✅ **SECURITY HEADERS**: OWASP-compliant HTTP security headers
- ✅ **PASSWORD STRENGTH**: zxcvbn entropy-based validation
- 📊 **PHASE 3 STATUS**: 95% complete - Production ready!

### 2025-01-10 - Session Management & Testing Complete! 🎉
- ✅ **SESSION MANAGEMENT**: Full session lifecycle implemented
- ✅ **CLIENT METADATA**: Custom ClientInfo extractor
- ✅ **TESTING**: Comprehensive integration testing
- ✅ **SECURITY**: All P0 auth tasks completed

### 2025-01-10 - Phase 2 Complete! 🎉
- ✅ **DATABASE MIGRATIONS**: All foundation tables created and tested
- ✅ **TOOLS SETUP**: sqlx-cli installed, migration helper script created
- ✅ **DOCUMENTATION**: ERD in DBML format, ARCHITECTURE.md updated

### 2025-01-09 - Major Refactor Complete! 🏗️
- ✅ **3-CRATE PATTERN**: User service migrated to production architecture
- ✅ **CLEAN ARCHITECTURE**: Core has zero infrastructure dependencies
- ✅ **TESTABILITY**: Generic handlers over service traits
- ✅ **SHARED LIBRARIES**: 6 shared crates created and working

### 2025-10-08 - Project Initialization Complete! 🚀
- ✅ **MICROSERVICES SKELETON**: All 5 services with basic structure
- ✅ **CARGO WORKSPACE**: Multi-crate setup working
- ✅ **DOCKER COMPOSE**: Local development environment
- ✅ **CI/CD**: GitHub Actions workflows

----

## 🚀 Quick Commands

```bash
# Start local dev environment
docker-compose up -d

# Build all services
cargo build --workspace

# Run a specific service
cargo run --bin user-service

# Run database migrations
sqlx migrate run

# Format code
cargo fmt --all

# Lint code
cargo clippy --all -- -D warnings

# Run tests
cargo test --workspace
```

----

**Cập nhật lần cuối**: 2025-11-04
**Tiến độ tổng thể**: ~30% (User Service Production Integration)
**MVP Target**: 2-3 tháng

### 📊 Progress Breakdown
- **Phase 1**: ✅ 100% complete (infrastructure, workspace, shared libs, auth crate)
- **Phase 2**: ✅ 100% complete (database migrations applied & tested)
- **Phase 3**: ✅ 95% complete (user service auth & security features complete)
- **Phase 4-12**: ⏳ 0% complete (not started)

### 🎯 Immediate Next Steps (Priority Order)
1. ✅ ~~Update User Service repositories to use new database schema~~ (COMPLETED)
2. ✅ ~~Integrate Casbin middleware into User Service API~~ (COMPLETED)
3. ✅ ~~Implement session management (store in database, logout endpoint)~~ (COMPLETED)
4. ✅ ~~Extract IP address & User-Agent in session management~~ (COMPLETED)
5. ✅ ~~Integration tests for auth endpoints~~ (COMPLETED - test_session_flow.sh)
6. ✅ ~~Tenant isolation testing~~ (VERIFIED - database-level isolation working)
7. 🟡 **P1** Implement tenant resolution in login
8. 🟡 **P1** Migrate password hashing to Argon2id
9. 🔵 **P2** Add rate limiting for auth endpoints
10. 🔵 **P2** Implement audit logging for auth events
