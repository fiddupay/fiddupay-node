# FidduPay Node.js SDK

[![npm version](https://img.shields.io/npm/v/@fiddupay/fiddupay-node.svg?style=flat-square&cacheSeconds=300)](https://www.npmjs.com/package/@fiddupay/fiddupay-node)
[![npm downloads](https://img.shields.io/npm/dm/@fiddupay/fiddupay-node.svg?style=flat-square)](https://www.npmjs.com/package/@fiddupay/fiddupay-node)
[![Build Status](https://github.com/fiddupay/fiddupay-node/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/fiddupay/fiddupay-node/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Official Node.js SDK for the FidduPay cryptocurrency payment gateway platform with **3-Mode Wallet System**.

## Daily Volume Limits

- **Non-KYC Merchants**: $1,000 USD daily volume limit (combined deposits + withdrawals)
- **KYC Verified Merchants**: No daily volume limits
- **Reset**: Daily limits reset at midnight UTC
- **Tracking**: Real-time volume tracking across all transaction types

```typescript
// Check your daily volume status
const profile = await client.merchants.getProfile();
console.log('KYC Status:', profile.kyc_verified);
console.log('Daily Volume Remaining:', profile.daily_volume_remaining);
```

## 3-Mode Wallet System

FidduPay offers three flexible wallet modes to suit different merchant needs:

### Mode 1: Generate Keys (Fully Managed)
FidduPay generates and manages wallet keys for you. Perfect for merchants who want a hands-off approach.

### Mode 2: Import Keys (Self-Managed)
Import your existing private keys. You maintain control while using FidduPay's infrastructure.

### Mode 3: Address-Only (Customer Wallets)
Customers pay directly from their own wallets to your addresses. No key management required.

## Quick Start

### Installation

```bash
npm install @fiddupay/fiddupay-node
```

### Basic Usage

```typescript
import { FidduPayClient } from '@fiddupay/fiddupay-node';

const client = new FidduPayClient({
  apiKey: 'sk_test_your_api_key',
  environment: 'sandbox' // or 'production'
});

// Create a payment
const payment = await client.payments.create({
  amount_usd: '100.50',
  crypto_type: 'ETH',
  description: 'Order #12345'
});

console.log('Payment created:', payment.id);
```

## Features

- **Payment Processing**: Create, retrieve, list, and cancel payments
- **Webhook Verification**: Secure HMAC-SHA256 signature validation
- **Merchant Management**: Profile, balance, and wallet configuration
- **Refund Operations**: Create and track refunds
- **Analytics**: Data retrieval and export
- **Security**: Input validation, rate limiting, retry logic
- **TypeScript**: Full type definitions included
- **Daily Volume Limits**: KYC status and volume tracking support

## Configuration

```typescript
const client = new FidduPayClient({
  apiKey: 'sk_test_your_api_key',
  environment: 'sandbox', // 'sandbox' or 'production'
  timeout: 30000, // Request timeout in milliseconds
  retries: 3, // Number of retry attempts
  baseURL: 'https://api.fiddupay.com' // Custom API base URL
});
```

## Payment Operations

### Create Payment

```typescript
const payment = await client.payments.create({
  amount_usd: '100.50',
  crypto_type: 'ETH',
  description: 'Order #12345',
  metadata: {
    orderId: '12345',
    customerId: 'cust_123'
  }
});
```

### Retrieve Payment

```typescript
const payment = await client.payments.retrieve('pay_123');
```

### List Payments

```typescript
const payments = await client.payments.list({
  limit: 10,
  status: 'completed'
});
```

## Webhook Handling

```typescript
import express from 'express';

const app = express();

app.post('/webhooks/fiddupay', express.raw({type: 'application/json'}), (req, res) => {
  const signature = req.headers['fiddupay-signature'] as string;
  
  try {
    const event = client.webhooks.constructEvent(
      req.body,
      signature,
      'your-webhook-secret'
    );

    switch (event.type) {
      case 'payment.completed':
        console.log('Payment completed:', event.data);
        break;
      case 'payment.failed':
        console.log('Payment failed:', event.data);
        break;
    }

    res.status(200).send('OK');
  } catch (error) {
    console.error('Webhook error:', error);
    res.status(400).send('Invalid signature');
  }
});
```

## Merchant Operations

```typescript
// Get merchant profile (includes KYC status and daily volume)
const profile = await client.merchants.getProfile();
console.log('KYC Verified:', profile.kyc_verified);
console.log('Daily Volume Remaining:', profile.daily_volume_remaining);

// Get account balance
const balance = await client.merchants.getBalance();

// Configure wallet
await client.merchants.configureWallet({
  currency: 'USDT',
  network: 'ethereum',
  address: '0x742d35Cc6634C0532925a3b8D4C9db96590c6C87'
});
```

## Refund Operations

```typescript
// Create refund
const refund = await client.refunds.create({
  paymentId: 'pay_123',
  amount: '50.25',
  reason: 'customer_request'
});

// List refunds
const refunds = await client.refunds.list({
  paymentId: 'pay_123'
});
```

## Analytics

```typescript
// Get analytics data
const analytics = await client.analytics.getData({
  startDate: '2026-01-01',
  endDate: '2026-01-31',
  metrics: ['revenue', 'transaction_count']
});

// Export data
const exportData = await client.analytics.exportData({
  format: 'csv',
  startDate: '2026-01-01',
  endDate: '2026-01-31'
});
```

## Error Handling

```typescript
import { FidduPayError, APIError, AuthenticationError, ValidationError, RateLimitError } from '@fiddupay/fiddupay-node';

try {
  const payment = await client.payments.create({
    amount_usd: '100',
    crypto_type: 'ETH'
  });
} catch (error) {
  if (error instanceof AuthenticationError) {
    console.error('Invalid API key');
  } else if (error instanceof ValidationError) {
    console.error('Invalid parameters:', error.details);
  } else if (error instanceof RateLimitError) {
    console.error('Rate limit exceeded, retry after:', error.retryAfter);
  } else if (error instanceof APIError) {
    console.error('API error:', error.message);
  }
}
```

## Security

- **API Key Security**: Never expose API keys in client-side code
- **Webhook Verification**: Always verify webhook signatures
- **HTTPS Only**: All API calls use HTTPS encryption
- **Input Validation**: All inputs are validated and sanitized

## Supported Cryptocurrencies

**5 Major Blockchain Networks:**
- **Solana** - SOL + USDT (SPL)
- **Ethereum** - ETH + USDT (ERC-20)
- **Binance Smart Chain** - BNB + USDT (BEP-20)
- **Polygon** - MATIC + USDT
- **Arbitrum** - ARB + USDT

**Total: 10 cryptocurrency options across 5 blockchains**

## API Reference

For complete API documentation, visit: [https://docs.fiddupay.com](https://docs.fiddupay.com)

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/new-feature`
3. Commit changes: `git commit -am 'Add new feature'`
4. Push to branch: `git push origin feature/new-feature`
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Support

- **Documentation**: [https://docs.fiddupay.com](https://docs.fiddupay.com)
- **Issues**: [GitHub Issues](https://github.com/fiddupay/fiddupay-node/issues)
- **Email**: support@fiddupay.com

---

**Made with care by the FidduPay Team**
