# TASKS_OVERVIEW.md - Project Task Overview

## 🎯 Project Goal

**Anthill** - Multi-tenant inventory management SaaS platform using Rust microservices architecture.

**Tech Stack**:
- Backend: Rust + Axum 0.8 + Tokio + SQLx + PostgreSQL
- Architecture: 3-Crate Pattern (Clean Architecture + DDD + Repository Pattern)
- Infrastructure: CapRover + Docker Swarm + NATS + Redis
- Status: Phase 3 - User Service Production Ready (~30% complete)

## 🏗️ Architecture Overview

**Multi-Tenancy Strategy**: Dual layer security (PostgreSQL RLS + Casbin RBAC)
**Service Structure**: Microservices with shared libraries
**Database**: Single PostgreSQL instance with tenant isolation
**Deployment**: CapRover PaaS with automatic deployments

## 🗂️ Roadmap & Status

### [✅] Phase 1: Infrastructure & Workspace - `Completed 95%`
    - [✅] 1.1 Basic Setup - `Completed`
          → [View folder](./V1_MVP/01_Infrastructure_Setup/1.1_Basic_Setup/)
          → Completed: 2025-01-10
          → 6/6 tasks done

    - [✅] 1.2 Microservices Skeleton - `Completed`
          → [View folder](./V1_MVP/01_Infrastructure_Setup/1.2_Microservices_Skeleton/)
          → Completed: 2025-01-09
          → 5/5 services created

    - [✅] 1.3 Shared Libraries - `Completed`
          → [View folder](./V1_MVP/01_Infrastructure_Setup/1.3_Shared_Libraries/)
          → Completed: 2025-01-09
          → 6/6 libraries implemented

### [✅] Phase 2: Database & Migrations - `Completed 100%`
    - [✅] 2.1 Database Design & Strategy - `Completed`
          → [View folder](./V1_MVP/02_Database_Foundations/2.1_Database_Design/)
          → Completed: 2025-01-10
          → 4/4 components completed

    - [✅] 2.2 Migration Testing & Deployment - `Completed`
          → [View folder](./V1_MVP/02_Database_Foundations/2.2_Migration_Testing/)
          → Completed: 2025-01-10
          → 3/3 migrations applied and tested

### [🔄] Phase 3: User Service (Kanidm Integration & Tenancy) - `In Progress 68%`

> **Timeline**: Weeks 3-5 (21 days)  
> **Dependencies**: Phase 1, 2 completed, Kanidm server
> **Progress**: 14/28 tasks Done + 5 NeedsReview + 2 InProgress (Updated: 2025-12-24)

    - [✅] 3.0 Architecture Implementation - `Completed`
          → [View folder](./V1_MVP/03_User_Service/3.0_Architecture/)
          → Completed: 2025-01-09

    - [🔄] 3.1 Kanidm Integration - `In Progress 67%`
          → [View folder](./V1_MVP/03_User_Service/3.1_Kanidm_Integration/)
          → Replaces custom JWT auth with Kanidm OAuth2/OIDC
          → Progress: 4/6 Done, 1 InProgress, 1 Todo (Updated: 2025-12-24)

    - [🔄] 3.2 Authorization with Casbin - `In Progress 92%`
          → [View folder](./V1_MVP/03_User_Service/3.2_Casbin_Authorization/)
          → Progress: 8/12 Done, 3 NeedsReview, 1 InProgress (Updated: 2025-12-24)

    - [🔄] 3.3 User Management - `In Progress 60%`
          → [View folder](./V1_MVP/03_User_Service/3.3_User_Management/)
          → Progress: 2/5 Done, 1 NeedsReview, 2 Todo (Updated: 2025-12-24)

    - [🔄] 3.4 Testing - `In Progress 20%`
          → [View folder](./V1_MVP/03_User_Service/3.4_Testing/)
          → Progress: 0/5 Done, 1 NeedsReview, 4 Todo (Updated: 2025-12-24)

### [🔄] Phase 4: Inventory Service - `In Progress 90%` *(Major Progress)*
    - [🔄] 4.1 Product Master - `In Progress 83%`
          → [View folder](./V1_MVP/04_Inventory_Service/4.1_Product_Master/)
          → Progress: 5/6 Done, 1 InProgress (Updated: 2025-12-24)

    - [✅] 4.2 Warehouse Management - `Done 100%`
          → [View folder](./V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/)
          → Progress: 1/1 tasks completed (Updated: 2025-12-24)

    - [✅] 4.3 Stock Operations Core - `Done 100%`
          → [View folder](./V1_MVP/04_Inventory_Service/4.3_Stock_Operations/)
          → Progress: 3/3 tasks completed (Updated: 2025-12-24)

    - [🔄] 4.4 Stock Transactions - `In Progress 95%`
          → [View folder](./V1_MVP/04_Inventory_Service/4.4_Stock_Operations/)
          → Progress: 18/19 Done, 1 NeedsReview (Updated: 2025-12-24)
          → **Note**: Consolidates GRN, DO, Transfers, Adjustments, RMA into unified workflow

    - [✅] 4.5 Lot & Serial Tracking - `Done 75%`
          → [View folder](./V1_MVP/04_Inventory_Service/4.5_Lot_Serial_Tracking/)
          → Progress: 3/4 Done, 1 NeedsReview (Updated: 2025-12-24)

    - [⏳] 4.6 Inventory Valuation - `Todo`
          → [View folder](./V1_MVP/04_Inventory_Service/4.6_Inventory_Valuation/)
          → Progress: 0/1 tasks completed
          → **Note**: Core valuation methods (FIFO, Average, Standard)

    - [🔄] 4.7 Stock Replenishment - `In Progress 50%`
          → [View folder](./V1_MVP/04_Inventory_Service/4.7_Stock_Replenishment/)
          → Progress: 0/2 Done, 1 NeedsReview, 1 Todo (Updated: 2025-12-24)

    - [✅] 4.8 Quality Management - `Done 100%`
          → [View folder](./V1_MVP/04_Inventory_Service/4.8_Quality_Management/)
          → Progress: 4/4 tasks completed (Updated: 2025-12-24)
          → **Features**: QC points, checks, alerts, inventory integration

    - [🔄] 4.9 Stock Reports & Analytics - `NeedsReview 100%`
          → [View folder](./V1_MVP/04_Inventory_Service/4.9_Stock_Reports_Analytics/)
          → Progress: 0/2 Done, 2 NeedsReview (Updated: 2025-12-24)

    - [✅] 4.10 Advanced Warehouse Features - `Done 100%`
          → [View folder](./V1_MVP/04_Inventory_Service/4.10_Advanced_Warehouse/)
          → Progress: 3/3 tasks completed (Updated: 2025-12-24)
          → **Features**: Putaway rules, advanced picking, removal strategies

    - [🔄] 4.11 Technical Implementation - `In Progress 67%`
          → [View folder](./V1_MVP/04_Inventory_Service/4.11_Technical_Implementation/)
          → Progress: 4/9 Done, 2 NeedsReview, 1 Todo (Updated: 2025-12-24)
          → **Note**: Removed mobile PWA from MVP scope

    - [⏳] 4.12 Multi-Echelon Inventory - `Todo`
          → [View folder](./V1_MVP/04_Inventory_Service/4.12_Multi_Echelon_Inventory/)
          → Progress: 0/2 tasks completed
          → **Note**: Simplified for MVP - basic distribution network only

    - [🔄] 4.13 Testing & Quality Assurance - `In Progress 83%`
          → [View folder](./V1_MVP/04_Inventory_Service/4.13_Testing_Quality_Assurance/)
          → Progress: 3/6 Done, 1 InProgress, 2 NeedsReview (Updated: 2025-12-24)

### [⏳] Phase 5: Order Service - `Todo 0%`
    - [⏳] 5.1 Order Management - `Todo`
          → [View folder](./V1_MVP/05_Order_Service/5.1_Order_Management/)
          → Progress: 0/4 tasks completed

    - [⏳] 5.2 Order Processing - `Todo`
          → [View folder](./V1_MVP/05_Order_Service/5.2_Order_Processing/)
          → Progress: 0/3 tasks completed

    - [⏳] 5.3 Fulfillment - `Todo`
          → [View folder](./V1_MVP/05_Order_Service/5.3_Fulfillment/)
          → Progress: 0/1 tasks completed

    - [⏳] 5.4 Testing - `Todo`
          → [View folder](./V1_MVP/05_Order_Service/5.4_Testing/)
          → Progress: 0/3 tasks completed

### [⏳] Phase 6: Integration Service (Marketplace) - `Todo 0%`
    - [⏳] 6.1 Adapter Pattern Setup - `Todo`
          → [View folder](./V1_MVP/06_Integration_Service/6.1_Adapter_Pattern/)
          → Progress: 0/3 tasks completed

    - [⏳] 6.2 Integration Management - `Todo`
          → [View folder](./V1_MVP/06_Integration_Service/6.2_Integration_Management/)
          → Progress: 0/4 tasks completed

    - [⏳] 6.3 Sync Logic - `Todo`
          → [View folder](./V1_MVP/06_Integration_Service/6.3_Sync_Logic/)
          → Progress: 0/3 tasks completed

    - [⏳] 6.4 Testing - `Todo`
          → [View folder](./V1_MVP/06_Integration_Service/6.4_Testing/)
          → Progress: 0/3 tasks completed

### [⏳] Phase 7: Payment Service - `Todo 0%`
    - [⏳] 7.1 Payment Gateway Integration - `Todo`
          → [View folder](./V1_MVP/07_Payment_Service/7.1_Payment_Gateway/)
          → Progress: 0/2 tasks completed

    - [⏳] 7.2 Payment Processing - `Todo`
          → [View folder](./V1_MVP/07_Payment_Service/7.2_Payment_Processing/)
          → Progress: 0/3 tasks completed

    - [⏳] 7.3 Refunds - `Todo`
          → [View folder](./V1_MVP/07_Payment_Service/7.3_Refunds/)
          → Progress: 0/1 tasks completed

    - [⏳] 7.4 Testing - `Todo`
          → [View folder](./V1_MVP/07_Payment_Service/7.4_Testing/)
          → Progress: 0/3 tasks completed

### [⏳] Phase 8: Frontend (SvelteKit) - `In Progress 8%`
    - [✅] 8.1 Project Setup - `Done 100%`
          → [View folder](./V1_MVP/08_Frontend/8.1_Project_Setup/)
          → Progress: 1/1 tasks completed (Updated: 2025-12-24)

    - [🔄] 8.2 Authentication UI (Email/Password + OAuth2) - `In Progress 25%`
          → [View folder](./V1_MVP/08_Frontend/8.2_Authentication_UI/)
          → Traditional email/password authentication foundation
          → OAuth2 integration with Kanidm identity provider
          → API infrastructure and client integration
          → Progress: 1/4 Done, 3 Todo (Updated: 2025-12-24)

    - [⏳] 8.3 Dashboard - `Todo`
          → [View folder](./V1_MVP/08_Frontend/8.3_Dashboard/)
          → Progress: 0/4 tasks completed

    - [⏳] 8.4 Product Management UI - `Todo`
          → [View folder](./V1_MVP/08_Frontend/8.4_Product_Management_UI/)
          → Progress: 0/4 tasks completed

    - [⏳] 8.5 Order Management UI - `Todo`
          → [View folder](./V1_MVP/08_Frontend/8.5_Order_Management_UI/)
          → Progress: 0/4 tasks completed

    - [⏳] 8.6 Integration UI - `Todo`
          → [View folder](./V1_MVP/08_Frontend/8.6_Integration_UI/)
          → Progress: 0/4 tasks completed

    - [⏳] 8.7 Settings - `Todo`
          → [View folder](./V1_MVP/08_Frontend/8.7_Settings/)
          → Progress: 0/4 tasks completed

### [⏳] Phase 9: Analytics (Cube) - `Todo 0%`
    - [⏳] 9.1 Cube Setup - `Todo`
          → [View folder](./V1_MVP/09_Analytics/9.1_Cube_Setup/)
          → Progress: 0/3 tasks completed

    - [⏳] 9.2 Frontend Integration - `Todo`
          → [View folder](./V1_MVP/09_Analytics/9.2_Frontend_Integration/)
          → Progress: 0/2 tasks completed

    - [⏳] 9.3 Pre-aggregations - `Todo`
          → [View folder](./V1_MVP/09_Analytics/9.3_Pre_aggregations/)
          → Progress: 0/2 tasks completed

### [⏳] Phase 10: Deployment (CapRover) - `Todo 0%`
    - [⏳] 10.1 CapRover Setup - `Todo`
          → [View folder](./V1_MVP/10_Deployment/10.1_CapRover_Setup/)
          → Progress: 0/2 tasks completed

    - [⏳] 10.2 Stateful Services Deployment - `Todo`
          → [View folder](./V1_MVP/10_Deployment/10.2_Stateful_Services/)
          → Progress: 0/4 tasks completed

    - [⏳] 10.3 Microservices Deployment - `Todo`
          → [View folder](./V1_MVP/10_Deployment/10.3_Microservices/)
          → Progress: 0/5 tasks completed

    - [⏳] 10.4 Frontend Deployment - `Todo`
          → [View folder](./V1_MVP/10_Deployment/10.4_Frontend/)
          → Progress: 0/2 tasks completed

    - [⏳] 10.5 CI/CD Pipeline - `Todo`
          → [View folder](./V1_MVP/10_Deployment/10.5_CI_CD/)
          → Progress: 0/2 tasks completed

### [⏳] Phase 11: Monitoring & Observability - `Todo 0%`
    - [⏳] 11.1 Logging Setup - `Todo`
          → [View folder](./V1_MVP/11_Monitoring/11.1_Logging_Setup/)
          → Progress: 0/3 tasks completed

    - [⏳] 11.2 Metrics & Monitoring - `Todo`
          → [View folder](./V1_MVP/11_Monitoring/11.2_Metrics_Monitoring/)
          → Progress: 0/3 tasks completed

    - [⏳] 11.3 Alerting System - `Todo`
          → [View folder](./V1_MVP/11_Monitoring/11.3_Alerting/)
          → Progress: 0/1 tasks completed

### [⏳] Phase 12: Testing & Quality Assurance - `In Progress 2%`
    - [⏳] 12.1 Unit Tests - `Todo`
          → [View folder](./V1_MVP/12_Testing/12.1_Unit_Tests/)
          → Progress: 0/1 tasks completed

    - [🔄] 12.2 Integration Tests - `In Progress`
          → [View folder](./V1_MVP/12_Testing/12.2_Integration_Tests/)
          → Progress: 0/1 Done, 1 InProgress (Updated: 2025-12-24)

    - [⏳] 12.3 E2E Tests - `Todo`
          → [View folder](./V1_MVP/12_Testing/12.3_E2E_Tests/)
          → Progress: 0/1 tasks completed

    - [⏳] 12.4 Load Testing - `Todo`
          → [View folder](./V1_MVP/12_Testing/12.4_Load_Testing/)
          → Progress: 0/30 tasks completed

    - [⏳] 12.5 Security Testing - `Todo`
          → [View folder](./V1_MVP/12_Testing/12.5_Security_Testing/)
          → Progress: 0/10 tasks completed

## 📊 Project Metrics

### Overall Progress: ~41% Complete (Updated: 2025-12-24)

| Phase | Total Tasks | Done | InProgress | NeedsReview | Todo | Completion |
|-------|-------------|------|------------|-------------|------|------------|
| Phase 1 (Infrastructure) | 12 | 5 | 1 | 1 | 5 | 42% |
| Phase 2 (Database) | 10 | 0 | 0 | 0 | 10 | 0%* |
| Phase 3 (User Service) | 28 | 14 | 2 | 5 | 7 | 68% |
| Phase 4 (Inventory) | 61 | 44 | 2 | 9 | 4 | 90% |
| Phase 5 (Order) | 1 | 0 | 0 | 0 | 1 | 0% |
| Phase 6 (Integration) | 3 | 0 | 0 | 0 | 3 | 0% |
| Phase 7 (Payment) | 2 | 0 | 0 | 0 | 2 | 0% |
| Phase 8 (Frontend) | 25 | 2 | 0 | 0 | 23 | 8% |
| Phase 9 (Analytics) | 4 | 0 | 0 | 0 | 4 | 0% |
| Phase 10 (Deployment) | 14 | 0 | 0 | 0 | 14 | 0% |
| Phase 11 (Monitoring) | 8 | 0 | 0 | 0 | 8 | 0% |
| Phase 12 (Testing) | 43 | 0 | 1 | 0 | 42 | 2% |
| **TOTAL** | **211** | **65** | **6** | **15** | **123** | **41%** |

*Note: Phase 2 shows 0% in tracking but core migrations are complete (tracked separately)

#### By Phase:
- **Phase 1** (Infrastructure): 🔄 42% - Auth library done, dev tools in progress
- **Phase 2** (Database): ✅ 100% - Foundation complete (core migrations done)
- **Phase 3** (User Service): 🔄 68% - Kanidm 67%, Casbin 92%, User Mgmt 60% (Updated: 2025-12-24)
- **Phase 4** (Inventory): 🔄 90% - Major progress! 44/61 Done, 9 NeedsReview (Updated: 2025-12-24)
- **Phase 5** (Order Service): ⏳ 0% - Not started
- **Phase 6** (Integration): ⏳ 0% - Not started
- **Phase 7** (Payment): ⏳ 0% - Not started
- **Phase 8** (Frontend): 🔄 8% - Project setup done, Auth UI 25% (Updated: 2025-12-24)
- **Phase 9** (Analytics): ⏳ 0% - Not started
- **Phase 10** (Deployment): ⏳ 0% - Not started
- **Phase 11** (Monitoring): ⏳ 0% - Not started
- **Phase 12** (Testing): 🔄 2% - Integration tests in progress

#### By Priority:
- **🔴 P0** (MVP Critical): 25+ tasks - 10% complete
- **🟡 P1** (Production Ready): 42+ tasks - 0% complete *(Phase 4 optimized)*
- **🔵 P2** (Enhancement): 35+ tasks - 0% complete

### 🎯 Critical Path for MVP

**Immediate Next Steps** (P0 tasks that block MVP):
1. **Kanidm Integration** (3.1.x) - OAuth2/OIDC authentication with Kanidm
2. **Casbin Authorization** (3.2.x) - Multi-tenant RBAC middleware (works with Kanidm JWT)
2. **User Management** (3.3.x) - Basic CRUD operations with tenant isolation
3. **Security Testing** (3.4.x) - Critical tenant isolation validation
4. **Core Inventory** (4.1-4.4) - Product catalog, warehouse, and stock operations *(optimized)*
5. **Quality Management** (4.8) - QC integration with inventory *(new)*
6. **Order Management** (5.1) - Basic order CRUD operations
7. **Marketplace Integration** (6.1-6.3) - Shopee/Lazada/Tiki adapters

**Estimated Effort**: 30 P0 tasks × 2-3 days each = **60-90 days** *(updated for optimized Phase 4)*

## 🚀 Quick Start for Contributors

### Development Environment
```bash
# 1. Setup environment variables
export DATABASE_URL="postgresql://user:pass@localhost/anthill"
export KANIDM_URL="https://idm.example.com"
export KANIDM_OAUTH2_CLIENT_ID="anthill"
export KANIDM_OAUTH2_CLIENT_SECRET="your-client-secret"
export REDIS_URL="redis://localhost:6379"

# 2. Run database migrations
./scripts/migrate.sh run

# 3. Start services
cargo run --bin user-service

# 4. Access API
curl http://localhost:8000/health
open http://localhost:8000/docs  # Swagger UI
```

### Task Management Workflow
1. **Browse tasks**: Check this TASKS_OVERVIEW.md for available work
2. **Claim task**: Update task status to `InProgress_By_[Your_Name]`
3. **Complete work**: Follow acceptance criteria and update sub-tasks
4. **Submit for review**: Change status to `NeedsReview`
5. **Get approval**: Project lead reviews and marks as `Completed`

### Current Hot Tasks 🔥

**🔴 P0 - Critical for MVP**:
- `V1_MVP/03_User_Service/3.2_Casbin_Authorization/3.2.1.1_add_dependencies.md`
- `V1_MVP/03_User_Service/3.2_Casbin_Authorization/3.2.2.1_create_model_file.md`
- `V1_MVP/03_User_Service/3.3_User_Management/3.3.1.1_tenant_isolation_test.md`
- `V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.04_api_infrastructure_core_setup.md`

**🟡 P1 - Important for Production**:
- `V1_MVP/03_User_Service/3.1_Authentication/3.1.4_tenant_resolution.md`
- `V1_MVP/03_User_Service/3.1_Authentication/3.1.5_password_migration.md`
- `V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.03_auth_api_client_integration.md`

## 📚 Key Documentation

- **ARCHITECTURE.md** - System design and deployment strategy
- **STRUCTURE.md** - Project structure and 3-crate pattern
- **WARP.md** - Development guide and best practices
- **TODO.md** - Comprehensive task breakdown (legacy)
- **AGENTS.md** - AI assistant guidance and rules

## 🔄 Current Sprint Focus

**Sprint Goal**: Complete Kanidm integration and Casbin authorization

**Note**: Authentication strategy changed to use Kanidm (Identity Provider) instead of custom JWT. This provides better security, OAuth2/OIDC compliance, and advanced features like Passkeys/WebAuthn.

**Sprint Tasks**:
1. Implement Casbin multi-tenant RBAC (12 tasks) - 🔄 8/12 Done, 3 NeedsReview
2. Create user management CRUD endpoints (5 tasks) - 🔄 2/5 Done, 1 NeedsReview
3. Comprehensive security testing (5 tasks) - 🔄 0/5 Done, 1 NeedsReview
4. Integration testing (3 tasks) - ⏳ 0/3 pending

**Success Criteria**:
**Achievements**:
- ✅ Kanidm server deployed and configured
- ✅ OAuth2/OIDC integration mostly complete (4/6 Done, 1 InProgress)
- ✅ Multi-tenant authorization (Casbin) 92% complete (8/12 Done, 3 NeedsReview)
- 🔄 User management partially implemented (2/5 Done, 1 NeedsReview)
- 🔄 Security testing started (1 NeedsReview)
- ⏳ Integration testing pending

## 📈 Progress Tracking

### Weekly Updates
- **Week 1**: Kanidm server setup and OAuth2 client configuration
- **Week 2**: Kanidm client library and OAuth2 endpoints
- **Week 3**: Group-tenant mapping and auth extractors update
- **Week 3**: User management endpoints
- **Week 4**: Security testing and validation

### Milestone Targets
- **Milestone 1** (Week 2): OAuth2 authentication flow working
- **Milestone 2** (Week 3): Casbin authorization with Kanidm JWT
- **Milestone 2** (Week 4): Complete user management system
- **Milestone 3** (Week 6): Security validation complete

---

**Last Updated**: 2025-12-24
**Project Status**: In Progress (Phase 3 User Service 68%, Phase 4 Inventory Service 90%)

**Recent Changes (2025-12-24)**:
- ✅ **Full Project Tracking Scan Completed**: Updated all phase progress based on actual task file statuses
      - Phase 3 (User Service): 68% complete (14 Done, 2 InProgress, 5 NeedsReview, 7 Todo)
      - Phase 4 (Inventory): 90% complete! (44 Done, 2 InProgress, 9 NeedsReview, 4 Todo)
      - Phase 8 (Frontend): 8% complete (2 Done, 23 Todo)
      - Phase 12 (Testing): 2% complete (1 InProgress, 42 Todo)
      - Overall: 41% complete (65 Done, 6 InProgress, 15 NeedsReview, 123 Todo)
- ✅ **Added Progress Table**: New metrics table with detailed breakdown by phase
- ✅ **Corrected Task Counts**: Updated actual task numbers in Phase 12 (Load Testing: 30, Security Testing: 10)

**Recent Changes (2025-11-28)**:
- ✅ **Phase 4 Inventory Service Progress Updated**: Corrected completion percentages based on actual task statuses
      - 4.1 Product Master: 5/6 completed (1 InProgress)
      - 4.2 Warehouse Management: 1/1 completed
      - 4.3 Stock Operations Core: 3/3 completed
      - 4.4 Stock Transactions: 11/19 completed (10 Done, 1 NeedsReview)
      - Overall Phase 4: 52% complete (27/52 tasks)
      - Updated overall project metrics accordingly

**Recent Changes (2025-11-12)**:
- ✅ **Authentication UI Issue Resolved**: task_08.02.01 session_expired error fixed
      - Root cause: API client sending Authorization headers to login endpoints + token storage inconsistency
      - Fixed API client to exclude auth headers for login/register endpoints
      - Synchronized AuthSession and tokenManager storage systems
      - Task status changed to Done
- ✅ **Authentication UI Tasks Re-authored**: Recreated all 4 tasks with clean folder-tasks format
      - task_08.02.01_create_login_registration_pages.md: Focus on SvelteKit 5 runes + shadcn-svelte UI + Valibot validation
      - task_08.02.02_integrate_oauth2_kanidm.md: Repurposed for email/password form actions, session cookies, logout flow
      - task_08.02.03_auth_api_client_integration.md: Centralised email/password auth client with typed DTOs and retries
      - task_08.02.04_api_infrastructure_core_setup.md: Shared fetch layer with retries, tenant headers, AppError mapping
- ✅ **Task Structure Cleaned**: Removed all corrupted content and recreated clean task files
- ✅ **Status Reset**: All tasks now ready for fresh implementation from scratch
- ✅ **OpenAPI Compliance**: Tasks prepared to follow user-service OpenAPI specification
- ✅ **Context7 Integration**: Tasks ready to work with latest Context7 documentation
