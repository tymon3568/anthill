#!/bin/bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123","full_name":"Test User","tenant_name":"Test Tenant"}' \
  http://localhost:8000/auth/register
