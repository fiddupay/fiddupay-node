import express from 'express';
import FidduPay from 'fiddupay-node';

const app = express();
const fiddupay = new FidduPay({
  apiKey: process.env.FIDDUPAY_API_KEY || 'sk_test_...',
  environment: 'sandbox'
});

app.use(express.json());

// Create payment endpoint
app.post('/create-payment', async (req, res) => {
  try {
    const { amount_usd, crypto_type, description } = req.body;
    
    const payment = await fiddupay.payments.create({
      amount_usd,
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
        amount: payment.amount_usd,
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

// Get payment status
app.get('/payment/:id', async (req, res) => {
  try {
    const payment = await fiddupay.payments.retrieve(req.params.id);
    
    res.json({
      success: true,
      payment: {
        id: payment.payment_id,
        status: payment.status,
        amount: payment.amount_usd,
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

// Webhook endpoint
app.post('/webhooks/fiddupay', express.raw({type: 'application/json'}), (req, res) => {
  const sig = req.headers['fiddupay-signature'] as string;
  const webhookSecret = process.env.FIDDUPAY_WEBHOOK_SECRET || 'whsec_test123';
  
  try {
    const event = fiddupay.webhooks.constructEvent(req.body, sig, webhookSecret);
    
    console.log('Received webhook event:', event.type, event.id);
    
    switch (event.type) {
      case 'payment.confirmed':
        console.log('✅ Payment confirmed:', event.data.payment_id);
        // Update your database, send confirmation email, etc.
        break;
        
      case 'payment.failed':
        console.log('❌ Payment failed:', event.data.payment_id);
        // Handle failed payment
        break;
        
      case 'payment.expired':
        console.log('⏰ Payment expired:', event.data.payment_id);
        // Handle expired payment
        break;
        
      case 'refund.completed':
        console.log('💰 Refund completed:', event.data.refund_id);
        // Handle completed refund
        break;
        
      case 'refund.failed':
        console.log('❌ Refund failed:', event.data.refund_id);
        // Handle failed refund
        break;
        
      default:
        console.log('Unknown event type:', event.type);
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
    
    const payments = await fiddupay.payments.list({
      limit: limit ? parseInt(limit as string) : 20,
      status: status as any,
      crypto_type: crypto_type as any
    });
    
    res.json({
      success: true,
      payments: payments.payments.map(p => ({
        id: p.payment_id,
        amount: p.amount_usd,
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
    const balance = await fiddupay.merchants.getBalance();
    
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
  console.log(`🚀 FidduPay Express.js example running on port ${PORT}`);
  console.log(`📝 Endpoints:`);
  console.log(`   POST /create-payment - Create a new payment`);
  console.log(`   GET  /payment/:id - Get payment status`);
  console.log(`   GET  /payments - List payments`);
  console.log(`   GET  /balance - Get merchant balance`);
  console.log(`   POST /webhooks/fiddupay - Webhook endpoint`);
});

export default app;
