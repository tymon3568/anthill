// ============================================================================
// Product Search Load Test
// ============================================================================
// Tests the product search endpoint under load
//
// Run:
//   k6 run --env BASE_URL=http://localhost:8082 product_search.js
//
// With custom stage:
//   k6 run --env STAGE_TYPE=load product_search.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';
import {
    BASE_URL,
    defaultHeaders,
    getStages,
    thresholds,
    randomSearchQuery,
} from './config.js';

// Custom metrics
const searchSuccessRate = new Rate('search_success_rate');
const searchDuration = new Trend('search_duration_ms');
const resultsReturned = new Trend('search_results_count');

// Test configuration
export const options = {
    stages: getStages(),
    thresholds: thresholds.productSearch,
};

// Main test function
export default function () {
    const query = randomSearchQuery();
    const url = `${BASE_URL}/api/v1/products/search`;

    const payload = JSON.stringify({
        query: query,
        page: 1,
        page_size: 20,
        filters: {
            is_active: true,
        },
    });

    const response = http.post(url, payload, {
        headers: defaultHeaders,
    });

    // Record metrics - use k6 built-in timing for accuracy
    searchDuration.add(response.timings.duration);

    // Parse response body once
    const body = response.status === 200 ? response.json() : null;

    const success = check(response, {
        'status is 200': (r) => r.status === 200,
        'response has products': () => body && body.products !== undefined,
        'response time < 200ms': (r) => r.timings.duration < 200,
    });

    searchSuccessRate.add(success);

    // Track results count (reuse parsed body)
    if (body && body.products) {
        resultsReturned.add(body.products.length);
    }

    // Small delay between requests (realistic user behavior)
    sleep(Math.random() * 0.5 + 0.1);
}

// Setup function - runs once per VU at start
export function setup() {
    console.log(`Starting product search load test against: ${BASE_URL}`);

    // Verify service is accessible
    const health = http.get(`${BASE_URL}/health`);
    if (health.status !== 200) {
        console.warn(`Warning: Health check failed with status ${health.status}`);
    }

    return { startTime: Date.now() };
}

// Teardown function - runs once at end
export function teardown(data) {
    const duration = (Date.now() - data.startTime) / 1000;
    console.log(`Test completed in ${duration.toFixed(2)} seconds`);
}
