// ============================================================================
// Stock Moves Load Test
// ============================================================================
// Tests stock movement endpoints under load
//
// Run:
//   k6 run --env BASE_URL=http://localhost:8082 stock_moves.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import {
    BASE_URL,
    defaultHeaders,
    getStages,
    thresholds,
    randomWarehouseId,
    randomProductId,
} from './config.js';

// Custom metrics
const stockMoveSuccessRate = new Rate('stock_move_success_rate');
const stockMoveDuration = new Trend('stock_move_duration_ms');
const stockMoveConflicts = new Counter('stock_move_conflicts');

// Test configuration
export const options = {
    stages: getStages(),
    thresholds: thresholds.stockMoves,
};

// Generate unique idempotency key
function generateIdempotencyKey() {
    return `k6-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`;
}

// Stock reservation test
function testStockReservation() {
    const url = `${BASE_URL}/api/v1/stock/reserve`;
    const payload = JSON.stringify({
        warehouse_id: randomWarehouseId(),
        product_id: randomProductId(),
        quantity: Math.floor(Math.random() * 10) + 1,
        reference_type: 'load_test',
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
    const duration = Date.now() - startTime;

    stockMoveDuration.add(duration);

    // Parse response body once
    const body = (response.status === 200 || response.status === 201) ? response.json() : null;

    const success = check(response, {
        'reservation status 200, 201, or 409': (r) => r.status === 200 || r.status === 201 || r.status === 409,
        'reservation has id (for 200/201 only)': () => {
            // 409 conflicts don't need reservation_id - they're expected under contention
            if (response.status === 409) return true;
            return body && body.reservation_id !== undefined;
        },
    });

    stockMoveSuccessRate.add(success);

    // Track conflicts (409) separately for analysis
    if (response.status === 409) {
        stockMoveConflicts.add(1);
    }

    return success;
}

// Stock adjustment test
function testStockAdjustment() {
    const url = `${BASE_URL}/api/v1/stock/adjust`;
    const payload = JSON.stringify({
        warehouse_id: randomWarehouseId(),
        product_id: randomProductId(),
        quantity_change: Math.floor(Math.random() * 21) - 10, // -10 to +10
        reason: 'load_test_adjustment',
        notes: 'Automated load test',
    });

    const startTime = Date.now();
    const response = http.post(url, payload, {
        headers: {
            ...defaultHeaders,
            'X-Idempotency-Key': generateIdempotencyKey(),
        },
    });
    const duration = Date.now() - startTime;

    stockMoveDuration.add(duration);

    const success = check(response, {
        'adjustment status 200': (r) => r.status === 200,
    });

    stockMoveSuccessRate.add(success);
    return success;
}

// Stock level query test
function testStockLevel() {
    const warehouseId = randomWarehouseId();
    const productId = randomProductId();
    const url = `${BASE_URL}/api/v1/stock/levels/${warehouseId}/${productId}`;

    const startTime = Date.now();
    const response = http.get(url, {
        headers: defaultHeaders,
    });
    const duration = Date.now() - startTime;

    stockMoveDuration.add(duration);

    const success = check(response, {
        'level query status 200': (r) => r.status === 200,
        'level has quantity': (r) => {
            const body = r.json();
            return body && body.available_quantity !== undefined;
        },
    });

    stockMoveSuccessRate.add(success);
    return success;
}

// Main test function - randomly choose operation
export default function () {
    const operations = [
        { weight: 50, fn: testStockLevel },       // 50% reads
        { weight: 30, fn: testStockReservation }, // 30% reservations
        { weight: 20, fn: testStockAdjustment },  // 20% adjustments
    ];

    const rand = Math.random() * 100;
    let cumulative = 0;

    for (const op of operations) {
        cumulative += op.weight;
        if (rand < cumulative) {
            op.fn();
            break;
        }
    }

    // Small delay between requests
    sleep(Math.random() * 0.3 + 0.1);
}

// Setup function
export function setup() {
    console.log(`Starting stock moves load test against: ${BASE_URL}`);
    return { startTime: Date.now() };
}

// Teardown function
export function teardown(data) {
    const duration = (Date.now() - data.startTime) / 1000;
    console.log(`Test completed in ${duration.toFixed(2)} seconds`);
}
