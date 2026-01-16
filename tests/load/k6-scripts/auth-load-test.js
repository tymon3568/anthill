// Authentication Load Test - k6 Script
// Tests user authentication endpoints under load
// Usage: k6 run auth-load-test.js

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const loginSuccessRate = new Rate('login_success_rate');
const loginDuration = new Trend('login_duration');
const registrationDuration = new Trend('registration_duration');
const profileDuration = new Trend('profile_duration');
const errorCount = new Counter('errors');

// Configuration
const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';
const API_PREFIX = '/api/v1';

// Test configuration
export const options = {
    scenarios: {
        // Smoke test - verify system works
        smoke: {
            executor: 'constant-vus',
            vus: 1,
            duration: '30s',
            startTime: '0s',
            tags: { scenario: 'smoke' },
        },
        // Load test - normal traffic
        load: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '1m', target: 10 },   // Ramp up to 10 users
                { duration: '3m', target: 10 },   // Stay at 10 users
                { duration: '1m', target: 50 },   // Ramp up to 50 users
                { duration: '3m', target: 50 },   // Stay at 50 users
                { duration: '2m', target: 0 },    // Ramp down
            ],
            startTime: '30s',
            tags: { scenario: 'load' },
        },
        // Stress test - beyond normal capacity
        stress: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '2m', target: 100 },  // Ramp to 100 users
                { duration: '5m', target: 100 },  // Stay at 100
                { duration: '2m', target: 200 },  // Push to 200
                { duration: '5m', target: 200 },  // Stay at 200
                { duration: '2m', target: 0 },    // Ramp down
            ],
            startTime: '11m',
            tags: { scenario: 'stress' },
        },
    },
    thresholds: {
        http_req_duration: ['p(95)<500', 'p(99)<1000'],    // 95% < 500ms, 99% < 1s
        http_req_failed: ['rate<0.01'],                    // < 1% errors
        login_success_rate: ['rate>0.95'],                 // > 95% login success
        login_duration: ['p(95)<300'],                     // Login 95% < 300ms
        registration_duration: ['p(95)<500'],              // Registration 95% < 500ms
        profile_duration: ['p(95)<200'],                   // Profile 95% < 200ms
    },
};

// Test data
const testTenantId = __ENV.TENANT_ID || '00000000-0000-0000-0000-000000000001';

function generateTestUser() {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(7);
    return {
        email: `loadtest_${timestamp}_${random}@example.com`,
        password: 'LoadTest123!@#',
        display_name: `Load Test User ${random}`,
        tenant_id: testTenantId,
    };
}

// Main test function
export default function () {
    const user = generateTestUser();
    let authToken = null;

    group('User Registration', () => {
        const startTime = Date.now();
        const registerPayload = JSON.stringify({
            email: user.email,
            password: user.password,
            display_name: user.display_name,
        });

        const registerRes = http.post(
            `${BASE_URL}${API_PREFIX}/auth/register`,
            registerPayload,
            {
                headers: {
                    'Content-Type': 'application/json',
                    'X-Tenant-ID': testTenantId,
                },
            }
        );

        const duration = Date.now() - startTime;
        registrationDuration.add(duration);

        const success = check(registerRes, {
            'registration status is 201 or 409': (r) => r.status === 201 || r.status === 409,
            'registration response has body': (r) => r.body && r.body.length > 0,
        });

        if (!success) {
            errorCount.add(1);
            console.error(`Registration failed: ${registerRes.status} - ${registerRes.body}`);
        }
    });

    sleep(0.5);

    group('User Login', () => {
        const startTime = Date.now();
        const loginPayload = JSON.stringify({
            email: user.email,
            password: user.password,
        });

        const loginRes = http.post(
            `${BASE_URL}${API_PREFIX}/auth/login`,
            loginPayload,
            {
                headers: {
                    'Content-Type': 'application/json',
                    'X-Tenant-ID': testTenantId,
                },
            }
        );

        const duration = Date.now() - startTime;
        loginDuration.add(duration);

        const success = check(loginRes, {
            'login status is 200': (r) => r.status === 200,
            'login response has access_token': (r) => {
                try {
                    const body = JSON.parse(r.body);
                    return body.access_token !== undefined;
                } catch {
                    return false;
                }
            },
        });

        loginSuccessRate.add(success);

        if (success) {
            try {
                const body = JSON.parse(loginRes.body);
                authToken = body.access_token;
            } catch (e) {
                console.error('Failed to parse login response');
            }
        } else {
            errorCount.add(1);
            console.error(`Login failed: ${loginRes.status} - ${loginRes.body}`);
        }
    });

    sleep(0.5);

    if (authToken) {
        group('Get Profile', () => {
            const startTime = Date.now();
            const profileRes = http.get(
                `${BASE_URL}${API_PREFIX}/profile`,
                {
                    headers: {
                        'Authorization': `Bearer ${authToken}`,
                        'X-Tenant-ID': testTenantId,
                    },
                }
            );

            const duration = Date.now() - startTime;
            profileDuration.add(duration);

            const success = check(profileRes, {
                'profile status is 200': (r) => r.status === 200,
                'profile has user data': (r) => {
                    try {
                        const body = JSON.parse(r.body);
                        return body.email !== undefined;
                    } catch {
                        return false;
                    }
                },
            });

            if (!success) {
                errorCount.add(1);
            }
        });

        sleep(0.3);

        group('Token Refresh', () => {
            const refreshRes = http.post(
                `${BASE_URL}${API_PREFIX}/auth/refresh`,
                null,
                {
                    headers: {
                        'Authorization': `Bearer ${authToken}`,
                        'X-Tenant-ID': testTenantId,
                    },
                }
            );

            check(refreshRes, {
                'refresh status is 200': (r) => r.status === 200,
            });
        });
    }

    sleep(1);
}

// Setup - runs once before tests
export function setup() {
    console.log(`Starting load test against ${BASE_URL}`);
    console.log(`Tenant ID: ${testTenantId}`);

    // Verify server is reachable
    const healthRes = http.get(`${BASE_URL}/health`);
    if (healthRes.status !== 200) {
        console.warn(`Health check failed: ${healthRes.status}`);
    }

    return { startTime: new Date().toISOString() };
}

// Teardown - runs once after tests
export function teardown(data) {
    console.log(`Load test completed. Started at: ${data.startTime}`);
    console.log(`Ended at: ${new Date().toISOString()}`);
}
