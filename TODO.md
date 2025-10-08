# TODO - Inventory SaaS Platform

## 📊 Tổng Quan Tiến Độ

- **Giai đoạn hiện tại**: Phase 1 - Thiết lập cơ sở hạ tầng
- **Ngày bắt đầu**: 2025-10-08
- **Mục tiêu**: MVP trong 2-3 tháng

---

## Phase 1: Thiết Lập Cơ Sở Hạ Tầng & Workspace

### 1.1 Thiết Lập Môi Trường Phát Triển
- [x] ✅ Tạo thư mục dự án và khởi tạo git repo
- [x] ✅ Tạo file ARCHITECTURE.md với kiến trúc CapRover
- [x] ✅ Tạo cấu trúc thư mục cho các microservices
- [x] ✅ Tạo Cargo workspace (Cargo.toml gốc)
- [x] ✅ Tạo docker-compose.yml cho môi trường local
- [ ] 🔄 Cài đặt Rust toolchain (stable + nightly)
  - `rustup default stable`
  - `rustup toolchain add nightly`
  - `rustup component add clippy rustfmt`
- [ ] 🔄 Cài đặt công cụ phát triển
  - `cargo install cargo-watch` (auto-reload)
  - `cargo install sqlx-cli --features postgres` (database migrations)
  - `cargo install cargo-make` (task runner)
- [ ] 🔄 Thiết lập Docker & Docker Compose trên máy local
- [ ] 🔄 Khởi động môi trường local dev
  - `cd infra/docker-compose && docker-compose up -d`

### 1.2 Khởi Tạo Các Microservices
- [x] ✅ Tạo skeleton cho user-service
- [x] ✅ Tạo skeleton cho inventory-service  
- [x] ✅ Tạo skeleton cho order-service
- [x] ✅ Tạo skeleton cho integration-service
- [x] ✅ Tạo skeleton cho payment-service
- [ ] 🔄 Test build tất cả services: `cargo build --workspace`
- [ ] 🔄 Test chạy từng service riêng lẻ

### 1.3 Thiết Lập Shared Libraries
- [ ] ⏳ Tạo `shared/common` crate
  - Error types (thiserror)
  - Result wrappers
  - Tracing setup helpers
  - Configuration management (config/figment)
- [ ] ⏳ Tạo `shared/db` crate
  - SQLx connection pool setup
  - Tenant context extractor
  - Common query helpers
- [ ] ⏳ Tạo `shared/auth` crate  
  - JWT token generation/validation (jsonwebtoken)
  - Casbin enforcer setup
  - Axum middleware cho authentication
  - Axum middleware cho authorization
- [ ] ⏳ Tạo `shared/events` crate
  - Event type definitions (serde)
  - NATS client wrapper
  - Publish/Subscribe helpers

---

## Phase 2: Database & Migrations

### 2.1 Thiết Kế Database Schema
- [ ] ⏳ Thiết kế schema cho multi-tenancy
  - Quyết định chiến lược: Shared schema với tenant_id
  - Row-Level Security policies (nếu dùng)
- [ ] ⏳ Tạo ERD (Entity Relationship Diagram)
- [ ] ⏳ Viết SQL migration files trong `infra/sql-migrations/`

### 2.2 Core Tables
- [ ] ⏳ Bảng `tenants`
  - tenant_id (UUID, PK)
  - name, plan, settings (JSONB)
  - created_at, updated_at
- [ ] ⏳ Bảng `users`
  - user_id (UUID, PK)
  - tenant_id (FK)
  - email, password_hash, role
  - created_at, updated_at
- [ ] ⏳ Bảng `sessions`
  - session_id, user_id, tenant_id
  - access_token_hash, refresh_token_hash
  - expires_at
- [ ] ⏳ Bảng `products`
  - product_id (UUID, PK)
  - tenant_id (FK)
  - sku, name, description, variants (JSONB)
- [ ] ⏳ Bảng `inventory_levels`
  - tenant_id, product_id, warehouse_id
  - quantity, reserved_quantity
  - Composite PK hoặc unique constraint
- [ ] ⏳ Bảng `warehouses`
  - warehouse_id, tenant_id
  - name, location (JSONB)
- [ ] ⏳ Bảng `orders`
  - order_id, tenant_id
  - customer_info (JSONB), status
  - channel (marketplace/web), created_at
- [ ] ⏳ Bảng `order_items`
  - order_id, product_id, quantity, price
- [ ] ⏳ Bảng `integrations`
  - integration_id, tenant_id
  - platform (shopee/lazada/tiki...), credentials (encrypted), status
- [ ] ⏳ Bảng `payments`
  - payment_id, tenant_id, order_id
  - gateway, amount, status, transaction_id

### 2.3 Indexes & Optimization
- [ ] ⏳ Tạo composite indexes cho multi-tenant queries
  - `(tenant_id, sku)` on products
  - `(tenant_id, status, created_at)` on orders
- [ ] ⏳ Tạo partial indexes cho performance
  - Active integrations
  - Pending orders

### 2.4 Chạy Migrations
- [ ] ⏳ Chạy migrations: `sqlx migrate run --database-url postgres://...`
- [ ] ⏳ Verify schema trong PostgreSQL

---

## Phase 3: User Service (Auth & Tenancy)

### 3.1 Core Authentication
- [ ] ⏳ Implement user registration endpoint
  - POST `/api/v1/auth/register`
  - Tạo tenant mới cho user đầu tiên
  - Hash password (argon2/bcrypt)
- [ ] ⏳ Implement login endpoint
  - POST `/api/v1/auth/login`
  - Generate JWT access token + refresh token
  - Lưu session vào database
- [ ] ⏳ Implement refresh token endpoint
  - POST `/api/v1/auth/refresh`
- [ ] ⏳ Implement logout endpoint
  - POST `/api/v1/auth/logout`

### 3.2 Authorization với Casbin
- [ ] ⏳ Tạo Casbin model file (`model.conf`)
  - Multi-tenant RBAC: `sub, dom, obj, act`
- [ ] ⏳ Tạo Casbin adapter cho PostgreSQL
  - Store policies trong bảng `casbin_rule`
- [ ] ⏳ Implement Axum middleware cho authorization
  - Extract JWT → Extract tenant_id + user_id
  - Load enforcer với policies của tenant
  - Enforce quyền truy cập

### 3.3 User Management
- [ ] ⏳ Endpoint: List users trong tenant
  - GET `/api/v1/users`
- [ ] ⏳ Endpoint: Invite user mới
  - POST `/api/v1/users/invite`
- [ ] ⏳ Endpoint: Cập nhật user role
  - PATCH `/api/v1/users/:user_id/role`

### 3.4 Testing
- [ ] ⏳ Viết unit tests cho authentication logic
- [ ] ⏳ Viết integration tests cho API endpoints
- [ ] ⏳ Test authorization với Casbin

---

## Phase 4: Inventory Service

### 4.1 Product Master Data (Item Master)
- [ ] ⏳ Bảng `products` (Item Master - Single Source of Truth)
  - product_id (UUID, PK)
  - tenant_id (FK)
  - sku (unique per tenant)
  - name, description
  - item_group_id (FK) - Phân loại sản phẩm theo nhóm
  - product_type (storable, consumable, service, digital)
  - track_inventory (boolean)
  - created_at, updated_at
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
- [ ] ⏳ Bảng `warehouses`
  - warehouse_id (UUID, PK)
  - tenant_id (FK)
  - name, code (unique per tenant)
  - warehouse_type (physical, virtual, transit)
  - address (JSONB)
  - is_active (boolean)
  - parent_warehouse_id (FK) - Tree structure cho multi-level warehouses
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
- [ ] ⏳ Bảng `stock_moves` (Stock Ledger - audit trail)
  - move_id (UUID, PK)
  - tenant_id, product_id
  - source_location_id, destination_location_id
  - move_type (receipt, delivery, transfer, adjustment, return)
  - quantity
  - unit_cost (valuation at time of move)
  - reference_type (order_id, transfer_id, adjustment_id)
  - reference_id (UUID)
  - move_date, created_by
  - status (draft, confirmed, done, cancelled)
- [ ] ⏳ Endpoint: Get stock levels by warehouse
  - GET `/api/v1/inventory/stock`
  - Filter by warehouse_id, location_id, product_id
  - Show available vs reserved quantities
- [ ] ⏳ Endpoint: Stock movement history
  - GET `/api/v1/inventory/stock/movements`
  - Audit trail của tất cả stock moves
  - Filter by product, date range, move_type

### 4.4 Stock Operations
- [ ] ⏳ **Stock Receipts** (Nhập kho)
  - POST `/api/v1/inventory/receipts`
  - Create stock move từ Supplier → Warehouse
  - Update inventory_levels
  - Publish event: `inventory.receipt.completed`
- [ ] ⏳ **Stock Deliveries** (Xuất kho)
  - POST `/api/v1/inventory/deliveries`
  - Create stock move từ Warehouse → Customer
  - Validate stock availability
  - Update inventory_levels
  - Publish event: `inventory.delivery.completed`
- [ ] ⏳ **Internal Transfers** (Chuyển kho nội bộ)
  - POST `/api/v1/inventory/transfers`
  - Move stock giữa warehouses hoặc locations
  - Support multi-step transfers (WH1 → Transit → WH2)
  - Use "Inter-warehouse Transit" virtual location
- [ ] ⏳ **Stock Adjustments/Reconciliation** (Kiểm kê)
  - POST `/api/v1/inventory/adjustments`
  - Correct discrepancies (physical count vs system)
  - Adjust quantity_on_hand
  - Log adjustments vào stock_moves với move_type="adjustment"
  - Sử dụng "Inventory Loss" virtual location cho negative adjustments
  - Reasons: damaged, expired, stolen, counting error
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

### 4.5 Lot & Serial Number Tracking (Traceability)
- [ ] ⏳ Bảng `lots_serial_numbers`
  - lot_serial_id (UUID, PK)
  - tenant_id, product_id
  - tracking_type (lot, serial)
  - lot_number / serial_number (unique)
  - manufacturing_date, expiry_date
  - supplier_info (JSONB)
  - quantity (for lots), always 1 for serial numbers
  - status (available, reserved, sold, returned, quarantined)
  - location_id (current location)
  - created_at
- [ ] ⏳ Bảng `lot_serial_moves` (Lot/Serial traceability)
  - move_id (FK to stock_moves)
  - lot_serial_id (FK)
  - quantity
  - source_location, dest_location
- [ ] ⏳ Enable Lot/Serial Number tracking per product
  - Add field `tracking_method` in products table (none, lot, serial)
  - Serial numbers: unique per unit (1 serial = 1 product)
  - Lot numbers: batch tracking (1 lot = multiple units)
- [ ] ⏳ Endpoint: Assign lot/serial numbers during receipt
  - POST `/api/v1/inventory/receipts/:id/assign-tracking`
  - Bulk generation of serial numbers
  - Import serial/lot numbers from CSV
- [ ] ⏳ Endpoint: Track lot/serial lifecycle
  - GET `/api/v1/inventory/tracking/:lot_serial_id`
  - Full traceability: origin → warehouses → customer
  - Show all movements and current status
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

### 4.7 Stock Replenishment (Tự động đặt hàng bổ sung)
- [ ] ⏳ Bảng `reorder_rules`
  - rule_id (UUID, PK)
  - tenant_id, product_id, warehouse_id
  - min_quantity (reorder level)
  - max_quantity (max stock level)
  - reorder_quantity (quantity to order)
  - lead_time_days
  - is_active (boolean)
- [ ] ⏳ Automated reorder detection
  - Background job check inventory_levels.quantity_available
  - If quantity < min_quantity → trigger reorder
  - Create Material Request or Purchase Order
  - Publish event: `inventory.reorder.triggered`
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
- [ ] ⏳ Stock aging report
  - GET `/api/v1/inventory/reports/aging`
  - Identify slow-moving và dead stock
- [ ] ⏳ Stock movement report
  - GET `/api/v1/inventory/reports/movements`
  - Inbound vs outbound by period
  - By product, warehouse, item group
- [ ] ⏳ Inventory turnover ratio
  - GET `/api/v1/inventory/reports/turnover`
  - COGS / Average Inventory Value
- [ ] ⏳ Low stock alerts
  - GET `/api/v1/inventory/reports/low-stock`
  - Products below reorder level
- [ ] ⏳ Inventory valuation report
  - GET `/api/v1/inventory/reports/valuation`
  - Total value by warehouse, product category
  - Historical valuation comparison

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

### 4.11 Testing & Quality Assurance
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

**Cập nhật lần cuối**: 2025-10-08  
**Tiến độ tổng thể**: ~10% (Hoàn thành thiết lập cơ bản)
