# FidduPay Merchant API Test Suite

Comprehensive test suite for all FidduPay merchant-related API endpoints. This test suite covers merchant registration, profile management, wallet configuration, payment operations, analytics, webhooks, and all merchant functionality.

## 🚀 Quick Start

### Prerequisites
- Node.js 16+ installed
- FidduPay backend server running on `http://127.0.0.1:8080`
- Internet connection for API calls

### Installation
```bash
cd tests
npm install
```

### Running Tests

#### Comprehensive Test Suite (Recommended)
```bash
npm test
# or
node merchant-api-comprehensive.js
```

#### Quick Test Suite (Fast)
```bash
npm run test:quick
# or
node merchant-api-quick.js
```

#### Verbose Mode
```bash
npm run test:verbose
```

## 📋 Test Coverage

### Merchant Management
- ✅ Merchant Registration
- ✅ Merchant Login
- ✅ Get Merchant Profile
- ✅ API Key Generation
- ✅ API Key Rotation
- ✅ Environment Switching (Sandbox/Live)

### Wallet Configuration
- ✅ Set Wallet Address (SOL, USDT_SPL, USDT_BEP20, ETH, USDT_ETH)
- ✅ Get Wallet Configurations
- ✅ Wallet Management Operations
- ✅ Gas Requirements Check

### Payment Operations
- ✅ Create Payment
- ✅ Get Payment Details
- ✅ List Payments
- ✅ Payment Verification
- ✅ Payment Status Tracking

### Refund Operations
- ✅ Create Refund
- ✅ Get Refund Details
- ✅ Complete Refund

### Analytics & Reporting
- ✅ Get Analytics Data
- ✅ Export Analytics (CSV)
- ✅ Balance Information
- ✅ Balance History

### Webhook Management
- ✅ Set Webhook URL
- ✅ Webhook Configuration

### Sandbox Operations
- ✅ Enable Sandbox Mode
- ✅ Simulate Payment Confirmation
- ✅ Sandbox Environment Testing

### Security Features
- ✅ IP Whitelist Management
- ✅ Audit Log Access
- ✅ Security Monitoring

### Withdrawal Operations
- ✅ Create Withdrawal
- ✅ List Withdrawals
- ✅ Get Withdrawal Details
- ✅ Cancel Withdrawal

### System Information
- ✅ Supported Currencies
- ✅ System Status
- ✅ Health Checks

## 🔧 Test Configuration

### Environment Variables
The test suite uses the following configuration:
- `BASE_URL`: API base URL (default: `http://127.0.0.1:8080/api/v1`)
- `TEST_EMAIL`: Unique test email generated per run
- `TEST_BUSINESS_NAME`: Test business name

### Test Data
Each test run creates:
- Fresh merchant account with unique email
- Test wallet addresses for multiple cryptocurrencies
- Sample payments and refunds
- Webhook configurations
- Security settings

## 📊 Test Results

The test suite provides detailed results including:
- ✅ **Pass/Fail Status** for each endpoint
- ⏱️ **Execution Time** for performance monitoring
- 📈 **Success Rate** percentage
- 🔑 **Generated API Keys** for debugging
- ❌ **Failed Test Details** for troubleshooting

### Sample Output
```
🚀 Starting Comprehensive Merchant API Test Suite
============================================================

📝 Testing Merchant Registration...
✅ Merchant Registration ID: 123

👤 Testing Get Merchant Profile...
✅ Get Merchant Profile Email: test_1234567890@example.com

💰 Testing Wallet Configuration...
✅ Set SOL Wallet 7xKXtg2CW...
✅ Set USDT_SPL Wallet 7xKXtg2CW...
✅ Set USDT_BEP20 Wallet 0x742d35C...

============================================================
📊 TEST RESULTS SUMMARY
============================================================
✅ Passed: 45/48 tests
❌ Failed: 3/48 tests
⏱️  Duration: 12.34 seconds
📈 Success Rate: 93.8%
```

## 🛠️ Troubleshooting

### Common Issues

#### Connection Refused
```
Error: connect ECONNREFUSED 127.0.0.1:8080
```
**Solution**: Ensure the FidduPay backend server is running on port 8080.

#### Authentication Errors
```
Error: 401 Unauthorized
```
**Solution**: Check that merchant registration is successful and API key is properly generated.

#### Test Failures
```
❌ Create Payment Failed to create payment
```
**Solution**: Verify wallet configuration is complete before running payment tests.

### Debug Mode
Run tests with verbose logging:
```bash
DEBUG=* node merchant-api-comprehensive.js
```

## 🔍 Test Architecture

### Test Structure
```
tests/
├── merchant-api-comprehensive.js  # Full test suite
├── merchant-api-quick.js         # Quick test suite
├── package.json                  # Dependencies
└── README.md                     # This file
```

### Test Flow
1. **Setup Phase**: Register fresh merchant account
2. **Configuration Phase**: Set up wallets and webhooks
3. **Operation Phase**: Test all API endpoints
4. **Validation Phase**: Verify responses and data integrity
5. **Cleanup Phase**: Generate test report

### Error Handling
- Graceful error handling for network issues
- Detailed error reporting for debugging
- Continuation of tests even if individual tests fail
- Proper exit codes for CI/CD integration

## 🚦 CI/CD Integration

### Exit Codes
- `0`: All tests passed
- `1`: One or more tests failed

### Usage in CI/CD
```yaml
- name: Run Merchant API Tests
  run: |
    cd tests
    npm install
    npm test
```

## 📝 Adding New Tests

To add new test cases:

1. Create a new test function:
```javascript
async function testNewFeature() {
  console.log('\n🆕 Testing New Feature...');
  
  try {
    const response = await axios.get(`${BASE_URL}/new-endpoint`, {
      headers: createAuthHeaders(testApiKey)
    });
    
    if (response.status === 200) {
      logTest('New Feature', 'PASS', 'Feature works');
    } else {
      logTest('New Feature', 'FAIL', 'Feature failed');
    }
  } catch (error) {
    logTest('New Feature', 'FAIL', error.message);
  }
}
```

2. Add to main test runner:
```javascript
await testNewFeature();
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add your test cases
4. Ensure all tests pass
5. Submit a pull request

## 📄 License

This test suite is part of the FidduPay project and follows the same licensing terms.

---

**Note**: This test suite creates real API calls and generates actual data. Use only in development/testing environments.