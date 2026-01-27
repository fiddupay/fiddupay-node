import express from 'express';
import { FidduPayClient } from '@fiddupay/fiddupay-node';

const app = express();
const client = new FidduPayClient({
  apiKey: process.env.FIDDUPAY_API_KEY || 'sk_test_...',
  environment: 'sandbox'
});

app.use(express.json());

// Create payment with USD amount
app.post('/payments/create-usd', async (req, res) => {
  try {
    const { amount_usd, crypto_type, description } = req.body;
    
    const payment = await client.payments.create({
      amount_usd,
      crypto_type,
      description,
      metadata: {
        mode: 'usd_amount',
        timestamp: new Date().toISOString()
      }
    });
    
    res.json({
      success: true,
      mode: 'usd_amount',
      payment: {
        id: payment.payment_id,
        amount_usd: payment.amount_usd,
        crypto_amount: payment.crypto_amount,
        crypto_type: payment.crypto_type,
        status: payment.status,
        deposit_address: payment.deposit_address
      }
    });
  } catch (error) {
    res.status(400).json({ success: false, error: error.message });
  }
});

// Create payment with crypto amount
app.post('/payments/create-crypto', async (req, res) => {
  try {
    const { amount, crypto_type, description } = req.body;
    
    const payment = await client.payments.create({
      amount,
      crypto_type,
      description,
      metadata: {
        mode: 'crypto_amount',
        timestamp: new Date().toISOString()
      }
    });
    
    res.json({
      success: true,
      mode: 'crypto_amount',
      payment: {
        id: payment.payment_id,
        amount_usd: payment.amount_usd,
        crypto_amount: payment.crypto_amount,
        crypto_type: payment.crypto_type,
        status: payment.status,
        deposit_address: payment.deposit_address
      }
    });
  } catch (error) {
    res.status(400).json({ success: false, error: error.message });
  }
});

// Address-Only Payment
app.post('/payments/address-only', async (req, res) => {
  try {
    const { crypto_type, merchant_address, requested_amount, customer_pays_fee } = req.body;
    
    const payment = await client.payments.createAddressOnly({
      crypto_type,
      merchant_address,
      requested_amount,
      customer_pays_fee: customer_pays_fee || true
    });
    
    res.json({
      success: true,
      mode: 'address_only',
      payment: {
        payment_id: payment.payment_id,
        requested_amount: payment.requested_amount,
        customer_amount: payment.customer_amount,
        processing_fee: payment.processing_fee,
        customer_pays_fee: payment.customer_pays_fee,
        customer_instructions: payment.customer_instructions,
        supported_currencies: payment.supported_currencies
      }
    });
  } catch (error) {
    res.status(400).json({ success: false, error: error.message });
  }
});

// Fee toggle demonstration
app.post('/payments/fee-toggle-demo', async (req, res) => {
  try {
    const { crypto_type, merchant_address, amount } = req.body;
    
    // Create two scenarios to show fee difference
    const customerPaysPayment = await client.payments.createAddressOnly({
      crypto_type,
      merchant_address,
      requested_amount: amount,
      customer_pays_fee: true
    });
    
    const merchantPaysPayment = await client.payments.createAddressOnly({
      crypto_type,
      merchant_address,
      requested_amount: amount,
      customer_pays_fee: false
    });
    
    res.json({
      success: true,
      fee_comparison: {
        customer_pays_scenario: {
          customer_amount: customerPaysPayment.customer_amount,
          processing_fee: customerPaysPayment.processing_fee,
          merchant_receives: customerPaysPayment.requested_amount
        },
        merchant_pays_scenario: {
          customer_amount: merchantPaysPayment.customer_amount,
          processing_fee: merchantPaysPayment.processing_fee,
          merchant_receives: (parseFloat(merchantPaysPayment.requested_amount) - parseFloat(merchantPaysPayment.processing_fee)).toString()
        }
      }
    });
  } catch (error) {
    res.status(400).json({ success: false, error: error.message });
  }
});

// Webhook endpoint for payment events
app.post('/webhooks/fiddupay', express.raw({type: 'application/json'}), (req, res) => {
  const sig = req.headers['fiddupay-signature'] as string;
  const webhookSecret = process.env.FIDDUPAY_WEBHOOK_SECRET || 'whsec_test123';
  
  try {
    const event = client.webhooks.constructEvent(req.body, sig, webhookSecret);
    
    console.log(`📨 Webhook received: ${event.type} for ${event.data.payment_id}`);
    
    switch (event.type) {
      case 'payment.confirmed':
        console.log(`✅ Payment confirmed: ${event.data.payment_id}`);
        break;
        
      case 'payment.failed':
        console.log(`❌ Payment failed: ${event.data.payment_id}`);
        break;
        
      default:
        console.log(`🔔 Unknown event: ${event.type}`);
    }
    
    res.json({ received: true });
  } catch (error) {
    console.error('Webhook verification failed:', error.message);
    res.status(400).send('Webhook verification failed');
  }
});

const PORT = process.env.PORT || 3000;

app.listen(PORT, () => {
  console.log(`🚀 FidduPay Payment System Example running on port ${PORT}`);
  console.log(`📋 Available endpoints:`);
  console.log(`   POST /payments/create-usd - Create payment with USD amount`);
  console.log(`   POST /payments/create-crypto - Create payment with crypto amount`);
  console.log(`   POST /payments/address-only - Address-only payments`);
  console.log(`   POST /payments/fee-toggle-demo - Demonstrate fee toggle`);
  console.log(`   POST /webhooks/fiddupay - Webhook endpoint`);
  console.log(`\n💡 Payment Creation Options:`);
  console.log(`   USD Amount: Specify amount_usd (e.g., "10.50")`);
  console.log(`   Crypto Amount: Specify amount (e.g., "0.1" for 0.1 SOL)`);
  console.log(`   Never specify both amount_usd AND amount`);
});

export default app;
