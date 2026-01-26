# FidduPay Sandbox Environment

Local sandbox environment for testing FidduPay API integration.

##  Quick Start

```bash
# Install dependencies
npm install

# Start sandbox server
npm start

# Run API tests
npm test
```

Server runs on: http://localhost:3001

##  Available Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | API documentation |
| POST | `/payments` | Create payment |
| GET | `/payments/:id` | Get payment status |
| GET | `/payments` | List payments |
| POST | `/webhooks/test` | Test webhook |
| GET | `/health` | Health check |

##  Testing

### Create Payment
```bash
curl -X POST http://localhost:3001/payments \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 100.50,
    "currency": "USDT",
    "network": "ethereum",
    "description": "Test payment"
  }'
```

### Test Webhook
```bash
curl -X POST http://localhost:3001/webhooks/test \
  -H "Content-Type: application/json" \
  -d '{
    "event": "payment.completed",
    "payment_id": "payment_123"
  }'
```

##  Configuration

Environment variables:
- `PORT` - Server port (default: 3001)
- `FIDDUPAY_API_KEY` - API key for main service
- `FIDDUPAY_API_URL` - Main API URL (default: http://localhost:8080)

##  Integration

Use with FidduPay Node.js SDK:

```javascript
const { FidduPayClient } = require('@fiddupay/fiddupay-node');

const client = new FidduPayClient({
  apiKey: 'sandbox_test_key',
  baseUrl: 'http://localhost:3001',
  environment: 'sandbox'
});
```

##  Related

- [Postman Collection](../docs/postman/)
- [API Reference](../docs/API_REFERENCE.md)
- [Node.js SDK](../fiddupay-node-sdk/)
