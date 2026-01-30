# 8.11 Pricing UI

## Overview

This module implements the frontend for the Pricing system, enabling management of price lists, pricing rules, and discounts/promotions.

## Scope

| Feature | Description |
|---------|-------------|
| Price Lists | Create and manage multiple price lists (wholesale, retail, VIP) |
| Price List Items | Set prices per product/category with quantity breaks |
| Customer Assignment | Assign price lists to specific customers |
| Pricing Rules | Create discount/promotion rules with conditions |
| Price Preview | Calculate and preview prices in real-time |

## Out of Scope

- Inventory Costing (FIFO/AVCO/Standard) - Already in inventory_valuations
- Base product pricing (sale_price, cost_price) - In Product Management (8.4)
- Backend API implementation - Separate backend module

## Dependencies

- 8.4 Product Management UI (products, categories, variants)
- Database schema: `docs/database-erd.dbml` (Pricing Tables section)
- Strategy document: `docs/pricing-strategy.md`

## Route Structure

```
/inventory/pricing/
├── price-lists/                    # Price list management
│   ├── (list)                      # List all price lists
│   ├── new                         # Create new price list
│   └── [id]/                       # Price list details
│       ├── (view)                  # View price list items
│       └── edit                    # Edit price list
├── rules/                          # Pricing rules management
│   ├── (list)                      # List all rules
│   ├── new                         # Create new rule
│   └── [id]/                       # Rule details
│       ├── (view)                  # View rule
│       └── edit                    # Edit rule
└── calculator/                     # Price calculator/preview tool
```

## Tasks

| Task | Description | Status |
|------|-------------|--------|
| 08.11.01 | Price List Management UI | Pending |
| 08.11.02 | Pricing Rules UI | Pending |
| 08.11.03 | Price Calculation & Preview | Pending |

## Technical Stack

- SvelteKit with Svelte 5 Runes
- shadcn-svelte components
- TypeScript
- Mock data (until backend ready)

## UI/UX Guidelines

### Design Principles

1. **Progressive Disclosure**: Show simple options first, advanced features on demand
2. **Inline Editing**: Allow quick edits in tables where possible
3. **Real-time Preview**: Show price impact as rules are configured
4. **Bulk Actions**: Support bulk price updates via import/export

### Color Coding

| Element | Color | Usage |
|---------|-------|-------|
| Discount | Green | Negative percentage, savings |
| Markup | Blue | Positive percentage, increase |
| Fixed Price | Gray | Override prices |
| Inactive | Muted | Disabled items |
| Expired | Red | Past validity period |

### Navigation Integration

Price Lists and Rules should be accessible from:
1. Main sidebar under "Inventory" > "Pricing"
2. Product detail page (quick link to set prices)
3. Command palette (Ctrl+K)
