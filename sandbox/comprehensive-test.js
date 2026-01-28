const { FidduPayClient } = require('@fiddupay/fiddupay-node');

async function runComprehensiveTest() {
  console.log(' Running Comprehensive FidduPay SDK Test...\n');
  
  let passedTests = 0;
  let totalTests = 0;

  try {
    // Step 1: Register a new merchant
    console.log('1⃣  Registering test merchant...');
    totalTests++;
    
    const tempClient = new FidduPayClient({
      apiKey: 'registration_key',
      environment: 'sandbox',
      baseURL: 'http://127.0.0.1:8080'
    });

    const testEmail = `comprehensive_${Date.now()}@example.com`;
    const registration = await tempClient.merchants.register({
      email: testEmail,
      business_name: 'Comprehensive Test Business',
      password: 'TestPassword123!'
    });
    
    console.log('    Merchant registered:', registration.api_key.substring(0, 10) + '...');
    passedTests++;
    
    // Step 2: Create authenticated client
    const client = new FidduPayClient({
      apiKey: registration.api_key,
      environment: 'sandbox',
      baseURL: 'http://127.0.0.1:8080'
    });

    // Step 3: Get merchant profile
    console.log('\n2⃣  Testing merchant profile...');
    totalTests++;
    
    const profile = await client.merchants.retrieve();
    console.log(`    Business: ${profile.business_name}`);
    console.log(`    KYC Status: ${profile.kyc_verified ? 'Verified' : 'Not Verified'}`);
    
    // Test daily volume limit information
    if (!profile.kyc_verified && profile.daily_volume_remaining !== undefined) {
      console.log(`    Daily Volume Remaining: $${profile.daily_volume_remaining}`);
    } else if (profile.kyc_verified) {
      console.log('    Daily Volume: Unlimited (KYC Verified)');
    }
    
    console.log('    Merchant profile retrieved successfully');
    passedTests++;

    // Step 4: Set up wallet for SOL
    console.log('\n3⃣  Setting up SOL wallet...');
    totalTests++;
    
    await client.merchants.setWallet({
      crypto_type: 'SOL',
      address: '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM'
    });
    console.log('    SOL wallet configured successfully');
    passedTests++;

    // Step 5: Create payment
    console.log('\n4⃣  Creating payment...');
    totalTests++;
    
    const payment = await client.payments.create({
      amount_usd: '10.00',
      crypto_type: 'SOL',
      description: 'Comprehensive test payment'
    });
    
    console.log(`    Payment ID: ${payment.payment_id}`);
    console.log(`    Amount: ${payment.amount} SOL ($${payment.amount_usd})`);
    console.log(`    Address: ${payment.to_address}`);
    console.log('    Payment created successfully');
    passedTests++;

    // Step 6: Retrieve payment
    console.log('\n5⃣  Retrieving payment...');
    totalTests++;
    
    const retrievedPayment = await client.payments.retrieve(payment.payment_id);
    console.log(`    Retrieved payment: ${retrievedPayment.payment_id}`);
    console.log(`    Status: ${retrievedPayment.status}`);
    console.log('    Payment retrieved successfully');
    passedTests++;

    // Step 7: List payments
    console.log('\n6⃣  Listing payments...');
    totalTests++;
    
    const payments = await client.payments.list({ limit: 5 });
    const paymentCount = payments.data?.length || payments.length || 0;
    console.log(`    Found ${paymentCount} payments`);
    console.log('    Payment listing successful');
    passedTests++;

    // Step 8: Test analytics
    console.log('\n7⃣  Testing analytics...');
    totalTests++;
    
    const analytics = await client.analytics.retrieve({
      start_date: '2024-01-01',
      end_date: '2024-12-31'
    });
    console.log('    Analytics retrieved successfully');
    console.log('    Analytics test passed');
    passedTests++;

  } catch (error) {
    console.log(`    Test failed: ${error.message}`);
    if (error.statusCode) {
      console.log(`    Status Code: ${error.statusCode}`);
    }
  }

  // Final Results
  console.log('\n Test Results Summary:');
  console.log(`    Passed: ${passedTests}/${totalTests} tests`);
  console.log(`    Failed: ${totalTests - passedTests}/${totalTests} tests`);
  
  if (passedTests === totalTests) {
    console.log('\n All tests passed! FidduPay SDK is fully functional.');
    return { passed: passedTests, total: totalTests, success: true };
  } else if (passedTests > 0) {
    console.log(`\n ${passedTests} test(s) passed. SDK is functional with some limitations.`);
    return { passed: passedTests, total: totalTests, success: false };
  } else {
    console.log(`\n  All tests failed. Please check the API server and configuration.`);
    return { passed: passedTests, total: totalTests, success: false };
  }
}

// Run the test
runComprehensiveTest()
  .then(results => {
    console.log(`\n Test execution completed: ${results.passed}/${results.total} passed`);
    process.exit(results.success ? 0 : 1);
  })
  .catch(error => {
    console.error(' Test execution failed:', error);
    process.exit(1);
  });
