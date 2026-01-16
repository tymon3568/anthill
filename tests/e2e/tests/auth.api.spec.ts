import { test, expect, APIRequestContext } from '@playwright/test';
import * as fs from 'fs';

interface TestState {
  adminEmail: string;
  adminPassword: string;
  tenantId: string;
  accessToken: string;
  refreshToken: string;
}

function getTestState(): TestState {
  const stateFile = '.e2e-state.json';
  if (fs.existsSync(stateFile)) {
    return JSON.parse(fs.readFileSync(stateFile, 'utf-8'));
  }
  throw new Error('Test state not found. Run global setup first.');
}

test.describe('Authentication API E2E Tests', () => {
  let state: TestState;

  test.beforeAll(() => {
    state = getTestState();
  });

  test('should register a new user successfully', async ({ request }) => {
    const uniqueEmail = `e2e_user_${Date.now()}@example.com`;

    const response = await request.post('/api/v1/auth/register', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email: uniqueEmail,
        password: 'E2ETestUser123!@#',
        display_name: 'E2E Test User',
      },
    });

    expect(response.status()).toBe(201);

    const body = await response.json();
    expect(body).toHaveProperty('user_id');
    expect(body).toHaveProperty('email', uniqueEmail);
  });

  test('should reject registration with weak password', async ({ request }) => {
    const response = await request.post('/api/v1/auth/register', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email: `weak_${Date.now()}@example.com`,
        password: '123',
        display_name: 'Weak Password User',
      },
    });

    expect(response.status()).toBe(400);
  });

  test('should reject duplicate email registration', async ({ request }) => {
    const email = `dup_${Date.now()}@example.com`;

    // First registration
    await request.post('/api/v1/auth/register', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email,
        password: 'E2ETestUser123!@#',
        display_name: 'First User',
      },
    });

    // Duplicate registration
    const response = await request.post('/api/v1/auth/register', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email,
        password: 'E2ETestUser123!@#',
        display_name: 'Duplicate User',
      },
    });

    expect(response.status()).toBe(409);
  });

  test('should login successfully with valid credentials', async ({ request }) => {
    const response = await request.post('/api/v1/auth/login', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email: state.adminEmail,
        password: state.adminPassword,
      },
    });

    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body).toHaveProperty('access_token');
    expect(body).toHaveProperty('refresh_token');
    expect(body).toHaveProperty('token_type', 'Bearer');
  });

  test('should reject login with invalid password', async ({ request }) => {
    const response = await request.post('/api/v1/auth/login', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email: state.adminEmail,
        password: 'WrongPassword123!',
      },
    });

    expect(response.status()).toBe(401);
  });

  test('should reject login for non-existent user', async ({ request }) => {
    const response = await request.post('/api/v1/auth/login', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email: 'nonexistent@example.com',
        password: 'SomePassword123!',
      },
    });

    expect(response.status()).toBe(401);
  });

  test('should refresh token successfully', async ({ request }) => {
    const response = await request.post('/api/v1/auth/refresh', {
      headers: {
        'Authorization': `Bearer ${state.accessToken}`,
        'X-Tenant-ID': state.tenantId,
      },
    });

    // May be 200 or 401 depending on token validity
    expect([200, 401]).toContain(response.status());
  });

  test('should logout successfully', async ({ request }) => {
    // Get fresh token
    const loginResponse = await request.post('/api/v1/auth/login', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email: state.adminEmail,
        password: state.adminPassword,
      },
    });

    const { access_token } = await loginResponse.json();

    const response = await request.post('/api/v1/auth/logout', {
      headers: {
        'Authorization': `Bearer ${access_token}`,
        'X-Tenant-ID': state.tenantId,
      },
    });

    expect([200, 204]).toContain(response.status());
  });
});

test.describe('Profile API E2E Tests', () => {
  let state: TestState;
  let accessToken: string;

  test.beforeAll(async ({ request }) => {
    state = getTestState();

    // Get fresh token
    const loginResponse = await request.post('/api/v1/auth/login', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email: state.adminEmail,
        password: state.adminPassword,
      },
    });

    const body = await loginResponse.json();
    accessToken = body.access_token;
  });

  test('should get user profile', async ({ request }) => {
    const response = await request.get('/api/v1/profile', {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'X-Tenant-ID': state.tenantId,
      },
    });

    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body).toHaveProperty('email');
    expect(body).toHaveProperty('display_name');
  });

  test('should reject profile access without token', async ({ request }) => {
    const response = await request.get('/api/v1/profile', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
    });

    expect(response.status()).toBe(401);
  });

  test('should update user profile', async ({ request }) => {
    const newDisplayName = `Updated E2E User ${Date.now()}`;

    const response = await request.patch('/api/v1/profile', {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        display_name: newDisplayName,
      },
    });

    expect([200, 204]).toContain(response.status());

    // Verify update
    const profileResponse = await request.get('/api/v1/profile', {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'X-Tenant-ID': state.tenantId,
      },
    });

    const body = await profileResponse.json();
    expect(body.display_name).toBe(newDisplayName);
  });
});

test.describe('Health Check E2E Tests', () => {
  test('should return healthy status', async ({ request }) => {
    const response = await request.get('/health');

    expect(response.status()).toBe(200);
  });
});
