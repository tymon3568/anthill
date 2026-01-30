# Business Flows Documentation

Thư mục này chứa documentation về các nghiệp vụ logic giữa các module trong hệ thống Anthill ERP.

## Approach

Mỗi business flow được document bằng 2 kỹ thuật chính:

### 1. Event Storming
- **Domain Events** (🟧): Sự kiện đã xảy ra (past tense)
- **Commands** (🟦): Hành động được request
- **Aggregates** (🟨): Entity xử lý command
- **Policies** (🟪): Phản ứng tự động ("When X happens, do Y")
- **Read Models** (🟩): Data cần thiết để ra quyết định
- **Hotspots** (🔴): Câu hỏi, vấn đề cần giải quyết

### 2. Sequence Diagrams
- Chi tiết kỹ thuật về flow giữa các components
- Happy path và error scenarios
- API calls, database operations, event publishing

## Folder Structure

```
docs/
├── business-flows/                    # Cross-module business logic
│   ├── README.md                      # This file
│   ├── product-creation-flow.md       # Product creation flow
│   ├── stock-receipt-flow.md          # GRN (Goods Receipt Note) flow
│   ├── stock-transfer-flow.md         # Stock transfer between warehouses
│   ├── price-calculation-flow.md      # Pricing resolution flow
│   └── inventory-valuation-flow.md    # FIFO/AVCO calculation flow
│
├── templates/
│   └── business-flow-template.md      # Template for new flows
│
└── modules/                           # Per-module PRDs
    ├── inventory/
    │   └── PRD.md
    ├── pricing/
    │   └── PRD.md
    └── orders/
        └── PRD.md
```

## Flow Categories

### Core Inventory Flows
| Flow | Status | Description |
|------|--------|-------------|
| [Product Creation](./product-creation-flow.md) | ✅ Draft | Create product, set valuation method |
| Stock Receipt (GRN) | 📝 Planned | Receive goods, update stock, create valuation layers |
| Stock Transfer | 📝 Planned | Transfer between warehouses |
| Stock Adjustment | 📝 Planned | Adjust stock levels (loss, damage, etc.) |

### Pricing Flows
| Flow | Status | Description |
|------|--------|-------------|
| Price Calculation | 📝 Planned | Resolve price from base → price list → rules |
| Price List Management | 📝 Planned | Create and manage price lists |

### Valuation Flows
| Flow | Status | Description |
|------|--------|-------------|
| FIFO Costing | 📝 Planned | Calculate COGS using FIFO method |
| AVCO Costing | 📝 Planned | Calculate COGS using Average Cost |
| Standard Costing | 📝 Planned | Variance analysis for standard cost |

## How to Create New Flow Documentation

1. Copy template: `cp templates/business-flow-template.md business-flows/{flow-name}.md`

2. Fill in sections:
   - **Overview**: Purpose, actors, triggers
   - **Event Storming**: Visual event flow with all components
   - **Sequence Diagram**: Technical implementation details
   - **State Machine**: If applicable (ERP documents)
   - **Domain Events**: Event definitions and subscriptions
   - **Business Rules**: Validation, computation, authorization
   - **Error Scenarios**: Expected and system errors
   - **Implementation Checklist**: Track progress

3. Add to this README's Flow Categories table

4. Link from relevant module PRDs

## Event Naming Convention

```
{bounded_context}.{aggregate}.{event_name}

Examples:
- inventory.product.created
- inventory.grn.confirmed
- inventory.stock_level.updated
- pricing.price_list.activated
- orders.sales_order.confirmed
```

## Cross-Module Event Subscriptions

| Source | Event | Target Module | Reaction |
|--------|-------|---------------|----------|
| Product Master | `product.created` | Valuation | Configure valuation method |
| Product Master | `product.created` | Audit | Log creation |
| GRN | `grn.confirmed` | Stock Levels | Increase quantity |
| GRN | `grn.confirmed` | Valuation | Create valuation layer |
| Stock Transfer | `transfer.shipped` | Stock Levels | Decrease source, create in-transit |
| Stock Transfer | `transfer.received` | Stock Levels | Increase destination |
| Price List | `price_list.activated` | Cache | Invalidate price cache |

## Related Documentation

- [Module Implementation Workflow](../module-implementation-workflow.md) - Development standards
- [Database ERD](../database-erd.dbml) - Data model
- [Pricing Strategy](../pricing-strategy.md) - Pricing architecture
- [UI Architecture](../ui-architecture-proposal.md) - Frontend structure
