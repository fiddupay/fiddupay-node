# FidduPay API Postman Collections

This directory contains the complete Postman collection and environment files for the FidduPay Cryptocurrency Payment Gateway API.

## Files

### Collections
- **FidduPay-Complete-API.postman_collection.json** - Complete API collection with all merchant and admin endpoints (v2.3.6)

### Environments
- **Local-Development.postman_environment.json** - Local development environment (localhost:8080)
- **Production.postman_environment.json** - Production environment (api.fiddupay.com)

## Features

### Merchant API Endpoints
- Authentication (register, login, profile)
- Payment management (create, status, list)
- Wallet configuration
- Withdrawal processing
- Balance management
- Analytics and reporting

### Admin API Endpoints (40+ endpoints)
- Admin authentication and dashboard
- Merchant management (view, suspend, activate, delete)
- Security management (events, alerts, settings)
- System configuration (environment, fees, limits)
- Payment management (view all, force confirm/fail)
- Withdrawal management (approve, reject)
- Analytics and reporting
- Wallet management (hot/cold wallets, balances)
- User management
- System maintenance

### Sandbox Testing
- Test payment creation
- Payment simulation
- Network testing utilities

## Usage

1. Import the collection file into Postman
2. Import the appropriate environment file (Local Development or Production)
3. Set your API key and admin credentials in the environment variables
4. Start testing the API endpoints

## Authentication

### Merchant API
Uses Bearer token authentication with API keys:
```
Authorization: Bearer your_api_key_here
```

### Admin API
Uses session-based authentication with cookies. Login first to establish a session.

## Environment Variables

### Required for Merchant API
- `baseUrl` - API base URL
- `apiKey` - Your merchant API key

### Required for Admin API
- `baseUrl` - API base URL  
- `adminUsername` - Admin username
- `adminPassword` - Admin password

## Security Features

- 10/10 security score with XSS/CSRF protection
- SQL injection protection
- Advanced rate limiting (100 req/min, burst 200/10s)
- Real-time threat detection
- Account lockout protection

## Supported Cryptocurrencies

**10 cryptocurrencies across 5 blockchains:**
- **Solana**: SOL, USDT (SPL)
- **Ethereum**: ETH, USDT (ERC-20)
- **BSC**: BNB, USDT (BEP-20)
- **Polygon**: MATIC, USDT
- **Arbitrum**: ARB, USDT

## Related Documentation

- [API Reference](../API_REFERENCE.md)
- [Node.js SDK](../NODE_SDK.md)
- [Sandbox Setup](../../sandbox/README.md)