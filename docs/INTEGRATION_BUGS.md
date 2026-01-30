# Integration Bugs

> This document contains bugs that occur at the BOUNDARY between modules.
> These are issues that arise when Frontend ↔ Backend, Service ↔ Service, or Service ↔ Database communicate.
> AI agents MUST read this file before implementing any cross-module features.

---

## IB-1: Casbin Policies for New Endpoints

**Date discovered:** 2026-01-28

**Modules involved:** Frontend ↔ Any Backend Service

### Bug Description
- New API endpoint returns 403 Forbidden even though user is authenticated
- Only happens on new endpoints, existing endpoints work fine

### Root Cause
1. **Missing Casbin policy migration** for the new endpoint
2. **Casbin uses tenant_id UUID, not slug** - policies must use `tenant_id::text`
3. **Nested routes need OriginalUri** - Axum modifies `parts.uri` in nested routers

### Fix

**Step 1: Create Casbin policy migration**
```sql
-- migrations/{timestamp}_add_{module}_casbin_policies.sql

-- Use tenant_id::text, NOT slug
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/adjustments', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Add for all roles: owner, admin, manager, user (with appropriate permissions)
-- Add for all HTTP methods: GET, POST, PUT, DELETE
-- Add for nested routes with wildcards: /warehouses/*/zones
```

**Step 2: Use OriginalUri in extractors (if using RequirePermission)**
```rust
// In extractors.rs
let resource = parts
    .extensions
    .get::<OriginalUri>()
    .map(|uri| uri.path().to_string())
    .unwrap_or_else(|| parts.uri.path().to_string());
```

**Step 3: Run migration and RESTART backend**
```bash
sqlx migrate run
# Kill and restart the service - policies are loaded at startup
```

### Prevention Checklist
- [ ] Create Casbin migration for EVERY new endpoint
- [ ] Use `tenant_id::text` not slug
- [ ] Include all CRUD methods (GET, POST, PUT, DELETE)
- [ ] Include nested routes with wildcards (`/parent/*/child`)
- [ ] RESTART backend after migration (policies cached at startup)

**Reference**: `docs/archive/BUGS_FIXED_LEGACY.md` Bugs #3, #5

---

## IB-2: API Proxy Token Refresh

**Date discovered:** 2026-01-29

**Modules involved:** Frontend (SvelteKit) ↔ Backend Services

### Bug Description
- After 15 minutes, all API calls start failing
- Frontend shows "Internal server error" (500) instead of redirecting to login
- User must manually refresh and re-login

### Root Cause
- SvelteKit API proxy routes (`/api/v1/inventory/[...path]/+server.ts`) forward requests to backend
- Proxy did NOT implement token refresh logic
- When access_token expired, proxy caught the 401 and returned generic 500

### Fix

**Implement token refresh in API proxy:**
```typescript
// /routes/api/v1/inventory/[...path]/+server.ts

import { shouldRefreshToken, isTokenExpired } from '$lib/auth/jwt';

async function refreshAccessToken(cookies: Cookies): Promise<string | null> {
  const refreshToken = cookies.get('refresh_token');
  if (!refreshToken) return null;
  
  const response = await fetch(`${USER_SERVICE_URL}/api/v1/auth/refresh`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ refresh_token: refreshToken })
  });
  
  if (response.ok) {
    const data = await response.json();
    cookies.set('access_token', data.access_token, { path: '/', httpOnly: true, secure: true });
    if (data.refresh_token) {
      cookies.set('refresh_token', data.refresh_token, { path: '/', httpOnly: true, secure: true });
    }
    return data.access_token;
  }
  
  // Refresh failed - clear cookies
  if (response.status === 401 || response.status === 403) {
    cookies.delete('access_token', { path: '/' });
    cookies.delete('refresh_token', { path: '/' });
  }
  return null;
}

async function proxyRequest(request: Request, cookies: Cookies): Promise<Response> {
  let accessToken = cookies.get('access_token');
  
  // 1. Proactive refresh if token about to expire
  if (accessToken && (isTokenExpired(accessToken) || shouldRefreshToken(accessToken))) {
    const newToken = await refreshAccessToken(cookies);
    if (newToken) accessToken = newToken;
  }
  
  // 2. No token: return structured 401
  if (!accessToken) {
    return json({
      error: 'Authentication required',
      code: 'NO_SESSION',
      message: 'Please log in'
    }, { status: 401 });
  }
  
  // 3. Forward request
  let response = await fetch(backendUrl, {
    headers: { Authorization: `Bearer ${accessToken}` }
  });
  
  // 4. Retry on 401
  if (response.status === 401) {
    const newToken = await refreshAccessToken(cookies);
    if (newToken) {
      response = await fetch(backendUrl, {
        headers: { Authorization: `Bearer ${newToken}` }
      });
    } else {
      return json({
        error: 'Session expired',
        code: 'SESSION_EXPIRED',
        message: 'Please log in again'
      }, { status: 401 });
    }
  }
  
  return response;
}
```

### Prevention Checklist
- [ ] Every API proxy route MUST implement token refresh
- [ ] Use `isTokenExpired()` and `shouldRefreshToken()` from `$lib/auth/jwt`
- [ ] Return structured 401 with `code` field (not generic 500)
- [ ] Reference `hooks.server.ts` for refresh pattern

**Reference**: `docs/archive/BUGS_FIXED_LEGACY.md` Bug #6

---

## IB-3: Backend Response Format Mismatch

**Date discovered:** 2026-01-28

**Modules involved:** Frontend ↔ Backend API

### Bug Description
- API returns 200 with data
- Frontend list shows "0 items" or empty
- No errors in console

### Root Cause
- Backend returns raw array: `[{...}, {...}]`
- Frontend expects wrapped object: `{ items: [...], pagination: {...} }`

### Fix

**Transform in API client layer:**
```typescript
// frontend/src/lib/api/inventory/warehouses.ts
async list(): Promise<ApiResponse<WarehouseListResponse>> {
  // Backend returns raw array
  const response = await apiClient.get<WarehouseResponse[]>('/warehouses');
  
  if (response.success && response.data) {
    const items = response.data;
    // Wrap in expected format
    return {
      success: true,
      data: {
        warehouses: items,
        pagination: {
          page: 1,
          pageSize: items.length,
          totalItems: items.length,
          totalPages: 1,
          hasNext: false,
          hasPrev: false
        }
      }
    };
  }
  return { success: false, error: response.error };
}
```

### Prevention Checklist
- [ ] Test API with curl/Postman to see actual response format
- [ ] Document expected vs actual response format
- [ ] Transform in API client layer, not in components
- [ ] Consider standardizing backend response format across all services

**Reference**: `docs/archive/BUGS_FIXED_LEGACY.md` Bug #4

---

## IB-4: Type Mismatch Rust ↔ TypeScript

**Date discovered:** 2026-01-29

**Modules involved:** Frontend (TypeScript) ↔ Backend (Rust)

### Bug Description
- Rust struct uses `Option<String>` but TypeScript expects `string`
- Rust uses `i64` but TypeScript receives as string from JSON
- Rust enum serializes differently than TypeScript expects

### Root Cause
- No automatic type synchronization between Rust and TypeScript
- Serde serialization defaults may not match frontend expectations

### Fix

**Option 1: Manual type sync (current approach)**
```rust
// Backend: Rust struct
#[derive(Serialize)]
pub struct AdjustmentResponse {
    pub adjustment_id: Uuid,
    pub quantity: i32,
    pub notes: Option<String>,  // Can be null
}
```

```typescript
// Frontend: TypeScript interface
interface AdjustmentResponse {
  adjustmentId: string;  // UUID serializes to string
  quantity: number;
  notes: string | null;  // Match Option<String>
}
```

**Option 2: Generate types from Rust (recommended for future)**
```bash
# Use ts-rs or similar to generate TypeScript from Rust
cargo install ts-rs
# Add #[derive(TS)] to structs
```

### Common Mismatches

| Rust Type | JSON | TypeScript |
|-----------|------|------------|
| `Uuid` | `"string"` | `string` |
| `i32`, `i64` | `number` | `number` |
| `Option<T>` | `null` or value | `T \| null` |
| `DateTime<Utc>` | `"ISO string"` | `string` (parse with `new Date()`) |
| `enum` (unit) | `"VariantName"` | `string` or enum |
| `enum` (data) | `{"variant": {...}}` | discriminated union |

### Prevention Checklist
- [ ] Document Rust struct ↔ TypeScript interface mappings
- [ ] Handle `Option<T>` as `T | null` in TypeScript
- [ ] Parse date strings when needed
- [ ] Test with actual API responses, not mocks

---

## IB-5: Service Port Conflicts

**Date discovered:** 2026-01-28

**Modules involved:** Frontend Proxy ↔ Backend Services

### Bug Description
- API calls go to wrong service
- 404 errors on existing endpoints
- Connection refused

### Root Cause
- Services started on wrong ports
- Frontend proxy configured for different port
- Port already in use by another process

### Standard Port Convention

| Service | Port |
|---------|------|
| Frontend (SvelteKit) | 5173 |
| User Service | 8000 |
| Inventory Service | 8001 |
| Order Service | 8002 |
| Notification Service | 8003 |

### Prevention Checklist
- [ ] Always use standard ports from convention table
- [ ] Check `vite.config.ts` proxy configuration
- [ ] Kill old processes before starting: `lsof -i :8001`
- [ ] Verify service is running: `curl http://localhost:8001/health`

---

## IB-6: Missing Required Headers (Idempotency)

**Date discovered:** 2026-01-30

**Modules involved:** Frontend ↔ Backend API (POST/PUT/PATCH requests)

### Bug Description
- POST/PUT/PATCH requests return "Empty reply from server" or connection reset
- GET requests work fine, only mutating operations fail
- No useful error message in browser DevTools or server logs

### Root Cause
- Anthill middleware requires `X-Idempotency-Key` header for all mutating operations
- Without this header, the request is rejected at middleware level BEFORE reaching handler
- Connection closes abruptly without proper HTTP response

### Fix

**Option 1: Add header manually in API calls**
```typescript
// frontend/src/lib/api/inventory/products.ts
async create(data: CreateProductRequest): Promise<ApiResponse<ProductResponse>> {
  return apiClient.post('/products', data, {
    headers: {
      'X-Idempotency-Key': crypto.randomUUID(),
    }
  });
}
```

**Option 2: Add automatic header in API client (RECOMMENDED)**
```typescript
// frontend/src/lib/api/client.ts
async request<T>(config: RequestConfig): Promise<ApiResponse<T>> {
  const headers = new Headers(config.headers);
  
  // Auto-add idempotency key for mutating methods
  if (['POST', 'PUT', 'PATCH'].includes(config.method?.toUpperCase() || '')) {
    if (!headers.has('X-Idempotency-Key')) {
      headers.set('X-Idempotency-Key', crypto.randomUUID());
    }
  }
  
  // ... rest of request logic
}
```

**Option 3: Verify in SvelteKit proxy (+server.ts)**
```typescript
// routes/api/v1/inventory/[...path]/+server.ts
async function proxyRequest(request: Request, cookies: Cookies): Promise<Response> {
  const headers = new Headers(request.headers);
  
  // Ensure idempotency key for mutating operations
  if (['POST', 'PUT', 'PATCH'].includes(request.method)) {
    if (!headers.has('X-Idempotency-Key')) {
      headers.set('X-Idempotency-Key', crypto.randomUUID());
    }
  }
  
  // Forward to backend
  return fetch(backendUrl, { method: request.method, headers, body });
}
```

### Testing Checklist
```bash
# Test without idempotency key - should fail or auto-add
curl -X POST http://localhost:8001/api/v1/inventory/products \
  -H "Authorization: Bearer $TOKEN" \
  -H "X-Tenant-ID: $TENANT_ID" \
  -H "Content-Type: application/json" \
  -d '{"name":"Test"}'

# Test with idempotency key - should succeed
curl -X POST http://localhost:8001/api/v1/inventory/products \
  -H "Authorization: Bearer $TOKEN" \
  -H "X-Tenant-ID: $TENANT_ID" \
  -H "X-Idempotency-Key: $(uuidgen)" \
  -H "Content-Type: application/json" \
  -d '{"name":"Test"}'
```

### Required Headers Summary

| Header | Required For | Purpose |
|--------|--------------|---------|
| `Authorization` | All authenticated requests | JWT Bearer token |
| `X-Tenant-ID` | All requests | Multi-tenant routing |
| `Content-Type` | POST/PUT/PATCH with body | Request body format |
| `X-Idempotency-Key` | POST/PUT/PATCH | Prevent duplicate operations |

### Prevention Checklist
- [ ] API client automatically adds `X-Idempotency-Key` for POST/PUT/PATCH
- [ ] SvelteKit proxy forwards all required headers
- [ ] Test mutating operations with curl before UI testing
- [ ] Document all required headers in API specs
- [ ] Check middleware order in backend (idempotency before auth?)

**Reference**: `docs/test-reports/8.10.6_products_enhancement_test_report.md` BUG-003

---

## Quick Reference Table

| Bug | Symptom | Quick Fix |
|-----|---------|-----------|
| IB-1 | 403 on new endpoint | Add Casbin migration + restart |
| IB-2 | 500 after 15 min | Add token refresh to API proxy |
| IB-3 | List shows 0 items | Transform response format in API client |
| IB-4 | Type errors / undefined | Sync Rust ↔ TypeScript types |
| IB-5 | 404 / connection refused | Check port configuration |
| IB-6 | Empty reply on POST/PUT/PATCH | Add X-Idempotency-Key header |

---

**Document Version:** 1.0
**Created:** 2026-01-29
**Author:** Claude (AI Agent)
