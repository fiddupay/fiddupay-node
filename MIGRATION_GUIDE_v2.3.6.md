# FidduPay Node.js SDK v2.3.6 Migration Guide

## Overview

Version 2.3.6 introduces **API Centralization** - a major architectural improvement that organizes all endpoints under logical prefixes while maintaining complete backward compatibility for SDK users.

## What Changed

### API Endpoint Organization
- **Merchant endpoints**: Now centralized under `/api/v1/merchant/`
- **Admin endpoints**: Organized under `/api/v1/admin/`
- **Sandbox endpoints**: Moved to `/api/v1/merchant/sandbox/`
- **Security endpoints**: Organized under `/api/v1/merchant/security/`

### SDK Internal Updates
- All internal endpoint paths updated automatically
- Enhanced TypeScript definitions
- Improved error handling and validation
- Better response type definitions

## Migration Steps

### Step 1: Update the SDK

```bash
npm update @fiddupay/fiddupay-node
```

### Step 2: Verify Version

```bash
npm list @fiddupay/fiddupay-node
```

Should show: `@fiddupay/fiddupay-node@2.3.6`

### Step 3: No Code Changes Required!

Your existing code continues to work unchanged:

```typescript
// This code works exactly the same in v2.3.6
import { FidduPayClient } from '@fiddupay/fiddupay-node';

const client = new FidduPayClient({
  apiKey: 'sk_test_your_api_key',
  environment: 'sandbox'
});

// All these methods work unchanged
const payment = await client.payments.create({
  amount_usd: '100.50',
  crypto_type: 'ETH',
  description: 'Order #12345'
});

const profile = await client.merchants.getProfile();
const balance = await client.merchants.getBalance();
const refund = await client.refunds.create({
  paymentId: 'pay_123',
  amount: '50.25'
});
```

## What's New in v2.3.6

### Enhanced Features
- **Better Organization**: Logical endpoint grouping for improved maintainability
- **Enhanced Security**: Role-based access control with proper authentication boundaries
- **Improved Performance**: Optimized request routing and response handling
- **Better Documentation**: Updated API reference with new endpoint structure

### New Capabilities
- **Enhanced Analytics**: More detailed reporting and data export options
- **Improved Security Monitoring**: Real-time threat detection and alerts
- **Better Wallet Management**: Enhanced wallet configuration and monitoring
- **Advanced Audit Logging**: Comprehensive activity tracking

### TypeScript Improvements
- **Better Type Definitions**: More accurate response types
- **Enhanced IntelliSense**: Improved code completion and documentation
- **Stricter Validation**: Better compile-time error detection

## Verification Steps

### 1. Test Basic Operations

```typescript
// Test payment creation
const payment = await client.payments.create({
  amount_usd: '1.00',
  crypto_type: 'ETH',
  description: 'Test payment'
});
console.log('Payment created:', payment.id);

// Test merchant profile
const profile = await client.merchants.getProfile();
console.log('Profile loaded:', profile.merchant_id);

// Test balance retrieval
const balance = await client.merchants.getBalance();
console.log('Balance:', balance);
```

### 2. Verify Webhook Handling

```typescript
// Webhook handling remains unchanged
app.post('/webhooks/fiddupay', express.raw({type: 'application/json'}), (req, res) => {
  const signature = req.headers['fiddupay-signature'] as string;
  
  try {
    const event = client.webhooks.constructEvent(
      req.body,
      signature,
      'your-webhook-secret'
    );
    
    // Handle events as before
    console.log('Event received:', event.type);
    res.status(200).send('OK');
  } catch (error) {
    res.status(400).send('Invalid signature');
  }
});
```

### 3. Test Error Handling

```typescript
try {
  const payment = await client.payments.create({
    amount_usd: 'invalid',
    crypto_type: 'ETH'
  });
} catch (error) {
  // Error handling works the same
  console.log('Error type:', error.constructor.name);
  console.log('Error message:', error.message);
}
```

## Breaking Changes

**None!** This release maintains complete backward compatibility. All existing method signatures, response formats, and error handling remain unchanged.

## Performance Improvements

- **Faster Response Times**: Optimized endpoint routing
- **Better Caching**: Improved response caching strategies
- **Reduced Latency**: More efficient request processing
- **Enhanced Reliability**: Better error recovery and retry logic

## Security Enhancements

- **10/10 Security Score Maintained**: All existing security protections intact
- **Enhanced Authentication**: Improved role-based access control
- **Better Rate Limiting**: More sophisticated rate limiting algorithms
- **Advanced Threat Detection**: Real-time security monitoring

## Troubleshooting

### Common Issues

#### 1. Import Errors
```typescript
// If you see import errors, ensure you're using the correct import
import { FidduPayClient } from '@fiddupay/fiddupay-node';
// Not: import FidduPayClient from '@fiddupay/fiddupay-node';
```

#### 2. TypeScript Errors
```bash
# Update TypeScript definitions
npm install --save-dev @types/node@latest
```

#### 3. Environment Issues
```typescript
// Ensure environment is correctly set
const client = new FidduPayClient({
  apiKey: process.env.FIDDUPAY_API_KEY,
  environment: process.env.NODE_ENV === 'production' ? 'production' : 'sandbox'
});
```

### Getting Help

If you encounter any issues:

1. **Check the Documentation**: [https://docs.fiddupay.com](https://docs.fiddupay.com)
2. **Review Examples**: Check the `/examples` directory in the SDK
3. **Open an Issue**: [GitHub Issues](https://github.com/fiddupay/fiddupay-node/issues)
4. **Contact Support**: support@fiddupay.com

## Rollback Instructions

If you need to rollback to the previous version:

```bash
npm install @fiddupay/fiddupay-node@2.3.5
```

However, we recommend staying on v2.3.6 for the latest features and security improvements.

## Next Steps

1. **Update your SDK**: `npm update @fiddupay/fiddupay-node`
2. **Test your integration**: Run your existing tests to verify everything works
3. **Review new features**: Explore the enhanced capabilities in v2.3.6
4. **Update documentation**: Update any internal documentation to reference v2.3.6

## Summary

Version 2.3.6 is a **seamless upgrade** that:
- ✅ Requires no code changes
- ✅ Maintains all existing functionality
- ✅ Adds new capabilities and improvements
- ✅ Enhances security and performance
- ✅ Provides better developer experience

The API centralization is completely transparent to SDK users while providing a more organized and maintainable backend architecture.

---

**Questions?** Contact us at support@fiddupay.com or open an issue on GitHub.