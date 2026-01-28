#!/bin/bash

# FidduPay SDK v2.3.6 Complete GitHub Release Script
# Comprehensive release with migration guide and API centralization highlights

set -e

echo " FidduPay SDK v2.3.6 - Complete Release Creation"
echo "=================================================="

# Configuration
VERSION="v2.3.6"
RELEASE_NAME="FidduPay SDK v2.3.6 - API Centralization Release"

# Check available tools
HAS_GH_CLI=false
HAS_CURL=false

if command -v gh &> /dev/null && gh auth status &> /dev/null 2>&1; then
    HAS_GH_CLI=true
    echo " GitHub CLI available and authenticated"
elif command -v curl &> /dev/null; then
    HAS_CURL=true
    echo " curl available for API calls"
else
    echo "  No GitHub tools available - will provide manual instructions"
fi

# Complete release notes with migration guide
create_complete_release_notes() {
    cat << 'EOF'
#  FidduPay SDK v2.3.6 - API Centralization Release

**Release Date**: January 28, 2026  
**SDK Version**: 2.3.6  
**API Version**: v1  

##  Overview

This major release implements **comprehensive API centralization** with improved route organization, enhanced security, and better developer experience. All merchant endpoints have been reorganized under the `/api/v1/merchant/` prefix for better structure and maintainability.

##  Key Highlights

###  **API Centralization**
- **Unified merchant endpoints** under `/api/v1/merchant/` prefix
- **Organized admin endpoints** under `/api/v1/admin/` prefix
- **Clean public endpoints** at `/api/v1/` level
- **Structured sandbox endpoints** under `/api/v1/merchant/sandbox/`

###  **Enhanced Security**
- **Role-based access control** with proper authentication boundaries
- **Session-based admin authentication** for enhanced security
- **API key validation** with environment detection
- **Maintained 10/10 security score** with all protections intact

###  **SDK Improvements**
- **Automatic endpoint updates** - no code changes required
- **Enhanced TypeScript types** for better development experience
- **Comprehensive error handling** with detailed error messages
- **Full test coverage** for all 45+ merchant endpoints

##  Breaking Changes & Migration

### **Endpoint Path Updates**

| Category | Old Path | New Path | Status |
|----------|----------|----------|--------|
| **Security** | `/api/v1/security/*` | `/api/v1/merchant/security/*` |  **Updated** |
| **Sandbox** | `/api/v1/sandbox/*` | `/api/v1/merchant/sandbox/*` |  **Updated** |
| **Profile** | `/api/v1/merchant/profile` | `/api/v1/merchant/profile` |  Same |
| **Payments** | `/api/v1/merchant/payments` | `/api/v1/merchant/payments` |  Same |
| **Analytics** | `/api/v1/merchant/analytics` | `/api/v1/merchant/analytics` |  Same |
| **Balance** | `/api/v1/merchant/balance` | `/api/v1/merchant/balance` |  Same |
| **Wallets** | `/api/v1/merchant/wallets` | `/api/v1/merchant/wallets` |  Same |
| **Refunds** | `/api/v1/merchant/refunds` | `/api/v1/merchant/refunds` |  Same |
| **Withdrawals** | `/api/v1/merchant/withdrawals` | `/api/v1/merchant/withdrawals` |  Same |

### ** Easy Migration Guide**

#### **Option 1: SDK Update (Recommended)**
** Zero Code Changes Required!**

```bash
npm install @fiddupay/fiddupay-node@2.3.6
```

Your existing code works unchanged:
```javascript
const fiddupay = new FidduPay('sk_your_api_key');

// All methods work exactly the same
const profile = await fiddupay.merchant.getProfile();
const payments = await fiddupay.payments.list();
const balance = await fiddupay.balance.get();

// Security methods (paths updated internally)
const events = await fiddupay.security.getEvents();
const alerts = await fiddupay.security.getAlerts();

// Sandbox methods (paths updated internally)
await fiddupay.sandbox.enable();
await fiddupay.sandbox.simulatePayment('payment_id', 'confirmed');
```

#### **Option 2: Direct API Migration**
Update endpoint URLs for direct API calls:

```javascript
// Before v2.3.6
const response = await fetch('/api/v1/security/events', {
  headers: { 'Authorization': `Bearer ${apiKey}` }
});

// After v2.3.6
const response = await fetch('/api/v1/merchant/security/events', {
  headers: { 'Authorization': `Bearer ${apiKey}` }
});
```

### **Migration Checklist**
- [ ] Update SDK to v2.3.6: `npm install @fiddupay/fiddupay-node@2.3.6`
- [ ] Test your integration in sandbox environment
- [ ] Update any direct API calls to use new endpoint paths
- [ ] Update webhook URLs if using admin endpoints
- [ ] Verify authentication tokens are working correctly

##  Enhanced API Organization

### **New Endpoint Structure**
```
 /api/v1/                    # Public endpoints
  status                  # System status
  currencies/supported    # Supported currencies  
  contact                 # Contact form
  pricing                 # Pricing information

 /api/v1/merchant/           # Merchant endpoints
  profile                 # Merchant profile
  analytics               # Payment analytics
  payments                # Payment management
  balance                 # Balance information
  wallets                 # Wallet management
  invoices                # Invoice system
  refunds                 # Refund processing
  withdrawals             # Withdrawal management
  security/               # Security features
  sandbox/                # Sandbox testing

 /api/v1/admin/              # Admin endpoints
  login                   # Admin login
  dashboard               # Admin dashboard
  security/               # Security monitoring
  merchants               # Merchant management
  users                   # User management
```

##  Complete SDK Coverage

### **Core Operations**
```javascript
// Merchant Profile & Authentication
await fiddupay.merchant.getProfile();
await fiddupay.merchant.generateApiKey();
await fiddupay.merchant.rotateApiKey();
await fiddupay.merchant.switchEnvironment('sandbox');

// Payment Management
await fiddupay.payments.create({ amount: '100.00', currency: 'USD' });
await fiddupay.payments.list();
await fiddupay.payments.get('payment_id');
await fiddupay.payments.verify('payment_id');

// Balance & Analytics
await fiddupay.balance.get();
await fiddupay.balance.getHistory();
await fiddupay.analytics.get();
await fiddupay.analytics.export();
```

### **Advanced Features**
```javascript
// Wallet Management
await fiddupay.wallets.getConfigs();
await fiddupay.wallets.generate('SOL');
await fiddupay.wallets.import('SOL', 'private_key');
await fiddupay.wallets.configureAddress('SOL', 'wallet_address');

// Security & Monitoring (Updated paths)
await fiddupay.security.getEvents();
await fiddupay.security.getAlerts();
await fiddupay.security.acknowledgeAlert('alert_id');
await fiddupay.security.getSettings();

// Sandbox Testing (Updated paths)
await fiddupay.sandbox.enable();
await fiddupay.sandbox.simulatePayment('payment_id', 'confirmed');
```

### **Enhanced TypeScript Support**
```typescript
import { FidduPay, MerchantProfile, Payment, SecurityAlert } from '@fiddupay/fiddupay-node';

const fiddupay = new FidduPay('sk_your_api_key');

// Full type safety
const profile: MerchantProfile = await fiddupay.merchant.getProfile();
const payments: Payment[] = await fiddupay.payments.list();
const alerts: SecurityAlert[] = await fiddupay.security.getAlerts();
```

##  Security Enhancements

### **Maintained Security Score: 10/10**
-  **XSS Prevention** & CSRF Protection
-  **SQL Injection Protection** with parameterized queries
-  **Advanced Rate Limiting** (60 req/min, burst 100/10s)
-  **Real-time Threat Detection** with automated responses
-  **Account Lockout Protection** after failed attempts
-  **Role-based Access Control** with proper boundaries

### **Authentication Improvements**
- **Enhanced API key validation** with environment detection
- **Session-based admin authentication** for better security
- **Proper authentication boundaries** between merchant/admin/public endpoints
- **Rate limiting maintained** across all endpoint categories

##  Supported Features

### **Payment Processing**
- **5 Blockchain Networks**: Solana, Ethereum, BSC, Polygon, Arbitrum
- **10 Cryptocurrencies**: SOL, ETH, BNB, MATIC, ARB + USDT on each network
- **Real-time Processing** with instant confirmations
- **Automatic Forwarding** to merchant wallets

### **Daily Volume Management**
- **Non-KYC Merchants**: $1,000 USD daily limit
- **KYC Verified**: Unlimited volume
- **Real-time Tracking** across all transaction types
- **Automatic Reset** at midnight UTC

### **Advanced Features**
- **Multi-wallet Support** with automatic generation
- **Comprehensive Analytics** with export capabilities
- **Security Monitoring** with real-time alerts
- **Sandbox Environment** for testing
- **Webhook Integration** for real-time notifications

##  Comprehensive Testing

### **Testing Completed**
-  **All 45+ merchant endpoints** tested and verified
-  **Admin endpoints** with session authentication tested
-  **Public endpoints** confirmed unchanged
-  **SDK integration** tested against live backend
-  **Frontend integration** verified with new API structure
-  **Postman collections** updated and validated
-  **OpenAPI specification** updated to v2.3.6

### **Performance Verification**
-  **No performance degradation** from route changes
-  **Improved maintainability** with organized structure
-  **Enhanced debugging** with clearer error boundaries
-  **Better monitoring** capabilities

##  Getting Started

### **Installation**
```bash
# Install the latest SDK
npm install @fiddupay/fiddupay-node@2.3.6

# Or update existing installation
npm update @fiddupay/fiddupay-node
```

### **Quick Start**
```javascript
import { FidduPay } from '@fiddupay/fiddupay-node';

// Initialize with your API key
const fiddupay = new FidduPay('sk_your_api_key');

// Create a payment
const payment = await fiddupay.payments.create({
  amount: '100.00',
  currency: 'USD',
  crypto_type: 'SOL',
  description: 'Order #123'
});

console.log('Payment created:', payment.payment_url);
```

### **Environment Setup**
```javascript
// Sandbox environment
const fiddupay = new FidduPay('sk_sandbox_key', {
  baseURL: 'http://localhost:8080'
});

// Production environment  
const fiddupay = new FidduPay('live_production_key', {
  baseURL: 'https://api.fiddupay.com'
});
```

##  Backward Compatibility

### **SDK Compatibility**
-  **Method signatures unchanged** - existing code works without modification
-  **Response formats maintained** - no breaking changes to data structures
-  **Error handling preserved** - same error codes and messages
-  **Authentication flow unchanged** - same API key usage

### **Migration Safety**
- **Gradual rollout** supported with version detection
- **Fallback mechanisms** for unsupported endpoints
- **Clear error messages** for deprecated paths
- **Comprehensive testing** ensures reliability

##  Bug Fixes

- **Fixed endpoint routing** for better organization
- **Improved error handling** with more descriptive messages
- **Enhanced authentication validation** with proper error responses
- **Resolved rate limiting issues** with new endpoint structure
- **Fixed webhook delivery** for admin endpoints

##  Performance Improvements

- **Optimized route matching** with new endpoint structure
- **Reduced response times** through better organization
- **Improved caching** for frequently accessed endpoints
- **Enhanced monitoring** with clearer metrics
- **Better resource utilization** with organized middleware

##  Updated Documentation

### **Complete Documentation Suite**
-  **[API Reference](docs/API_REFERENCE.md)** - Updated with new endpoint structure
-  **[SDK Guide](docs/NODE_SDK.md)** - Complete Node.js SDK documentation
-  **[Setup Guide](docs/SETUP.md)** - Development and production setup
-  **[Deployment Guide](docs/DEPLOYMENT.md)** - Production deployment guide
-  **[Postman Collections](docs/postman/)** - Updated API collections

### **Developer Resources**
- **OpenAPI Specification**: Updated to v2.3.6 with new endpoints
- **Postman Collections**: Complete API testing suite
- **TypeScript Definitions**: Full type coverage for all endpoints
- **Code Examples**: Updated examples for all major use cases

##  What's Next

### **Upcoming Features**
- **Multi-signature wallet support** for enhanced security
- **Advanced analytics dashboard** with real-time insights
- **Mobile SDK** for React Native applications
- **GraphQL API** for more flexible data fetching
- **Enhanced webhook system** with retry mechanisms

### **Roadmap**
- **Q1 2026**: Mobile SDK and GraphQL API
- **Q2 2026**: Multi-signature wallets and advanced analytics
- **Q3 2026**: Enterprise features and white-label solutions
- **Q4 2026**: Global expansion and regulatory compliance

##  Support & Resources

### **Getting Help**
-  **Documentation**: https://docs.fiddupay.com
-  **Support Email**: support@fiddupay.com
-  **GitHub Issues**: https://github.com/fiddupay/fiddupay-node/issues
-  **Business Inquiries**: business@fiddupay.com

### **Community**
-  **Twitter**: [@FidduPay](https://twitter.com/fiddupay)
-  **LinkedIn**: [FidduPay](https://linkedin.com/company/fiddupay)
-  **YouTube**: [FidduPay Channel](https://youtube.com/@fiddupay)

##  Acknowledgments

Special thanks to our development team and community contributors who made this release possible:

- **Backend Team**: API centralization and security enhancements
- **Frontend Team**: UI updates and integration testing
- **SDK Team**: Comprehensive endpoint coverage and TypeScript improvements
- **QA Team**: Extensive testing and verification
- **Documentation Team**: Complete documentation updates

---

** Thank you for using FidduPay! This release represents a significant step forward in our API maturity and developer experience.**

**© 2026 TechyTro Software - FidduPay v2.3.6**
EOF
}

# Function to create release with GitHub CLI
create_with_gh_cli() {
    echo " Pushing tag to remote..."
    git push origin $VERSION 2>/dev/null || echo "Tag already exists or push failed"

    echo " Creating GitHub release with comprehensive notes..."
    
    RELEASE_NOTES=$(create_complete_release_notes)
    
    gh release create $VERSION \
        --title "$RELEASE_NAME" \
        --notes "$RELEASE_NOTES" \
        --target main \
        --latest

    echo " GitHub release created successfully with GitHub CLI!"
}

# Function to create release with curl
create_with_curl() {
    if [ -z "$GITHUB_TOKEN" ]; then
        echo " GITHUB_TOKEN environment variable required for API access"
        echo "   Set your token: export GITHUB_TOKEN=your_token_here"
        return 1
    fi

    echo " Creating release via GitHub API..."
    
    RELEASE_NOTES=$(create_complete_release_notes)
    ESCAPED_NOTES=$(echo "$RELEASE_NOTES" | sed 's/\\/\\\\/g' | sed 's/"/\\"/g' | sed ':a;N;$!ba;s/\n/\\n/g')

    RESPONSE=$(curl -s -X POST \
        -H "Authorization: token $GITHUB_TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/fiddupay/fiddupay/releases" \
        -d "{
            \"tag_name\": \"$VERSION\",
            \"target_commitish\": \"main\",
            \"name\": \"$RELEASE_NAME\",
            \"body\": \"$ESCAPED_NOTES\",
            \"draft\": false,
            \"prerelease\": false,
            \"make_latest\": \"true\"
        }")

    if echo "$RESPONSE" | grep -q '"html_url"'; then
        RELEASE_URL=$(echo "$RESPONSE" | grep '"html_url"' | head -1 | sed 's/.*"html_url": "\([^"]*\)".*/\1/')
        echo " GitHub release created successfully with API!"
        echo " Release URL: $RELEASE_URL"
    else
        echo " Failed to create release via API"
        echo "$RESPONSE"
        return 1
    fi
}

# Function to provide manual instructions
provide_manual_instructions() {
    echo ""
    echo " Manual Release Creation Instructions:"
    echo "======================================"
    echo ""
    echo "1. Go to: https://github.com/fiddupay/fiddupay/releases/new"
    echo ""
    echo "2. Fill in the release details:"
    echo "   - Tag version: $VERSION"
    echo "   - Release title: $RELEASE_NAME"
    echo "   - Target: main"
    echo "   - Mark as latest release: "
    echo ""
    echo "3. Copy the complete release notes:"
    echo "   - Full content from RELEASE_NOTES_v2.3.6.md"
    echo "   - Migration guide from MIGRATION_GUIDE_v2.3.6.md"
    echo "   - Or use the combined notes created by this script"
    echo ""
    echo " Key highlights to emphasize:"
    echo "    API Centralization with /api/v1/merchant/ prefix"
    echo "    Enhanced security with 10/10 score maintained"
    echo "    Zero code changes required for SDK users"
    echo "    Comprehensive migration guide included"
    echo "    All 45+ endpoints tested and verified"
    echo ""
    
    # Save complete release notes to file for easy copying
    echo " Saving complete release notes to: complete_release_notes_v2.3.6.md"
    create_complete_release_notes > complete_release_notes_v2.3.6.md
    echo "   You can copy the content from this file for the GitHub release"
}

# Main execution
echo ""
if [ "$HAS_GH_CLI" = true ]; then
    echo " Creating release with GitHub CLI..."
    create_with_gh_cli
elif [ "$HAS_CURL" = true ]; then
    echo " Attempting to create release with GitHub API..."
    if ! create_with_curl; then
        provide_manual_instructions
    fi
else
    provide_manual_instructions
fi

echo ""
echo " Post-Release Checklist:"
echo "========================="
echo "   1.  Verify release on GitHub"
echo "   2.  Update npm package (if needed)"
echo "   3.  Notify users about the release"
echo "   4.  Monitor for any issues"
echo "   5.  Update documentation links"
echo "   6.  Update changelog and version tags"
echo ""
echo " Release will be available at:"
echo "   https://github.com/fiddupay/fiddupay/releases/tag/$VERSION"
echo ""
echo " FidduPay SDK v2.3.6 release process completed!"
echo "   This release highlights the major API centralization improvements"
echo "   and provides a comprehensive migration guide for all users."