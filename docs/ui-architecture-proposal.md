# UI Architecture Proposal for Anthill Platform

> **Document Version:** 1.1  
> **Created:** 2026-01-26  
> **Updated:** 2026-01-30  
> **Status:** Proposal  
> **References:** Odoo 17/18, SAP Fiori, ERPNext, NetSuite, Zoho Inventory, Akeneo PIM, Salsify

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [ERP UI Research](#2-erp-ui-research)
3. [Proposed Module Structure](#3-proposed-module-structure)
4. [Navigation Design](#4-navigation-design)
5. [Route Architecture](#5-route-architecture)
6. [Page Templates](#6-page-templates)
7. [Implementation Roadmap](#7-implementation-roadmap)
8. [Products Module - Feature Gap Analysis](#8-products-module---feature-gap-analysis)

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

## 8. Products Module - Feature Gap Analysis

> **Research Date:** 2026-01-30  
> **Sources:** Odoo 17/18, NetSuite, ERPNext, Akeneo PIM, Salsify, user surveys 2024-2025

### 8.1 Current Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| Product CRUD | ✅ Complete | Create, Read, Update, Soft Delete |
| SKU uniqueness per tenant | ✅ Complete | Enforced at DB level |
| Product Types | ✅ Complete | goods/service/consumable |
| Categories (hierarchical) | ✅ Complete | Tree structure |
| Product Variants | ✅ Complete | Key-value attributes, price delta |
| Pricing | ✅ Complete | Sale price, cost price, profit margin |
| Multi-currency | ✅ Complete | VND/USD support |
| Physical attributes | ✅ Complete | Weight, dimensions |
| Tracking methods | ✅ Complete | none/lot/serial |
| Status flags | ✅ Complete | active, sellable, purchasable |
| Soft delete | ✅ Complete | Mark as deleted |
| Bulk operations | ✅ Complete | Activate/deactivate/delete |
| Search & Filter | ✅ Complete | Full-text search |
| Pagination & Sorting | ✅ Complete | Configurable |
| Default UOM | ✅ Complete | Per product |

### 8.2 Missing Features - Priority Analysis

#### 🔴 CRITICAL (85%+ user demand)

| Feature | Description | User Demand | Effort | ERP Reference |
|---------|-------------|-------------|--------|---------------|
| **Product Images/Media** | Upload, manage multiple images per product with drag-drop reordering, primary image selection | **92%** | Medium | Odoo, Shopify, NetSuite |
| **Import/Export CSV** | Bulk import/export products from CSV/Excel with field mapping | **89%** | Medium | All ERPs |
| **Barcode/GTIN Field** | Dedicated barcode field (EAN-13, UPC, ISBN, custom) | **85%** | Low | Odoo, ERPNext |
| **Product Bundles/Kits** | Combine multiple products into sellable combo/kit | **82%** | High | Odoo (BoM), NetSuite |
| **UOM Conversions** | Unit conversion rules (1 box = 12 pcs, 1 kg = 1000 g) | **80%** | Medium | Odoo, SAP |

#### 🟠 HIGH PRIORITY (60-80% user demand)

| Feature | Description | User Demand | Effort |
|---------|-------------|-------------|--------|
| **SEO Fields** | Meta title, meta description, URL slug for ecommerce | **78%** | Low |
| **Custom/Dynamic Attributes** | User-defined fields per category (e.g., RAM for electronics) | **75%** | High |
| **Product Templates** | Create products from predefined templates | **72%** | Medium |
| **Related Products** | Cross-sell, upsell, accessories linking | **70%** | Medium |
| **Product Tags** | Flexible tagging for filtering/search | **68%** | Low |
| **Supplier Info** | Preferred vendors, vendor SKU, lead time, MOQ | **65%** | Medium |
| **Brand/Manufacturer** | Dedicated brand field with brand management | **63%** | Low |
| **Product History/Audit** | View change log (who changed what, when) | **62%** | Medium |

#### 🟡 MEDIUM PRIORITY (40-60% user demand)

| Feature | Description | User Demand | Effort |
|---------|-------------|-------------|--------|
| **Multi-language** | Product name/description in multiple languages | **58%** | High |
| **Product Documents** | Attach PDFs, datasheets, manuals | **55%** | Medium |
| **Variant Matrix** | Create variants via matrix (size × color) | **52%** | Medium |
| **Product Lifecycle** | Status workflow: Draft → Active → Discontinued → Archived | **50%** | Low |
| **Minimum Order Qty (MOQ)** | Minimum quantity for purchase/sale | **48%** | Low |
| **Lead Time** | Default delivery/production time | **45%** | Low |
| **Country of Origin** | Product origin for customs/compliance | **42%** | Low |
| **HS Code** | Harmonized System code for import/export | **40%** | Low |

#### 🟢 NICE-TO-HAVE (20-40% user demand)

| Feature | Description | User Demand |
|---------|-------------|-------------|
| AI Product Description | Auto-generate descriptions using AI | 35% |
| Product Comparison | Side-by-side product comparison | 32% |
| 3D/AR Preview | 3D product visualization | 28% |
| Product Reviews Integration | Sync reviews from ecommerce channels | 25% |
| Sustainability/ESG Data | Carbon footprint, eco-labels | 22% |
| Product Configurator | Build-to-order configuration | 20% |

### 8.3 Competitor Comparison Matrix

| Feature | Anthill | Odoo | NetSuite | ERPNext | Akeneo |
|---------|---------|------|----------|---------|--------|
| **Core CRUD** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Variants** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Categories** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Pricing** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Physical Attrs** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Images/Media** | ❌ | ✅ | ✅ | ✅ | ✅ |
| **Import/Export** | ❌ | ✅ | ✅ | ✅ | ✅ |
| **Barcode** | ❌ | ✅ | ✅ | ✅ | ✅ |
| **Bundles/Kits** | ❌ | ✅ | ✅ | ✅ | ❌ |
| **UOM Conversion** | ❌ | ✅ | ✅ | ✅ | ❌ |
| **SEO Fields** | ❌ | ✅ | ✅ | ❌ | ✅ |
| **Custom Attrs** | ❌ | ✅ | ✅ | ✅ | ✅ |
| **Related Products** | ❌ | ✅ | ✅ | ❌ | ✅ |
| **Supplier Info** | ❌ | ✅ | ✅ | ✅ | ❌ |
| **Brand** | ❌ | ✅ | ✅ | ✅ | ✅ |
| **Multi-language** | ❌ | ✅ | ✅ | ❌ | ✅ |
| **Documents** | ❌ | ✅ | ✅ | ✅ | ✅ |
| **History/Audit** | ❌ | ✅ | ✅ | ✅ | ✅ |

### 8.4 Proposed Products UI Structure

```
├── 📦 INVENTORY
│   ├── Products
│   │   ├── Product List
│   │   │   ├── Grid/List View Toggle                    🆕
│   │   │   ├── Quick Filters (by tag, brand, status)   🆕
│   │   │   └── Bulk Import/Export                       🆕
│   │   │
│   │   ├── Product Detail (Tabbed Interface)
│   │   │   ├── Basic Info Tab
│   │   │   │   ├── **Images Gallery**                   🆕 (drag-drop, reorder, primary)
│   │   │   │   ├── **Barcode/GTIN**                     🆕
│   │   │   │   ├── **Brand/Manufacturer**               🆕
│   │   │   │   └── **Tags**                             🆕
│   │   │   │
│   │   │   ├── Pricing Tab (existing)
│   │   │   │
│   │   │   ├── Inventory Tab
│   │   │   │   └── **Supplier Info**                    🆕 (vendors, vendor SKU)
│   │   │   │
│   │   │   ├── Variants Tab
│   │   │   │   └── **Variant Matrix Creator**           🆕
│   │   │   │
│   │   │   ├── **SEO Tab**                              🆕
│   │   │   │   ├── Meta Title
│   │   │   │   ├── Meta Description
│   │   │   │   └── URL Slug
│   │   │   │
│   │   │   ├── **Related Tab**                          🆕
│   │   │   │   ├── Cross-sell Products
│   │   │   │   ├── Upsell Products
│   │   │   │   └── Accessories
│   │   │   │
│   │   │   ├── **Documents Tab**                        🆕
│   │   │   │   └── Attachments (PDF, specs)
│   │   │   │
│   │   │   └── **History Tab**                          🆕
│   │   │       └── Audit Log
│   │   │
│   │   ├── **Bundles/Kits**                             🆕
│   │   │   ├── Bundle List
│   │   │   ├── Create Bundle (add components)
│   │   │   └── Bundle Detail
│   │   │
│   │   ├── **Import/Export**                            🆕
│   │   │   ├── Import Products (CSV/Excel)
│   │   │   ├── Export Products
│   │   │   └── Import Templates
│   │   │
│   │   ├── **Brands**                                   🆕
│   │   │   └── Brand Management (CRUD)
│   │   │
│   │   ├── **Attributes**                               🆕
│   │   │   ├── Attribute Groups
│   │   │   └── Attribute Values
│   │   │
│   │   └── **UOM Management**                           🆕
│   │       ├── Units of Measure
│   │       └── UOM Conversions
│   │
│   ├── Categories (existing)
│   └── ... (other inventory sections)
```

### 8.5 Products Implementation Roadmap

#### Sprint 1 (Week 1-2): Critical Features

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Product Images/Media management | P0 | 4 days | File storage service |
| Barcode field (frontend + backend) | P0 | 1 day | None |
| Import/Export CSV functionality | P0 | 3 days | None |
| Image gallery component (drag-drop) | P0 | 2 days | Images feature |

#### Sprint 2 (Week 3-4): High Priority Features

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Product Bundles/Kits module | P1 | 5 days | None |
| UOM Conversions management | P1 | 3 days | None |
| SEO fields (meta, slug) | P1 | 1 day | None |
| Product Tags system | P1 | 2 days | None |

#### Sprint 3 (Week 5-6): Enhancement Features

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Brand management | P2 | 2 days | None |
| Related Products (cross-sell/upsell) | P2 | 3 days | None |
| Supplier Info per product | P2 | 2 days | Purchase module |
| Product History/Audit viewer | P2 | 2 days | Audit log service |

#### Sprint 4 (Week 7-8): Advanced Features

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Custom/Dynamic Attributes | P2 | 5 days | Attribute schema |
| Variant Matrix creator | P2 | 3 days | Variants feature |
| Product Documents/Attachments | P3 | 2 days | File storage |
| Product Templates | P3 | 3 days | None |

### 8.6 Database Schema Additions

```sql
-- Product Images
CREATE TABLE product_images (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    product_id UUID NOT NULL REFERENCES products(id),
    tenant_id UUID NOT NULL,
    url TEXT NOT NULL,
    alt_text TEXT,
    position INTEGER DEFAULT 0,
    is_primary BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(product_id, position)
);

-- Product Barcodes
ALTER TABLE products ADD COLUMN barcode VARCHAR(50);
ALTER TABLE products ADD COLUMN barcode_type VARCHAR(20); -- EAN13, UPC, ISBN, CUSTOM

-- Product SEO
ALTER TABLE products ADD COLUMN meta_title VARCHAR(70);
ALTER TABLE products ADD COLUMN meta_description VARCHAR(160);
ALTER TABLE products ADD COLUMN url_slug VARCHAR(255);

-- Product Tags
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL,
    name VARCHAR(50) NOT NULL,
    color VARCHAR(7), -- Hex color
    UNIQUE(tenant_id, name)
);

CREATE TABLE product_tags (
    product_id UUID REFERENCES products(id),
    tag_id UUID REFERENCES tags(id),
    PRIMARY KEY(product_id, tag_id)
);

-- Brands
CREATE TABLE brands (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    logo_url TEXT,
    website TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

ALTER TABLE products ADD COLUMN brand_id UUID REFERENCES brands(id);

-- Product Bundles
CREATE TABLE product_bundles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL,
    bundle_product_id UUID NOT NULL REFERENCES products(id),
    component_product_id UUID NOT NULL REFERENCES products(id),
    quantity DECIMAL(10,3) NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- UOM Conversions
CREATE TABLE uom_conversions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL,
    from_uom_id UUID NOT NULL REFERENCES uom(id),
    to_uom_id UUID NOT NULL REFERENCES uom(id),
    factor DECIMAL(15,6) NOT NULL, -- 1 from_uom = factor * to_uom
    product_id UUID REFERENCES products(id), -- NULL = global
    UNIQUE(tenant_id, from_uom_id, to_uom_id, product_id)
);

-- Related Products
CREATE TABLE related_products (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    product_id UUID NOT NULL REFERENCES products(id),
    related_product_id UUID NOT NULL REFERENCES products(id),
    relation_type VARCHAR(20) NOT NULL, -- cross_sell, upsell, accessory
    position INTEGER DEFAULT 0,
    UNIQUE(product_id, related_product_id, relation_type)
);

-- Product Suppliers
CREATE TABLE product_suppliers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    product_id UUID NOT NULL REFERENCES products(id),
    supplier_id UUID NOT NULL, -- REFERENCES suppliers(id)
    vendor_sku VARCHAR(50),
    vendor_name VARCHAR(200),
    purchase_price BIGINT,
    currency VARCHAR(3) DEFAULT 'VND',
    lead_time_days INTEGER,
    min_order_qty DECIMAL(10,3),
    is_preferred BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Product Documents
CREATE TABLE product_documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    product_id UUID NOT NULL REFERENCES products(id),
    tenant_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    file_url TEXT NOT NULL,
    file_type VARCHAR(50),
    file_size INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### 8.7 API Endpoints (New)

```
# Product Images
POST   /api/v1/products/{id}/images          Upload image
GET    /api/v1/products/{id}/images          List images
PUT    /api/v1/products/{id}/images/reorder  Reorder images
DELETE /api/v1/products/{id}/images/{imgId}  Delete image

# Import/Export
POST   /api/v1/products/import               Import from CSV
GET    /api/v1/products/export               Export to CSV
GET    /api/v1/products/import/template      Download template

# Bundles
GET    /api/v1/bundles                       List bundles
POST   /api/v1/bundles                       Create bundle
GET    /api/v1/bundles/{id}                  Get bundle
PUT    /api/v1/bundles/{id}                  Update bundle
DELETE /api/v1/bundles/{id}                  Delete bundle

# Brands
GET    /api/v1/brands                        List brands
POST   /api/v1/brands                        Create brand
PUT    /api/v1/brands/{id}                   Update brand
DELETE /api/v1/brands/{id}                   Delete brand

# Tags
GET    /api/v1/tags                          List tags
POST   /api/v1/tags                          Create tag
DELETE /api/v1/tags/{id}                     Delete tag
POST   /api/v1/products/{id}/tags            Add tags to product

# UOM Conversions
GET    /api/v1/uom-conversions               List conversions
POST   /api/v1/uom-conversions               Create conversion
DELETE /api/v1/uom-conversions/{id}          Delete conversion

# Related Products
GET    /api/v1/products/{id}/related         Get related products
POST   /api/v1/products/{id}/related         Add related product
DELETE /api/v1/products/{id}/related/{relId} Remove relation

# Product Suppliers
GET    /api/v1/products/{id}/suppliers       List suppliers
POST   /api/v1/products/{id}/suppliers       Add supplier
PUT    /api/v1/products/{id}/suppliers/{sid} Update supplier
DELETE /api/v1/products/{id}/suppliers/{sid} Remove supplier

# Documents
POST   /api/v1/products/{id}/documents       Upload document
GET    /api/v1/products/{id}/documents       List documents
DELETE /api/v1/products/{id}/documents/{did} Delete document
```

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
