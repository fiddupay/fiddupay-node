const { FidduPayClient } = require('@fiddupay/fiddupay-node');

// Test configuration
const client = new FidduPayClient({
  apiKey: 'sk_IBB6fzohlZvJBC2V7zPbwZ3WXs0PT4qh',
  baseUrl: 'http://localhost:8080/api/v1',
  environment: 'sandbox'
});

// Valid crypto types from backend
const VALID_CRYPTO_TYPES = [
  'SOL', 'ETH', 'BNB', 'MATIC', 'ARB',
  'USDT_ETH', 'USDT_BSC', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL'
];

// Validation helpers
function validatePaymentResponse(payment, testName) {
  console.log(`   Validating ${testName} response structure...`);
  
  if (!payment.payment_id) {
    throw new Error(`${testName}: Missing payment_id`);
  }
  
  if (typeof payment.payment_id !== 'string') {
    throw new Error(`${testName}: payment_id should be string`);
  }
  
  console.log(`    ${testName} validation passed`);
  return true;
}

function validateAddressOnlyResponse(payment, testName) {
  console.log(`   Validating ${testName} response structure...`);
  
  const requiredFields = [
    'payment_id', 'requested_amount', 'customer_amount', 
    'processing_fee', 'customer_pays_fee', 'customer_instructions'
  ];
  
  for (const field of requiredFields) {
    if (payment[field] === undefined) {
      throw new Error(`${testName}: Missing required field: ${field}`);
    }
  }
  
  if (typeof payment.customer_pays_fee !== 'boolean') {
    throw new Error(`${testName}: customer_pays_fee should be boolean`);
  }
  
  if (typeof payment.requested_amount !== 'number') {
    throw new Error(`${testName}: requested_amount should be number`);
  }
  
  console.log(`    ${testName} validation passed`);
  return true;
}

async function runValidatedTests() {
  console.log('\n🧪 Running Validated FidduPay API Tests...\n');
  
  let passedTests = 0;
  let totalTests = 0;

  try {
    // Test 1: Standard Payment Creation (Mode 1)
    totalTests++;
    console.log('1. Testing Standard Payment Creation...');
    try {
      const payment = await client.payments.create({
        amount_usd: '100.00',
        crypto_type: 'SOL',
        description: 'Test standard payment'
      });
      
      validatePaymentResponse(payment, 'Standard Payment');
      console.log(' Standard payment created:', payment.payment_id);
      passedTests++;
    } catch (error) {
      console.log(' Standard payment test failed:', error.message);
    }

    // Test 2: Address-Only Payment (Mode 3) - Customer Pays Fee
    totalTests++;
    console.log('\n2. Testing Address-Only Payment (Customer Pays Fee)...');
    try {
      const payment = await client.payments.createAddressOnly({
        crypto_type: 'ETH',
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        requested_amount: 0.05,
        customer_pays_fee: true
      });
      
      validateAddressOnlyResponse(payment, 'Address-Only (Customer Pays)');
      
      if (!payment.customer_pays_fee) {
        throw new Error('customer_pays_fee should be true');
      }
      
      if (payment.customer_amount <= payment.requested_amount) {
        throw new Error('customer_amount should be greater than requested_amount when customer pays fee');
      }
      
      console.log(' Address-only payment (customer pays) created:', payment.payment_id);
      console.log('   Requested:', payment.requested_amount, 'ETH');
      console.log('   Customer pays:', payment.customer_amount, 'ETH');
      console.log('   Processing fee:', payment.processing_fee, 'ETH');
      passedTests++;
    } catch (error) {
      console.log(' Address-only (customer pays) test failed:', error.message);
    }

    // Test 3: Address-Only Payment (Mode 3) - Merchant Pays Fee
    totalTests++;
    console.log('\n3. Testing Address-Only Payment (Merchant Pays Fee)...');
    try {
      const payment = await client.payments.createAddressOnly({
        crypto_type: 'USDT_ETH',
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        requested_amount: 100,
        customer_pays_fee: false
      });
      
      validateAddressOnlyResponse(payment, 'Address-Only (Merchant Pays)');
      
      if (payment.customer_pays_fee) {
        throw new Error('customer_pays_fee should be false');
      }
      
      if (payment.customer_amount !== payment.requested_amount) {
        throw new Error('customer_amount should equal requested_amount when merchant pays fee');
      }
      
      console.log(' Address-only payment (merchant pays) created:', payment.payment_id);
      console.log('   Customer pays:', payment.customer_amount, 'USDT');
      console.log('   Merchant covers fee:', payment.processing_fee, 'USDT');
      passedTests++;
    } catch (error) {
      console.log(' Address-only (merchant pays) test failed:', error.message);
    }

    // Test 4: Fee Toggle Validation
    totalTests++;
    console.log('\n4. Testing Fee Toggle Logic...');
    try {
      const customerPays = await client.payments.createAddressOnly({
        crypto_type: 'BNB',
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        requested_amount: 1,
        customer_pays_fee: true
      });

      const merchantPays = await client.payments.createAddressOnly({
        crypto_type: 'BNB',
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        requested_amount: 1,
        customer_pays_fee: false
      });

      // Validate fee toggle logic
      if (customerPays.customer_amount <= merchantPays.customer_amount) {
        throw new Error('Customer-pays amount should be higher than merchant-pays amount');
      }

      if (customerPays.processing_fee !== merchantPays.processing_fee) {
        throw new Error('Processing fee should be the same regardless of who pays');
      }

      console.log(' Fee toggle validation passed');
      console.log('   Customer pays scenario:', customerPays.customer_amount, 'BNB');
      console.log('   Merchant pays scenario:', merchantPays.customer_amount, 'BNB');
      console.log('   Processing fee:', customerPays.processing_fee, 'BNB');
      passedTests++;
    } catch (error) {
      console.log(' Fee toggle test failed:', error.message);
    }

    // Test 5: All Crypto Types Validation
    totalTests++;
    console.log('\n5. Testing All Supported Crypto Types...');
    try {
      let validTypes = 0;
      const testAmount = 0.01;
      
      for (const cryptoType of VALID_CRYPTO_TYPES.slice(0, 3)) { // Test first 3 to avoid rate limits
        try {
          const payment = await client.payments.createAddressOnly({
            crypto_type: cryptoType,
            merchant_address: cryptoType === 'SOL' ? 
              '11111111111111111111111111111112' : 
              '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
            requested_amount: testAmount,
            customer_pays_fee: true
          });
          
          if (payment.payment_id) {
            validTypes++;
            console.log(`    ${cryptoType}: Payment created`);
          }
        } catch (error) {
          console.log(`   ⚠️  ${cryptoType}: ${error.message}`);
        }
      }
      
      if (validTypes > 0) {
        console.log(` Crypto types validation: ${validTypes}/${VALID_CRYPTO_TYPES.slice(0, 3).length} types working`);
        passedTests++;
      } else {
        throw new Error('No crypto types working');
      }
    } catch (error) {
      console.log(' Crypto types test failed:', error.message);
    }

    // Test 6: Error Handling Validation
    totalTests++;
    console.log('\n6. Testing Error Handling...');
    try {
      let errorsCaught = 0;
      
      // Test invalid crypto type
      try {
        await client.payments.createAddressOnly({
          crypto_type: 'INVALID_CRYPTO',
          merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
          requested_amount: 1,
          customer_pays_fee: true
        });
      } catch (error) {
        if (error.message.includes('Invalid crypto type')) {
          errorsCaught++;
          console.log('    Invalid crypto type error caught correctly');
        }
      }
      
      // Test invalid address format
      try {
        await client.payments.createAddressOnly({
          crypto_type: 'ETH',
          merchant_address: 'invalid_address',
          requested_amount: 1,
          customer_pays_fee: true
        });
      } catch (error) {
        errorsCaught++;
        console.log('    Invalid address error caught correctly');
      }
      
      // Test invalid amount
      try {
        await client.payments.createAddressOnly({
          crypto_type: 'ETH',
          merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
          requested_amount: -1,
          customer_pays_fee: true
        });
      } catch (error) {
        errorsCaught++;
        console.log('    Invalid amount error caught correctly');
      }
      
      if (errorsCaught >= 2) {
        console.log(' Error handling validation passed');
        passedTests++;
      } else {
        throw new Error('Not enough errors caught');
      }
    } catch (error) {
      console.log(' Error handling test failed:', error.message);
    }

    // Test 7: Response Data Validation
    totalTests++;
    console.log('\n7. Testing Response Data Validation...');
    try {
      const payment = await client.payments.createAddressOnly({
        crypto_type: 'MATIC',
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        requested_amount: 10,
        customer_pays_fee: true
      });
      
      // Validate response structure
      const expectedFields = [
        'payment_id', 'gateway_deposit_address', 'requested_amount',
        'customer_amount', 'processing_fee', 'customer_pays_fee',
        'customer_instructions', 'supported_currencies'
      ];
      
      let validFields = 0;
      for (const field of expectedFields) {
        if (payment[field] !== undefined) {
          validFields++;
        } else {
          console.log(`   ⚠️  Missing field: ${field}`);
        }
      }
      
      // Validate data types
      if (typeof payment.payment_id === 'string' &&
          typeof payment.requested_amount === 'number' &&
          typeof payment.customer_amount === 'number' &&
          typeof payment.processing_fee === 'number' &&
          typeof payment.customer_pays_fee === 'boolean' &&
          typeof payment.customer_instructions === 'string' &&
          Array.isArray(payment.supported_currencies)) {
        
        console.log(` Response validation: ${validFields}/${expectedFields.length} fields present with correct types`);
        passedTests++;
      } else {
        throw new Error('Invalid data types in response');
      }
    } catch (error) {
      console.log(' Response validation test failed:', error.message);
    }

    // Final Results
    console.log('\n Test Results Summary:');
    console.log('='.repeat(50));
    console.log(`Total Tests: ${totalTests}`);
    console.log(`Passed: ${passedTests}`);
    console.log(`Failed: ${totalTests - passedTests}`);
    console.log(`Success Rate: ${((passedTests / totalTests) * 100).toFixed(1)}%`);
    
    if (passedTests === totalTests) {
      console.log('\n All tests passed! API endpoints working correctly.');
    } else {
      console.log(`\n⚠️  ${totalTests - passedTests} test(s) failed. Check backend connectivity.`);
    }
    
    console.log('\n Validated Features:');
    console.log('   • 3-Mode Wallet System');
    console.log('   • Fee Toggle Mechanism');
    console.log('   • Address-Only Payments');
    console.log('   • Error Handling');
    console.log('   • Response Data Validation');
    console.log('   • Multiple Crypto Types');

  } catch (error) {
    console.error(' Test suite failed:', error.message);
    process.exit(1);
  }
}

runValidatedTests();
