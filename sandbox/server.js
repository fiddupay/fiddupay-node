const express = require('express');
const cors = require('cors');
const bodyParser = require('body-parser');
const { FidduPayClient } = require('@fiddupay/fiddupay-node');

const app = express();
const PORT = process.env.PORT || 3001;

// Middleware
app.use(cors());
app.use(bodyParser.json());
app.use(express.static('public'));

// Initialize FidduPay client
const client = new FidduPayClient({
  apiKey: process.env.FIDDUPAY_API_KEY || 'sk_sandbox_test_key',
  baseURL: process.env.FIDDUPAY_API_URL || 'http://localhost:8080/api/v1',
  environment: 'sandbox'
});

// Routes
app.get('/', (req, res) => {
  res.json({
    message: 'FidduPay Sandbox API',
    version: '1.0.0',
    endpoints: {
      'POST /payments': 'Create payment',
      'GET /payments/:id': 'Get payment status',
      'GET /payments': 'List payments',
      'POST /webhooks/test': 'Test webhook',
      'GET /health': 'Health check'
    }
  });
});

// Create payment
app.post('/payments', async (req, res) => {
  try {
    const payment = await client.payments.create(req.body);
    res.json(payment);
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});

// Get payment
app.get('/payments/:id', async (req, res) => {
  try {
    const payment = await client.payments.get(req.params.id);
    res.json(payment);
  } catch (error) {
    res.status(404).json({ error: error.message });
  }
});

// List payments
app.get('/payments', async (req, res) => {
  try {
    const payments = await client.payments.list(req.query);
    res.json(payments);
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});

// Test webhook
app.post('/webhooks/test', (req, res) => {
  console.log('Webhook received:', req.body);
  res.json({ received: true, timestamp: new Date().toISOString() });
});

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'healthy', timestamp: new Date().toISOString() });
});

app.listen(PORT, () => {
  console.log(`FidduPay Sandbox running on port ${PORT}`);
  console.log(`Visit http://localhost:${PORT} for API documentation`);
});
