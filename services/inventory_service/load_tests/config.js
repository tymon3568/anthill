// ============================================================================
// k6 Load Test Configuration
// ============================================================================
// Shared configuration for all inventory service load tests
//
// Usage:
//   k6 run --env BASE_URL=http://localhost:8082 product_search.js
//
// Prerequisites:
//   - k6 installed: https://k6.io/docs/get-started/installation/
//   - Inventory service running at BASE_URL
//   - Test tenant and auth token available

// Base URL for inventory service API
export const BASE_URL = __ENV.BASE_URL || 'http://localhost:8082';

// Authentication token (JWT) for API requests
export const AUTH_TOKEN = __ENV.AUTH_TOKEN || 'test-token';

// Test tenant ID
export const TENANT_ID = __ENV.TENANT_ID || '550e8400-e29b-41d4-a716-446655440001';

// Default headers for all requests
export const defaultHeaders = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${AUTH_TOKEN}`,
    'X-Tenant-ID': TENANT_ID,
};

// ============================================================================
// Performance Thresholds
// ============================================================================

export const thresholds = {
    productSearch: {
        // Product search should complete within 200ms for 95% of requests
        http_req_duration: ['p(95)<200', 'p(99)<500'],
        // Error rate should be below 1%
        http_req_failed: ['rate<0.01'],
    },
    stockMoves: {
        // Stock moves can be slower due to transaction overhead
        http_req_duration: ['p(95)<500', 'p(99)<1000'],
        // Critical: error rate should be below 0.5%
        http_req_failed: ['rate<0.005'],
    },
    mixedWorkload: {
        http_req_duration: ['p(95)<300', 'p(99)<600'],
        http_req_failed: ['rate<0.01'],
    },
};

// ============================================================================
// Load Stages Configuration
// ============================================================================

export const stages = {
    // Quick smoke test
    smoke: [
        { duration: '10s', target: 5 },
        { duration: '30s', target: 5 },
        { duration: '10s', target: 0 },
    ],
    // Standard load test
    load: [
        { duration: '30s', target: 50 },   // Ramp up
        { duration: '2m', target: 100 },   // Sustained load
        { duration: '30s', target: 200 },  // Peak load
        { duration: '1m', target: 200 },   // Sustained peak
        { duration: '30s', target: 0 },    // Ramp down
    ],
    // Stress test
    stress: [
        { duration: '1m', target: 100 },
        { duration: '2m', target: 300 },
        { duration: '2m', target: 500 },
        { duration: '1m', target: 0 },
    ],
    // Spike test
    spike: [
        { duration: '10s', target: 10 },
        { duration: '1s', target: 500 },   // Spike!
        { duration: '30s', target: 500 },
        { duration: '1s', target: 10 },
        { duration: '10s', target: 0 },
    ],
};

// Select stage based on environment variable
export function getStages() {
    const stageType = __ENV.STAGE_TYPE || 'smoke';
    return stages[stageType] || stages.smoke;
}

// ============================================================================
// Test Data Generation
// ============================================================================

// Generate random product search query
export function randomSearchQuery() {
    const terms = [
        'laptop', 'phone', 'tablet', 'keyboard', 'mouse',
        'monitor', 'camera', 'headphones', 'speaker', 'cable',
        'adapter', 'charger', 'case', 'stand', 'dock',
    ];
    return terms[Math.floor(Math.random() * terms.length)];
}

// Generate random warehouse ID from test set
export function randomWarehouseId() {
    const warehouses = [
        '550e8400-e29b-41d4-a716-446655440101',
        '550e8400-e29b-41d4-a716-446655440102',
        '550e8400-e29b-41d4-a716-446655440103',
    ];
    return warehouses[Math.floor(Math.random() * warehouses.length)];
}

// Generate random product ID from test set
export function randomProductId() {
    const products = [
        '550e8400-e29b-41d4-a716-446655440201',
        '550e8400-e29b-41d4-a716-446655440202',
        '550e8400-e29b-41d4-a716-446655440203',
        '550e8400-e29b-41d4-a716-446655440204',
        '550e8400-e29b-41d4-a716-446655440205',
    ];
    return products[Math.floor(Math.random() * products.length)];
}

// Generate random stock move request with guaranteed different warehouses
export function randomStockMoveRequest() {
    const fromWarehouse = randomWarehouseId();
    let toWarehouse = randomWarehouseId();
    // Ensure from and to warehouses are different
    while (toWarehouse === fromWarehouse) {
        toWarehouse = randomWarehouseId();
    }
    return {
        from_warehouse_id: fromWarehouse,
        to_warehouse_id: toWarehouse,
        product_id: randomProductId(),
        quantity: Math.floor(Math.random() * 100) + 1,
        reason: 'load_test',
    };
}
