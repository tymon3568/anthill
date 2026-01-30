# Task 08.11.03: Price Calculation & Preview

## Status: Pending

## Objective

Implement price calculation service and UI components for real-time price preview across the application.

## Database Reference

Uses all pricing tables from `docs/database-erd.dbml`:
- `price_lists`, `price_list_items`
- `pricing_rules`, `pricing_rule_usage`
- `products`, `product_variants`

## User Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                   PRICE CALCULATION FLOW                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  INPUT                                                           │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Product ID: prod-1                                        │   │
│  │ Variant ID: var-1 (optional)                              │   │
│  │ Customer ID: cust-1 (optional)                            │   │
│  │ Quantity: 10                                              │   │
│  │ Date: 2026-01-24 (optional, default: now)                 │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              │                                   │
│                              ▼                                   │
│  STEP 1: BASE PRICE                                              │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ product.sale_price + variant.price_difference             │   │
│  │ = 25,000,000 + 3,000,000 = 28,000,000₫                   │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              │                                   │
│                              ▼                                   │
│  STEP 2: FIND PRICE LIST                                         │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ 1. Customer-specific: customer_price_lists               │   │
│  │ 2. Customer group: (future)                               │   │
│  │ 3. Default price list                                     │   │
│  │                                                           │   │
│  │ Found: "Wholesale" (-15% for qty >= 10)                   │   │
│  │ List Price: 28,000,000 × 0.85 = 23,800,000₫              │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              │                                   │
│                              ▼                                   │
│  STEP 3: APPLY PRICING RULES                                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Active rules matching conditions:                         │   │
│  │                                                           │   │
│  │ ✓ "VIP Extra 5%" - combinable, priority 50               │   │
│  │   Discount: 23,800,000 × 0.05 = 1,190,000₫               │   │
│  │                                                           │   │
│  │ After rules: 23,800,000 - 1,190,000 = 22,610,000₫        │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              │                                   │
│                              ▼                                   │
│  OUTPUT                                                          │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ {                                                         │   │
│  │   basePrice: 28,000,000,                                  │   │
│  │   listPrice: 23,800,000,                                  │   │
│  │   finalPrice: 22,610,000,                                 │   │
│  │   unitPrice: 22,610,000,                                  │   │
│  │   lineTotal: 226,100,000,                                 │   │
│  │   discounts: [                                            │   │
│  │     { type: 'pricelist', name: 'Wholesale', amount: 4.2M }│   │
│  │     { type: 'rule', name: 'VIP 5%', amount: 1.19M }       │   │
│  │   ],                                                      │   │
│  │   totalSavings: 5,390,000,                                │   │
│  │   currency: 'VND'                                         │   │
│  │ }                                                         │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## UI Specifications

### 1. Price Calculator Page (`/inventory/pricing/calculator`)

```
┌─────────────────────────────────────────────────────────────────┐
│  Price Calculator                                                │
│  Preview pricing for any product and customer combination        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─ Product Selection ─────────────────────────────────────────┐│
│  │                                                              ││
│  │  Product *                                                   ││
│  │  [🔍 Search products...                             ▼]       ││
│  │                                                              ││
│  │  Selected: Laptop Pro 15" (LAPTOP-001)                       ││
│  │  Base Price: 25,000,000₫ │ Cost: 20,000,000₫                 ││
│  │                                                              ││
│  │  Variant                                                     ││
│  │  [Black / 512GB                                    ▼]        ││
│  │  Price Difference: +3,000,000₫                               ││
│  │                                                              ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                  │
│  ┌─ Customer & Quantity ───────────────────────────────────────┐│
│  │                                                              ││
│  │  Customer (optional)                                         ││
│  │  [🔍 Search customers...                            ▼]       ││
│  │                                                              ││
│  │  Selected: ABC Corporation                                   ││
│  │  Price Lists: Wholesale, VIP                                 ││
│  │                                                              ││
│  │  Quantity                   Date                             ││
│  │  [10          ]             [📅 2026-01-24        ]          ││
│  │                                                              ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                  │
│  ┌─ Price Breakdown ───────────────────────────────────────────┐│
│  │                                                              ││
│  │  ┌──────────────────────────────────────────────────────┐   ││
│  │  │                    PRICE CALCULATION                  │   ││
│  │  ├──────────────────────────────────────────────────────┤   ││
│  │  │                                                       │   ││
│  │  │  Base Price                           28,000,000₫    │   ││
│  │  │  └── Product: 25,000,000₫                            │   ││
│  │  │  └── Variant: +3,000,000₫                            │   ││
│  │  │                                                       │   ││
│  │  │  ─────────────────────────────────────────────────── │   ││
│  │  │                                                       │   ││
│  │  │  Price List: Wholesale                                │   ││
│  │  │  └── -15% (qty ≥ 10)              -4,200,000₫        │   ││
│  │  │                                   ────────────        │   ││
│  │  │  List Price                           23,800,000₫    │   ││
│  │  │                                                       │   ││
│  │  │  ─────────────────────────────────────────────────── │   ││
│  │  │                                                       │   ││
│  │  │  Pricing Rules Applied:                               │   ││
│  │  │  └── VIP Extra 5%                 -1,190,000₫        │   ││
│  │  │                                   ────────────        │   ││
│  │  │                                                       │   ││
│  │  │  ═══════════════════════════════════════════════════ │   ││
│  │  │  UNIT PRICE                           22,610,000₫    │   ││
│  │  │  × Quantity                                    10    │   ││
│  │  │  ═══════════════════════════════════════════════════ │   ││
│  │  │  LINE TOTAL                          226,100,000₫    │   ││
│  │  │                                                       │   ││
│  │  │  ─────────────────────────────────────────────────── │   ││
│  │  │  💰 Total Savings: 53,900,000₫ (19.25%)              │   ││
│  │  │  📊 Margin: 26,100,000₫ (11.5%)                      │   ││
│  │  │                                                       │   ││
│  │  └──────────────────────────────────────────────────────┘   ││
│  │                                                              ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2. Inline Price Preview Component

Used in Product Detail, Order Forms, etc.

```
┌─────────────────────────────────────────────────────────────────┐
│  Pricing                                            [Calculator] │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ Quantity   │    1    │   10    │   50    │   100   │       │ │
│  ├────────────┼─────────┼─────────┼─────────┼─────────┤       │ │
│  │ Unit Price │ 25.0M₫  │ 21.25M₫ │ 20.0M₫  │ 18.75M₫ │       │ │
│  │ Discount   │    -    │  -15%   │  -20%   │  -25%   │       │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  Active Promotions:                                              │
│  🏷️ Tết 2026: Extra 10% off orders > 500K (ends 10 Feb)         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3. Quick Price Check Widget

Floating widget accessible from any page:

```
┌───────────────────────────────────────┐
│  Quick Price Check              [×]   │
├───────────────────────────────────────┤
│                                       │
│  [🔍 Product...]     Qty: [1   ]      │
│                                       │
│  Laptop Pro 15"                       │
│  ──────────────────────────────────   │
│  Base:     25,000,000₫                │
│  Final:    22,500,000₫  (-10%)        │
│  ──────────────────────────────────   │
│  Save:      2,500,000₫                │
│                                       │
│  [Open Full Calculator]               │
│                                       │
└───────────────────────────────────────┘
```

## Price Calculation Service

```typescript
// src/lib/services/pricing.service.ts

export interface PriceRequest {
  productId: string;
  variantId?: string;
  customerId?: string;
  quantity: number;
  date?: Date;
  currencyCode?: string;
}

export interface PriceResult {
  // Prices
  basePrice: number;
  listPrice: number;
  finalPrice: number;
  unitPrice: number;
  lineTotal: number;
  
  // Breakdown
  priceListUsed?: {
    id: string;
    name: string;
    code: string;
  };
  
  discounts: Array<{
    type: 'pricelist' | 'rule';
    id: string;
    name: string;
    description?: string;
    percentage?: number;
    amount: number;
  }>;
  
  // Totals
  totalDiscount: number;
  totalSavings: number;
  savingsPercentage: number;
  
  // Margin (if cost price available)
  marginAmount?: number;
  marginPercentage?: number;
  
  // Meta
  currency: string;
  calculatedAt: Date;
}

export interface BulkPriceRequest {
  items: Array<{
    productId: string;
    variantId?: string;
    quantity: number;
  }>;
  customerId?: string;
  date?: Date;
}

export interface BulkPriceResult {
  items: PriceResult[];
  subtotal: number;
  totalDiscount: number;
  grandTotal: number;
  currency: string;
}

export class PricingService {
  /**
   * Calculate price for a single product
   */
  async calculatePrice(request: PriceRequest): Promise<PriceResult>;
  
  /**
   * Calculate prices for multiple products (cart/order)
   */
  async calculateBulkPrice(request: BulkPriceRequest): Promise<BulkPriceResult>;
  
  /**
   * Get quantity break prices for display
   */
  async getQuantityBreaks(
    productId: string,
    variantId?: string,
    customerId?: string
  ): Promise<Array<{ minQty: number; maxQty?: number; unitPrice: number; discount?: number }>>;
  
  /**
   * Get active promotions for a product
   */
  async getActivePromotions(
    productId: string,
    categoryId?: string
  ): Promise<Array<{ id: string; name: string; description: string; validTo?: Date }>>;
}
```

## Component Structure

```
src/lib/components/pricing/
├── calculator/
│   ├── PriceCalculator.svelte        # Full calculator page
│   ├── PriceBreakdown.svelte         # Detailed price breakdown
│   ├── QuantityBreaksTable.svelte    # Quantity tier display
│   ├── QuickPriceCheck.svelte        # Floating widget
│   └── InlinePricePreview.svelte     # Embedded preview
├── shared/
│   ├── PriceDisplay.svelte           # Formatted price display
│   ├── DiscountBadge.svelte          # Discount percentage badge
│   ├── SavingsBadge.svelte           # Savings amount badge
│   └── PromotionBanner.svelte        # Active promotion display
```

## Integration Points

### 1. Product Detail Page
```svelte
<!-- In product detail page -->
<InlinePricePreview 
  productId={product.productId}
  variantId={selectedVariant?.variantId}
/>
```

### 2. Order/Cart Forms
```svelte
<!-- In order line item -->
<script>
  const priceResult = await pricingService.calculatePrice({
    productId: item.productId,
    variantId: item.variantId,
    customerId: order.customerId,
    quantity: item.quantity
  });
</script>

<PriceBreakdown result={priceResult} compact />
```

### 3. Command Palette
```typescript
// Quick price check from Ctrl+K
{
  id: 'price-check',
  name: 'Quick Price Check',
  shortcut: 'Ctrl+Shift+P',
  action: () => openQuickPriceCheck()
}
```

## Mock Data

```typescript
// src/lib/api/pricing.mock.ts

export const mockPriceLists: PriceList[] = [
  {
    priceListId: 'pl-1',
    name: 'Retail Price',
    code: 'RETAIL',
    currencyCode: 'VND',
    priceListType: 'sale',
    basedOn: 'fixed',
    priority: 100,
    isDefault: true,
    isActive: true,
    itemCount: 150
  },
  {
    priceListId: 'pl-2',
    name: 'Wholesale',
    code: 'WHOLESALE',
    currencyCode: 'VND',
    priceListType: 'sale',
    basedOn: 'base_price',
    defaultPercentage: -15,
    priority: 50,
    isDefault: false,
    isActive: true,
    itemCount: 150
  },
  {
    priceListId: 'pl-3',
    name: 'VIP Members',
    code: 'VIP',
    currencyCode: 'VND',
    priceListType: 'sale',
    basedOn: 'base_price',
    defaultPercentage: -20,
    priority: 40,
    isDefault: false,
    isActive: true,
    itemCount: 45
  }
];

export const mockPricingRules: PricingRule[] = [
  {
    ruleId: 'rule-1',
    name: 'VIP Extra 5% Off',
    code: 'VIP_EXTRA',
    ruleType: 'discount_percentage',
    discountPercentage: 5,
    conditions: { customerGroups: ['vip'] },
    priority: 50,
    isCombinable: true,
    isActive: true,
    usageCount: 45
  },
  {
    ruleId: 'rule-2',
    name: 'Tết 2026 - 10% Off',
    code: 'TET2026',
    ruleType: 'discount_percentage',
    discountPercentage: 10,
    maxDiscountAmount: 500000,
    conditions: { minOrderAmount: 500000 },
    validFrom: new Date('2026-01-25'),
    validTo: new Date('2026-02-10'),
    priority: 10,
    isCombinable: false,
    isActive: true,
    usageCount: 128
  }
];
```

## Acceptance Criteria

1. [ ] Price Calculator page with full breakdown
2. [ ] Real-time calculation as inputs change
3. [ ] Quantity breaks display
4. [ ] Active promotions display
5. [ ] Inline price preview component for product pages
6. [ ] Quick price check floating widget
7. [ ] Bulk price calculation for carts/orders
8. [ ] Margin calculation when cost price available
9. [ ] Currency formatting (VND, USD)
10. [ ] Mobile responsive design

## Performance Considerations

- Cache price list data (5 minute TTL)
- Debounce quantity input (300ms)
- Batch API calls for bulk calculations
- Lazy load pricing components

## Dependencies

- Task 08.11.01 (Price Lists)
- Task 08.11.02 (Pricing Rules)
- Task 08.04 (Product Management)
