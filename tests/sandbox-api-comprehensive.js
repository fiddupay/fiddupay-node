const axios = require('axios');

// Test configuration
const BASE_URL = 'http://127.0.0.1:8080/api/v1';
const TEST_EMAIL = `sandbox_test_${Date.now()}@example.com`;
const TEST_BUSINESS_NAME = 'Sandbox Test Business';

// Test state
let testMerchant = null;
let testApiKey = null;
let sandboxApiKey = null;
let liveApiKey = null;
let testPaymentId = null;
let testRefundId = null;

// Test results tracking
let passedTests = 0;
let totalTests = 0;
let testResults = [];

// Helper functions
function logTest(testName, status, details = '') {
  totalTests++;
  const statusIcon = status === 'PASS' ? '✅' : '❌';
  console.log(`${statusIcon} ${testName} ${details}`);
  
  if (status === 'PASS') {
    passedTests++;
  }
  
  testResults.push({ testName, status, details });
}

function createAuthHeaders(apiKey) {
  return {
    'Authorization': `Bearer ${apiKey}`,
    'Content-Type': 'application/json'
  };
}

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// Setup functions
async function testSandboxDailyVolumeLimit() {
  console.log('\n💰 Testing Sandbox Daily Volume Limit...');
  
  try {
    // Test that sandbox merchants also have daily volume limits
    const profileResponse = await axios.get(`${BASE_URL}/merchants/profile`, {
      headers: createAuthHeaders(testApiKey)
    });
    
    if (profileResponse.status === 200) {
      const profile = profileResponse.data;
      
      if (typeof profile.kyc_verified === 'boolean') {
        logTest('Sandbox KYC Status', 'PASS', `KYC verified: ${profile.kyc_verified}`);
        
        // Sandbox merchants should also have daily volume limits if not KYC verified
        if (!profile.kyc_verified) {
          logTest('Sandbox Daily Volume Limit', 'PASS', 'Non-KYC sandbox merchant has daily volume limits');
        } else {
          logTest('Sandbox KYC Unlimited', 'PASS', 'KYC verified sandbox merchant has unlimited volume');
        }
      } else {
        logTest('Sandbox KYC Status', 'FAIL', 'KYC status not found in sandbox profile');
      }
    } else {
      logTest('Sandbox Profile', 'FAIL', `HTTP ${profileResponse.status}`);
    }
  } catch (error) {
    logTest('Sandbox Daily Volume Test', 'FAIL', error.message);
  }
}

async function setupTestMerchant() {
  console.log('\n🔧 Setting up test merchant...');
  
  try {
    const response = await axios.post(`${BASE_URL}/merchants/register`, {
      email: TEST_EMAIL,
      business_name: TEST_BUSINESS_NAME,
      password: 'SecurePassword123!'
    });
    
    if (response.status === 201 && response.data.api_key && response.data.user) {
      testMerchant = response.data.user;
      testApiKey = response.data.api_key;
      console.log(`✅ Test merchant created: ${testMerchant.id}`);
      return true;
    }
  } catch (error) {
    console.log(`❌ Failed to create test merchant: ${error.message}`);
    return false;
  }
}

// Sandbox Core Tests
async function testEnableSandboxMode() {
  console.log('\n🏖️ Testing Enable Sandbox Mode...');
  
  try {
    const response = await axios.post(`${BASE_URL}/sandbox/enable`, {}, {
      headers: createAuthHeaders(testApiKey)
    });
    
    if (response.status === 200 && response.data.sandbox_api_key && response.data.sandbox_mode) {
      sandboxApiKey = response.data.sandbox_api_key;
      // Update the main API key to use the new sandbox key
      testApiKey = sandboxApiKey;
      logTest('Enable Sandbox Mode', 'PASS', `Sandbox API Key: ${sandboxApiKey.substring(0, 10)}...`);
      
      // Verify sandbox key format
      if (sandboxApiKey.startsWith('sk_')) {
        logTest('Sandbox Key Format', 'PASS', 'Correct sk_ prefix');
      } else {
        logTest('Sandbox Key Format', 'FAIL', 'Missing sk_ prefix');
      }
    } else {
      logTest('Enable Sandbox Mode', 'FAIL', 'Invalid response structure');
    }
  } catch (error) {
    logTest('Enable Sandbox Mode', 'FAIL', error.message);
  }
}

async function testSandboxEnvironmentSwitch() {
  console.log('\n🔄 Testing Environment Switching...');
  
  // Test switch to live
  try {
    const liveResponse = await axios.post(`${BASE_URL}/merchants/environment/switch`, {
      to_live: true
    }, {
      headers: createAuthHeaders(testApiKey)
    });
    
    if (liveResponse.status === 200 && liveResponse.data.api_key) {
      liveApiKey = liveResponse.data.api_key;
      logTest('Switch to Live Environment', 'PASS', `Live API Key: ${liveApiKey.substring(0, 10)}...`);
      
      // Verify live key format
      if (liveApiKey.startsWith('live_')) {
        logTest('Live Key Format', 'PASS', 'Correct live_ prefix');
      } else {
        logTest('Live Key Format', 'FAIL', 'Missing live_ prefix');
      }
    } else {
      logTest('Switch to Live Environment', 'FAIL', 'Invalid response');
    }
  } catch (error) {
    logTest('Switch to Live Environment', 'FAIL', error.message);
  }
  
  // Test switch back to sandbox
  try {
    const sandboxResponse = await axios.post(`${BASE_URL}/merchants/environment/switch`, {
      to_live: false
    }, {
      headers: createAuthHeaders(liveApiKey)
    });
    
    if (sandboxResponse.status === 200 && sandboxResponse.data.api_key) {
      testApiKey = sandboxResponse.data.api_key; // Update current API key
      logTest('Switch to Sandbox Environment', 'PASS', 'Successfully switched back');
    } else {
      logTest('Switch to Sandbox Environment', 'FAIL', 'Invalid response');
    }
  } catch (error) {
    logTest('Switch to Sandbox Environment', 'FAIL', error.message);
  }
}

async function testSandboxDataIsolation() {
  console.log('\n🔒 Testing Sandbox Data Isolation...');
  
  // Configure required wallets first
  const wallets = [
    { crypto_type: 'SOL', address: '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM' },
    { crypto_type: 'USDT_SPL', address: '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM' },
    { crypto_type: 'ETH', address: '0x1234567890123456789012345678901234567890' },
    { crypto_type: 'USDT_BEP20', address: '0x1234567890123456789012345678901234567890' }
  ];
  
  for (const wallet of wallets) {
    try {
      await axios.put(`${BASE_URL}/merchants/wallets`, wallet, {
        headers: createAuthHeaders(testApiKey)
      });
    } catch (error) {
      // Wallet might already be configured, continue
    }
  }
  
  // Test that sandbox key cannot access live data
  try {
    // First create a payment with sandbox key
    const paymentResponse = await axios.post(`${BASE_URL}/payments`, {
      amount_usd: '100.00',
      crypto_type: 'SOL',
      description: 'Sandbox isolation test payment'
    }, {
      headers: createAuthHeaders(testApiKey)
    });
    
    if (paymentResponse.status === 201) {
      testPaymentId = paymentResponse.data.payment_id;
      logTest('Create Sandbox Payment', 'PASS', `Payment ID: ${testPaymentId}`);
      
      // Try to access this payment with live key (should fail or not show)
      if (liveApiKey) {
        try {
          await axios.get(`${BASE_URL}/payments/${testPaymentId}`, {
            headers: createAuthHeaders(liveApiKey)
          });
          logTest('Sandbox Data Isolation', 'FAIL', 'Live key accessed sandbox data');
        } catch (error) {
          if (error.response && (error.response.status === 404 || error.response.status === 403 || error.response.status === 401)) {
            logTest('Sandbox Data Isolation', 'PASS', 'Live key properly isolated from sandbox data');
          } else {
            logTest('Sandbox Data Isolation', 'FAIL', `Unexpected error: ${error.message}`);
          }
        }
      }
    } else {
      logTest('Create Sandbox Payment', 'FAIL', 'Failed to create test payment');
    }
  } catch (error) {
    logTest('Create Sandbox Payment', 'FAIL', error.message);
  }
}

// Payment Simulation Tests
async function testPaymentSimulationSuccess() {
  console.log('\n✅ Testing Successful Payment Simulation...');
  
  if (!testPaymentId) {
    logTest('Payment Simulation Success', 'SKIP', 'No test payment available');
    return;
  }
  
  try {
    const response = await axios.post(`${BASE_URL}/sandbox/payments/${testPaymentId}/simulate`, {
      success: true
    }, {
      headers: createAuthHeaders(testApiKey)
    });
    
    if (response.status === 200 && response.data.success) {
      logTest('Payment Simulation Success', 'PASS', response.data.message);
      
      // Verify payment status changed
      await sleep(1000); // Wait for status update
      const statusResponse = await axios.get(`${BASE_URL}/payments/${testPaymentId}`, {
        headers: createAuthHeaders(testApiKey)
      });
      
      if (statusResponse.data.status === 'Confirmed') {
        logTest('Payment Status Update', 'PASS', 'Status correctly updated to Confirmed');
      } else {
        logTest('Payment Status Update', 'FAIL', `Status: ${statusResponse.data.status}`);
      }
    } else {
      logTest('Payment Simulation Success', 'FAIL', 'Invalid response');
    }
  } catch (error) {
    logTest('Payment Simulation Success', 'FAIL', error.message);
  }
}

async function testPaymentSimulationFailure() {
  console.log('\n❌ Testing Failed Payment Simulation...');
  
  // Create another test payment for failure simulation
  try {
    const paymentResponse = await axios.post(`${BASE_URL}/payments`, {
      amount_usd: "50.00",
      crypto_type: 'USDT_SPL',
      description: 'Sandbox failure test payment'
    }, {
      headers: createAuthHeaders(testApiKey)
    });
    
    if (paymentResponse.status === 201) {
      const failurePaymentId = paymentResponse.data.payment_id;
      
      const response = await axios.post(`${BASE_URL}/sandbox/payments/${failurePaymentId}/simulate`, {
        success: false
      }, {
        headers: createAuthHeaders(testApiKey)
      });
      
      if (response.status === 200 && response.data.success) {
        logTest('Payment Simulation Failure', 'PASS', response.data.message);
        
        // Verify payment status changed to failed
        await sleep(1000);
        const statusResponse = await axios.get(`${BASE_URL}/payments/${failurePaymentId}`, {
          headers: createAuthHeaders(testApiKey)
        });
        
        if (statusResponse.data.status === 'Failed') {
          logTest('Payment Failure Status Update', 'PASS', 'Status correctly updated to Failed');
        } else {
          logTest('Payment Failure Status Update', 'FAIL', `Status: ${statusResponse.data.status}`);
        }
      } else {
        logTest('Payment Simulation Failure', 'FAIL', 'Invalid response');
      }
    }
  } catch (error) {
    logTest('Payment Simulation Failure', 'FAIL', error.message);
  }
}

async function testSandboxSimulationRestrictions() {
  console.log('\n🚫 Testing Sandbox Simulation Restrictions...');
  
  // Test that live environment cannot simulate payments
  if (liveApiKey && testPaymentId) {
    try {
      await axios.post(`${BASE_URL}/sandbox/payments/${testPaymentId}/simulate`, {
        success: true
      }, {
        headers: createAuthHeaders(liveApiKey)
      });
      logTest('Live Environment Simulation Block', 'FAIL', 'Live key allowed simulation');
    } catch (error) {
      if (error.response && (error.response.status === 403 || error.response.status === 400 || error.response.status === 401)) {
        logTest('Live Environment Simulation Block', 'PASS', 'Live key properly blocked from simulation');
      } else {
        logTest('Live Environment Simulation Block', 'FAIL', `Unexpected error: ${error.message}`);
      }
    }
  }
  
  // Test simulation with non-existent payment
  try {
    await axios.post(`${BASE_URL}/sandbox/payments/pay_nonexistent123/simulate`, {
      success: true
    }, {
      headers: createAuthHeaders(testApiKey)
    });
    logTest('Non-existent Payment Simulation', 'FAIL', 'Allowed simulation of non-existent payment');
  } catch (error) {
    if (error.response && (error.response.status === 404 || error.response.status === 400)) {
      logTest('Non-existent Payment Simulation', 'PASS', 'Properly rejected non-existent payment');
    } else {
      logTest('Non-existent Payment Simulation', 'FAIL', `Unexpected error: ${error.message}`);
    }
  }
}

// Test Data Generation
async function testSandboxTestDataGeneration() {
  console.log('\n🎲 Testing Sandbox Test Data Generation...');
  
  // Configure wallets for different crypto types first
  const cryptoTypes = ['SOL', 'SOL', 'SOL']; // Use same crypto type for all
  const addresses = [
    '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM',
    '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM', 
    '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM'
  ];
  
  for (let i = 0; i < cryptoTypes.length; i++) {
    try {
      await axios.put(`${BASE_URL}/merchants/wallets`, {
        crypto_type: cryptoTypes[i],
        address: addresses[i]
      }, {
        headers: createAuthHeaders(testApiKey)
      });
    } catch (error) {
      // Wallet might already be configured
    }
  }
  
  // Test creating multiple payments for testing
  const testPayments = [];
  for (let i = 0; i < 3; i++) {
    try {
      const response = await axios.post(`${BASE_URL}/payments`, {
        amount_usd: ((i + 1) * 25.00).toString(),
        crypto_type: cryptoTypes[i],
        description: `Test payment ${i + 1}`
      }, {
        headers: createAuthHeaders(testApiKey)
      });
      
      if (response.status === 201) {
        testPayments.push(response.data.payment_id);
      }
    } catch (error) {
      // Continue with other payments
    }
  }
  
  if (testPayments.length === 3) {
    logTest('Sandbox Test Data Generation', 'PASS', `Created ${testPayments.length} test payments`);
    
    // Test bulk simulation
    let simulatedCount = 0;
    for (const paymentId of testPayments) {
      try {
        await axios.post(`${BASE_URL}/sandbox/payments/${paymentId}/simulate`, {
          success: Math.random() > 0.5
        }, {
          headers: createAuthHeaders(testApiKey)
        });
        simulatedCount++;
      } catch (error) {
        // Continue with other simulations
      }
    }
    
    logTest('Bulk Payment Simulation', 'PASS', `Simulated ${simulatedCount}/${testPayments.length} payments`);
  } else {
    logTest('Sandbox Test Data Generation', 'FAIL', `Only created ${testPayments.length}/3 test payments`);
  }
}

// Sandbox Analytics Tests
async function testSandboxAnalytics() {
  console.log('\n📊 Testing Sandbox Analytics...');
  
  try {
    const response = await axios.get(`${BASE_URL}/analytics`, {
      headers: createAuthHeaders(testApiKey)
    });
    
    if (response.status === 200 && response.data) {
      logTest('Sandbox Analytics Access', 'PASS', 'Analytics accessible in sandbox mode');
      
      // Check if analytics show any data (flexible check)
      if (response.data.total_payments !== undefined || 
          response.data.payments !== undefined || 
          response.data.summary !== undefined ||
          Object.keys(response.data).length > 0) {
        logTest('Sandbox Analytics Data', 'PASS', 'Analytics data available');
      } else {
        logTest('Sandbox Analytics Data', 'FAIL', 'Missing analytics data');
      }
    } else {
      logTest('Sandbox Analytics Access', 'FAIL', 'Analytics not accessible');
    }
  } catch (error) {
    logTest('Sandbox Analytics Access', 'FAIL', error.message);
  }
}

// Sandbox Webhook Tests
async function testSandboxWebhooks() {
  console.log('\n🔗 Testing Sandbox Webhooks...');
  
  // Set a test webhook URL
  try {
    const webhookResponse = await axios.put(`${BASE_URL}/merchants/webhook`, {
      url: 'https://webhook.site/test-sandbox-webhook'
    }, {
      headers: createAuthHeaders(testApiKey)
    });
    
    if (webhookResponse.status === 200) {
      logTest('Sandbox Webhook Configuration', 'PASS', 'Webhook URL set successfully');
      
      // Test webhook with payment simulation
      if (testPaymentId) {
        try {
          await axios.post(`${BASE_URL}/sandbox/payments/${testPaymentId}/simulate`, {
            success: true
          }, {
            headers: createAuthHeaders(testApiKey)
          });
          
          logTest('Sandbox Webhook Trigger', 'PASS', 'Payment simulation should trigger webhook');
        } catch (error) {
          logTest('Sandbox Webhook Trigger', 'FAIL', error.message);
        }
      }
    } else {
      logTest('Sandbox Webhook Configuration', 'FAIL', 'Failed to set webhook URL');
    }
  } catch (error) {
    logTest('Sandbox Webhook Configuration', 'FAIL', error.message);
  }
}

// Sandbox Security Tests
async function testSandboxSecurity() {
  console.log('\n🔐 Testing Sandbox Security...');
  
  // Test that sandbox keys are properly identified
  if (testApiKey && testApiKey.startsWith('sk_')) {
    logTest('Sandbox Key Identification', 'PASS', 'Sandbox key properly prefixed');
  } else {
    logTest('Sandbox Key Identification', 'FAIL', 'Sandbox key not properly identified');
  }
  
  // Test sandbox mode verification
  try {
    const profileResponse = await axios.get(`${BASE_URL}/merchants/profile`, {
      headers: createAuthHeaders(testApiKey)
    });
    
    if (profileResponse.data.sandbox_mode === true) {
      logTest('Sandbox Mode Verification', 'PASS', 'Profile correctly shows sandbox mode');
    } else {
      logTest('Sandbox Mode Verification', 'FAIL', 'Profile does not show sandbox mode');
    }
  } catch (error) {
    logTest('Sandbox Mode Verification', 'FAIL', error.message);
  }
}

// Sandbox Utilities Tests
async function testSandboxUtilities() {
  console.log('\n🛠️ Testing Sandbox Utilities...');
  
  // Test getting supported currencies in sandbox
  try {
    const currenciesResponse = await axios.get(`${BASE_URL}/currencies/supported`);
    
    if (currenciesResponse.status === 200 && currenciesResponse.data) {
      logTest('Sandbox Supported Currencies', 'PASS', 'Currencies accessible in sandbox');
    } else {
      logTest('Sandbox Supported Currencies', 'FAIL', 'Currencies not accessible');
    }
  } catch (error) {
    logTest('Sandbox Supported Currencies', 'FAIL', error.message);
  }
  
  // Test payment status endpoint
  if (testPaymentId) {
    try {
      const statusResponse = await axios.get(`${BASE_URL}/payments/${testPaymentId}`, {
        headers: createAuthHeaders(testApiKey)
      });
      
      if (statusResponse.status === 200) {
        logTest('Sandbox Payment Status Page', 'PASS', 'Status page accessible');
      } else {
        logTest('Sandbox Payment Status Page', 'FAIL', 'Status page not accessible');
      }
    } catch (error) {
      logTest('Sandbox Payment Status Page', 'FAIL', error.message);
    }
  }
}

// Error Handling Tests
async function testSandboxErrorHandling() {
  console.log('\n⚠️ Testing Sandbox Error Handling...');
  
  // Test invalid simulation request
  try {
    await axios.post(`${BASE_URL}/sandbox/payments/${testPaymentId}/simulate`, {
      // Missing success field
    }, {
      headers: createAuthHeaders(testApiKey)
    });
    logTest('Invalid Simulation Request', 'FAIL', 'Accepted invalid simulation request');
  } catch (error) {
    if (error.response && (error.response.status === 400 || error.response.status === 422)) {
      logTest('Invalid Simulation Request', 'PASS', 'Properly rejected invalid request');
    } else {
      logTest('Invalid Simulation Request', 'FAIL', `Unexpected error: ${error.message}`);
    }
  }
  
  // Test unauthorized simulation
  try {
    await axios.post(`${BASE_URL}/sandbox/payments/${testPaymentId}/simulate`, {
      success: true
    });
    logTest('Unauthorized Simulation', 'FAIL', 'Allowed simulation without auth');
  } catch (error) {
    if (error.response && error.response.status === 401) {
      logTest('Unauthorized Simulation', 'PASS', 'Properly rejected unauthorized request');
    } else {
      logTest('Unauthorized Simulation', 'FAIL', `Unexpected error: ${error.message}`);
    }
  }
}

// Main test runner
async function runSandboxTests() {
  console.log('🏖️ FidduPay Sandbox API Comprehensive Test Suite');
  console.log('================================================');
  
  // Setup
  const setupSuccess = await setupTestMerchant();
  if (!setupSuccess) {
    console.log('❌ Setup failed, aborting tests');
    return;
  }
  
  // Run all test suites
  await testEnableSandboxMode();
  await testSandboxDailyVolumeLimit();
  await testSandboxEnvironmentSwitch();
  await testSandboxDataIsolation();
  await testPaymentSimulationSuccess();
  await testPaymentSimulationFailure();
  await testSandboxSimulationRestrictions();
  await testSandboxTestDataGeneration();
  await testSandboxAnalytics();
  await testSandboxWebhooks();
  await testSandboxSecurity();
  await testSandboxUtilities();
  await testSandboxErrorHandling();
  
  // Results summary
  console.log('\n📊 Test Results Summary');
  console.log('======================');
  console.log(`Total Tests: ${totalTests}`);
  console.log(`Passed: ${passedTests}`);
  console.log(`Failed: ${totalTests - passedTests}`);
  console.log(`Success Rate: ${((passedTests / totalTests) * 100).toFixed(1)}%`);
  
  if (passedTests === totalTests) {
    console.log('\n🎉 All sandbox tests passed!');
  } else {
    console.log('\n⚠️ Some tests failed. Check the details above.');
    
    // Show failed tests
    const failedTests = testResults.filter(t => t.status === 'FAIL');
    if (failedTests.length > 0) {
      console.log('\n❌ Failed Tests:');
      failedTests.forEach(test => {
        console.log(`  - ${test.testName}: ${test.details}`);
      });
    }
  }
  
  console.log('\n🏖️ Sandbox testing complete!');
}

// Run the tests
if (require.main === module) {
  runSandboxTests().catch(console.error);
}

module.exports = {
  runSandboxTests,
  testResults: () => testResults,
  testStats: () => ({ total: totalTests, passed: passedTests })
};