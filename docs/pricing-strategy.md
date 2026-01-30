# Pricing Strategy for Anthill Inventory Platform

> **Document Version:** 1.0  
> **Created:** 2026-01-24  
> **Status:** Proposal  
> **References:** Odoo, SAP S/4HANA, ERPNext, Oracle NetSuite

## Table of Contents

1. [Overview](#1-overview)
   - 1.1 [Purpose](#11-purpose)
   - 1.2 [Goals](#12-goals)
   - 1.3 [Scope](#13-scope)
   - 1.4 [Module Separation Principle](#14-module-separation-principle) ⭐ *New*
2. [Research Summary](#2-research-summary)
3. [Current State Analysis](#3-current-state-analysis)
4. [Proposed Architecture](#4-proposed-architecture)
5. [Database Schema](#5-database-schema)
6. [Price Calculation Algorithm](#6-price-calculation-algorithm)
7. [API Design](#7-api-design)
8. [Implementation Roadmap](#8-implementation-roadmap)
9. [Comparison with Enterprise ERPs](#9-comparison-with-enterprise-erps)

---

## 1. Overview

### 1.1 Purpose

This document outlines the pricing strategy and implementation plan for Anthill Inventory Platform. It covers both **Inventory Costing** (cost of goods) and **Sales Pricing** (selling price to customers).

### 1.2 Goals

- Flexible pricing that supports B2B and B2C scenarios
- Multi-currency support (VND, USD, etc.)
- Customer-specific and quantity-based pricing
- Promotional and discount rule engine
- Integration with existing inventory valuation system

### 1.3 Scope

| Component | Status | Description |
|-----------|--------|-------------|
| Inventory Costing | ✅ Implemented | FIFO, AVCO, Standard costing methods |
| Base Product Pricing | ✅ Implemented | sale_price, cost_price on products table |
| Price Lists | ❌ Not Implemented | Customer/group-specific price lists |
| Pricing Rules | ❌ Not Implemented | Discounts, promotions, conditions |
| Contract Pricing | ❌ Not Implemented | Long-term customer agreements |

### 1.4 Module Separation Principle

> **Key Design Decision:** Anthill separates **cost management** (Inventory) from **revenue management** (Orders/Sales) following industry best practices from Odoo, SAP, and ERPNext.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        MODULE RESPONSIBILITY MATRIX                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────┐    ┌─────────────────────────────────┐ │
│  │     INVENTORY MODULE            │    │     ORDERS/SALES MODULE         │ │
│  │     (Cost Management)           │    │     (Revenue Management)        │ │
│  ├─────────────────────────────────┤    ├─────────────────────────────────┤ │
│  │                                 │    │                                 │ │
│  │  📦 Products & Variants        │    │  💰 Price Lists                 │ │
│  │     - SKU, barcode, attributes │    │     - Customer-specific prices  │ │
│  │     - Product categories       │    │     - Quantity breaks           │ │
│  │                                 │    │     - Multi-currency pricing    │ │
│  │  💵 Cost Price (cost_price)    │    │                                 │ │
│  │     - Purchase cost            │    │  🏷️ Sale Price (sale_price)    │ │
│  │     - Landed cost              │    │     - Base selling price        │ │
│  │     - Manufacturing cost       │    │     - Markup from cost          │ │
│  │                                 │    │                                 │ │
│  │  📊 Inventory Valuation        │    │  🎁 Pricing Rules & Discounts  │ │
│  │     - FIFO costing             │    │     - Percentage discounts      │ │
│  │     - AVCO (Moving Average)    │    │     - Fixed amount discounts    │ │
│  │     - Standard costing         │    │     - Buy X Get Y promotions    │ │
│  │                                 │    │     - Seasonal campaigns        │ │
│  │  🏭 Stock & Warehouses         │    │                                 │ │
│  │     - Stock levels             │    │  📋 Orders                      │ │
│  │     - Stock movements          │    │     - Sales Orders              │ │
│  │     - Warehouse locations      │    │     - Purchase Orders           │ │
│  │     - Stock adjustments        │    │     - Returns & Refunds         │ │
│  │                                 │    │                                 │ │
│  └─────────────────────────────────┘    └─────────────────────────────────┘ │
│                                                                              │
│  ════════════════════════════════════════════════════════════════════════   │
│                                                                              │
│  📍 Navigation Structure:                                                    │
│                                                                              │
│  /inventory/                          /orders/                               │
│  ├── products/                        ├── sales/                             │
│  ├── categories/                      ├── purchase/                          │
│  ├── stock/                           ├── returns/                           │
│  ├── adjustments/                     └── pricing/          ◄── Price Lists │
│  └── warehouses/                          ├── price-lists/                   │
│                                           └── rules/                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Why This Separation?

| Aspect | Inventory (Cost) | Orders/Sales (Revenue) |
|--------|------------------|------------------------|
| **Primary User** | Warehouse staff, Accountants | Sales team, Managers |
| **Focus** | "How much did it cost us?" | "How much do we sell it for?" |
| **Data Source** | Supplier invoices, Production | Market strategy, Customer value |
| **Volatility** | Changes with purchases | Changes with promotions, seasons |
| **Visibility** | Internal only | Customer-facing |

#### Industry References

| ERP System | Cost Management | Sales Pricing |
|------------|-----------------|---------------|
| **Odoo** | Inventory → Valuation | Sales → Configuration → Pricelists |
| **SAP S/4HANA** | Materials Management (MM) | Sales & Distribution (SD) |
| **ERPNext** | Stock → Item Valuation | Selling → Price List |
| **NetSuite** | Inventory → Costing | Sales → Pricing |
| **Dynamics 365** | Inventory Costing | Sales Pricing |

> **Note:** While `sale_price` is stored on the `products` table for convenience (default base price), the **Price Lists** and **Pricing Rules** that modify this base price belong to the Orders/Sales domain because they are customer-facing and sales-driven.

---

## 2. Research Summary

### 2.1 Odoo Pricing Model

**Key Features:**
- **Pricelists**: Multiple price lists per company
- **Pricelist Items**: Rules with conditions (quantity, date, product/category)
- **Compute Methods**: Fixed price, percentage discount, formula
- **Customer Assignment**: Each customer has a default pricelist

**Strengths:**
- Highly flexible pricelist hierarchy
- Formula-based pricing for complex calculations
- Good quantity break support

**Reference:** [Odoo Pricelist Documentation](https://www.odoo.com/documentation/17.0/applications/sales/sales/products_prices/prices/pricing.html)

### 2.2 SAP S/4HANA Pricing Model

**Key Features:**
- **Condition Records**: Price components stored as conditions
- **Pricing Procedure**: Sequential application of conditions
- **Material Price**: Standard Price vs Moving Average Price (MAP)
- **Customer Hierarchy**: Prices cascade through customer groups

**Strengths:**
- Very granular control over pricing components
- Strong integration with financial accounting
- Supports complex B2B pricing scenarios

**Reference:** SAP Product Costing and Pricing Procedure documentation

### 2.3 ERPNext Pricing Model

**Key Features:**
- **Price Lists**: Buying and Selling price lists
- **Pricing Rules**: Discount rules with multiple conditions
- **Item Price**: Price per item per price list
- **Valuation**: FIFO and Moving Average

**Strengths:**
- Simple and intuitive
- Open-source and customizable
- Good documentation

**Reference:** [ERPNext Pricing Rule Documentation](https://docs.erpnext.com/docs/user/manual/en/pricing-rule)

### 2.4 Key Takeaways

| Feature | Odoo | SAP | ERPNext | Industry Best Practice |
|---------|------|-----|---------|------------------------|
| Price List Hierarchy | ✅ | ✅ | ✅ | Essential |
| Quantity Breaks | ✅ | ✅ | ✅ | Essential |
| Customer Groups | ✅ | ✅ | ✅ | Essential |
| Date Validity | ✅ | ✅ | ✅ | Essential |
| Multi-currency | ✅ | ✅ | ✅ | Essential |
| Formula Pricing | ✅ | ✅ | ❌ | Nice to have |
| Condition-based Rules | ✅ | ✅ | ✅ | Essential |
| Priority/Sequence | ✅ | ✅ | ✅ | Essential |

---

## 3. Current State Analysis

### 3.1 Existing Pricing in Anthill

#### Products Table
```sql
-- Already implemented
products.sale_price    -- Base selling price (BIGINT, cents)
products.cost_price    -- Base cost price (BIGINT, cents)
products.currency_code -- Currency (VARCHAR, default 'VND')
```

#### Product Variants Table
```sql
-- Already implemented
product_variants.price_difference  -- Delta from parent product (BIGINT, cents)
```

#### Inventory Valuation Tables
```sql
-- Already implemented
inventory_valuations.valuation_method     -- 'fifo', 'avco', 'standard'
inventory_valuations.current_unit_cost    -- Current cost per unit
inventory_valuation_layers                -- FIFO cost layers
inventory_valuation_settings              -- Method per tenant/category/product
```

### 3.2 What's Missing

1. **Price Lists** - No support for multiple price lists
2. **Customer-specific Pricing** - Cannot set different prices per customer
3. **Quantity Breaks** - No tiered pricing based on order quantity
4. **Promotional Rules** - No discount/promotion engine
5. **Time-based Pricing** - No seasonal or promotional pricing
6. **Contract Pricing** - No long-term pricing agreements

---

## 4. Proposed Architecture

### 4.1 Three-Layer Pricing Model

```
┌─────────────────────────────────────────────────────────────────┐
│                     PRICING ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ LAYER 1: BASE PRICE                                       │  │
│  │                                                           │  │
│  │  Source: products.sale_price + variants.price_difference  │  │
│  │  Purpose: Default price when no other rules apply         │  │
│  │  Currency: products.currency_code                         │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              ↓                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ LAYER 2: PRICE LISTS                                      │  │
│  │                                                           │  │
│  │  Features:                                                │  │
│  │  ├── Customer group pricing (B2B, B2C, Wholesale, VIP)   │  │
│  │  ├── Multi-currency support (VND, USD, EUR)              │  │
│  │  ├── Time-based validity (seasonal, promotional)         │  │
│  │  ├── Quantity breaks (buy more, pay less)                │  │
│  │  └── Category-level pricing                              │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              ↓                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ LAYER 3: PRICING RULES                                    │  │
│  │                                                           │  │
│  │  Features:                                                │  │
│  │  ├── Discount rules (percentage, fixed amount)           │  │
│  │  ├── Promotional campaigns (buy X get Y)                 │  │
│  │  ├── Coupon/voucher support                              │  │
│  │  ├── Conditional rules (min qty, min amount, etc.)       │  │
│  │  └── Combinable vs exclusive discounts                   │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              ↓                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ OUTPUT: FINAL PRICE                                       │  │
│  │                                                           │  │
│  │  {                                                        │  │
│  │    base_price: 1000000,                                   │  │
│  │    list_price: 900000,                                    │  │
│  │    discounts: [...],                                      │  │
│  │    final_price: 810000,                                   │  │
│  │    currency: 'VND'                                        │  │
│  │  }                                                        │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Price Resolution Priority

When multiple prices are available, the system resolves using this priority:

1. **Contract Price** (highest priority) - Customer-specific agreements
2. **Customer Price List** - Assigned directly to customer
3. **Customer Group Price List** - Based on customer's group (VIP, Wholesale)
4. **Variant Price List Item** - Specific to product variant
5. **Product Price List Item** - Specific to product
6. **Category Price List Item** - Based on product category
7. **Default Price List** - Tenant's default price list
8. **Base Price** (lowest priority) - product.sale_price + variant.price_difference

---

## 5. Database Schema

### 5.1 Price Lists Table

```sql
-- Main price list definition
CREATE TABLE price_lists (
    price_list_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    
    -- Basic info
    name VARCHAR(255) NOT NULL,
    code VARCHAR(50) NOT NULL,
    description TEXT,
    
    -- Currency
    currency_code VARCHAR(3) NOT NULL DEFAULT 'VND',
    
    -- Type: 'sale' for selling, 'purchase' for buying
    price_list_type VARCHAR(20) NOT NULL DEFAULT 'sale'
        CHECK (price_list_type IN ('sale', 'purchase')),
    
    -- Computation method
    based_on VARCHAR(20) NOT NULL DEFAULT 'fixed'
        CHECK (based_on IN ('fixed', 'base_price', 'other_pricelist')),
    
    -- Parent price list (for cascading)
    parent_price_list_id UUID REFERENCES price_lists(price_list_id),
    
    -- Default markup/discount when based on parent
    default_percentage DECIMAL(10, 4) DEFAULT 0,
    
    -- Validity period
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    
    -- Priority (lower = higher priority)
    priority INTEGER NOT NULL DEFAULT 100,
    
    -- Status
    is_default BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(user_id),
    updated_by UUID REFERENCES users(user_id),
    deleted_at TIMESTAMPTZ,
    
    -- Constraints
    CONSTRAINT price_lists_code_unique UNIQUE (tenant_id, code) 
        WHERE deleted_at IS NULL,
    CONSTRAINT price_lists_one_default UNIQUE (tenant_id, is_default, price_list_type) 
        WHERE is_default = true AND deleted_at IS NULL
);

-- Indexes
CREATE INDEX idx_price_lists_tenant ON price_lists(tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_price_lists_active ON price_lists(tenant_id, is_active, priority) WHERE deleted_at IS NULL;
CREATE INDEX idx_price_lists_validity ON price_lists(tenant_id, valid_from, valid_to) WHERE deleted_at IS NULL;
```

### 5.2 Price List Items Table

```sql
-- Individual price entries within a price list
CREATE TABLE price_list_items (
    item_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    price_list_id UUID NOT NULL REFERENCES price_lists(price_list_id),
    
    -- What this price applies to (mutually exclusive)
    apply_to VARCHAR(20) NOT NULL DEFAULT 'product'
        CHECK (apply_to IN ('product', 'variant', 'category', 'all')),
    product_id UUID REFERENCES products(product_id),
    variant_id UUID REFERENCES product_variants(variant_id),
    category_id UUID REFERENCES product_categories(category_id),
    
    -- Quantity range for tiered pricing
    min_quantity BIGINT NOT NULL DEFAULT 1 CHECK (min_quantity >= 1),
    max_quantity BIGINT CHECK (max_quantity IS NULL OR max_quantity >= min_quantity),
    
    -- Computation method
    compute_method VARCHAR(20) NOT NULL DEFAULT 'fixed'
        CHECK (compute_method IN ('fixed', 'percentage', 'formula', 'margin')),
    
    -- Price values (based on compute_method)
    fixed_price BIGINT CHECK (fixed_price IS NULL OR fixed_price >= 0),
    percentage DECIMAL(10, 4),  -- Can be negative for discount
    margin_percentage DECIMAL(10, 4),  -- Target margin based on cost
    formula TEXT,  -- Advanced: e.g., "base_price * 0.9 - 10000"
    
    -- Rounding
    rounding_method VARCHAR(20) DEFAULT 'none'
        CHECK (rounding_method IN ('none', 'round_up', 'round_down', 'round_nearest', 'round_to_99')),
    rounding_precision INTEGER DEFAULT 0,  -- Decimal places or rounding unit
    
    -- Validity (can override parent price list)
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    
    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraint: must have exactly one target
    CONSTRAINT price_list_items_target_check CHECK (
        (apply_to = 'product' AND product_id IS NOT NULL AND variant_id IS NULL AND category_id IS NULL) OR
        (apply_to = 'variant' AND variant_id IS NOT NULL AND product_id IS NULL AND category_id IS NULL) OR
        (apply_to = 'category' AND category_id IS NOT NULL AND product_id IS NULL AND variant_id IS NULL) OR
        (apply_to = 'all' AND product_id IS NULL AND variant_id IS NULL AND category_id IS NULL)
    ),
    
    -- Constraint: must have price value based on method
    CONSTRAINT price_list_items_value_check CHECK (
        (compute_method = 'fixed' AND fixed_price IS NOT NULL) OR
        (compute_method = 'percentage' AND percentage IS NOT NULL) OR
        (compute_method = 'margin' AND margin_percentage IS NOT NULL) OR
        (compute_method = 'formula' AND formula IS NOT NULL)
    )
);

-- Indexes for fast lookup
CREATE INDEX idx_price_list_items_list ON price_list_items(tenant_id, price_list_id);
CREATE INDEX idx_price_list_items_product ON price_list_items(tenant_id, product_id) WHERE product_id IS NOT NULL;
CREATE INDEX idx_price_list_items_variant ON price_list_items(tenant_id, variant_id) WHERE variant_id IS NOT NULL;
CREATE INDEX idx_price_list_items_category ON price_list_items(tenant_id, category_id) WHERE category_id IS NOT NULL;
CREATE INDEX idx_price_list_items_qty ON price_list_items(tenant_id, price_list_id, min_quantity, max_quantity);
```

### 5.3 Customer Price Lists Table

```sql
-- Assignment of price lists to customers
CREATE TABLE customer_price_lists (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    customer_id UUID NOT NULL,  -- References future customers table
    price_list_id UUID NOT NULL REFERENCES price_lists(price_list_id),
    
    -- Priority when customer has multiple price lists
    priority INTEGER NOT NULL DEFAULT 0,
    
    -- Validity
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    
    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(user_id),
    
    -- Unique constraint
    CONSTRAINT customer_price_lists_unique UNIQUE (tenant_id, customer_id, price_list_id)
);

-- Indexes
CREATE INDEX idx_customer_price_lists_customer ON customer_price_lists(tenant_id, customer_id);
CREATE INDEX idx_customer_price_lists_validity ON customer_price_lists(tenant_id, customer_id, valid_from, valid_to);
```

### 5.4 Pricing Rules Table

```sql
-- Discount and promotion rules
CREATE TABLE pricing_rules (
    rule_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    
    -- Basic info
    name VARCHAR(255) NOT NULL,
    code VARCHAR(50),
    description TEXT,
    
    -- Rule type
    rule_type VARCHAR(30) NOT NULL
        CHECK (rule_type IN (
            'discount_percentage',  -- Apply percentage discount
            'discount_amount',      -- Apply fixed discount amount
            'fixed_price',          -- Override to fixed price
            'free_item',            -- Add free item
            'buy_x_get_y',          -- Buy X quantity, get Y free
            'bundle_price'          -- Special price for product bundle
        )),
    
    -- Conditions (flexible JSON structure)
    conditions JSONB NOT NULL DEFAULT '{}',
    /*
    Example conditions:
    {
        "min_quantity": 10,
        "max_quantity": 100,
        "min_order_amount": 1000000,
        "max_order_amount": null,
        "products": ["uuid1", "uuid2"],
        "categories": ["uuid3"],
        "variants": ["uuid4"],
        "customer_ids": ["uuid5"],
        "customer_groups": ["wholesale", "vip"],
        "price_lists": ["uuid6"],
        "weekdays": [1, 2, 3, 4, 5],  -- Monday to Friday
        "time_range": {
            "start": "09:00",
            "end": "17:00"
        },
        "first_order_only": false,
        "new_customer_days": 30
    }
    */
    
    -- Discount values (based on rule_type)
    discount_percentage DECIMAL(10, 4),  -- e.g., 10.0000 for 10%
    discount_amount BIGINT,              -- Fixed amount in cents
    fixed_price BIGINT,                  -- Override price in cents
    
    -- For free item / buy X get Y rules
    free_product_id UUID REFERENCES products(product_id),
    free_variant_id UUID REFERENCES product_variants(variant_id),
    free_quantity BIGINT DEFAULT 1,
    buy_quantity BIGINT,                 -- For buy_x_get_y
    get_quantity BIGINT,                 -- For buy_x_get_y
    
    -- Limits
    max_discount_amount BIGINT,          -- Cap the discount
    usage_limit INTEGER,                 -- Total uses allowed
    usage_count INTEGER NOT NULL DEFAULT 0,
    per_customer_limit INTEGER,          -- Uses per customer
    
    -- Validity
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    
    -- Priority and combination
    priority INTEGER NOT NULL DEFAULT 100,  -- Lower = higher priority
    is_combinable BOOLEAN NOT NULL DEFAULT false,
    exclusive_group VARCHAR(50),  -- Only one rule per group can apply
    
    -- Application scope
    apply_on VARCHAR(20) NOT NULL DEFAULT 'line'
        CHECK (apply_on IN ('line', 'order')),  -- Line item or whole order
    
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(user_id),
    updated_by UUID REFERENCES users(user_id),
    deleted_at TIMESTAMPTZ,
    
    -- Constraints
    CONSTRAINT pricing_rules_code_unique UNIQUE (tenant_id, code) 
        WHERE code IS NOT NULL AND deleted_at IS NULL
);

-- Indexes
CREATE INDEX idx_pricing_rules_tenant ON pricing_rules(tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_pricing_rules_active ON pricing_rules(tenant_id, is_active, priority) 
    WHERE deleted_at IS NULL AND is_active = true;
CREATE INDEX idx_pricing_rules_validity ON pricing_rules(tenant_id, valid_from, valid_to) 
    WHERE deleted_at IS NULL AND is_active = true;
CREATE INDEX idx_pricing_rules_type ON pricing_rules(tenant_id, rule_type) 
    WHERE deleted_at IS NULL AND is_active = true;
CREATE INDEX idx_pricing_rules_conditions ON pricing_rules USING GIN (conditions) 
    WHERE deleted_at IS NULL AND is_active = true;
```

### 5.5 Pricing Rule Usage Table

```sql
-- Track usage of pricing rules per customer
CREATE TABLE pricing_rule_usage (
    usage_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    rule_id UUID NOT NULL REFERENCES pricing_rules(rule_id),
    customer_id UUID,
    order_id UUID,  -- References future orders table
    
    -- Usage details
    discount_amount BIGINT NOT NULL,
    used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Indexes
    CONSTRAINT pricing_rule_usage_order_unique UNIQUE (tenant_id, rule_id, order_id)
);

CREATE INDEX idx_pricing_rule_usage_rule ON pricing_rule_usage(tenant_id, rule_id);
CREATE INDEX idx_pricing_rule_usage_customer ON pricing_rule_usage(tenant_id, customer_id, rule_id);
```

### 5.6 Price History Table

```sql
-- Audit trail for price changes
CREATE TABLE price_history (
    history_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    
    -- What changed
    entity_type VARCHAR(20) NOT NULL
        CHECK (entity_type IN ('product', 'variant', 'price_list_item')),
    entity_id UUID NOT NULL,
    
    -- Change details
    field_name VARCHAR(50) NOT NULL,
    old_value TEXT,
    new_value TEXT,
    
    -- Context
    change_reason TEXT,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by UUID REFERENCES users(user_id),
    
    -- Metadata
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_price_history_entity ON price_history(tenant_id, entity_type, entity_id, changed_at DESC);
CREATE INDEX idx_price_history_time ON price_history(tenant_id, changed_at DESC);
```

---

## 6. Price Calculation Algorithm

### 6.1 Pseudocode

```typescript
interface PriceRequest {
  tenantId: string;
  productId: string;
  variantId?: string;
  customerId?: string;
  quantity: number;
  date?: Date;  // Default: now
  currencyCode?: string;  // Default: tenant currency
}

interface PriceResult {
  basePrice: number;
  listPrice: number;
  finalPrice: number;
  currency: string;
  discounts: Discount[];
  priceListUsed?: string;
  rulesApplied: string[];
}

async function calculatePrice(request: PriceRequest): Promise<PriceResult> {
  const { tenantId, productId, variantId, customerId, quantity, date = new Date() } = request;
  
  // Step 1: Get base price
  const product = await getProduct(tenantId, productId);
  let basePrice = product.salePrice;
  
  if (variantId) {
    const variant = await getVariant(tenantId, variantId);
    basePrice += variant.priceDifference;
  }
  
  // Step 2: Find applicable price list
  const priceLists = await getApplicablePriceLists(tenantId, customerId, date);
  let listPrice = basePrice;
  let priceListUsed: string | undefined;
  
  for (const priceList of priceLists) {
    const item = await findPriceListItem(priceList, productId, variantId, product.categoryId, quantity, date);
    
    if (item) {
      listPrice = computePriceFromItem(basePrice, item, product.costPrice);
      priceListUsed = priceList.id;
      break;  // Use first matching price list (ordered by priority)
    }
  }
  
  // Step 3: Apply pricing rules
  const rules = await getApplicablePricingRules(tenantId, {
    productId,
    variantId,
    categoryId: product.categoryId,
    customerId,
    quantity,
    amount: listPrice * quantity,
    date
  });
  
  const discounts: Discount[] = [];
  let finalPrice = listPrice;
  const rulesApplied: string[] = [];
  
  // Group rules by exclusive_group
  const ruleGroups = groupBy(rules, r => r.exclusiveGroup || r.id);
  
  for (const [groupId, groupRules] of Object.entries(ruleGroups)) {
    // Take highest priority rule from each group
    const rule = groupRules.sort((a, b) => a.priority - b.priority)[0];
    
    // Check if combinable or first rule
    if (!rule.isCombinable && discounts.length > 0) {
      continue;
    }
    
    // Check usage limits
    if (!await checkRuleUsage(rule, customerId)) {
      continue;
    }
    
    const discount = applyRule(rule, finalPrice, quantity);
    if (discount.amount > 0) {
      discounts.push(discount);
      finalPrice -= discount.amount;
      rulesApplied.push(rule.id);
    }
  }
  
  // Step 4: Apply rounding and validation
  finalPrice = Math.max(0, Math.round(finalPrice));
  
  // Step 5: Currency conversion if needed
  if (request.currencyCode && request.currencyCode !== product.currencyCode) {
    finalPrice = await convertCurrency(finalPrice, product.currencyCode, request.currencyCode);
    basePrice = await convertCurrency(basePrice, product.currencyCode, request.currencyCode);
    listPrice = await convertCurrency(listPrice, product.currencyCode, request.currencyCode);
  }
  
  return {
    basePrice,
    listPrice,
    finalPrice,
    currency: request.currencyCode || product.currencyCode,
    discounts,
    priceListUsed,
    rulesApplied
  };
}
```

### 6.2 Price List Item Resolution

```typescript
async function findPriceListItem(
  priceList: PriceList,
  productId: string,
  variantId: string | undefined,
  categoryId: string | undefined,
  quantity: number,
  date: Date
): Promise<PriceListItem | null> {
  
  // Query items in priority order:
  // 1. Variant-specific
  // 2. Product-specific  
  // 3. Category-specific
  // 4. All products
  
  const items = await db.query(`
    SELECT * FROM price_list_items
    WHERE tenant_id = $1
      AND price_list_id = $2
      AND min_quantity <= $3
      AND (max_quantity IS NULL OR max_quantity >= $3)
      AND (valid_from IS NULL OR valid_from <= $4)
      AND (valid_to IS NULL OR valid_to >= $4)
    ORDER BY
      CASE apply_to
        WHEN 'variant' THEN 1
        WHEN 'product' THEN 2
        WHEN 'category' THEN 3
        WHEN 'all' THEN 4
      END,
      min_quantity DESC  -- Prefer higher quantity tier
  `, [tenantId, priceList.id, quantity, date]);
  
  for (const item of items) {
    if (item.applyTo === 'variant' && item.variantId === variantId) {
      return item;
    }
    if (item.applyTo === 'product' && item.productId === productId) {
      return item;
    }
    if (item.applyTo === 'category' && item.categoryId === categoryId) {
      return item;
    }
    if (item.applyTo === 'all') {
      return item;
    }
  }
  
  return null;
}
```

### 6.3 Compute Price from Item

```typescript
function computePriceFromItem(
  basePrice: number,
  item: PriceListItem,
  costPrice?: number
): number {
  switch (item.computeMethod) {
    case 'fixed':
      return item.fixedPrice!;
      
    case 'percentage':
      // Positive percentage = markup, negative = discount
      return Math.round(basePrice * (1 + item.percentage! / 100));
      
    case 'margin':
      // Calculate price to achieve target margin
      // margin = (price - cost) / price
      // price = cost / (1 - margin)
      if (!costPrice) throw new Error('Cost price required for margin calculation');
      return Math.round(costPrice / (1 - item.marginPercentage! / 100));
      
    case 'formula':
      // Evaluate formula with context
      return evaluateFormula(item.formula!, { basePrice, costPrice });
      
    default:
      return basePrice;
  }
}
```

---

## 7. API Design

### 7.1 Price Calculation Endpoint

```http
POST /api/v1/pricing/calculate
Content-Type: application/json

{
  "items": [
    {
      "productId": "uuid",
      "variantId": "uuid",  // optional
      "quantity": 10
    }
  ],
  "customerId": "uuid",  // optional
  "currencyCode": "VND",  // optional
  "date": "2026-01-24T00:00:00Z"  // optional, for future pricing
}
```

**Response:**
```json
{
  "items": [
    {
      "productId": "uuid",
      "variantId": "uuid",
      "quantity": 10,
      "basePrice": 1000000,
      "listPrice": 900000,
      "unitPrice": 810000,
      "lineTotal": 8100000,
      "currency": "VND",
      "discounts": [
        {
          "type": "pricelist",
          "name": "Wholesale Price",
          "amount": 100000
        },
        {
          "type": "rule",
          "ruleId": "uuid",
          "name": "Quantity Discount 10%",
          "amount": 90000
        }
      ],
      "priceListUsed": "uuid"
    }
  ],
  "subtotal": 8100000,
  "totalDiscounts": 1900000,
  "currency": "VND"
}
```

### 7.2 Price List CRUD

```http
# List price lists
GET /api/v1/pricing/price-lists

# Create price list
POST /api/v1/pricing/price-lists

# Update price list
PUT /api/v1/pricing/price-lists/:id

# Delete price list
DELETE /api/v1/pricing/price-lists/:id

# Get price list items
GET /api/v1/pricing/price-lists/:id/items

# Add/update price list item
POST /api/v1/pricing/price-lists/:id/items
PUT /api/v1/pricing/price-lists/:id/items/:itemId
```

### 7.3 Pricing Rules CRUD

```http
# List pricing rules
GET /api/v1/pricing/rules

# Create rule
POST /api/v1/pricing/rules

# Update rule
PUT /api/v1/pricing/rules/:id

# Delete rule
DELETE /api/v1/pricing/rules/:id

# Test rule (preview what it would apply to)
POST /api/v1/pricing/rules/:id/test
```

---

## 8. Implementation Roadmap

### Phase 1: Core Pricing (MVP) - 2-3 weeks

**Goal:** Basic price list functionality

| Task | Priority | Effort |
|------|----------|--------|
| Create migration for price_lists table | P0 | 1 day |
| Create migration for price_list_items table | P0 | 1 day |
| Create migration for customer_price_lists table | P0 | 0.5 day |
| Backend: Price list CRUD API | P0 | 3 days |
| Backend: Price calculation service | P0 | 3 days |
| Frontend: Price list management UI | P0 | 3 days |
| Frontend: Product price display with list prices | P0 | 2 days |
| Testing and bug fixes | P0 | 2 days |

**Deliverables:**
- Multiple price lists per tenant
- Quantity-based pricing
- Customer-specific price lists
- Price calculation API

### Phase 2: Pricing Rules - 2 weeks

**Goal:** Discount and promotion engine

| Task | Priority | Effort |
|------|----------|--------|
| Create migration for pricing_rules table | P0 | 1 day |
| Create migration for pricing_rule_usage table | P0 | 0.5 day |
| Backend: Pricing rules CRUD API | P0 | 2 days |
| Backend: Rule evaluation engine | P0 | 3 days |
| Backend: Usage tracking | P1 | 1 day |
| Frontend: Pricing rules management UI | P0 | 3 days |
| Frontend: Rule condition builder | P1 | 2 days |
| Testing and bug fixes | P0 | 2 days |

**Deliverables:**
- Percentage and fixed discounts
- Quantity-based rules
- Time-limited promotions
- Rule priority and combination

### Phase 3: Advanced Features - 2 weeks

**Goal:** Enterprise pricing features

| Task | Priority | Effort |
|------|----------|--------|
| Formula-based pricing | P1 | 2 days |
| Margin-based pricing | P1 | 1 day |
| Buy X Get Y promotions | P1 | 2 days |
| Price history tracking | P1 | 1 day |
| Multi-currency support | P2 | 2 days |
| Price import/export | P2 | 2 days |
| Analytics dashboard | P2 | 3 days |

**Deliverables:**
- Complex pricing formulas
- Promotional campaigns
- Price change audit trail
- Currency conversion

---

## 9. Comparison with Enterprise ERPs

| Feature | Odoo | SAP | ERPNext | Anthill (Proposed) |
|---------|------|-----|---------|-------------------|
| **Price Lists** | | | | |
| Multiple price lists | ✅ | ✅ | ✅ | ✅ |
| Price list hierarchy | ✅ | ✅ | ❌ | ✅ |
| Quantity breaks | ✅ | ✅ | ✅ | ✅ |
| Date validity | ✅ | ✅ | ✅ | ✅ |
| **Customer Pricing** | | | | |
| Customer-specific | ✅ | ✅ | ✅ | ✅ |
| Customer groups | ✅ | ✅ | ✅ | ✅ Planned |
| Contract pricing | ✅ | ✅ | ✅ | ❌ Future |
| **Computation** | | | | |
| Fixed price | ✅ | ✅ | ✅ | ✅ |
| Percentage markup/discount | ✅ | ✅ | ✅ | ✅ |
| Formula-based | ✅ | ✅ | ❌ | ✅ |
| Margin-based | ✅ | ✅ | ❌ | ✅ |
| **Discounts** | | | | |
| Percentage discount | ✅ | ✅ | ✅ | ✅ |
| Fixed amount discount | ✅ | ✅ | ✅ | ✅ |
| Conditional rules | ✅ | ✅ | ✅ | ✅ |
| Buy X Get Y | ✅ | ✅ | ✅ | ✅ |
| Combinable discounts | ✅ | ✅ | ❌ | ✅ |
| Usage limits | ✅ | ✅ | ✅ | ✅ |
| **Multi-currency** | ✅ | ✅ | ✅ | ✅ |
| **Inventory Costing** | | | | |
| FIFO | ✅ | ✅ | ✅ | ✅ Already |
| Moving Average | ✅ | ✅ | ✅ | ✅ Already |
| Standard Cost | ✅ | ✅ | ❌ | ✅ Already |

---

## 10. References

- [Odoo Pricelist Documentation](https://www.odoo.com/documentation/17.0/applications/sales/sales/products_prices/prices/pricing.html)
- [ERPNext Pricing Rule](https://docs.erpnext.com/docs/user/manual/en/pricing-rule)
- [ERPNext Item Valuation](https://docs.erpnext.com/docs/v14/user/manual/en/stock/articles/item-valuation-fifo-and-moving-average)
- SAP S/4HANA Product Costing and Condition Records Documentation

---

## Appendix A: Example Configurations

### A.1 Wholesale Price List

```json
{
  "name": "Wholesale Pricing",
  "code": "WHOLESALE",
  "currency_code": "VND",
  "based_on": "base_price",
  "default_percentage": -15.0,
  "items": [
    {
      "apply_to": "all",
      "min_quantity": 1,
      "compute_method": "percentage",
      "percentage": -15.0
    },
    {
      "apply_to": "all",
      "min_quantity": 100,
      "compute_method": "percentage",
      "percentage": -20.0
    },
    {
      "apply_to": "all",
      "min_quantity": 500,
      "compute_method": "percentage",
      "percentage": -25.0
    }
  ]
}
```

### A.2 VIP Customer Rule

```json
{
  "name": "VIP Extra 5% Off",
  "code": "VIP_EXTRA",
  "rule_type": "discount_percentage",
  "discount_percentage": 5.0,
  "conditions": {
    "customer_groups": ["vip"]
  },
  "is_combinable": true,
  "priority": 50
}
```

### A.3 Seasonal Promotion

```json
{
  "name": "Lunar New Year 2026",
  "code": "TET2026",
  "rule_type": "discount_percentage",
  "discount_percentage": 10.0,
  "conditions": {
    "min_order_amount": 500000
  },
  "valid_from": "2026-01-25T00:00:00Z",
  "valid_to": "2026-02-10T23:59:59Z",
  "max_discount_amount": 500000,
  "is_combinable": false,
  "priority": 10
}
```
