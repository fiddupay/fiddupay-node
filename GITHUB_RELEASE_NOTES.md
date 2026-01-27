# 🚀 FidduPay Node.js SDK v2.3.4 - Complete Backend API Coverage

## 🎉 Major Release - Production Ready!

This release provides **complete coverage of all backend merchant API endpoints** with comprehensive security measures and extensive testing.

## ✨ What's New

### 🔧 **Complete API Coverage (45+ Endpoints)**
- ✅ **Merchant Management**: Registration, login, profile, API keys
- ✅ **Payment Processing**: Create, retrieve, cancel, list payments  
- ✅ **Wallet Operations**: Single/batch wallet configuration
- ✅ **Withdrawal Management**: Create, process, cancel withdrawals
- ✅ **Balance & History**: Real-time balances and transaction history
- ✅ **Security Features**: IP whitelist, audit logs, 2FA support
- ✅ **Webhook Integration**: Signature validation and event handling
- ✅ **Analytics & Reporting**: Transaction analytics and insights
- ✅ **Sandbox Testing**: Complete testing utilities

### 🔒 **Security Enhancements**
- **Contact Form Security**: 30+ malicious pattern detection and blocking
- **Input Sanitization**: XSS and injection prevention
- **Admin Endpoint Exclusion**: Zero admin endpoints in SDK (merchant-only access)
- **Public Endpoint Audit**: Verified no sensitive data exposure

### 🧪 **Comprehensive Testing**
- **189 Tests Passing** across 8 comprehensive test suites
- **Contact Form Tests**: Security validation and error handling
- **Integration Tests**: End-to-end SDK functionality
- **Webhook Tests**: Signature validation and event processing
- **Error Handling**: Comprehensive error scenario coverage

## 📦 Installation

```bash
npm install @fiddupay/fiddupay-node
```

## 🚀 Quick Start

```javascript
import FidduPay from '@fiddupay/fiddupay-node';

const client = new FidduPay({
  apiKey: 'sk_test_your_api_key',
  environment: 'sandbox' // or 'production'
});

// Create a payment
const payment = await client.payments.create({
  amount: '100.00',
  crypto_type: 'ETH',
  description: 'Test payment'
});

// Get merchant balance
const balance = await client.merchants.getBalance();

// Configure webhook
await client.webhooks.configure({
  url: 'https://your-site.com/webhook',
  events: ['payment.completed', 'payment.failed']
});
```

## 🔧 Breaking Changes

- Removed public endpoints (`pricing`, `currencies`) from authenticated SDK
- Contact endpoint kept for convenience with clear public documentation

## 📊 Package Stats

- **Size**: 24.9 kB (125.7 kB unpacked)
- **Files**: 62 total files
- **Dependencies**: axios ^1.13.3
- **TypeScript**: Full type definitions included

## 🌐 Supported Cryptocurrencies

**5 Major Blockchain Networks:**
- **Solana**: SOL + USDT (SPL)
- **Ethereum**: ETH + USDT (ERC-20)  
- **Binance Smart Chain**: BNB + USDT (BEP-20)
- **Polygon**: MATIC + USDT
- **Arbitrum**: ARB + USDT

**Total: 10 cryptocurrency options**

## 📚 Documentation

- **[API Reference](https://github.com/fiddupay/fiddupay-node#api-reference)**
- **[Setup Guide](https://github.com/fiddupay/fiddupay-node#setup)**
- **[Examples](https://github.com/fiddupay/fiddupay-node#examples)**

## 🛡️ Security

- **10/10 Security Score** with comprehensive protection
- **XSS Prevention** & **CSRF Protection**
- **SQL Injection Protection**
- **Advanced Rate Limiting**
- **Real-time Threat Detection**

## 🤝 Support

- **GitHub Issues**: [Report bugs or request features](https://github.com/fiddupay/fiddupay-node/issues)
- **Documentation**: [Complete API documentation](https://github.com/fiddupay/fiddupay-node)
- **Contact**: Use the SDK's contact form feature for support

---

**Full Changelog**: https://github.com/fiddupay/fiddupay-node/compare/v1.0.0...v2.3.4
