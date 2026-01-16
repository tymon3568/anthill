// API Endpoint Load Test - k6 Script
// Tests various API endpoints under concurrent load
// Usage: k6 run api-load-test.js

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import { SharedArray } from 'k6/data';

// Custom metrics
const apiSuccessRate = new Rate('api_success_rate');
const getUsersDuration = new Trend('get_users_duration');
const getUserByIdDuration = new Trend('get_user_by_id_duration');
const updateProfileDuration = new Trend('update_profile_duration');
const searchDuration = new Trend('search_duration');
const errorCount = new Counter('errors');

// Configuration
const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';
const API_PREFIX = '/api/v1';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || '';
const TENANT_ID = __ENV.TENANT_ID || '00000000-0000-0000-0000-000000000001';

// Test configuration
export const options = {
    scenarios: {
        // Constant load for baseline
        constant_load: {
            executor: 'constant-arrival-rate',
            rate: 50,              // 50 requests per second
            timeUnit: '1s',
            duration: '5m',
            preAllocatedVUs: 20,
            maxVUs: 100,
            tags: { scenario: 'constant' },
        },
        // Spike test
        spike: {
            executor: 'ramping-arrival-rate',
            startRate: 10,
            timeUnit: '1s',
            stages: [
                { duration: '1m', target: 10 },
                { duration: '30s', target: 200 },  // Spike to 200 rps
                { duration: '1m', target: 200 },
                { duration: '30s', target: 10 },   // Back to normal
                { duration: '2m', target: 10 },
            ],
            preAllocatedVUs: 50,
            maxVUs: 300,
            startTime: '6m',
            tags: { scenario: 'spike' },
        },
    },
    thresholds: {
        http_req_duration: ['p(95)<500', 'p(99)<1500'],
        http_req_failed: ['rate<0.05'],
        api_success_rate: ['rate>0.90'],
        get_users_duration: ['p(95)<400'],
        get_user_by_id_duration: ['p(95)<200'],
        update_profile_duration: ['p(95)<300'],
        search_duration: ['p(95)<500'],
    },
};

function getAuthHeaders() {
    return {
        'Authorization': `Bearer ${AUTH_TOKEN}`,
        'Content-Type': 'application/json',
        'X-Tenant-ID': TENANT_ID,
    };
}

// Main test function
export default function () {
    const headers = getAuthHeaders();

    // Randomly choose an endpoint to test
    const endpoints = [
        { name: 'list_users', weight: 30 },
        { name: 'get_profile', weight: 25 },
        { name: 'update_profile', weight: 15 },
        { name: 'list_permissions', weight: 20 },
        { name: 'health_check', weight: 10 },
    ];

    const totalWeight = endpoints.reduce((sum, e) => sum + e.weight, 0);
    let random = Math.random() * totalWeight;
    let selectedEndpoint = endpoints[0].name;

    for (const endpoint of endpoints) {
        random -= endpoint.weight;
        if (random <= 0) {
            selectedEndpoint = endpoint.name;
            break;
        }
    }

    switch (selectedEndpoint) {
        case 'list_users':
            testListUsers(headers);
            break;
        case 'get_profile':
            testGetProfile(headers);
            break;
        case 'update_profile':
            testUpdateProfile(headers);
            break;
        case 'list_permissions':
            testListPermissions(headers);
            break;
        case 'health_check':
            testHealthCheck();
            break;
    }

    sleep(0.1);
}

function testListUsers(headers) {
    group('List Users', () => {
        const startTime = Date.now();
        const res = http.get(
            `${BASE_URL}${API_PREFIX}/admin/users?page=1&page_size=20`,
            { headers }
        );
        const duration = Date.now() - startTime;
        getUsersDuration.add(duration);

        const success = check(res, {
            'list users status is 200 or 403': (r) => r.status === 200 || r.status === 403,
        });

        apiSuccessRate.add(success);
        if (!success) errorCount.add(1);
    });
}

function testGetProfile(headers) {
    group('Get Profile', () => {
        const startTime = Date.now();
        const res = http.get(
            `${BASE_URL}${API_PREFIX}/profile`,
            { headers }
        );
        const duration = Date.now() - startTime;
        getUserByIdDuration.add(duration);

        const success = check(res, {
            'get profile status is 200': (r) => r.status === 200,
        });

        apiSuccessRate.add(success);
        if (!success) errorCount.add(1);
    });
}

function testUpdateProfile(headers) {
    group('Update Profile', () => {
        const startTime = Date.now();
        const payload = JSON.stringify({
            display_name: `Updated User ${Date.now()}`,
        });

        const res = http.patch(
            `${BASE_URL}${API_PREFIX}/profile`,
            payload,
            { headers }
        );
        const duration = Date.now() - startTime;
        updateProfileDuration.add(duration);

        const success = check(res, {
            'update profile status is 200 or 204': (r) => r.status === 200 || r.status === 204,
        });

        apiSuccessRate.add(success);
        if (!success) errorCount.add(1);
    });
}

function testListPermissions(headers) {
    group('List Permissions', () => {
        const startTime = Date.now();
        const res = http.get(
            `${BASE_URL}${API_PREFIX}/permissions`,
            { headers }
        );
        const duration = Date.now() - startTime;
        searchDuration.add(duration);

        const success = check(res, {
            'list permissions status is 200 or 403': (r) => r.status === 200 || r.status === 403,
        });

        apiSuccessRate.add(success);
        if (!success) errorCount.add(1);
    });
}

function testHealthCheck() {
    group('Health Check', () => {
        const res = http.get(`${BASE_URL}/health`);

        const success = check(res, {
            'health check status is 200': (r) => r.status === 200,
        });

        apiSuccessRate.add(success);
        if (!success) errorCount.add(1);
    });
}

export function setup() {
    console.log(`API Load Test starting against ${BASE_URL}`);

    if (!AUTH_TOKEN) {
        console.warn('No AUTH_TOKEN provided. Some tests may fail.');
    }

    // Verify connectivity
    const healthRes = http.get(`${BASE_URL}/health`);
    if (healthRes.status !== 200) {
        throw new Error(`Server not reachable: ${healthRes.status}`);
    }

    return { startTime: new Date().toISOString() };
}

export function teardown(data) {
    console.log(`API Load Test completed`);
    console.log(`Started: ${data.startTime}`);
    console.log(`Ended: ${new Date().toISOString()}`);
}
