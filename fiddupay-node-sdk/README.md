# FidduPay Node.js SDK

Official Node.js SDK for the FidduPay cryptocurrency payment gateway platform with **3-Mode Wallet System**.

[![npm version](https://badge.fury.io/js/@fiddupay/fiddupay-node.svg)](https://www.npmjs.com/package/@fiddupay/fiddupay-node)
[![Build Status](https://github.com/fiddupay/fiddupay-node/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/fiddupay/fiddupay-node/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

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

##  3-Mode Wallet System

FidduPay offers three flexible wallet modes to suit different merchant needs:

### Mode 1: Generate Keys (Fully Managed)
FidduPay generates and manages wallet keys for you. Perfect for merchants who want a hands-off approach.

### Mode 2: Import Keys (Self-Managed)
Import your existing private keys. You maintain control while using FidduPay's infrastructure.

### Mode 3: Address-Only (Customer Wallets)
Customers pay directly from their own wallets to your addresses. No key management required.

## Installation

```bash
npm install @fiddupay/fiddupay-node
```

## Quick Start

```typescript
import { FidduPayClient } from '@fiddupay/fiddupay-node';

const client = new FidduPayClient({
  apiKey: 'sk_test_your_api_key',
  environment: 'sandbox' // or 'production'
});

// USD-based payment
const payment = await client.payments.create({
  amount_usd: '100.50',
  crypto_type: 'USDT_ETH',
  description: 'Premium subscription'
});

// Crypto-based payment
const cryptoPayment = await client.payments.create({
  amount: '2.5',
  crypto_type: 'SOL',
  description: 'Premium subscription'
});

// Address-Only Payment
const addressOnlyPayment = await client.payments.createAddressOnly({
  crypto_type: 'ETH',
  merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
  requested_amount: '0.05'
});
```

## Supported Cryptocurrencies

FidduPay supports 10 cryptocurrencies across 5 major blockchain networks:

| Currency | Network | Type |
|----------|---------|------|
| SOL | Solana | Native |
| USDT | Solana | SPL Token |
| ETH | Ethereum | Native |
| USDT | Ethereum | ERC-20 |
| BNB | BSC | Native |
| USDT | BSC | BEP-20 |
| MATIC | Polygon | Native |
| USDT | Polygon | Polygon |
| ARB | Arbitrum | Native |
| USDT | Arbitrum | Arbitrum |

## Configuration

### Basic Configuration

```typescript
import { FidduPayClient } from '@fiddupay/fiddupay-node';

const client = new FidduPayClient({
  apiKey: 'your_api_key',
  environment: 'production', // 'sandbox' or 'production'
  timeout: 30000, // Request timeout in milliseconds
  maxRetries: 3 // Maximum number of retries
});
```

### Environment Variables

You can also configure using environment variables:

```bash
FIDDUPAY_API_KEY=your_api_key
FIDDUPAY_ENVIRONMENT=production
FIDDUPAY_TIMEOUT=30000
FIDDUPAY_MAX_RETRIES=3
```

## API Reference

###  Wallet Management

#### Mode 1: Generate Wallet Keys

```typescript
// Generate new wallet for a crypto type
const wallet = await client.wallets.generate({
  crypto_type: 'ETH'
});

console.log('Address:', wallet.address);
console.log('Private Key:', wallet.private_key); // Store securely!
```

#### Mode 2: Import Existing Keys

```typescript
// Import your existing private key
const wallet = await client.wallets.import({
  crypto_type: 'SOL',
  private_key: 'your_base58_private_key_here'
});

console.log('Imported address:', wallet.address);
```

#### Get Wallet Configuration

```typescript
const config = await client.wallets.getConfig();
console.log('Wallet modes:', config.supported_modes);
console.log('Current mode:', config.current_mode);
```

###  Payments

#### Standard Payments

```typescript
// USD-based payment
const payment = await client.payments.create({
  amount_usd: '100.50',
  crypto_type: 'USDT_ETH',
  description: 'Order payment',
  metadata: {
    orderId: 'order-123'
  },
  webhook_url: 'https://your-site.com/webhooks/fiddupay',
  expiration_minutes: 30
});

// Crypto-based payment
const cryptoPayment = await client.payments.create({
  amount: '2.5',
  crypto_type: 'SOL',
  description: 'Order payment',
  metadata: {
    orderId: 'order-124'
  },
  webhook_url: 'https://your-site.com/webhooks/fiddupay',
  expiration_minutes: 30
});
```

#### Address-Only Payments

```typescript
// Create address-only payment request
const addressPayment = await client.payments.createAddressOnly({
  crypto_type: 'ETH',
  merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
  requested_amount: '0.05',
  customer_pays_fee: true // or false for merchant pays fee
});

console.log('Payment ID:', addressPayment.payment_id);
console.log('Customer amount:', addressPayment.customer_amount);
console.log('Instructions:', addressPayment.customer_instructions);
```

#### Get Payment

```typescript
const payment = await client.payments.get('payment_123');
```

#### List Payments

```typescript
const payments = await client.payments.list({
  limit: 20,
  offset: 0,
  status: 'pending',
  currency: 'USDT',
  network: 'ethereum'
});
```

#### Cancel Payment

```typescript
const payment = await client.payments.cancel('payment_123');
```

### Merchants

#### Register Merchant

```typescript
const merchant = await client.merchants.register({
  business_name: 'My Business',
  email: 'merchant@example.com',
  password: 'secure_password',
  website_url: 'https://mybusiness.com'
});
```

#### Get API Keys

```typescript
const keys = await client.merchants.getApiKeys();
```

### Webhooks

#### Verify Webhook Signature

```typescript
import { Webhooks } from '@fiddupay/fiddupay-node';

const isValid = Webhooks.verifySignature(
  payload,
  signature,
  webhookSecret
);
```

## Error Handling

The SDK provides comprehensive error handling:

```typescript
import { 
  FidduPayAPIError, 
  FidduPayAuthenticationError,
  FidduPayRateLimitError 
} from '@fiddupay/fiddupay-node';

try {
  const payment = await client.payments.create(paymentData);
} catch (error) {
  if (error instanceof FidduPayAuthenticationError) {
    console.error('Authentication failed:', error.message);
  } else if (error instanceof FidduPayRateLimitError) {
    console.error('Rate limit exceeded. Retry after:', error.retryAfter);
  } else if (error instanceof FidduPayAPIError) {
    console.error('API error:', error.message, 'Status:', error.statusCode);
  }
}
```

## Webhook Integration

### Express.js Example

```typescript
import express from 'express';
import { Webhooks } from '@fiddupay/fiddupay-node';

const app = express();

app.post('/webhooks/fiddupay', express.raw({type: 'application/json'}), (req, res) => {
  const signature = req.headers['x-fiddupay-signature'] as string;
  const payload = req.body;
  
  if (!Webhooks.verifySignature(payload, signature, process.env.WEBHOOK_SECRET)) {
    return res.status(400).send('Invalid signature');
  }
  
  const event = JSON.parse(payload.toString());
  
  switch (event.type) {
    case 'payment.completed':
      console.log('Payment completed:', event.data.id);
      break;
    case 'payment.failed':
      console.log('Payment failed:', event.data.id);
      break;
  }
  
  res.status(200).send('OK');
});
```

## Testing

### Unit Tests

```bash
npm test
```

### Integration Tests

```bash
npm run test:integration
```

### Coverage Report

```bash
npm run test:coverage
```

## TypeScript Support

The SDK is written in TypeScript and includes full type definitions:

```typescript
import { 
  FidduPayClient, 
  Payment, 
  PaymentStatus,
  CreatePaymentRequest 
} from '@fiddupay/fiddupay-node';

const client = new FidduPayClient({
  apiKey: 'your_api_key',
  environment: 'production'
});

// Full type safety
const paymentRequest: CreatePaymentRequest = {
  amount: 100.50,
  currency: 'USDT',
  network: 'ethereum'
};

const payment: Payment = await client.payments.create(paymentRequest);
```

## Examples

### Mode 1: Generate Keys E-commerce

```typescript
import { FidduPayClient } from '@fiddupay/fiddupay-node';

class PaymentService {
  private client: FidduPayClient;
  
  constructor() {
    this.client = new FidduPayClient({
      apiKey: process.env.FIDDUPAY_API_KEY,
      environment: process.env.NODE_ENV === 'production' ? 'production' : 'sandbox'
    });
  }
  
  async setupMerchantWallet() {
    // Generate wallets for all supported crypto types
    const cryptoTypes = ['ETH', 'SOL', 'BNB', 'MATIC', 'ARB'];
    const wallets = {};
    
    for (const crypto of cryptoTypes) {
      wallets[crypto] = await this.client.wallets.generate({
        crypto_type: crypto
      });
    }
    
    return wallets;
  }
  
  async createOrderPayment(order: Order) {
    return await this.client.payments.create({
      amount: order.total,
      currency: 'USDT',
      network: 'ethereum',
      wallet_mode: 'generate_keys',
      description: `Order #${order.id}`,
      metadata: {
        orderId: order.id,
        customerId: order.customerId
      }
    });
  }
}
```

### Mode 2: Import Keys Integration

```typescript
async function setupImportedWallets() {
  const client = new FidduPayClient({
    apiKey: process.env.FIDDUPAY_API_KEY,
    environment: 'production'
  });
  
  // Import your existing Ethereum wallet
  const ethWallet = await client.wallets.import({
    crypto_type: 'ETH',
    private_key: process.env.ETH_PRIVATE_KEY
  });
  
  // Import your existing Solana wallet
  const solWallet = await client.wallets.import({
    crypto_type: 'SOL',
    private_key: process.env.SOL_PRIVATE_KEY
  });
  
  console.log('Imported ETH wallet:', ethWallet.address);
  console.log('Imported SOL wallet:', solWallet.address);
}
```

### Mode 3: Address-Only Payments

```typescript
async function createAddressOnlyPayment(customerOrder: Order) {
  const client = new FidduPayClient({
    apiKey: process.env.FIDDUPAY_API_KEY,
    environment: 'production'
  });
  
  // Create address-only payment where customer pays from their wallet
  const payment = await client.payments.createAddressOnly({
    crypto_type: 'ETH',
    merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    requested_amount: customerOrder.total_eth,
    customer_pays_fee: true // Customer covers network fees
  });
  
  return {
    paymentId: payment.payment_id,
    customerAmount: payment.customer_amount,
    instructions: payment.customer_instructions,
    supportedCurrencies: payment.supported_currencies
  };
}
```

### Fee Toggle System

```typescript
async function createPaymentWithFeeToggle(order: Order, customerPaysFee: boolean) {
  const payment = await client.payments.createAddressOnly({
    crypto_type: 'USDT',
    merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    requested_amount: order.amount,
    customer_pays_fee: customerPaysFee
  });
  
  if (customerPaysFee) {
    console.log(`Customer pays: ${payment.customer_amount} USDT (includes ${payment.processing_fee} USDT fee)`);
  } else {
    console.log(`Customer pays: ${payment.requested_amount} USDT (merchant covers ${payment.processing_fee} USDT fee)`);
  }
  
  return payment;
}
```

## Migration Guide

### From v1.x to v2.x

The main class has been renamed from `FidduPay` to `FidduPayClient`:

```typescript
// v1.x
import { FidduPay } from '@fiddupay/fiddupay-node';
const client = new FidduPay(config);

// v2.x
import { FidduPayClient } from '@fiddupay/fiddupay-node';
const client = new FidduPayClient(config);

// Backward compatibility alias is available
import { FidduPay } from '@fiddupay/fiddupay-node';
const client = new FidduPay(config); // Still works
```

## Support

- Documentation: [https://docs.fiddupay.com](https://docs.fiddupay.com)
- API Reference: [https://docs.fiddupay.com/api](https://docs.fiddupay.com/api)
- Support Email: support@fiddupay.com
- GitHub Issues: [https://github.com/fiddupay/fiddupay-node/issues](https://github.com/fiddupay/fiddupay-node/issues)

## License

MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Changelog

### v2.0.0
- Renamed main class from `FidduPay` to `FidduPayClient`
- Added backward compatibility alias
- Updated API endpoints to match v2.0 specification
- Enhanced error handling
- Improved TypeScript support

### v1.0.0
- Initial release
- Basic payment functionality
- Webhook support
- TypeScript definitions
