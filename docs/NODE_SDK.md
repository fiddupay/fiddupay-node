# FidduPay Node.js SDK Development Guide

**Official Node.js SDK for FidduPay Cryptocurrency Payment Gateway**

## 🎯 Overview

The FidduPay Node.js SDK provides a simple, secure way to integrate cryptocurrency payments into Node.js applications. Built to work seamlessly with the Rust backend API.

---

## 📦 SDK Architecture

### Core Components
```
fiddupay-node/
├── src/
│   ├── index.ts              # Main SDK export
│   ├── client.ts             # HTTP client wrapper
│   ├── resources/            # API resource classes
│   │   ├── payments.ts       # Payment operations
│   │   ├── merchants.ts      # Merchant management
│   │   ├── webhooks.ts       # Webhook utilities
│   │   ├── analytics.ts      # Analytics and reporting
│   │   └── refunds.ts        # Refund operations
│   ├── types/                # TypeScript definitions
│   │   ├── payments.ts       # Payment types
│   │   ├── merchants.ts      # Merchant types
│   │   ├── common.ts         # Shared types
│   │   └── index.ts          # Type exports
│   ├── utils/                # Utility functions
│   │   ├── validation.ts     # Input validation
│   │   ├── crypto.ts         # Cryptographic utilities
│   │   └── webhooks.ts       # Webhook verification
│   └── errors/               # Custom error classes
│       ├── api-error.ts      # API error handling
│       └── validation-error.ts
├── tests/                    # Test suites
├── examples/                 # Usage examples
├── docs/                     # SDK documentation
└── package.json
```

---

## 🚀 SDK Features

### ✅ Core Functionality
- **Payment Processing**: Create, retrieve, and manage payments
- **Webhook Handling**: Secure webhook verification and parsing
- **Merchant Management**: Account settings and configuration
- **Analytics**: Transaction reporting and insights
- **Refund Processing**: Full and partial refund support
- **TypeScript Support**: Full type definitions included
- **Error Handling**: Comprehensive error types and messages
- **Retry Logic**: Automatic retry for failed requests
- **Rate Limiting**: Built-in rate limit handling

### 🔒 Security Features
- **API Key Management**: Secure key storage and rotation
- **Webhook Verification**: HMAC signature validation
- **Input Sanitization**: Prevent injection attacks
- **TLS/SSL**: Encrypted communication with backend
- **Request Signing**: Optional request signing for enhanced security

---

## 📋 API Reference

### Installation
```bash
npm install fiddupay-node
# or
yarn add fiddupay-node
```

### Basic Usage
```typescript
import FidduPay from 'fiddupay-node';

const fiddupay = new FidduPay({
  apiKey: 'sk_test_...',
  environment: 'sandbox' // or 'production'
});
```

### Payment Operations
```typescript
// Create a payment
const payment = await fiddupay.payments.create({
  amount_usd: '100.00',
  crypto_type: 'USDT_ETH',
  description: 'Order #12345',
  metadata: {
    order_id: '12345',
    customer_id: 'cust_abc123'
  },
  expiration_minutes: 30
});

// Retrieve a payment
const payment = await fiddupay.payments.retrieve('pay_1234567890');

// List payments
const payments = await fiddupay.payments.list({
  limit: 20,
  status: 'confirmed',
  crypto_type: 'USDT_ETH'
});
```

### Webhook Handling
```typescript
import express from 'express';

const app = express();

app.post('/webhooks/fiddupay', express.raw({type: 'application/json'}), (req, res) => {
  const sig = req.headers['fiddupay-signature'];
  
  try {
    const event = fiddupay.webhooks.constructEvent(
      req.body,
      sig,
      process.env.FIDDUPAY_WEBHOOK_SECRET
    );
    
    switch (event.type) {
      case 'payment.confirmed':
        console.log('Payment confirmed:', event.data);
        break;
      case 'payment.failed':
        console.log('Payment failed:', event.data);
        break;
    }
    
    res.json({received: true});
  } catch (err) {
    console.log('Webhook signature verification failed:', err.message);
    res.status(400).send('Webhook signature verification failed');
  }
});
```

### Merchant Operations
```typescript
// Get merchant profile
const merchant = await fiddupay.merchants.retrieve();

// Update webhook URL
await fiddupay.merchants.updateWebhook({
  url: 'https://example.com/webhooks/fiddupay'
});

// Get account balance
const balance = await fiddupay.merchants.getBalance();

// Configure wallet addresses
await fiddupay.merchants.setWallets({
  USDT_ETH: '0x742d35Cc6634C0532925a3b8D4C0532925a3b8D4',
  SOL: '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM'
});
```

### Analytics
```typescript
// Get analytics data
const analytics = await fiddupay.analytics.retrieve({
  start_date: '2026-01-01',
  end_date: '2026-01-31',
  granularity: 'day'
});

// Export analytics
const exportData = await fiddupay.analytics.export({
  format: 'csv',
  start_date: '2026-01-01',
  end_date: '2026-01-31'
});
```

### Refund Operations
```typescript
// Create a refund
const refund = await fiddupay.refunds.create({
  payment_id: 'pay_1234567890',
  amount: '50.00', // Partial refund
  reason: 'Customer request'
});

// Retrieve refund status
const refund = await fiddupay.refunds.retrieve('ref_1234567890');
```

---

## 🛠️ Development Specifications

### TypeScript Definitions
```typescript
// Core SDK interface
interface FidduPayConfig {
  apiKey: string;
  environment?: 'sandbox' | 'production';
  apiVersion?: string;
  timeout?: number;
  maxRetries?: number;
  baseURL?: string;
}

// Payment types
interface CreatePaymentRequest {
  amount_usd: string;
  crypto_type: CryptoType;
  description?: string;
  metadata?: Record<string, any>;
  expiration_minutes?: number;
  webhook_url?: string;
}

interface Payment {
  payment_id: string;
  amount_usd: string;
  crypto_amount: string;
  crypto_type: CryptoType;
  status: PaymentStatus;
  deposit_address: string;
  transaction_hash?: string;
  confirmations?: number;
  created_at: string;
  confirmed_at?: string;
  expires_at: string;
  description?: string;
  metadata?: Record<string, any>;
}

type CryptoType = 'SOL' | 'USDT_ETH' | 'USDT_BSC' | 'USDT_POLYGON' | 'USDT_ARBITRUM' | 'USDT_SPL';
type PaymentStatus = 'PENDING' | 'CONFIRMING' | 'CONFIRMED' | 'FAILED' | 'EXPIRED';
```

### Error Handling
```typescript
// Custom error classes
class FidduPayError extends Error {
  type: string;
  code?: string;
  statusCode?: number;
  requestId?: string;
}

class FidduPayAPIError extends FidduPayError {
  constructor(message: string, statusCode: number, code?: string, requestId?: string) {
    super(message);
    this.type = 'api_error';
    this.statusCode = statusCode;
    this.code = code;
    this.requestId = requestId;
  }
}

class FidduPayValidationError extends FidduPayError {
  constructor(message: string, param?: string) {
    super(message);
    this.type = 'validation_error';
    this.param = param;
  }
}
```

### HTTP Client Implementation
```typescript
class FidduPayClient {
  private apiKey: string;
  private baseURL: string;
  private timeout: number;
  private maxRetries: number;

  constructor(config: FidduPayConfig) {
    this.apiKey = config.apiKey;
    this.baseURL = config.baseURL || this.getBaseURL(config.environment);
    this.timeout = config.timeout || 30000;
    this.maxRetries = config.maxRetries || 3;
  }

  async request<T>(
    method: string,
    path: string,
    data?: any,
    options?: RequestOptions
  ): Promise<T> {
    // Implementation with retry logic, error handling, and rate limiting
  }
}
```

---

## 🧪 Testing Strategy

### Unit Tests
```typescript
// Payment creation test
describe('Payments', () => {
  it('should create a payment successfully', async () => {
    const mockPayment = {
      payment_id: 'pay_test123',
      amount_usd: '100.00',
      crypto_type: 'USDT_ETH',
      status: 'PENDING'
    };

    nock('https://api-sandbox.fiddupay.com')
      .post('/v1/payments')
      .reply(201, mockPayment);

    const payment = await fiddupay.payments.create({
      amount_usd: '100.00',
      crypto_type: 'USDT_ETH'
    });

    expect(payment.payment_id).toBe('pay_test123');
  });
});
```

### Integration Tests
```typescript
// Webhook verification test
describe('Webhooks', () => {
  it('should verify webhook signatures correctly', () => {
    const payload = JSON.stringify({ type: 'payment.confirmed' });
    const secret = 'whsec_test123';
    const signature = fiddupay.webhooks.generateSignature(payload, secret);
    
    const event = fiddupay.webhooks.constructEvent(payload, signature, secret);
    expect(event.type).toBe('payment.confirmed');
  });
});
```

---

## 📚 Usage Examples

### Express.js Integration
```typescript
import express from 'express';
import FidduPay from 'fiddupay-node';

const app = express();
const fiddupay = new FidduPay({ apiKey: process.env.FIDDUPAY_API_KEY });

// Create payment endpoint
app.post('/create-payment', async (req, res) => {
  try {
    const payment = await fiddupay.payments.create({
      amount_usd: req.body.amount,
      crypto_type: req.body.crypto_type,
      description: req.body.description
    });
    
    res.json({ payment });
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});
```

### Next.js API Route
```typescript
// pages/api/payments.ts
import type { NextApiRequest, NextApiResponse } from 'next';
import FidduPay from 'fiddupay-node';

const fiddupay = new FidduPay({ apiKey: process.env.FIDDUPAY_API_KEY });

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  if (req.method === 'POST') {
    try {
      const payment = await fiddupay.payments.create(req.body);
      res.status(201).json(payment);
    } catch (error) {
      res.status(400).json({ error: error.message });
    }
  }
}
```

### NestJS Service
```typescript
import { Injectable } from '@nestjs/common';
import FidduPay from 'fiddupay-node';

@Injectable()
export class PaymentService {
  private fiddupay: FidduPay;

  constructor() {
    this.fiddupay = new FidduPay({
      apiKey: process.env.FIDDUPAY_API_KEY,
      environment: process.env.NODE_ENV === 'production' ? 'production' : 'sandbox'
    });
  }

  async createPayment(data: CreatePaymentRequest) {
    return await this.fiddupay.payments.create(data);
  }
}
```

---

## 🔧 Development Roadmap

### Phase 1: Core SDK (Q1 2026)
- ✅ Basic payment operations
- ✅ Webhook handling
- ✅ TypeScript definitions
- ✅ Error handling
- ✅ Unit tests

### Phase 2: Advanced Features (Q2 2026)
- 🔄 Merchant management
- 🔄 Analytics integration
- 🔄 Refund operations
- 🔄 Rate limiting
- 🔄 Retry logic

### Phase 3: Enterprise Features (Q3 2026)
- 📋 Request signing
- 📋 Advanced webhook features
- 📋 Bulk operations
- 📋 Custom middleware support
- 📋 Performance optimizations

### Phase 4: Ecosystem Integration (Q4 2026)
- 📋 Framework-specific packages
- 📋 CLI tools
- 📋 Development plugins
- 📋 Monitoring integrations
- 📋 Advanced documentation

---

## 📖 Documentation Structure

### SDK Documentation
```
docs/
├── README.md                 # Getting started guide
├── API_REFERENCE.md          # Complete API reference
├── EXAMPLES.md              # Usage examples
├── WEBHOOKS.md              # Webhook integration guide
├── ERROR_HANDLING.md        # Error handling guide
├── TESTING.md               # Testing strategies
├── MIGRATION.md             # Version migration guides
└── CHANGELOG.md             # Version history
```

### Code Examples Repository
```
examples/
├── express-basic/           # Basic Express.js integration
├── nextjs-ecommerce/        # Next.js e-commerce example
├── nestjs-microservice/     # NestJS microservice
├── webhook-handler/         # Standalone webhook handler
├── bulk-operations/         # Bulk payment processing
└── testing-examples/        # Testing implementations
```

---

## 🚀 Distribution Strategy

### NPM Package
- **Package Name**: `fiddupay-node`
- **Scope**: `@fiddupay/node` (future consideration)
- **Versioning**: Semantic versioning (semver)
- **License**: MIT License
- **Keywords**: cryptocurrency, payments, blockchain, fintech, api

### GitHub Repository
- **Repository**: `fiddupay/fiddupay-node`
- **Branches**: `main`, `develop`, `release/*`
- **CI/CD**: GitHub Actions for testing and publishing
- **Documentation**: GitHub Pages for SDK docs

### Developer Experience
- **TypeScript**: Full type definitions included
- **IDE Support**: IntelliSense and autocomplete
- **Debugging**: Source maps and debug logging
- **Examples**: Comprehensive example repository
- **Community**: Discord channel for developer support

---

## 📊 Success Metrics

### Adoption Metrics
- NPM downloads per month
- GitHub stars and forks
- Developer community size
- Integration implementations

### Quality Metrics
- Test coverage (target: >95%)
- Documentation completeness
- Issue resolution time
- Developer satisfaction score

### Performance Metrics
- API response times
- Error rates
- Retry success rates
- Memory usage optimization

---

**Document Version**: 1.0  
**Last Updated**: January 25, 2026  
**Next Review**: March 1, 2026  
**Owner**: TechyTro Software - FidduPay SDK Team
