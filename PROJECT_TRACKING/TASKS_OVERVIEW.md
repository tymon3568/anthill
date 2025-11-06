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

### [🔄] Phase 3: User Service (Kanidm Integration & Tenancy) - `In Progress 52%`

> **Timeline**: Weeks 3-5 (21 days)  
> **Dependencies**: Phase 1, 2 completed, Kanidm server
> **Progress**: 16/25 tasks completed (Updated: 2025-11-05)

    - [✅] 3.0 Architecture Implementation - `Completed`
          → [View folder](./V1_MVP/03_User_Service/3.0_Architecture/)
          → Completed: 2025-01-09

    - [✅] 3.1 Kanidm Integration - `Completed 83%`
          → [View folder](./V1_MVP/03_User_Service/3.1_Kanidm_Integration/)
          → Replaces custom JWT auth with Kanidm OAuth2/OIDC
          → Started: 2025-01-10
          → Progress: 5/6 tasks completed (Updated: 2025-11-05)

    - [✅] 3.2 Authorization with Casbin - `Completed 90%`
          → [View folder](./V1_MVP/03_User_Service/3.2_Casbin_Authorization/)
          → Started: 2025-01-10
          → Progress: 9/10 tasks completed (Updated: 2025-11-05)

    - [🔄] 3.3 User Management - `In Progress 40%`
          → [View folder](./V1_MVP/03_User_Service/3.3_User_Management/)
          → Progress: 2/5 tasks completed (Updated: 2025-11-05)

    - [⏳] 3.4 Testing - `Todo`
          → [View folder](./V1_MVP/03_User_Service/3.4_Testing/)
          → Progress: 0/4 tasks completed

### [⏳] Phase 4: Inventory Service - `Todo 0%`
    - [⏳] 4.1 Product Master Data - `Todo`
          → [View folder](./V1_MVP/04_Inventory_Service/4.1_Product_Master/)
          → Progress: 0/8 tasks completed

    - [⏳] 4.2 Warehouse Management - `Todo`
          → [View folder](./V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/)
          → Progress: 0/5 tasks completed

    - [⏳] 4.3 Stock Operations - `Todo`
          → [View folder](./V1_MVP/04_Inventory_Service/4.3_Stock_Operations/)
          → Progress: 0/15 tasks completed

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

### [⏳] Phase 8: Frontend (SvelteKit) - `Todo 0%`
    - [✅] 8.1 Project Setup - `Done 100%`
          → [View folder](./V1_MVP/08_Frontend/8.1_Project_Setup/)
          → Progress: 1/1 tasks completed (Updated: 2025-11-05)

    - [🔄] 8.2 Authentication UI (OAuth2) - `In Progress 50%`
          → [View folder](./V1_MVP/08_Frontend/8.2_Authentication_UI/)
          → Backend auth integration completed, needs testing and docs
          → Progress: 1/2 tasks completed (Updated: 2025-11-06)

    - [⏳] 8.3 Dashboard - `Todo`
          → [View folder](./V1_MVP/08_Frontend/8.3_Dashboard/)
          → Progress: 0/3 tasks completed

    - [⏳] 8.4 Product Management UI - `Todo`
          → [View folder](./V1_MVP/08_Frontend/8.4_Product_Management_UI/)
          → Progress: 0/3 tasks completed

    - [⏳] 8.5 Order Management UI - `Todo`
          → [View folder](./V1_MVP/08_Frontend/8.5_Order_Management_UI/)
          → Progress: 0/3 tasks completed

    - [⏳] 8.6 Integration UI - `Todo`
          → [View folder](./V1_MVP/08_Frontend/8.6_Integration_UI/)
          → Progress: 0/3 tasks completed

    - [⏳] 8.7 Settings - `Todo`
          → [View folder](./V1_MVP/08_Frontend/8.7_Settings/)
          → Progress: 0/3 tasks completed

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

### [⏳] Phase 12: Testing & Quality Assurance - `Todo 0%`
    - [⏳] 12.1 Unit Tests - `Todo`
          → [View folder](./V1_MVP/12_Testing/12.1_Unit_Tests/)
          → Progress: 0/1 tasks completed

    - [⏳] 12.2 Integration Tests - `Todo`
          → [View folder](./V1_MVP/12_Testing/12.2_Integration_Tests/)
          → Progress: 0/1 tasks completed

    - [⏳] 12.3 E2E Tests - `Todo`
          → [View folder](./V1_MVP/12_Testing/12.3_E2E_Tests/)
          → Progress: 0/1 tasks completed

    - [⏳] 12.4 Load Testing - `Todo`
          → [View folder](./V1_MVP/12_Testing/12.4_Load_Testing/)
          → Progress: 0/2 tasks completed

    - [⏳] 12.5 Security Testing - `Todo`
          → [View folder](./V1_MVP/12_Testing/12.5_Security_Testing/)
          → Progress: 0/4 tasks completed

## 📊 Project Metrics

### Overall Progress: ~36% Complete (Updated: 2025-11-06)

#### By Phase:
- **Phase 1** (Infrastructure): ✅ 95% - Production ready
- **Phase 2** (Database): ✅ 100% - Foundation complete
- **Phase 3** (User Service): 🔄 52% - Kanidm integration mostly complete, Casbin authorization nearly done (Updated: 2025-11-05)
- **Phase 4** (Inventory): ⏳ 0% - Not started (core business logic)
- **Phase 5** (Order Service): ⏳ 0% - Not started
- **Phase 6** (Integration): ⏳ 0% - Not started
- **Phase 7** (Payment): ⏳ 0% - Not started
- **Phase 8** (Frontend): 🔄 ~15% - Project setup complete, auth integration in progress
- **Phase 9** (Analytics): ⏳ 0% - Not started
- **Phase 10** (Deployment): ⏳ 0% - Not started
- **Phase 11** (Monitoring): ⏳ 0% - Not started
- **Phase 12** (Testing): ⏳ 0% - Not started

#### By Priority:
- **🔴 P0** (MVP Critical): 25+ tasks - 10% complete
- **🟡 P1** (Production Ready): 20+ tasks - 0% complete
- **🔵 P2** (Enhancement): 35+ tasks - 0% complete

### 🎯 Critical Path for MVP

**Immediate Next Steps** (P0 tasks that block MVP):
1. **Kanidm Integration** (3.1.x) - OAuth2/OIDC authentication with Kanidm
2. **Casbin Authorization** (3.2.x) - Multi-tenant RBAC middleware (works with Kanidm JWT)
2. **User Management** (3.3.x) - Basic CRUD operations with tenant isolation
3. **Security Testing** (3.4.x) - Critical tenant isolation validation
4. **Core Inventory** (4.1-4.3) - Product catalog and stock tracking
5. **Order Management** (5.1) - Basic order CRUD operations
6. **Marketplace Integration** (6.1-6.3) - Shopee/Lazada/Tiki adapters

**Estimated Effort**: 25 P0 tasks × 2-3 days each = **50-75 days**

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
curl http://localhost:3000/health
open http://localhost:3000/docs  # Swagger UI
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

**🟡 P1 - Important for Production**:
- `V1_MVP/03_User_Service/3.1_Authentication/3.1.4_tenant_resolution.md`
- `V1_MVP/03_User_Service/3.1_Authentication/3.1.5_password_migration.md`

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
1. Implement Casbin multi-tenant RBAC (10 tasks) - ✅ 9/10 completed
2. Create user management CRUD endpoints (5 tasks) - 🔄 2/5 completed
3. Comprehensive security testing (4 tasks) - ⏳ 0/4 pending
4. Integration testing (3 tasks) - ⏳ 0/3 pending

**Success Criteria**:
**Achievements**:
- ✅ Kanidm server deployed and configured
- ✅ OAuth2/OIDC integration completed (5/6 tasks)
- ✅ Multi-tenant authorization (Casbin) nearly complete (9/10 tasks)
- 🔄 User management partially implemented (2/5 tasks)
- ⏳ Security testing pending
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

**Last Updated**: 2025-11-06
**Project Status**: In Progress (Phase 3 Kanidm Integration)

**Recent Changes (2025-11-06)**:
- ✅ **Backend Auth Integration**: Completed task_08.02.02_integrate_backend_auth.md (12/16 sub-tasks done)
- ✅ **Task File Updates**: Synchronized checkboxes and progress tracking per folder-tasks workflow
- ✅ **Progress Update**: Updated Phase 8 progress from 0% to ~15% (1/2 auth tasks completed)
- ✅ **Frontend Setup**: Completed SvelteKit project foundation (task_08.01.01_setup_sveltekit_project.md)
- ✅ **Progress Update**: Updated Phase 3 progress from 60% to 64% (16/25 tasks completed)
- ✅ **Kanidm Integration**: 5/6 tasks completed (83% complete)
- ✅ **Casbin Authorization**: 9/10 tasks completed (90% complete) 
- ✅ **User Management**: 2/5 tasks completed (40% complete)
- ✅ **Task Tracking**: Fixed missing Assignee fields in completed tasks
- ✅ **Documentation**: Updated folder-tasks workflow from Context7
- 🔄 Migrating from custom JWT authentication to Kanidm (OAuth2/OIDC)
- 📚 Updated all documentation (ARCHITECTURE.md, README.md, etc.)
- 📋 See `docs/KANIDM_MIGRATION_PLAN.md` for detailed migration plan
**Next Milestone**: Complete remaining Phase 3 tasks (Week 2)
