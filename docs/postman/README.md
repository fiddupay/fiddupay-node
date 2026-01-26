# FidduPay Postman Collection

Complete Postman collection for testing FidduPay API endpoints.

##  Files

- `FidduPay-API.postman_collection.json` - Main API collection
- `Local-Development.postman_environment.json` - Local development environment
- `Production.postman_environment.json` - Production environment template

##  Quick Setup

### 1. Import Collection
1. Open Postman
2. Click **Import** 
3. Select `FidduPay-API.postman_collection.json`

### 2. Import Environment
1. Click **Import** again
2. Select `Local-Development.postman_environment.json`
3. Select the environment from dropdown (top-right)

### 3. Update API Key
1. Go to **Environments** tab
2. Update `apiKey` variable with your actual API key
3. For production, use `Production.postman_environment.json`

##  Available Requests

### Payments
- **Create Payment** - Create new payment request
- **Get Payment** - Retrieve payment by ID
- **List Payments** - Get paginated payment list
- **Cancel Payment** - Cancel pending payment

### Wallets
- **List Wallets** - Get merchant wallets
- **Create Wallet** - Create new wallet

### System
- **Health Check** - API health status
- **System Status** - Detailed system status
- **Supported Currencies** - Available cryptocurrencies

### Sandbox
- **Sandbox - Create Payment** - Test payment creation
- **Sandbox - Test Webhook** - Test webhook endpoint

##  Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `baseUrl` | Main API URL | `http://localhost:8080` |
| `sandboxUrl` | Sandbox API URL | `http://localhost:3001` |
| `apiKey` | Your API key | `sandbox_test_key_12345` |

##  Usage Tips

1. **Start with Sandbox**: Use sandbox endpoints for testing
2. **Check Responses**: Review response structure for integration
3. **Test Webhooks**: Use sandbox webhook endpoint for testing
4. **Environment Switching**: Switch between Local/Production environments easily

##  Related Documentation

- [API Reference](../API_REFERENCE.md)
- [Node.js SDK](../NODE_SDK.md)
- [Sandbox Setup](../../sandbox/README.md)
