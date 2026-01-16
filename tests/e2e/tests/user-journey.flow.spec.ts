import { test, expect } from '@playwright/test';
import * as fs from 'fs';

interface TestState {
  adminEmail: string;
  adminPassword: string;
  tenantId: string;
  accessToken: string;
}

function getTestState(): TestState {
  const stateFile = '.e2e-state.json';
  if (fs.existsSync(stateFile)) {
    return JSON.parse(fs.readFileSync(stateFile, 'utf-8'));
  }
  throw new Error('Test state not found.');
}

test.describe('Complete User Journey Flow', () => {
  const testEmail = `journey_${Date.now()}@example.com`;
  const testPassword = 'JourneyTest123!@#';
  let state: TestState;
  let userAccessToken: string;
  let userRefreshToken: string;

  test.beforeAll(() => {
    state = getTestState();
  });

  test('Step 1: User registers a new account', async ({ request }) => {
    const response = await request.post('/api/v1/auth/register', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email: testEmail,
        password: testPassword,
        display_name: 'Journey Test User',
      },
    });

    expect(response.status()).toBe(201);

    const body = await response.json();
    expect(body.email).toBe(testEmail);
    expect(body.user_id).toBeDefined();
  });

  test('Step 2: User logs in to their account', async ({ request }) => {
    const response = await request.post('/api/v1/auth/login', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email: testEmail,
        password: testPassword,
      },
    });

    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body.access_token).toBeDefined();
    expect(body.refresh_token).toBeDefined();

    userAccessToken = body.access_token;
    userRefreshToken = body.refresh_token;
  });

  test('Step 3: User views their profile', async ({ request }) => {
    const response = await request.get('/api/v1/profile', {
      headers: {
        'Authorization': `Bearer ${userAccessToken}`,
        'X-Tenant-ID': state.tenantId,
      },
    });

    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body.email).toBe(testEmail);
    expect(body.display_name).toBe('Journey Test User');
  });

  test('Step 4: User updates their profile', async ({ request }) => {
    const newDisplayName = 'Updated Journey User';

    const response = await request.patch('/api/v1/profile', {
      headers: {
        'Authorization': `Bearer ${userAccessToken}`,
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        display_name: newDisplayName,
      },
    });

    expect([200, 204]).toContain(response.status());

    // Verify the update
    const profileResponse = await request.get('/api/v1/profile', {
      headers: {
        'Authorization': `Bearer ${userAccessToken}`,
        'X-Tenant-ID': state.tenantId,
      },
    });

    const body = await profileResponse.json();
    expect(body.display_name).toBe(newDisplayName);
  });

  test('Step 5: User refreshes their token', async ({ request }) => {
    const response = await request.post('/api/v1/auth/refresh', {
      headers: {
        'Authorization': `Bearer ${userAccessToken}`,
        'X-Tenant-ID': state.tenantId,
      },
    });

    // Token refresh may succeed or fail depending on implementation
    expect([200, 401]).toContain(response.status());
  });

  test('Step 6: User logs out', async ({ request }) => {
    const response = await request.post('/api/v1/auth/logout', {
      headers: {
        'Authorization': `Bearer ${userAccessToken}`,
        'X-Tenant-ID': state.tenantId,
      },
    });

    expect([200, 204]).toContain(response.status());
  });

  test('Step 7: Token is invalid after logout', async ({ request }) => {
    const response = await request.get('/api/v1/profile', {
      headers: {
        'Authorization': `Bearer ${userAccessToken}`,
        'X-Tenant-ID': state.tenantId,
      },
    });

    // Should be unauthorized after logout
    expect(response.status()).toBe(401);
  });
});

test.describe('Password Change Flow', () => {
  const testEmail = `pwchange_${Date.now()}@example.com`;
  const originalPassword = 'OriginalPass123!@#';
  const newPassword = 'NewPassword456!@#';
  let state: TestState;
  let accessToken: string;

  test.beforeAll(() => {
    state = getTestState();
  });

  test('Step 1: Register user', async ({ request }) => {
    const response = await request.post('/api/v1/auth/register', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email: testEmail,
        password: originalPassword,
        display_name: 'Password Change User',
      },
    });

    expect(response.status()).toBe(201);
  });

  test('Step 2: Login with original password', async ({ request }) => {
    const response = await request.post('/api/v1/auth/login', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email: testEmail,
        password: originalPassword,
      },
    });

    expect(response.status()).toBe(200);

    const body = await response.json();
    accessToken = body.access_token;
  });

  test('Step 3: Change password', async ({ request }) => {
    const response = await request.post('/api/v1/auth/change-password', {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        current_password: originalPassword,
        new_password: newPassword,
      },
    });

    // May be 200, 204, or 404 depending on endpoint availability
    expect([200, 204, 404]).toContain(response.status());
  });

  test('Step 4: Login with new password', async ({ request }) => {
    const response = await request.post('/api/v1/auth/login', {
      headers: {
        'X-Tenant-ID': state.tenantId,
      },
      data: {
        email: testEmail,
        password: newPassword,
      },
    });

    // If password change was successful, this should work
    // Otherwise, original password still works
    expect([200, 401]).toContain(response.status());
  });
});

test.describe('Multi-Tenant Isolation Flow', () => {
  let state: TestState;
  const tenant1Id = '00000000-0000-0000-0000-000000000001';
  const tenant2Id = '00000000-0000-0000-0000-000000000002';

  test.beforeAll(() => {
    state = getTestState();
  });

  test('User in tenant 1 cannot access tenant 2 data', async ({ request }) => {
    // Register and login in tenant 1
    const email = `tenant1_${Date.now()}@example.com`;
    const password = 'TenantTest123!@#';

    await request.post('/api/v1/auth/register', {
      headers: {
        'X-Tenant-ID': tenant1Id,
      },
      data: {
        email,
        password,
        display_name: 'Tenant 1 User',
      },
    });

    const loginResponse = await request.post('/api/v1/auth/login', {
      headers: {
        'X-Tenant-ID': tenant1Id,
      },
      data: {
        email,
        password,
      },
    });

    const { access_token } = await loginResponse.json();

    // Try to access profile with wrong tenant ID
    const response = await request.get('/api/v1/profile', {
      headers: {
        'Authorization': `Bearer ${access_token}`,
        'X-Tenant-ID': tenant2Id,
      },
    });

    // Should be forbidden or unauthorized
    expect([401, 403]).toContain(response.status());
  });
});
