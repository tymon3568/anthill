// Concurrent Sessions Load Test - k6 Script
// Tests system behavior with many concurrent authenticated sessions
// Usage: k6 run concurrent-sessions-test.js

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter, Gauge } from 'k6/metrics';

// Custom metrics
const sessionSuccessRate = new Rate('session_success_rate');
const activeSessionsGauge = new Gauge('active_sessions');
const sessionCreationDuration = new Trend('session_creation_duration');
const sessionValidationDuration = new Trend('session_validation_duration');
const concurrentErrors = new Counter('concurrent_errors');

// Configuration
const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';
const API_PREFIX = '/api/v1';
const TENANT_ID = __ENV.TENANT_ID || '00000000-0000-0000-0000-000000000001';

// Test configuration - simulate many concurrent sessions
export const options = {
    scenarios: {
        // Simulate gradual session buildup
        session_buildup: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '2m', target: 50 },   // Build up 50 sessions
                { duration: '5m', target: 50 },   // Maintain
                { duration: '2m', target: 100 },  // Scale to 100
                { duration: '5m', target: 100 },  // Maintain
                { duration: '2m', target: 200 },  // Scale to 200
                { duration: '5m', target: 200 },  // Maintain peak
                { duration: '3m', target: 0 },    // Graceful shutdown
            ],
            gracefulRampDown: '30s',
        },
    },
    thresholds: {
        http_req_duration: ['p(95)<1000'],
        http_req_failed: ['rate<0.05'],
        session_success_rate: ['rate>0.90'],
        session_creation_duration: ['p(95)<500'],
        session_validation_duration: ['p(95)<100'],
    },
};

// Each VU represents a unique user session
export default function () {
    const vuId = __VU;
    const iterationId = __ITER;
    const uniqueId = `${vuId}_${iterationId}_${Date.now()}`;

    let authToken = null;
    let refreshToken = null;

    // Create new session (login)
    group('Create Session', () => {
        const email = `loaduser_${uniqueId}@example.com`;
        const password = 'LoadTest123!@#';

        // First, try to register (might already exist)
        const registerRes = http.post(
            `${BASE_URL}${API_PREFIX}/auth/register`,
            JSON.stringify({
                email: email,
                password: password,
                display_name: `Load User ${uniqueId}`,
            }),
            {
                headers: {
                    'Content-Type': 'application/json',
                    'X-Tenant-ID': TENANT_ID,
                },
            }
        );

        // Login
        const startTime = Date.now();
        const loginRes = http.post(
            `${BASE_URL}${API_PREFIX}/auth/login`,
            JSON.stringify({
                email: email,
                password: password,
            }),
            {
                headers: {
                    'Content-Type': 'application/json',
                    'X-Tenant-ID': TENANT_ID,
                },
            }
        );
        const duration = Date.now() - startTime;
        sessionCreationDuration.add(duration);

        const success = check(loginRes, {
            'login successful': (r) => r.status === 200,
        });

        sessionSuccessRate.add(success);

        if (success) {
            try {
                const body = JSON.parse(loginRes.body);
                authToken = body.access_token;
                refreshToken = body.refresh_token;
                activeSessionsGauge.add(1);
            } catch (e) {
                concurrentErrors.add(1);
            }
        } else {
            concurrentErrors.add(1);
        }
    });

    if (!authToken) {
        sleep(1);
        return;
    }

    // Simulate active session behavior
    const sessionDuration = Math.random() * 10 + 5; // 5-15 seconds
    const requestsPerSession = Math.floor(Math.random() * 5) + 3; // 3-7 requests

    for (let i = 0; i < requestsPerSession; i++) {
        group('Session Activity', () => {
            const startTime = Date.now();

            // Random activity
            const activities = ['profile', 'permissions', 'health'];
            const activity = activities[Math.floor(Math.random() * activities.length)];

            let res;
            switch (activity) {
                case 'profile':
                    res = http.get(`${BASE_URL}${API_PREFIX}/profile`, {
                        headers: {
                            'Authorization': `Bearer ${authToken}`,
                            'X-Tenant-ID': TENANT_ID,
                        },
                    });
                    break;
                case 'permissions':
                    res = http.get(`${BASE_URL}${API_PREFIX}/permissions`, {
                        headers: {
                            'Authorization': `Bearer ${authToken}`,
                            'X-Tenant-ID': TENANT_ID,
                        },
                    });
                    break;
                case 'health':
                    res = http.get(`${BASE_URL}/health`);
                    break;
            }

            const duration = Date.now() - startTime;
            sessionValidationDuration.add(duration);

            const success = check(res, {
                'request successful': (r) => r.status === 200 || r.status === 403,
            });

            if (!success) {
                concurrentErrors.add(1);
            }
        });

        sleep(sessionDuration / requestsPerSession);
    }

    // Optionally refresh token
    if (refreshToken && Math.random() > 0.7) {
        group('Refresh Token', () => {
            const refreshRes = http.post(
                `${BASE_URL}${API_PREFIX}/auth/refresh`,
                null,
                {
                    headers: {
                        'Authorization': `Bearer ${authToken}`,
                        'X-Tenant-ID': TENANT_ID,
                    },
                }
            );

            check(refreshRes, {
                'token refresh successful': (r) => r.status === 200,
            });
        });
    }

    // End session (logout)
    group('End Session', () => {
        const logoutRes = http.post(
            `${BASE_URL}${API_PREFIX}/auth/logout`,
            null,
            {
                headers: {
                    'Authorization': `Bearer ${authToken}`,
                    'X-Tenant-ID': TENANT_ID,
                },
            }
        );

        check(logoutRes, {
            'logout successful': (r) => r.status === 200 || r.status === 204,
        });

        activeSessionsGauge.add(-1);
    });

    sleep(1);
}

export function setup() {
    console.log('Concurrent Sessions Load Test starting...');
    console.log(`Target: ${BASE_URL}`);
    console.log(`Tenant: ${TENANT_ID}`);

    const healthRes = http.get(`${BASE_URL}/health`);
    if (healthRes.status !== 200) {
        throw new Error('Server not reachable');
    }

    return { startTime: new Date().toISOString() };
}

export function teardown(data) {
    console.log('Concurrent Sessions Load Test completed');
    console.log(`Started: ${data.startTime}`);
    console.log(`Ended: ${new Date().toISOString()}`);
}
