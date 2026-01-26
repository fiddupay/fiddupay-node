# 🐛 CRITICAL BUG REPORT: Authentication Middleware Failure

## **ISSUE ID**: AUTH-001
**Priority**: CRITICAL  
**Status**: ACTIVE  
**Created**: 2026-01-26T20:36:00Z  
**Assignee**: Development Team  

---

## **PROBLEM SUMMARY**
Authentication middleware consistently rejects valid API keys despite perfect SHA256 hash matches between registration and authentication.

## **SYMPTOMS**
-  Merchant registration: **WORKING** (generates valid API keys with `sk_` prefix)
-  Hash generation: **WORKING** (SHA256 consistent across registration/auth)
-  Database storage: **WORKING** (hashes stored correctly)
-  Authentication middleware: **FAILING** (rejects valid API keys)
-  Protected endpoints: **BLOCKED** (wallet setup, payments fail with 401)

## **REPRODUCTION STEPS**
1. Register merchant: `POST /api/v1/merchants/register`
2. Extract API key from response (e.g., `sk_f1B326qkjTAuEWjDR1adqKJ0y8dVBbDC`)
3. Calculate SHA256 hash: `8514a2a886d03ab540c6d3a95daf99daa50b298dd66959f665054ee155226e48`
4. Verify hash in database: **MATCHES PERFECTLY**
5. Use API key for authentication: `PUT /api/v1/merchants/wallets` with `Authorization: Bearer sk_...`
6. **Result**: `{"error":"Invalid API key","message":"The provided API key is not valid"}`

## **INVESTIGATION RESULTS**

### ** VERIFIED WORKING**
- Database connection: OK
- Migration status: All applied
- Hash algorithm: SHA256 consistent
- API key format: Valid `sk_` prefix
- Merchant record: Active, sandbox mode enabled
- Database query: Direct hash lookup works in psql

### ** SUSPECTED ROOT CAUSES**
1. **Service instantiation issue**: MerchantService not properly initialized in AppState
2. **Connection pool issue**: Database pool not shared correctly between registration and auth
3. **Middleware execution order**: Auth middleware not receiving correct service instance
4. **Async/await issue**: Race condition in service method execution
5. **Logging disabled**: No debug logs appearing despite tracing statements

### **TECHNICAL DETAILS**
- **File**: `/backend/src/services/merchant_service.rs:160-188`
- **Method**: `authenticate(api_key: &str)`
- **Query**: Direct hash lookup with `WHERE api_key_hash = $1 AND is_active = true`
- **Expected**: Returns merchant record
- **Actual**: Returns `ServiceError::InvalidApiKey`

## **ENVIRONMENT**
- **Database**: PostgreSQL (restarted)
- **Backend**: Rust/Axum (clean rebuild)
- **Test Merchant ID**: 84
- **Test API Key**: `sk_f1B326qkjTAuEWjDR1adqKJ0y8dVBbDC`
- **Expected Hash**: `8514a2a886d03ab540c6d3a95daf99daa50b298dd66959f665054ee155226e48`

## **IMPACT**
- **Severity**: CRITICAL - Blocks all authenticated operations
- **Affected Features**: Wallet setup, payment creation, merchant dashboard
- **User Impact**: Merchants cannot use the system after registration
- **Business Impact**: Complete system unusable for production

## **NEXT ACTIONS**
1. **Enable debug logging** to see authentication flow
2. **Add service initialization logging** to verify AppState setup
3. **Test direct service method call** bypassing middleware
4. **Check middleware execution order** in route configuration
5. **Verify connection pool sharing** between services

## **WORKAROUND**
None available - authentication is required for all merchant operations.

## **ACCEPTANCE CRITERIA**
- [ ] API keys generated during registration can authenticate successfully
- [ ] Protected endpoints (wallet setup) return 200 instead of 401
- [ ] Complete E2E workflow passes without authentication errors
- [ ] Debug logs show successful authentication flow

---
**Last Updated**: 2026-01-26T20:36:00Z  
**Debug Script**: `/home/vibes/crypto-payment-gateway/debug-auth.sh`
