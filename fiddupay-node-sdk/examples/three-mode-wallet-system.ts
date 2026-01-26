import express from 'express';
import { FidduPayClient } from '@fiddupay/fiddupay-node';

const app = express();
const client = new FidduPayClient({
  apiKey: process.env.FIDDUPAY_API_KEY || 'sk_test_...',
  environment: 'sandbox'
});

app.use(express.json());

// Mode 1: Generate Keys - Create payment with auto-generated wallet
app.post('/payments/generate-keys', async (req, res) => {
  try {
    const { amount, currency, network, description } = req.body;
    
    const payment = await client.payments.create({
      amount,
      currency,
      network,
      wallet_mode: 'generate_keys',
      description,
      metadata: {
        mode: 'generate_keys',
        timestamp: new Date().toISOString()
      }
    });
    
    res.json({
      success: true,
      mode: 'generate_keys',
      payment: {
        id: payment.payment_id,
        amount: payment.amount,
        currency: payment.currency,
        status: payment.status,
        payment_url: payment.payment_url,
        deposit_address: payment.deposit_address
      }
    });
  } catch (error) {
    res.status(400).json({ success: false, error: error.message });
  }
});

// Mode 2: Import Keys - Setup wallet with existing private key
app.post('/wallets/import', async (req, res) => {
  try {
    const { crypto_type, private_key } = req.body;
    
    const wallet = await client.wallets.import({
      crypto_type,
      private_key
    });
    
    res.json({
      success: true,
      mode: 'import_keys',
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

// Mode 2: Create payment with imported keys
app.post('/payments/import-keys', async (req, res) => {
  try {
    const { amount, currency, network, description } = req.body;
    
    const payment = await client.payments.create({
      amount,
      currency,
      network,
      wallet_mode: 'import_keys',
      description,
      metadata: {
        mode: 'import_keys',
        timestamp: new Date().toISOString()
      }
    });
    
    res.json({
      success: true,
      mode: 'import_keys',
      payment
    });
  } catch (error) {
    res.status(400).json({ success: false, error: error.message });
  }
});

// Mode 3: Address-Only - Customer pays from their own wallet
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

// Generate wallet keys (Mode 1)
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
        // Note: In production, never return private keys in API responses
        // Store them securely and only show to merchant once
        private_key_preview: wallet.private_key.substring(0, 10) + '...'
      }
    });
  } catch (error) {
    res.status(400).json({ success: false, error: error.message });
  }
});

// Get wallet configuration
app.get('/wallets/config', async (req, res) => {
  try {
    const config = await client.wallets.getConfig();
    
    res.json({
      success: true,
      config: {
        supported_modes: config.supported_modes,
        current_mode: config.current_mode,
        supported_crypto_types: config.supported_crypto_types
      }
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

// Fee toggle demonstration
app.post('/payments/fee-toggle-demo', async (req, res) => {
  try {
    const { crypto_type, merchant_address, amount, customer_pays_fee } = req.body;
    
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
          merchant_receives: merchantPaysPayment.requested_amount - merchantPaysPayment.processing_fee
        }
      }
    });
  } catch (error) {
    res.status(400).json({ success: false, error: error.message });
  }
});

// Webhook endpoint for all modes
app.post('/webhooks/fiddupay', express.raw({type: 'application/json'}), (req, res) => {
  const sig = req.headers['fiddupay-signature'] as string;
  const webhookSecret = process.env.FIDDUPAY_WEBHOOK_SECRET || 'whsec_test123';
  
  try {
    const event = client.webhooks.constructEvent(req.body, sig, webhookSecret);
    
    console.log(`📨 Webhook received: ${event.type} for ${event.data.payment_id}`);
    
    switch (event.type) {
      case 'payment.confirmed':
        console.log(` Payment confirmed: ${event.data.payment_id} (Mode: ${event.data.wallet_mode || 'unknown'})`);
        break;
        
      case 'payment.failed':
        console.log(` Payment failed: ${event.data.payment_id}`);
        break;
        
      case 'address_only.payment_received':
        console.log(` Address-only payment received: ${event.data.payment_id}`);
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
  console.log(` FidduPay 3-Mode Wallet System Example running on port ${PORT}`);
  console.log(` Available endpoints:`);
  console.log(`   POST /payments/generate-keys - Mode 1: Auto-generate wallet keys`);
  console.log(`   POST /wallets/generate - Generate new wallet`);
  console.log(`   POST /wallets/import - Mode 2: Import existing private key`);
  console.log(`   POST /payments/import-keys - Create payment with imported keys`);
  console.log(`   POST /payments/address-only - Mode 3: Address-only payments`);
  console.log(`   POST /payments/fee-toggle-demo - Demonstrate fee toggle`);
  console.log(`   GET  /wallets/config - Get wallet configuration`);
  console.log(`   POST /webhooks/fiddupay - Webhook endpoint`);
});

export default app;
