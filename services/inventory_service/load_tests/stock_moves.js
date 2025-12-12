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
    return `k6-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
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

    const success = check(response, {
        'reservation status 200 or 201': (r) => r.status === 200 || r.status === 201,
        'reservation has id': (r) => {
            try {
                const body = JSON.parse(r.body);
                return body.reservation_id !== undefined;
            } catch {
                return false;
            }
        },
    });

    stockMoveSuccessRate.add(success);

    // Track conflicts (409)
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
        quantity_change: Math.floor(Math.random() * 20) - 10, // -10 to +10
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
            try {
                const body = JSON.parse(r.body);
                return body.available_quantity !== undefined;
            } catch {
                return false;
            }
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
