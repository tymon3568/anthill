# UI Architecture Proposal for Anthill Platform

> **Document Version:** 1.0  
> **Created:** 2026-01-26  
> **Status:** Proposal  
> **References:** Odoo 17/18, SAP Fiori, ERPNext, NetSuite, Zoho Inventory

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [ERP UI Research](#2-erp-ui-research)
3. [Proposed Module Structure](#3-proposed-module-structure)
4. [Navigation Design](#4-navigation-design)
5. [Route Architecture](#5-route-architecture)
6. [Page Templates](#6-page-templates)
7. [Implementation Roadmap](#7-implementation-roadmap)

---

## 1. Executive Summary

### Current State

Anthill hiện có cấu trúc frontend cơ bản với:
- ✅ Authentication & Authorization
- ✅ Product Management (CRUD đầy đủ)
- ✅ Admin Console (Users, Roles, Invitations)
- ✅ Settings Module
- ⚠️ Orders Module (placeholder)
- ⚠️ Integrations Module (placeholder)
- 🔄 Inventory Operations (đang phát triển)
- 🔄 Pricing System (đang phát triển)

### Proposed Changes

Đề xuất tái cấu trúc UI theo mô hình **Domain-Driven Design** tham khảo từ Odoo, SAP Fiori và ERPNext:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           ANTHILL UI ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │  DASHBOARD   │  │   SALES      │  │   PURCHASE   │  │  INVENTORY   │    │
│  │              │  │              │  │              │  │              │    │
│  │  • KPIs      │  │  • Customers │  │  • Suppliers │  │  • Products  │    │
│  │  • Charts    │  │  • Quotes    │  │  • PO        │  │  • Stock     │    │
│  │  • Alerts    │  │  • SO        │  │  • Receiving │  │  • Warehouse │    │
│  │  • Tasks     │  │  • Invoices  │  │  • Bills     │  │  • Transfers │    │
│  │              │  │  • Pricing   │  │              │  │  • Valuation │    │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   FINANCE    │  │ INTEGRATIONS │  │   REPORTS    │  │   SETTINGS   │    │
│  │              │  │              │  │              │  │              │    │
│  │  • Payments  │  │  • Channels  │  │  • Sales     │  │  • Profile   │    │
│  │  • Expenses  │  │  • Sync      │  │  • Inventory │  │  • Tenant    │    │
│  │  • Transfers │  │  • Webhooks  │  │  • Financial │  │  • Billing   │    │
│  │  • Cash Flow │  │  • Mappings  │  │  • Custom    │  │  • API Keys  │    │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                                              │
│  ┌──────────────┐                                                            │
│  │    ADMIN     │                                                            │
│  │              │                                                            │
│  │  • Users     │                                                            │
│  │  • Roles     │                                                            │
│  │  • Audit     │                                                            │
│  │  • Invites   │                                                            │
│  └──────────────┘                                                            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. ERP UI Research

### 2.1 Odoo 17/18 Navigation Model

| Component | Design Pattern |
|-----------|---------------|
| **App Launcher** | Grid-based app selector (top-left) |
| **Top Bar** | App switcher, global search, activities, messaging |
| **App Menu** | Horizontal tabs for main sections |
| **Sidebar** | Filters, favorites, groupings (contextual) |
| **View Modes** | List, Kanban, Calendar, Pivot, Graph |

**Odoo Module Organization:**
```
Sales
├── Orders
│   ├── Quotations
│   ├── Sales Orders
│   └── Invoices
├── Customers
├── Products
└── Configuration
    ├── Pricelists
    ├── Payment Terms
    └── Settings
```

### 2.2 SAP Fiori Navigation

| Component | Design Pattern |
|-----------|---------------|
| **Launchpad** | Role-based tile dashboard |
| **Shell Bar** | Global search, notifications, user menu |
| **Spaces & Pages** | Flexible grouping of apps |
| **In-App Nav** | Master-detail, full-screen, flexible column |

**Key Principles:**
- Intent-based navigation (semantic objects + actions)
- Deep linking support
- Responsive: Cozy / Compact / Condensed modes

### 2.3 ERPNext Workspace Model

| Component | Design Pattern |
|-----------|---------------|
| **Sidebar** | Module workspaces as collapsible sections |
| **Shortcuts** | Frequently used doctypes |
| **Links** | Grouped by: Masters, Transactions, Reports, Settings |
| **Awesome Bar** | Command palette for quick navigation |

**ERPNext Module Organization:**
```
Stock (Inventory)
├── Shortcuts: Item, Stock Entry, Stock Reconciliation
├── Masters
│   ├── Item
│   ├── Item Group
│   ├── Warehouse
│   └── UOM
├── Transactions
│   ├── Stock Entry
│   ├── Material Request
│   └── Stock Reconciliation
├── Reports
│   ├── Stock Balance
│   ├── Stock Ledger
│   └── Stock Ageing
└── Settings
    └── Stock Settings
```

### 2.4 Key Takeaways for Anthill

| Best Practice | Source | Implementation |
|---------------|--------|----------------|
| **Role-based navigation** | SAP Fiori | Show/hide modules based on user role |
| **Command palette (Ctrl+K)** | ERPNext | Already implemented ✅ |
| **Workspace shortcuts** | ERPNext | Pin frequently used pages |
| **Horizontal app tabs** | Odoo | Secondary nav within modules |
| **Collapsible sidebar** | All | Responsive navigation |
| **Search-first approach** | Modern SaaS | Global search as primary nav |
| **Consistent shell bar** | SAP Fiori | Top bar across all pages |

---

## 3. Proposed Module Structure

### 3.1 High-Level Modules

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           MODULE HIERARCHY                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  📊 DASHBOARD                    Core entry point                            │
│  │                                                                           │
│  ├── 💰 SALES                    Revenue & Customer Management               │
│  │   ├── Customers                                                           │
│  │   ├── Quotations                                                          │
│  │   ├── Sales Orders                                                        │
│  │   ├── Invoices                                                            │
│  │   └── Pricing                 ◄── Price Lists, Rules, Discounts          │
│  │                                                                           │
│  ├── 🛒 PURCHASE                 Procurement & Supplier Management           │
│  │   ├── Suppliers                                                           │
│  │   ├── Purchase Orders                                                     │
│  │   ├── Goods Receipt (GRN)                                                 │
│  │   └── Bills                                                               │
│  │                                                                           │
│  ├── 📦 INVENTORY                Stock & Warehouse Management                │
│  │   ├── Products                                                            │
│  │   ├── Categories                                                          │
│  │   ├── Variants                                                            │
│  │   ├── Stock Levels                                                        │
│  │   ├── Warehouses & Locations                                              │
│  │   ├── Stock Movements                                                     │
│  │   │   ├── Transfers                                                       │
│  │   │   ├── Adjustments                                                     │
│  │   │   └── Stock Take                                                      │
│  │   └── Valuation              ◄── FIFO, AVCO, Standard Costing            │
│  │                                                                           │
│  ├── 💵 FINANCE                  Cash & Financial Management (NEW)           │
│  │   ├── Payments                                                            │
│  │   │   ├── Customer Payments (Thu tiền khách hàng)                        │
│  │   │   └── Supplier Payments (Trả tiền nhà cung cấp)                      │
│  │   ├── Other Income (Thu khác)                                             │
│  │   ├── Expenses (Chi phí)                                                  │
│  │   ├── Internal Transfers (Lưu chuyển nội bộ)                              │
│  │   ├── Currency Exchange (Quy đổi tiền tệ)                                 │
│  │   ├── Cash Flow                                                           │
│  │   │   ├── Cash Book                                                       │
│  │   │   └── Bank Reconciliation                                             │
│  │   └── Financial Summary                                                   │
│  │                                                                           │
│  ├── 🔗 INTEGRATIONS             Marketplace & Channel Management            │
│  │   ├── Channels (Shopee, Lazada, TikTok, etc.)                            │
│  │   ├── Sync Status                                                         │
│  │   ├── Product Mappings                                                    │
│  │   └── Webhooks                                                            │
│  │                                                                           │
│  ├── 📈 REPORTS                  Analytics & Reporting                       │
│  │   ├── Sales Reports                                                       │
│  │   ├── Inventory Reports                                                   │
│  │   ├── Financial Summary                                                   │
│  │   └── Custom Reports                                                      │
│  │                                                                           │
│  ├── ⚙️ SETTINGS                 User & Organization Settings                │
│  │   ├── Profile                                                             │
│  │   ├── Organization                                                        │
│  │   ├── Billing & Subscription                                              │
│  │   ├── API Keys                                                            │
│  │   └── Notifications                                                       │
│  │                                                                           │
│  └── 🛡️ ADMIN                    System Administration (Admin Only)          │
│      ├── Users                                                               │
│      ├── Roles & Permissions                                                 │
│      ├── Invitations                                                         │
│      └── Audit Log                                                           │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Module Responsibility Matrix

| Module | Domain | Primary Users | Key Entities |
|--------|--------|---------------|--------------|
| **Dashboard** | Overview | All users | KPIs, Alerts, Tasks |
| **Sales** | Revenue | Sales team, Managers | Customers, Orders, Invoices, Pricing |
| **Purchase** | Procurement | Purchasing team | Suppliers, POs, GRN, Bills |
| **Inventory** | Stock | Warehouse staff, Inventory managers | Products, Stock, Warehouses, Movements |
| **Finance** | Cash Management | Finance team, Accountants | Payments, Expenses, Transfers, Cash Flow |
| **Integrations** | Channels | eCommerce managers | Channels, Sync, Mappings |
| **Reports** | Analytics | Managers, Analysts | Reports, Dashboards |
| **Settings** | Configuration | All users | Profile, Tenant, API |
| **Admin** | Administration | Admins only | Users, Roles, Audit |

### 3.3 Feature-to-Module Mapping

| Feature | Current Location | Proposed Location | Rationale |
|---------|-----------------|-------------------|-----------|
| Products | `/inventory/products` | `/inventory/products` | ✅ Correct |
| Categories | `/inventory/categories` | `/inventory/categories` | ✅ Correct |
| Warehouses | `/inventory/warehouses` | `/inventory/warehouses` | ✅ Correct |
| Price Lists | `/inventory/pricing` | `/sales/pricing` | Pricing is sales-driven |
| Pricing Rules | `/inventory/pricing/rules` | `/sales/pricing/rules` | Pricing is sales-driven |
| Sales Orders | `/orders/sales` | `/sales/orders` | Clearer module separation |
| Purchase Orders | `/orders/purchase` | `/purchase/orders` | Clearer module separation |
| Returns | `/orders/returns` | `/sales/returns` or `/purchase/returns` | Split by type |
| GRN | - | `/purchase/grn` | Receiving goods |
| Stock Transfers | `/inventory/transfers` | `/inventory/movements/transfers` | Group movements |
| Stock Take | `/inventory/stock-take` | `/inventory/movements/stock-take` | Group movements |

---

## 4. Navigation Design

### 4.1 Navigation Configuration

```typescript
// frontend/src/lib/config/navigation.ts

import {
  LayoutDashboardIcon,
  ShoppingCartIcon,
  PackageIcon,
  TruckIcon,
  PlugIcon,
  BarChartIcon,
  SettingsIcon,
  ShieldIcon
} from '@lucide/svelte/icons';

export const mainNavigation: NavItem[] = [
  {
    title: 'Dashboard',
    url: '/dashboard',
    icon: LayoutDashboardIcon
  },
  {
    title: 'Sales',
    url: '/sales',
    icon: ShoppingCartIcon,
    items: [
      { title: 'Customers', url: '/sales/customers' },
      { title: 'Quotations', url: '/sales/quotations' },
      { title: 'Sales Orders', url: '/sales/orders' },
      { title: 'Invoices', url: '/sales/invoices' },
      // Separator
      { title: 'Price Lists', url: '/sales/pricing' },
      { title: 'Pricing Rules', url: '/sales/pricing/rules' }
    ]
  },
  {
    title: 'Purchase',
    url: '/purchase',
    icon: TruckIcon,
    items: [
      { title: 'Suppliers', url: '/purchase/suppliers' },
      { title: 'Purchase Orders', url: '/purchase/orders' },
      { title: 'Goods Receipt', url: '/purchase/grn' },
      { title: 'Bills', url: '/purchase/bills' }
    ]
  },
  {
    title: 'Inventory',
    url: '/inventory',
    icon: PackageIcon,
    items: [
      { title: 'Products', url: '/inventory/products' },
      { title: 'Categories', url: '/inventory/categories' },
      { title: 'Stock Levels', url: '/inventory/stock' },
      { title: 'Warehouses', url: '/inventory/warehouses' },
      // Separator
      { title: 'Transfers', url: '/inventory/transfers' },
      { title: 'Adjustments', url: '/inventory/adjustments' },
      { title: 'Stock Take', url: '/inventory/stock-take' },
      // Separator
      { title: 'Valuation', url: '/inventory/valuation' }
    ]
  },
  {
    title: 'Integrations',
    url: '/integrations',
    icon: PlugIcon,
    items: [
      { title: 'Channels', url: '/integrations/channels' },
      { title: 'Sync Status', url: '/integrations/sync' },
      { title: 'Product Mappings', url: '/integrations/mappings' },
      { title: 'Webhooks', url: '/integrations/webhooks' }
    ]
  },
  {
    title: 'Finance',
    url: '/finance',
    icon: WalletIcon,
    items: [
      { title: 'Customer Payments', url: '/finance/payments/customers' },
      { title: 'Supplier Payments', url: '/finance/payments/suppliers' },
      // Separator
      { title: 'Other Income', url: '/finance/income' },
      { title: 'Expenses', url: '/finance/expenses' },
      // Separator
      { title: 'Internal Transfers', url: '/finance/transfers' },
      { title: 'Currency Exchange', url: '/finance/exchange' },
      // Separator
      { title: 'Cash Flow', url: '/finance/cash-flow' }
    ]
  },
  {
    title: 'Reports',
    url: '/reports',
    icon: BarChartIcon,
    items: [
      { title: 'Sales Reports', url: '/reports/sales' },
      { title: 'Inventory Reports', url: '/reports/inventory' },
      { title: 'Financial Summary', url: '/reports/financial' }
    ]
  }
];

export const settingsNavigation: NavItem[] = [
  {
    title: 'Admin',
    url: '/admin',
    icon: ShieldIcon,
    adminOnly: true,
    items: [
      { title: 'Users', url: '/admin/users' },
      { title: 'Roles', url: '/admin/roles' },
      { title: 'Invitations', url: '/admin/invitations' },
      { title: 'Audit Log', url: '/admin/audit' }
    ]
  },
  {
    title: 'Settings',
    url: '/settings',
    icon: SettingsIcon,
    items: [
      { title: 'Profile', url: '/settings/profile' },
      { title: 'Organization', url: '/settings/organization' },
      { title: 'Billing', url: '/settings/billing' },
      { title: 'API Keys', url: '/settings/api-keys' },
      { title: 'Notifications', url: '/settings/notifications' }
    ]
  }
];
```

### 4.2 Navigation Comparison

| Aspect | Current | Proposed | Benefit |
|--------|---------|----------|---------|
| **Top-level modules** | 4 (Dashboard, Inventory, Orders, Integrations) | 6 (+ Sales, Purchase, Reports) | Clearer domain separation |
| **Orders module** | Combined sales/purchase | Split into Sales & Purchase | Matches ERP best practices |
| **Pricing location** | Under Inventory | Under Sales | Revenue-focused, like Odoo/SAP |
| **Reports** | None | Dedicated module | Centralized analytics |
| **Settings depth** | 3 items | 5 items | More configuration options |

### 4.3 Visual Navigation Layout

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ [🐜 Anthill]  [Global Search Ctrl+K]              [🔔] [❓] [👤 User Menu] │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐  ┌─────────────────────────────────────────────────────┐  │
│  │              │  │                                                     │  │
│  │  📊 Dashboard│  │                    CONTENT AREA                     │  │
│  │              │  │                                                     │  │
│  │  💰 Sales    │  │  ┌─────────────────────────────────────────────┐   │  │
│  │    ├─ Custom │  │  │  Breadcrumb: Dashboard > Sales > Orders    │   │  │
│  │    ├─ Quotes │  │  └─────────────────────────────────────────────┘   │  │
│  │    ├─ Orders │  │                                                     │  │
│  │    ├─ Invoice│  │  ┌─────────────────────────────────────────────┐   │  │
│  │    └─ Pricing│  │  │  Page Header: Sales Orders                  │   │  │
│  │              │  │  │  [+ New Order]  [Export]  [Filter]          │   │  │
│  │  🛒 Purchase │  │  └─────────────────────────────────────────────┘   │  │
│  │    ├─ Supplie│  │                                                     │  │
│  │    ├─ Orders │  │  ┌─────────────────────────────────────────────┐   │  │
│  │    ├─ GRN    │  │  │                                             │   │  │
│  │    └─ Bills  │  │  │              DATA TABLE / CONTENT           │   │  │
│  │              │  │  │                                             │   │  │
│  │  📦 Inventory│  │  │  • List View (default)                      │   │  │
│  │    ├─ Product│  │  │  • Kanban View (optional)                   │   │  │
│  │    ├─ Categor│  │  │  • Calendar View (for dates)                │   │  │
│  │    ├─ Stock  │  │  │                                             │   │  │
│  │    ├─ Warehou│  │  └─────────────────────────────────────────────┘   │  │
│  │    └─ Transfe│  │                                                     │  │
│  │              │  │  ┌─────────────────────────────────────────────┐   │  │
│  │  🔗 Integrat │  │  │  Pagination: < 1 2 3 ... 10 >  |  50/page   │   │  │
│  │              │  │  └─────────────────────────────────────────────┘   │  │
│  │  📈 Reports  │  │                                                     │  │
│  │              │  └─────────────────────────────────────────────────────┘  │
│  │──────────────│                                                            │
│  │  🛡️ Admin    │                                                            │
│  │  ⚙️ Settings │                                                            │
│  │              │                                                            │
│  │  [Collapse ◀]│                                                            │
│  └──────────────┘                                                            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 5. Route Architecture

### 5.1 Proposed Route Structure

```
frontend/src/routes/
├── (auth)/                          # Unauthenticated routes
│   ├── login/
│   ├── register/
│   ├── forgot-password/
│   ├── reset-password/
│   └── verify-email/
│
├── (protected)/                     # Authenticated routes
│   ├── +layout.svelte               # Main app shell
│   ├── +layout.server.ts            # Auth guard
│   │
│   ├── dashboard/                   # 📊 Dashboard Module
│   │   └── +page.svelte
│   │
│   ├── sales/                       # 💰 Sales Module (NEW)
│   │   ├── +page.svelte             # Sales overview/dashboard
│   │   ├── customers/
│   │   │   ├── +page.svelte         # Customer list
│   │   │   ├── new/+page.svelte
│   │   │   └── [id]/
│   │   │       ├── +page.svelte     # Customer detail
│   │   │       └── edit/+page.svelte
│   │   ├── quotations/
│   │   │   ├── +page.svelte
│   │   │   ├── new/+page.svelte
│   │   │   └── [id]/+page.svelte
│   │   ├── orders/
│   │   │   ├── +page.svelte         # Sales order list
│   │   │   ├── new/+page.svelte
│   │   │   └── [id]/+page.svelte
│   │   ├── invoices/
│   │   │   ├── +page.svelte
│   │   │   └── [id]/+page.svelte
│   │   ├── returns/
│   │   │   ├── +page.svelte
│   │   │   └── [id]/+page.svelte
│   │   └── pricing/                 # 🏷️ Pricing (moved from inventory)
│   │       ├── +page.svelte         # Price lists
│   │       ├── new/+page.svelte
│   │       ├── [id]/+page.svelte
│   │       └── rules/
│   │           ├── +page.svelte
│   │           ├── new/+page.svelte
│   │           └── [id]/+page.svelte
│   │
│   ├── purchase/                    # 🛒 Purchase Module (NEW)
│   │   ├── +page.svelte             # Purchase overview
│   │   ├── suppliers/
│   │   │   ├── +page.svelte
│   │   │   ├── new/+page.svelte
│   │   │   └── [id]/+page.svelte
│   │   ├── orders/
│   │   │   ├── +page.svelte         # Purchase order list
│   │   │   ├── new/+page.svelte
│   │   │   └── [id]/+page.svelte
│   │   ├── grn/                     # Goods Receipt Notes
│   │   │   ├── +page.svelte
│   │   │   ├── new/+page.svelte
│   │   │   └── [id]/+page.svelte
│   │   └── bills/
│   │       ├── +page.svelte
│   │       └── [id]/+page.svelte
│   │
│   ├── inventory/                   # 📦 Inventory Module (Restructured)
│   │   ├── +page.svelte             # Inventory dashboard
│   │   ├── products/
│   │   │   ├── +page.svelte         # ✅ Already implemented
│   │   │   ├── new/+page.svelte
│   │   │   └── [id]/
│   │   │       ├── +page.svelte
│   │   │       └── edit/+page.svelte
│   │   ├── categories/
│   │   │   └── +page.svelte         # ✅ Already implemented
│   │   ├── stock/
│   │   │   └── +page.svelte         # Stock levels overview
│   │   ├── warehouses/
│   │   │   ├── +page.svelte         # ✅ Already implemented
│   │   │   └── [id]/+page.svelte
│   │   ├── transfers/
│   │   │   ├── +page.svelte
│   │   │   ├── new/+page.svelte
│   │   │   └── [id]/+page.svelte
│   │   ├── adjustments/
│   │   │   ├── +page.svelte
│   │   │   ├── new/+page.svelte
│   │   │   └── [id]/+page.svelte
│   │   ├── stock-take/
│   │   │   ├── +page.svelte
│   │   │   ├── new/+page.svelte
│   │   │   └── [id]/+page.svelte
│   │   └── valuation/
│   │       └── +page.svelte         # FIFO/AVCO reports
│   │
│   ├── integrations/                # 🔗 Integrations Module
│   │   ├── +page.svelte
│   │   ├── channels/
│   │   │   ├── +page.svelte
│   │   │   └── [id]/+page.svelte
│   │   ├── sync/
│   │   │   └── +page.svelte
│   │   ├── mappings/
│   │   │   └── +page.svelte
│   │   └── webhooks/
│   │       ├── +page.svelte
│   │       └── [id]/+page.svelte
│   │
│   ├── reports/                     # 📈 Reports Module (NEW)
│   │   ├── +page.svelte             # Report dashboard
│   │   ├── sales/
│   │   │   └── +page.svelte
│   │   ├── inventory/
│   │   │   └── +page.svelte
│   │   └── financial/
│   │       └── +page.svelte
│   │
│   ├── admin/                       # 🛡️ Admin Module
│   │   ├── +layout.svelte           # ✅ Already implemented
│   │   ├── users/+page.svelte
│   │   ├── roles/+page.svelte
│   │   ├── invitations/+page.svelte
│   │   └── audit/+page.svelte       # NEW: Audit log
│   │
│   ├── settings/                    # ⚙️ Settings Module
│   │   ├── +page.svelte             # Redirect to profile
│   │   ├── profile/+page.svelte
│   │   ├── organization/+page.svelte
│   │   ├── billing/+page.svelte
│   │   ├── api-keys/+page.svelte    # NEW
│   │   └── notifications/+page.svelte # NEW
│   │
│   └── profile/                     # User profile
│       └── +page.svelte
│
└── api/v1/                          # API proxy routes
    ├── auth/
    ├── users/
    ├── sales/                       # NEW
    ├── purchase/                    # NEW
    ├── inventory/
    ├── pricing/
    └── integrations/
```

### 5.2 Route Count Summary

| Module | Routes | Pages | Status |
|--------|--------|-------|--------|
| **Dashboard** | 1 | 1 | ✅ Implemented |
| **Sales** | 15 | 15 | 🆕 New module |
| **Purchase** | 12 | 12 | 🆕 New module |
| **Inventory** | 18 | 18 | 🔄 Partial |
| **Integrations** | 8 | 8 | ⚠️ Placeholder |
| **Reports** | 4 | 4 | 🆕 New module |
| **Admin** | 4 | 4 | ✅ Implemented |
| **Settings** | 5 | 5 | 🔄 Partial |
| **Total** | **67** | **67** | |

---

## 6. Page Templates

### 6.1 Standard Page Types

| Type | Usage | Components |
|------|-------|------------|
| **List Page** | View collection of items | Search, Filters, Table, Pagination, Bulk Actions |
| **Detail Page** | View single item | Header, Tabs, Info Cards, Related Items, Actions |
| **Form Page** | Create/Edit item | Form Sections, Validation, Save/Cancel |
| **Dashboard Page** | Overview with KPIs | Stats Cards, Charts, Recent Items, Alerts |
| **Settings Page** | Configuration | Form Groups, Toggles, Save Button |

### 6.2 List Page Template

```svelte
<!-- Standard List Page Structure -->
<script lang="ts">
  // State, filters, pagination logic
</script>

<div class="flex flex-col gap-6">
  <!-- Page Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold">Sales Orders</h1>
      <p class="text-muted-foreground">Manage your sales orders</p>
    </div>
    <div class="flex gap-2">
      <Button variant="outline">Export</Button>
      <Button href="/sales/orders/new">
        <Plus class="mr-2 h-4 w-4" />
        New Order
      </Button>
    </div>
  </div>

  <!-- Filters & Search -->
  <Card>
    <CardContent class="flex gap-4 pt-6">
      <Input placeholder="Search orders..." class="max-w-sm" />
      <Select placeholder="Status" />
      <Select placeholder="Customer" />
      <DateRangePicker />
      <Button variant="ghost">Clear</Button>
    </CardContent>
  </Card>

  <!-- Data Table -->
  <Card>
    <Table>
      <TableHeader>...</TableHeader>
      <TableBody>...</TableBody>
    </Table>
    
    <!-- Pagination -->
    <CardFooter class="flex justify-between">
      <span class="text-sm text-muted-foreground">
        Showing 1-10 of 156 orders
      </span>
      <Pagination />
    </CardFooter>
  </Card>
</div>
```

### 6.3 Detail Page Template

```svelte
<!-- Standard Detail Page Structure -->
<script lang="ts">
  // Fetch item, tabs state
</script>

<div class="flex flex-col gap-6">
  <!-- Page Header -->
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-4">
      <Button variant="ghost" size="icon" href="/sales/orders">
        <ArrowLeft class="h-4 w-4" />
      </Button>
      <div>
        <h1 class="text-2xl font-bold">Order #SO-2024-0001</h1>
        <div class="flex items-center gap-2">
          <Badge variant="success">Confirmed</Badge>
          <span class="text-muted-foreground">Customer: ABC Corp</span>
        </div>
      </div>
    </div>
    <div class="flex gap-2">
      <Button variant="outline">Print</Button>
      <Button variant="outline">Duplicate</Button>
      <Button href="/sales/orders/123/edit">Edit</Button>
    </div>
  </div>

  <!-- Tabs -->
  <Tabs value="details">
    <TabsList>
      <TabsTrigger value="details">Details</TabsTrigger>
      <TabsTrigger value="items">Line Items</TabsTrigger>
      <TabsTrigger value="history">History</TabsTrigger>
      <TabsTrigger value="documents">Documents</TabsTrigger>
    </TabsList>
    
    <TabsContent value="details">
      <!-- Info Cards Grid -->
      <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Order Information</CardTitle>
          </CardHeader>
          <CardContent>...</CardContent>
        </Card>
        <!-- More cards -->
      </div>
    </TabsContent>
    
    <TabsContent value="items">
      <Table>...</Table>
    </TabsContent>
  </Tabs>
</div>
```

### 6.4 Form Page Template

```svelte
<!-- Standard Form Page Structure -->
<script lang="ts">
  import { superForm } from 'sveltekit-superforms';
  // Form logic with Zod validation
</script>

<div class="flex flex-col gap-6">
  <!-- Page Header -->
  <div class="flex items-center gap-4">
    <Button variant="ghost" size="icon" onclick={() => history.back()}>
      <ArrowLeft class="h-4 w-4" />
    </Button>
    <h1 class="text-2xl font-bold">Create Sales Order</h1>
  </div>

  <!-- Form -->
  <form method="POST" use:enhance class="space-y-8">
    <!-- Section: Basic Information -->
    <Card>
      <CardHeader>
        <CardTitle>Basic Information</CardTitle>
        <CardDescription>Enter the order details</CardDescription>
      </CardHeader>
      <CardContent class="grid gap-6 md:grid-cols-2">
        <FormField name="customer">
          <Label>Customer</Label>
          <Select>...</Select>
          <FormError />
        </FormField>
        <!-- More fields -->
      </CardContent>
    </Card>

    <!-- Section: Line Items -->
    <Card>
      <CardHeader>
        <CardTitle>Line Items</CardTitle>
      </CardHeader>
      <CardContent>
        <LineItemsEditor />
      </CardContent>
    </Card>

    <!-- Actions -->
    <div class="flex justify-end gap-4">
      <Button variant="outline" type="button" onclick={() => history.back()}>
        Cancel
      </Button>
      <Button type="submit" disabled={$submitting}>
        {#if $submitting}
          <Loader2 class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        Create Order
      </Button>
    </div>
  </form>
</div>
```

---

## 7. Implementation Roadmap

### Phase 1: Navigation Restructure (1 week)

| Task | Priority | Effort |
|------|----------|--------|
| Update `navigation.ts` with new structure | P0 | 1 day |
| Create route placeholders for new modules | P0 | 1 day |
| Move pricing routes from `/inventory/pricing` to `/sales/pricing` | P0 | 1 day |
| Add redirects for old routes | P1 | 0.5 day |
| Update breadcrumb logic | P1 | 0.5 day |
| Test navigation across all modules | P0 | 1 day |

### Phase 2: Sales Module (2-3 weeks)

| Task | Priority | Effort |
|------|----------|--------|
| Customer management UI (CRUD) | P0 | 3 days |
| Sales order list & detail pages | P0 | 4 days |
| Sales order form with line items | P0 | 3 days |
| Invoice generation from orders | P1 | 2 days |
| Sales returns workflow | P1 | 2 days |
| Migrate pricing UI to sales module | P0 | 1 day |

### Phase 3: Purchase Module (2 weeks)

| Task | Priority | Effort |
|------|----------|--------|
| Supplier management UI | P0 | 2 days |
| Purchase order CRUD | P0 | 3 days |
| GRN (Goods Receipt Note) workflow | P0 | 3 days |
| Bills management | P1 | 2 days |

### Phase 4: Inventory Operations (2 weeks)

| Task | Priority | Effort |
|------|----------|--------|
| Stock levels overview page | P0 | 2 days |
| Stock transfers UI | P0 | 3 days |
| Stock adjustments UI | P0 | 2 days |
| Stock take / cycle count | P1 | 3 days |
| Valuation reports | P1 | 2 days |

### Phase 5: Reports Module (1 week)

| Task | Priority | Effort |
|------|----------|--------|
| Reports dashboard | P1 | 1 day |
| Sales reports page | P1 | 2 days |
| Inventory reports page | P1 | 2 days |
| Export functionality | P2 | 1 day |

### Phase 6: Polish & Integration (1 week)

| Task | Priority | Effort |
|------|----------|--------|
| Integrations module UI | P2 | 3 days |
| Settings expansion (API keys, notifications) | P2 | 2 days |
| Admin audit log | P2 | 1 day |
| End-to-end testing | P0 | 2 days |

---

## Appendix A: Icon Mapping

| Module | Icon | Lucide Name |
|--------|------|-------------|
| Dashboard | 📊 | `layout-dashboard` |
| Sales | 💰 | `shopping-cart` |
| Purchase | 🛒 | `truck` |
| Inventory | 📦 | `package` |
| Integrations | 🔗 | `plug` |
| Reports | 📈 | `bar-chart-3` |
| Settings | ⚙️ | `settings` |
| Admin | 🛡️ | `shield` |

## Appendix B: Color Coding

| Status/Type | Color | Usage |
|-------------|-------|-------|
| Draft | Gray | `bg-gray-100 text-gray-800` |
| Pending | Yellow | `bg-yellow-100 text-yellow-800` |
| Confirmed | Blue | `bg-blue-100 text-blue-800` |
| Completed | Green | `bg-green-100 text-green-800` |
| Cancelled | Red | `bg-red-100 text-red-800` |
| Overdue | Orange | `bg-orange-100 text-orange-800` |

## Appendix C: Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+K` | Open command palette |
| `Ctrl+N` | Create new item (context-aware) |
| `Ctrl+S` | Save current form |
| `Escape` | Close modal / Cancel |
| `?` | Show keyboard shortcuts |
