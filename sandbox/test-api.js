const { FidduPayClient } = require('@fiddupay/fiddupay-node');

// Test configuration
const client = new FidduPayClient({
  apiKey: 'sk_sandbox_test_key',
  baseUrl: 'http://localhost:3001',
  environment: 'sandbox'
});

async function runTests() {
  console.log(' Running FidduPay API Tests...\n');

  try {
    // Test 1: Create payment
    console.log('1. Creating payment...');
    const payment = await client.payments.create({
      amount: 100.50,
      currency: 'USDT',
      network: 'ethereum',
      description: 'Test payment',
      metadata: { orderId: 'test-123' }
    });
    console.log(' Payment created:', payment.id);

    // Test 2: Get payment
    console.log('\n2. Getting payment...');
    const retrievedPayment = await client.payments.get(payment.id);
    console.log(' Payment retrieved:', retrievedPayment.status);

    // Test 3: List payments
    console.log('\n3. Listing payments...');
    const payments = await client.payments.list({ limit: 5 });
    console.log(' Payments listed:', payments.data.length, 'payments');

    console.log('\n All tests passed!');
  } catch (error) {
    console.error(' Test failed:', error.message);
  }
}

runTests();
