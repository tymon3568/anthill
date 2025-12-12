// ============================================================================
// Mixed Workload Load Test
// ============================================================================
// Simulates realistic traffic patterns with 70% reads, 30% writes
//
// Run:
//   k6 run --env BASE_URL=http://localhost:8082 mixed_workload.js

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend } from 'k6/metrics';
import {
    BASE_URL,
    defaultHeaders,
    getStages,
    thresholds,
    randomSearchQuery,
    randomWarehouseId,
    randomProductId,
} from './config.js';

// Custom metrics
const overallSuccessRate = new Rate('overall_success_rate');
const readDuration = new Trend('read_duration_ms');
const writeDuration = new Trend('write_duration_ms');

// Test configuration
export const options = {
    stages: getStages(),
    thresholds: thresholds.mixedWorkload,
};

// Generate unique idempotency key
function generateIdempotencyKey() {
    return `k6-mixed-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`;
}

// ============================================================================
// Read Operations (70% of traffic)
// ============================================================================

function productSearch() {
    const query = randomSearchQuery();
    const url = `${BASE_URL}/api/v1/products/search`;
    const payload = JSON.stringify({
        query: query,
        page: 1,
        page_size: 20,
    });

    const startTime = Date.now();
    const response = http.post(url, payload, { headers: defaultHeaders });
    readDuration.add(Date.now() - startTime);

    const success = check(response, {
        'search: status 200': (r) => r.status === 200,
    });
    overallSuccessRate.add(success);
    return success;
}

function getProduct() {
    const productId = randomProductId();
    const url = `${BASE_URL}/api/v1/products/${productId}`;

    const startTime = Date.now();
    const response = http.get(url, { headers: defaultHeaders });
    readDuration.add(Date.now() - startTime);

    const success = check(response, {
        'get product: status 200 or 404': (r) => r.status === 200 || r.status === 404,
    });
    overallSuccessRate.add(success);
    return success;
}

function getStockLevel() {
    const warehouseId = randomWarehouseId();
    const productId = randomProductId();
    const url = `${BASE_URL}/api/v1/stock/levels/${warehouseId}/${productId}`;

    const startTime = Date.now();
    const response = http.get(url, { headers: defaultHeaders });
    readDuration.add(Date.now() - startTime);

    const success = check(response, {
        'stock level: status 200': (r) => r.status === 200,
    });
    overallSuccessRate.add(success);
    return success;
}

function getCategories() {
    const url = `${BASE_URL}/api/v1/categories/tree`;

    const startTime = Date.now();
    const response = http.get(url, { headers: defaultHeaders });
    readDuration.add(Date.now() - startTime);

    const success = check(response, {
        'categories: status 200': (r) => r.status === 200,
    });
    overallSuccessRate.add(success);
    return success;
}

// ============================================================================
// Write Operations (30% of traffic)
// ============================================================================

function reserveStock() {
    const url = `${BASE_URL}/api/v1/stock/reserve`;
    const payload = JSON.stringify({
        warehouse_id: randomWarehouseId(),
        product_id: randomProductId(),
        quantity: Math.floor(Math.random() * 5) + 1,
        reference_type: 'mixed_load_test',
        reference_id: generateIdempotencyKey(),
        expiry_minutes: 5,
    });

    const startTime = Date.now();
    const response = http.post(url, payload, {
        headers: {
            ...defaultHeaders,
            'X-Idempotency-Key': generateIdempotencyKey(),
        },
    });
    writeDuration.add(Date.now() - startTime);

    const success = check(response, {
        'reserve: status 200/201/409': (r) => [200, 201, 409].includes(r.status),
    });
    overallSuccessRate.add(success);
    return success;
}

function adjustStock() {
    const url = `${BASE_URL}/api/v1/stock/adjust`;
    const payload = JSON.stringify({
        warehouse_id: randomWarehouseId(),
        product_id: randomProductId(),
        quantity_change: Math.floor(Math.random() * 10) - 5,
        reason: 'mixed_load_test',
    });

    const startTime = Date.now();
    const response = http.post(url, payload, {
        headers: {
            ...defaultHeaders,
            'X-Idempotency-Key': generateIdempotencyKey(),
        },
    });
    writeDuration.add(Date.now() - startTime);

    const success = check(response, {
        'adjust: status 200': (r) => r.status === 200,
    });
    overallSuccessRate.add(success);
    return success;
}

// ============================================================================
// Main Test Function
// ============================================================================

export default function () {
    // 70% reads, 30% writes
    const rand = Math.random() * 100;

    if (rand < 70) {
        // Read operations
        group('read_operations', () => {
            const readRand = Math.random() * 100;
            if (readRand < 40) {
                productSearch();
            } else if (readRand < 60) {
                getProduct();
            } else if (readRand < 80) {
                getStockLevel();
            } else {
                getCategories();
            }
        });
    } else {
        // Write operations
        group('write_operations', () => {
            if (Math.random() < 0.6) {
                reserveStock();
            } else {
                adjustStock();
            }
        });
    }

    // Simulate user think time
    sleep(Math.random() * 0.5 + 0.2);
}

// Setup function
export function setup() {
    console.log(`Starting mixed workload test against: ${BASE_URL}`);
    console.log('Traffic distribution: 70% reads, 30% writes');
    return { startTime: Date.now() };
}

// Teardown function
export function teardown(data) {
    const duration = (Date.now() - data.startTime) / 1000;
    console.log(`Mixed workload test completed in ${duration.toFixed(2)} seconds`);
}
