# Task 8.3.04 - Dashboard Data API Integration

## Overview
Integrate dashboard with backend APIs to display real-time inventory metrics, recent orders, and tenant statistics.

## Sub-tasks

### 8.3.04.1 - Inventory Metrics API Client
- Create API client functions for inventory metrics (total products, low stock alerts, value)
- Implement tenant-aware requests with proper error handling
- Add loading states and error boundaries

### 8.3.04.2 - Recent Orders API Client
- Create API client for fetching recent orders with pagination
- Implement order status filtering and sorting
- Add real-time order updates via polling or websockets

### 8.3.04.3 - Tenant Statistics API Client
- Create API client for tenant-level statistics (revenue, order count, user activity)
- Implement date range filtering for metrics
- Add chart data transformation for visualization

### 8.3.04.4 - Dashboard State Management
- Implement reactive state for dashboard data
- Add auto-refresh functionality for live metrics
- Handle offline/error states gracefully

## Dependencies
- [8.2.04] API Infrastructure Setup (HTTP client, error handling, tenant context)
- Backend services: Inventory, Order, User services

## Acceptance Criteria
- [ ] Dashboard loads real-time inventory metrics
- [ ] Recent orders display with proper pagination
- [ ] Tenant statistics show revenue and activity data
- [ ] All API calls include tenant context
- [ ] Error states handled gracefully with user feedback
- [ ] Loading states prevent UI blocking
- [ ] Auto-refresh works without manual page reload

## Files to Create/Modify
- `src/lib/api/dashboard.ts` - Dashboard API client
- `src/routes/dashboard/+page.svelte` - Update dashboard page with API integration
- `src/lib/stores/dashboard.ts` - Dashboard state management
