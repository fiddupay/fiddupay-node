import express from 'express';
import { FidduPayClient } from '@fiddupay/fiddupay-node';

const app = express();
const client = new FidduPayClient({
  apiKey: process.env.FIDDUPAY_API_KEY || 'sk_test_...',
  environment: 'sandbox'
});

app.use(express.json());

// Mode 1: Generate Keys Payment
app.post('/create-payment', async (req, res) => {
  try {
    const { amount_usd, amount, crypto_type, description } = req.body;
    
    const payment = await client.payments.create({
      amount_usd,
      amount,
      crypto_type,
      description,
      metadata: {
        source: 'express-example',
        timestamp: new Date().toISOString()
      }
    });
    
    res.json({
      success: true,
      payment: {
        id: payment.payment_id,
        amount_usd: payment.amount_usd,
        crypto_amount: payment.crypto_amount,
        crypto_type: payment.crypto_type,
        status: payment.status,
        payment_link: payment.payment_link,
        expires_at: payment.expires_at
      }
    });
  } catch (error) {
    console.error('Payment creation failed:', error);
    res.status(400).json({
      success: false,
      error: error.message
    });
  }
});

// Mode 3: Address-Only Payment
app.post('/create-address-only-payment', async (req, res) => {
  try {
    const { crypto_type, merchant_address, requested_amount, customer_pays_fee } = req.body;
    
    const payment = await client.payments.createAddressOnly({
      crypto_type,
      merchant_address,
      requested_amount,
      customer_pays_fee: customer_pays_fee !== false // default true
    });
    
    res.json({
      success: true,
      payment: {
        payment_id: payment.payment_id,
        requested_amount: payment.requested_amount,
        customer_amount: payment.customer_amount,
        processing_fee: payment.processing_fee,
        customer_pays_fee: payment.customer_pays_fee,
        customer_instructions: payment.customer_instructions
      }
    });
  } catch (error) {
    console.error('Address-only payment creation failed:', error);
    res.status(400).json({
      success: false,
      error: error.message
    });
  }
});

// Wallet Management
app.post('/wallets/generate', async (req, res) => {
  try {
    const { crypto_type } = req.body;
    
    const wallet = await client.wallets.generate({
      crypto_type
    });
    
    res.json({
      success: true,
      wallet: {
        crypto_type: wallet.crypto_type,
        address: wallet.address,
        network: wallet.network
      }
    });
  } catch (error) {
    res.status(400).json({ success: false, error: error.message });
  }
});

app.post('/wallets/import', async (req, res) => {
  try {
    const { crypto_type, private_key } = req.body;
    
    const wallet = await client.wallets.import({
      crypto_type,
      private_key
    });
    
    res.json({
      success: true,
      wallet: {
        crypto_type: wallet.crypto_type,
        address: wallet.address,
        network: wallet.network
      }
    });
  } catch (error) {
    res.status(400).json({ success: false, error: error.message });
  }
});

// Get payment status
app.get('/payment/:id', async (req, res) => {
  try {
    const payment = await client.payments.retrieve(req.params.id);
    
    res.json({
      success: true,
      payment: {
        id: payment.payment_id,
        status: payment.status,
        amount_usd: payment.amount_usd,
        crypto_amount: payment.crypto_amount,
        crypto_type: payment.crypto_type,
        transaction_hash: payment.transaction_hash,
        confirmations: payment.confirmations,
        created_at: payment.created_at,
        confirmed_at: payment.confirmed_at
      }
    });
  } catch (error) {
    console.error('Payment retrieval failed:', error);
    res.status(404).json({
      success: false,
      error: error.message
    });
  }
});

// Webhook endpoint - handles payment events
app.post('/webhooks/fiddupay', express.raw({type: 'application/json'}), (req, res) => {
  const sig = req.headers['fiddupay-signature'] as string;
  const webhookSecret = process.env.FIDDUPAY_WEBHOOK_SECRET || 'whsec_test123';
  
  try {
    const event = client.webhooks.constructEvent(req.body, sig, webhookSecret);
    
    console.log('Received webhook event:', event.type, event.id);
    
    switch (event.type) {
      case 'payment.confirmed':
        console.log(' Payment confirmed:', event.data.payment_id);
        break;
        
      case 'payment.failed':
        console.log(' Payment failed:', event.data.payment_id);
        break;
        
      case 'payment.expired':
        console.log('⏰ Payment expired:', event.data.payment_id);
        break;
        
      case 'refund.completed':
        console.log(' Refund completed:', event.data.refund_id);
        break;
        
      case 'refund.failed':
        console.log(' Refund failed:', event.data.refund_id);
        break;
        
      default:
        console.log(' Unknown event type:', event.type);
    }
    
    res.json({ received: true });
  } catch (error) {
    console.error('Webhook signature verification failed:', error.message);
    res.status(400).send('Webhook signature verification failed');
  }
});

// List payments
app.get('/payments', async (req, res) => {
  try {
    const { limit, status, crypto_type } = req.query;
    
    const payments = await client.payments.list({
      limit: limit ? parseInt(limit as string) : 20,
      status: status as any,
      crypto_type: crypto_type as any
    });
    
    res.json({
      success: true,
      payments: payments.payments.map(p => ({
        id: p.payment_id,
        amount_usd: p.amount_usd,
        crypto_amount: p.crypto_amount,
        crypto_type: p.crypto_type,
        status: p.status,
        created_at: p.created_at
      })),
      total: payments.total,
      has_more: payments.has_more
    });
  } catch (error) {
    console.error('Failed to list payments:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

// Get merchant balance
app.get('/balance', async (req, res) => {
  try {
    const balance = await client.merchants.getBalance();
    
    res.json({
      success: true,
      balance
    });
  } catch (error) {
    console.error('Failed to get balance:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

const PORT = process.env.PORT || 3000;

app.listen(PORT, () => {
  console.log(` FidduPay Express.js example running on port ${PORT}`);
  console.log(` Endpoints:`);
  console.log(`   POST /create-payment - Create payment (amount_usd OR amount)`);
  console.log(`   POST /create-address-only-payment - Create address-only payment`);
  console.log(`   POST /wallets/generate - Generate new wallet keys`);
  console.log(`   POST /wallets/import - Import existing keys`);
  console.log(`   GET  /payment/:id - Get payment status`);
  console.log(`   GET  /payments - List payments`);
  console.log(`   GET  /balance - Get merchant balance`);
  console.log(`   POST /webhooks/fiddupay - Webhook endpoint`);
  console.log(`\n Payment Creation:`);
  console.log(`   Use either 'amount_usd' (USD amount) OR 'amount' (crypto amount)`);
  console.log(`   Supported crypto types: SOL, ETH, BNB, MATIC, ARB, USDT_*`);
});

export default app;
