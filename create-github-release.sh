#!/bin/bash

# Create GitHub Release for v2.3.4
# This is a one-time fix for the missing release

echo " Creating GitHub Release for v2.3.4..."

# Create release using GitHub API
curl -X POST \
  -H "Accept: application/vnd.github.v3+json" \
  -H "Authorization: token $GITHUB_TOKEN" \
  https://api.github.com/repos/fiddupay/fiddupay-node/releases \
  -d '{
    "tag_name": "v2.3.4",
    "target_commitish": "main",
    "name": "FidduPay Node.js SDK v2.3.4 - Complete Backend API Coverage",
    "body": "##  FidduPay Node.js SDK v2.3.4 - Complete Backend API Coverage\n\n##  Major Release - Production Ready!\n\nThis release provides **complete coverage of all backend merchant API endpoints** with comprehensive security measures and extensive testing.\n\n##  What'\''s New\n\n###  **Complete API Coverage (45+ Endpoints)**\n-  **Merchant Management**: Registration, login, profile, API keys\n-  **Payment Processing**: Create, retrieve, cancel, list payments  \n-  **Wallet Operations**: Single/batch wallet configuration\n-  **Withdrawal Management**: Create, process, cancel withdrawals\n-  **Balance & History**: Real-time balances and transaction history\n-  **Security Features**: IP whitelist, audit logs, 2FA support\n-  **Webhook Integration**: Signature validation and event handling\n-  **Analytics & Reporting**: Transaction analytics and insights\n-  **Sandbox Testing**: Complete testing utilities\n\n###  **Security Enhancements**\n- **Contact Form Security**: 30+ malicious pattern detection and blocking\n- **Input Sanitization**: XSS and injection prevention\n- **Admin Endpoint Exclusion**: Zero admin endpoints in SDK (merchant-only access)\n- **Public Endpoint Audit**: Verified no sensitive data exposure\n\n###  **Comprehensive Testing**\n- **189 Tests Passing** across 8 comprehensive test suites\n- **Contact Form Tests**: Security validation and error handling\n- **Integration Tests**: End-to-end SDK functionality\n- **Webhook Tests**: Signature validation and event processing\n- **Error Handling**: Comprehensive error scenario coverage\n\n##  Installation\n\n```bash\nnpm install @fiddupay/fiddupay-node\n```\n\n##  Quick Start\n\n```javascript\nimport FidduPay from '\''@fiddupay/fiddupay-node'\'';\n\nconst client = new FidduPay({\n  apiKey: '\''sk_test_your_api_key'\'',\n  environment: '\''sandbox'\'' // or '\''production'\''\n});\n\n// Create a payment\nconst payment = await client.payments.create({\n  amount: '\''100.00'\'',\n  crypto_type: '\''ETH'\'',\n  description: '\''Test payment'\''\n});\n```\n\n##  Supported Cryptocurrencies\n\n**5 Major Blockchain Networks:**\n- **Solana**: SOL + USDT (SPL)\n- **Ethereum**: ETH + USDT (ERC-20)  \n- **Binance Smart Chain**: BNB + USDT (BEP-20)\n- **Polygon**: MATIC + USDT\n- **Arbitrum**: ARB + USDT\n\n**Total: 10 cryptocurrency options**\n\n##  Package Stats\n\n- **Size**: 24.9 kB (125.7 kB unpacked)\n- **Files**: 62 total files\n- **Dependencies**: axios ^1.13.3\n- **TypeScript**: Full type definitions included\n\n**Full Changelog**: https://github.com/fiddupay/fiddupay-node/compare/v1.0.1...v2.3.4",
    "draft": false,
    "prerelease": false
  }'

echo " GitHub Release created!"
echo " View at: https://github.com/fiddupay/fiddupay-node/releases/tag/v2.3.4"
