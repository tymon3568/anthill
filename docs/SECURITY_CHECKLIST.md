# Security Checklist

> This document contains MANDATORY security tests that MUST be performed before releasing any module.
> AI agents MUST read this file and include these tests in every testing phase.

---

## Mandatory Security Tests

### 1. SQL Injection Testing

**Test Input:**
```
'; DROP TABLE users; --
' OR '1'='1
'; UPDATE users SET role='admin' WHERE '1'='1
```

**Where to Test:**
- [ ] All text input fields (name, description, notes, etc.)
- [ ] Search boxes
- [ ] Filter inputs
- [ ] URL query parameters

**Expected Behavior:**
- ✅ Payload stored as literal text in database
- ✅ No SQL execution (tables intact)
- ✅ No data leakage
- ❌ FAIL if: Error message reveals SQL structure
- ❌ FAIL if: Query returns unexpected data

**Verification:**
```sql
-- After test, check database directly
SELECT * FROM adjustments WHERE notes LIKE '%DROP TABLE%';
-- Should find the literal text, not execute it
```

---

### 2. XSS (Cross-Site Scripting) Testing

**Test Input:**
```html
<script>alert('XSS')</script>
<img src=x onerror=alert('XSS')>
<svg onload=alert('XSS')>
javascript:alert('XSS')
```

**Where to Test:**
- [ ] All text input fields
- [ ] Rich text editors
- [ ] User-generated content displays
- [ ] URL parameters reflected in page

**Expected Behavior:**
- ✅ Script tags displayed as escaped text: `&lt;script&gt;`
- ✅ No JavaScript execution
- ✅ No alert dialogs appear
- ❌ FAIL if: Alert dialog appears
- ❌ FAIL if: Script executes in any way

**Verification:**
1. Submit form with XSS payload
2. View the created/edited item
3. Check that payload appears as text, not executed
4. Check browser console for any script errors

---

### 3. Boundary Value Testing

**Test Values:**

| Type | Test Values |
|------|-------------|
| Quantity/Number | `0`, `-1`, `-999999`, `9999999999999`, `0.0001`, `NaN` |
| Text | Empty `""`, Very long (10000+ chars), Unicode `日本語`, Emoji `🎉` |
| UUID | Empty, Invalid format, Non-existent valid UUID |
| Date | Past dates, Future dates, Invalid format |

**Expected Behavior:**
- ✅ Form validation catches invalid values BEFORE submission
- ✅ Backend returns clear validation error messages
- ✅ System handles gracefully without crash
- ❌ FAIL if: Application crashes
- ❌ FAIL if: Invalid data saved to database
- ❌ FAIL if: Integer overflow causes negative values

**Specific Checks:**
- [ ] Quantity 0: Should be accepted or rejected based on business rules
- [ ] Negative quantity: Should be rejected with clear error
- [ ] Very large number: Should be rejected or handled without precision loss
- [ ] Empty required fields: Should show validation error

---

### 4. Authentication & Authorization Testing

**Tests:**

| Test | How | Expected |
|------|-----|----------|
| No token | Remove Authorization header | 401 Unauthorized |
| Expired token | Wait 15+ min or modify exp claim | 401 + redirect to login |
| Invalid token | Random string as token | 401 Unauthorized |
| Wrong tenant | Use token from tenant A on tenant B data | 403 Forbidden |
| Role escalation | User role trying admin action | 403 Forbidden |

**Where to Test:**
- [ ] All API endpoints
- [ ] Direct backend calls (bypass frontend)
- [ ] Cross-tenant data access attempts

---

### 5. Invalid ID/UUID Testing

**Test Values:**
```
not-a-uuid
00000000-0000-0000-0000-000000000000
aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee  (valid format, non-existent)
```

**Expected Behavior:**
- ✅ Invalid format: 400 Bad Request with clear error
- ✅ Non-existent: 404 Not Found
- ❌ FAIL if: 500 Internal Server Error
- ❌ FAIL if: Returns data from different entity

---

## Security Test Report Template

```markdown
## Security Test Report - [Module Name]

**Date:** YYYY-MM-DD
**Tester:** [Name/AI Agent]

### SQL Injection
- [ ] Tested in: [list fields]
- [ ] Result: PASS/FAIL
- [ ] Notes: [any observations]

### XSS Attack
- [ ] Tested in: [list fields]
- [ ] Result: PASS/FAIL
- [ ] Notes: [any observations]

### Boundary Values
- [ ] Tested values: [list values]
- [ ] Result: PASS/FAIL
- [ ] Notes: [any observations]

### Authentication
- [ ] Tested scenarios: [list scenarios]
- [ ] Result: PASS/FAIL
- [ ] Notes: [any observations]

### Invalid IDs
- [ ] Tested with: [list IDs]
- [ ] Result: PASS/FAIL
- [ ] Notes: [any observations]

### Overall Status: PASS/FAIL
### Vulnerabilities Found: [count]
### Recommendations: [list]
```

---

## Quick Security Commands

```bash
# Test SQL injection via curl
curl -X POST http://localhost:8001/api/v1/inventory/adjustments \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"notes": "'\'''; DROP TABLE adjustments; --"}'

# Test with invalid UUID
curl http://localhost:8001/api/v1/inventory/adjustments/not-a-uuid \
  -H "Authorization: Bearer $TOKEN"

# Test without token
curl http://localhost:8001/api/v1/inventory/adjustments
```

---

## Known Secure Patterns in This Project

1. **SQL Queries**: Using SQLx with parameterized queries (safe from injection)
2. **Svelte Rendering**: Auto-escapes HTML by default (safe from XSS)
3. **Authentication**: JWT with refresh token pattern
4. **Authorization**: Casbin RBAC with tenant isolation

---

**Document Version:** 1.0
**Created:** 2026-01-29
**Author:** Claude (AI Agent)
