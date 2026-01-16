import { request } from '@playwright/test';
import * as fs from 'fs';

async function globalSetup() {
  const baseURL = process.env.BASE_URL || 'http://localhost:3000';
  const tenantId = process.env.TENANT_ID || '00000000-0000-0000-0000-000000000001';

  console.log('🚀 E2E Test Global Setup');
  console.log(`   Base URL: ${baseURL}`);
  console.log(`   Tenant ID: ${tenantId}`);

  // Check server health
  const context = await request.newContext({ baseURL });

  try {
    const healthResponse = await context.get('/health');
    if (healthResponse.status() !== 200) {
      throw new Error(`Server health check failed: ${healthResponse.status()}`);
    }
    console.log('   ✓ Server is healthy');
  } catch (error) {
    console.error('   ✗ Server is not reachable');
    console.error(`   Make sure the server is running at ${baseURL}`);
    throw error;
  }

  // Create test admin user for E2E tests
  const adminEmail = `e2e_admin_${Date.now()}@example.com`;
  const adminPassword = 'E2ETestAdmin123!@#';

  try {
    const registerResponse = await context.post('/api/v1/auth/register', {
      headers: {
        'X-Tenant-ID': tenantId,
      },
      data: {
        email: adminEmail,
        password: adminPassword,
        display_name: 'E2E Test Admin',
      },
    });

    if (registerResponse.status() === 201 || registerResponse.status() === 409) {
      console.log('   ✓ Test admin user created/exists');
    }
  } catch (error) {
    console.warn('   ⚠ Could not create test admin user');
  }

  // Login and get token
  try {
    const loginResponse = await context.post('/api/v1/auth/login', {
      headers: {
        'X-Tenant-ID': tenantId,
      },
      data: {
        email: adminEmail,
        password: adminPassword,
      },
    });

    if (loginResponse.status() === 200) {
      const { access_token, refresh_token } = await loginResponse.json();

      // Save tokens for tests
      const testState = {
        adminEmail,
        adminPassword,
        tenantId,
        accessToken: access_token,
        refreshToken: refresh_token,
        timestamp: new Date().toISOString(),
      };

      fs.writeFileSync('.e2e-state.json', JSON.stringify(testState, null, 2));
      console.log('   ✓ Test state saved');
    }
  } catch (error) {
    console.warn('   ⚠ Could not login test admin');
  }

  await context.dispose();
  console.log('   ✓ Global setup complete\n');
}

export default globalSetup;
