#  FidduPay SDK v2.3.6 Migration Guide

##  Overview

This guide helps you migrate from previous versions of the FidduPay SDK to v2.3.6, which introduces API centralization with improved endpoint organization and enhanced security.

##  What Changed

### **API Centralization**
All merchant endpoints have been reorganized under the `/api/v1/merchant/` prefix for better structure and maintainability.

### **Key Changes**
- **Merchant endpoints**: Now use `/api/v1/merchant/` prefix
- **Admin endpoints**: Now use `/api/v1/admin/` prefix  
- **Sandbox endpoints**: Now use `/api/v1/merchant/sandbox/` prefix
- **Security endpoints**: Now use `/api/v1/merchant/security/` prefix

##  Migration Options

### **Option 1: SDK Update (Recommended)**

** Easiest Migration**: Update to SDK v2.3.6 - no code changes required!

```bash
npm install @fiddupay/fiddupay-node@2.3.6
```

**Benefits:**
-  No code changes needed
-  Automatic endpoint path updates
-  Enhanced TypeScript support
-  Improved error handling
-  Full backward compatibility

### **Option 2: Direct API Migration**

If you're using direct API calls, update your endpoint URLs:

##  Endpoint Migration Table

| Category | Old Endpoint | New Endpoint | Status |
|----------|--------------|--------------|--------|
| **Merchant Profile** | `/api/v1/merchant/profile` | `/api/v1/merchant/profile` |  Same |
| **Payments** | `/api/v1/merchant/payments` | `/api/v1/merchant/payments` |  Same |
| **Analytics** | `/api/v1/merchant/analytics` | `/api/v1/merchant/analytics` |  Same |
| **Balance** | `/api/v1/merchant/balance` | `/api/v1/merchant/balance` |  Same |
| **Wallets** | `/api/v1/merchant/wallets` | `/api/v1/merchant/wallets` |  Same |
| **Refunds** | `/api/v1/merchant/refunds` | `/api/v1/merchant/refunds` |  Same |
| **Withdrawals** | `/api/v1/merchant/withdrawals` | `/api/v1/merchant/withdrawals` |  Same |
| **Security Events** | `/api/v1/security/events` | `/api/v1/merchant/security/events` |  Updated |
| **Security Alerts** | `/api/v1/security/alerts` | `/api/v1/merchant/security/alerts` |  Updated |
| **Sandbox Enable** | `/api/v1/sandbox/enable` | `/api/v1/merchant/sandbox/enable` |  Updated |
| **Sandbox Simulate** | `/api/v1/sandbox/payments/{id}/simulate` | `/api/v1/merchant/sandbox/payments/{id}/simulate` |  Updated |

##  Step-by-Step Migration

### **Step 1: Update SDK Version**

```bash
# Check current version
npm list @fiddupay/fiddupay-node

# Update to v2.3.6
npm install @fiddupay/fiddupay-node@2.3.6

# Verify installation
npm list @fiddupay/fiddupay-node
```

### **Step 2: Test Your Integration**

Your existing code should work without changes:

```javascript
import { FidduPay } from '@fiddupay/fiddupay-node';

// Initialize (same as before)
const fiddupay = new FidduPay('sk_your_api_key');

// All methods work the same way
const profile = await fiddupay.merchant.getProfile();
const payments = await fiddupay.payments.list();
const balance = await fiddupay.balance.get();

// Security methods (paths updated internally)
const events = await fiddupay.security.getEvents();
const alerts = await fiddupay.security.getAlerts();

// Sandbox methods (paths updated internally)
await fiddupay.sandbox.enable();
await fiddupay.sandbox.simulatePayment('payment_id', 'confirmed');
```

### **Step 3: Update Direct API Calls (If Any)**

If you have any direct API calls, update the endpoints:

```javascript
// Before v2.3.6
const response = await fetch('/api/v1/security/events', {
  headers: { 'Authorization': `Bearer ${apiKey}` }
});

// After v2.3.6
const response = await fetch('/api/v1/merchant/security/events', {
  headers: { 'Authorization': `Bearer ${apiKey}` }
});
```

### **Step 4: Update Webhook URLs (If Using Admin Endpoints)**

Admin endpoints now use session-based authentication:

```javascript
// Before v2.3.6 - Admin endpoints mixed with merchant
// After v2.3.6 - Admin endpoints separated
// Update your webhook configurations if needed
```

### **Step 5: Test in Sandbox Environment**

```javascript
// Test all your critical flows
const fiddupay = new FidduPay('sk_sandbox_key', {
  baseURL: 'http://localhost:8080'
});

// Test payment creation
const payment = await fiddupay.payments.create({
  amount: '10.00',
  currency: 'USD',
  crypto_type: 'SOL',
  description: 'Test payment'
});

// Test other critical methods
const profile = await fiddupay.merchant.getProfile();
const balance = await fiddupay.balance.get();
```

##  Verification Checklist

### **Pre-Migration**
- [ ] Note your current SDK version
- [ ] Backup your current implementation
- [ ] List all FidduPay API calls in your code
- [ ] Identify any direct API calls (non-SDK)

### **During Migration**
- [ ] Update SDK to v2.3.6
- [ ] Run your test suite
- [ ] Test in sandbox environment
- [ ] Update any direct API calls
- [ ] Verify webhook configurations

### **Post-Migration**
- [ ] All existing functionality works
- [ ] No breaking changes in responses
- [ ] Error handling works correctly
- [ ] Authentication still works
- [ ] Webhooks are delivered correctly

##  Common Issues & Solutions

### **Issue 1: Import Errors**

```javascript
//  Old import might cause issues
const FidduPay = require('@fiddupay/fiddupay-node');

//  Use proper import
import { FidduPay } from '@fiddupay/fiddupay-node';
// OR
const { FidduPay } = require('@fiddupay/fiddupay-node');
```

### **Issue 2: TypeScript Errors**

```typescript
//  Update your TypeScript imports
import { 
  FidduPay, 
  MerchantProfile, 
  Payment, 
  SecurityAlert 
} from '@fiddupay/fiddupay-node';
```

### **Issue 3: Direct API Calls**

```javascript
//  Old direct API call
fetch('/api/v1/security/events')

//  Updated direct API call
fetch('/api/v1/merchant/security/events')

//  Better: Use SDK method
fiddupay.security.getEvents()
```

### **Issue 4: Environment Configuration**

```javascript
//  Ensure proper environment setup
const fiddupay = new FidduPay('sk_your_key', {
  baseURL: process.env.NODE_ENV === 'production' 
    ? 'https://api.fiddupay.com'
    : 'http://localhost:8080'
});
```

##  Feature Comparison

### **Before v2.3.6**
```javascript
// Mixed endpoint structure
await fetch('/api/v1/merchant/profile');
await fetch('/api/v1/security/events');
await fetch('/api/v1/sandbox/enable');
```

### **After v2.3.6**
```javascript
// Organized endpoint structure
await fetch('/api/v1/merchant/profile');
await fetch('/api/v1/merchant/security/events');
await fetch('/api/v1/merchant/sandbox/enable');

// Or better, use SDK methods
await fiddupay.merchant.getProfile();
await fiddupay.security.getEvents();
await fiddupay.sandbox.enable();
```

##  Security Considerations

### **Authentication**
- **Merchant endpoints**: Continue using API key authentication
- **Admin endpoints**: Now use session-based authentication
- **Public endpoints**: No authentication required

### **Rate Limiting**
- Same rate limits apply: 60 requests/minute, burst 100/10s
- Rate limiting maintained across all endpoint categories
- Headers: `X-RateLimit-Remaining`, `X-RateLimit-Reset`

##  Testing Your Migration

### **Unit Tests**

```javascript
describe('FidduPay SDK v2.3.6 Migration', () => {
  const fiddupay = new FidduPay('sk_test_key');

  test('merchant profile works', async () => {
    const profile = await fiddupay.merchant.getProfile();
    expect(profile).toBeDefined();
    expect(profile.id).toBeDefined();
  });

  test('payments work', async () => {
    const payments = await fiddupay.payments.list();
    expect(Array.isArray(payments)).toBe(true);
  });

  test('security events work', async () => {
    const events = await fiddupay.security.getEvents();
    expect(Array.isArray(events)).toBe(true);
  });

  test('sandbox methods work', async () => {
    await expect(fiddupay.sandbox.enable()).resolves.not.toThrow();
  });
});
```

### **Integration Tests**

```javascript
describe('Integration Tests', () => {
  test('complete payment flow', async () => {
    // Create payment
    const payment = await fiddupay.payments.create({
      amount: '10.00',
      currency: 'USD',
      crypto_type: 'SOL'
    });

    // Verify payment
    const verified = await fiddupay.payments.verify(payment.id);
    expect(verified).toBeDefined();

    // Check balance
    const balance = await fiddupay.balance.get();
    expect(balance).toBeDefined();
  });
});
```

##  Performance Impact

### **Improvements**
-  **Better route organization** for improved maintainability
-  **Enhanced caching** with organized endpoint structure
-  **Improved monitoring** with clearer metrics
-  **No performance degradation** from changes

### **Metrics**
- **Response times**: Same or better
- **Error rates**: Maintained low levels
- **Throughput**: No impact on request handling
- **Memory usage**: Optimized with better organization

##  Future Compatibility

### **Upcoming Features**
- **Multi-signature wallets**: Enhanced security features
- **GraphQL API**: More flexible data fetching
- **Mobile SDK**: React Native support
- **Advanced analytics**: Real-time insights

### **Backward Compatibility**
- **Method signatures**: Will remain stable
- **Response formats**: Consistent structure maintained
- **Error codes**: Same error handling approach
- **Authentication**: API key format unchanged

##  Getting Help

### **If You Need Assistance**

1. **Check Documentation**: https://docs.fiddupay.com
2. **Review Examples**: See updated code examples
3. **Test in Sandbox**: Use sandbox environment for testing
4. **Contact Support**: support@fiddupay.com

### **Common Support Requests**

**Q: Do I need to change my code?**  
A: No, if you're using the SDK. Just update to v2.3.6.

**Q: Will my API keys still work?**  
A: Yes, all existing API keys continue to work.

**Q: Are there any new features?**  
A: Enhanced security, better organization, improved TypeScript support.

**Q: What if I have issues?**  
A: Contact support@fiddupay.com with your specific issue.

##  Migration Success

### **You've Successfully Migrated When:**
- [ ] SDK updated to v2.3.6
- [ ] All tests pass
- [ ] No breaking changes in your application
- [ ] Sandbox testing works correctly
- [ ] Production deployment successful
- [ ] All features work as expected

### **Celebrate! **
You're now using the latest FidduPay SDK with:
-  Better organized API structure
-  Enhanced security features
-  Improved developer experience
-  Full backward compatibility
-  Future-ready architecture

---

**Need help with migration? Contact us at support@fiddupay.com**

**© 2026 TechyTro Software - FidduPay v2.3.6 Migration Guide**