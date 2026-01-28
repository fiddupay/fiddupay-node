#!/bin/bash

# FidduPay SDK v2.3.6 GitHub Release Script
# Creates a comprehensive GitHub release with migration guide and API centralization highlights

set -e

echo "🚀 Creating GitHub Release for FidduPay SDK v2.3.6..."

# Configuration
VERSION="v2.3.6"
RELEASE_NAME="FidduPay SDK v2.3.6 - API Centralization Release"
RELEASE_BRANCH="main"

# Check if gh CLI is available
if command -v gh &> /dev/null; then
    echo "✅ GitHub CLI found. Using GitHub CLI for release creation."
    USE_GH_CLI=true
else
    echo "⚠️  GitHub CLI not found. Will provide manual instructions."
    USE_GH_CLI=false
fi

# Function to create release with GitHub CLI
create_release_with_cli() {
    echo "📤 Pushing tag to remote..."
    git push origin $VERSION 2>/dev/null || echo "Tag already pushed or push failed"

    echo "📝 Creating GitHub release with comprehensive notes..."
    
    # Read the full release notes and migration guide
    RELEASE_NOTES=$(cat << 'EOF'
# 🚀 FidduPay SDK v2.3.6 - API Centralization Release

**Release Date**: January 28, 2026  
**SDK Version**: 2.3.6  
**API Version**: v1  

## 📋 Overview

This major release implements **comprehensive API centralization** with improved route organization, enhanced security, and better developer experience. All merchant endpoints have been reorganized under the `/api/v1/merchant/` prefix for better structure and maintainability.

## 🎯 Key Highlights

### ✨ **API Centralization**
- **Unified merchant endpoints** under `/api/v1/merchant/` prefix
- **Organized admin endpoints** under `/api/v1/admin/` prefix
- **Clean public endpoints** at `/api/v1/` level
- **Structured sandbox endpoints** under `/api/v1/merchant/sandbox/`

### 🔒 **Enhanced Security**
- **Role-based access control** with proper authentication boundaries
- **Session-based admin authentication** for enhanced security
- **API key validation** with environment detection
- **Maintained 10/10 security score** with all protections intact

### 📦 **SDK Improvements**
- **Automatic endpoint updates** - no code changes required
- **Enhanced TypeScript types** for better development experience
- **Comprehensive error handling** with detailed error messages
- **Full test coverage** for all 45+ merchant endpoints

## 🔧 Breaking Changes & Migration

### **Endpoint Path Updates**

| Category | Old Path | New Path | Status |
|----------|----------|----------|--------|
| **Security** | `/api/v1/security/*` | `/api/v1/merchant/security/*` | 🔄 **Updated** |
| **Sandbox** | `/api/v1/sandbox/*` | `/api/v1/merchant/sandbox/*` | 🔄 **Updated** |
| **Profile** | `/api/v1/merchant/profile` | `/api/v1/merchant/profile` | ✅ Same |
| **Payments** | `/api/v1/merchant/payments` | `/api/v1/merchant/payments` | ✅ Same |
| **Analytics** | `/api/v1/merchant/analytics` | `/api/v1/merchant/analytics` | ✅ Same |

### **🛠️ Easy Migration Guide**

#### **Option 1: SDK Update (Recommended)**
**✅ Zero Code Changes Required!**

```bash
npm install @fiddupay/fiddupay-node@2.3.6
```

Your existing code works unchanged:
```javascript
const fiddupay = new FidduPay('sk_your_api_key');

// All methods work exactly the same
const profile = await fiddupay.merchant.getProfile();
const payments = await fiddupay.payments.list();
const events = await fiddupay.security.getEvents(); // Path updated internally
```

#### **Option 2: Direct API Migration**
Update endpoint URLs for direct API calls:

```javascript
// Before v2.3.6
fetch('/api/v1/security/events')
fetch('/api/v1/sandbox/enable')

// After v2.3.6
fetch('/api/v1/merchant/security/events')
fetch('/api/v1/merchant/sandbox/enable')
```

### **Migration Checklist**
- [ ] Update SDK: `npm install @fiddupay/fiddupay-node@2.3.6`
- [ ] Test in sandbox environment
- [ ] Update direct API calls (if any)
- [ ] Verify webhook configurations
- [ ] Test all critical flows

## 🆕 Enhanced API Organization

### **New Endpoint Structure**
```
📁 /api/v1/                    # Public endpoints
├── 🌐 status                  # System status
├── 💱 currencies/supported    # Supported currencies  
├── 📧 contact                 # Contact form
└── 💰 pricing                 # Pricing information

📁 /api/v1/merchant/           # Merchant endpoints
├── 👤 profile                 # Merchant profile
├── 📊 analytics               # Payment analytics
├── 💳 payments                # Payment management
├── 💰 balance                 # Balance information
├── 👛 wallets                 # Wallet management
├── 🔄 refunds                 # Refund processing
├── 💸 withdrawals             # Withdrawal management
├── 🔒 security/               # Security features
└── 🧪 sandbox/                # Sandbox testing

📁 /api/v1/admin/              # Admin endpoints
├── 🔐 login                   # Admin login
├── 📈 dashboard               # Admin dashboard
├── 🛡️ security/               # Security monitoring
└── 🏪 merchants               # Merchant management
```

## 🚀 Complete SDK Coverage

### **Core Operations**
```javascript
// Merchant Profile & Authentication
await fiddupay.merchant.getProfile();
await fiddupay.merchant.generateApiKey();
await fiddupay.merchant.rotateApiKey();

// Payment Management
await fiddupay.payments.create({ amount: '100.00', currency: 'USD' });
await fiddupay.payments.list();
await fiddupay.payments.verify('payment_id');

// Balance & Analytics
await fiddupay.balance.get();
await fiddupay.analytics.get();
```

### **Advanced Features**
```javascript
// Security & Monitoring (Updated paths)
await fiddupay.security.getEvents();
await fiddupay.security.getAlerts();
await fiddupay.security.acknowledgeAlert('alert_id');

// Sandbox Testing (Updated paths)
await fiddupay.sandbox.enable();
await fiddupay.sandbox.simulatePayment('payment_id', 'confirmed');

// Wallet Management
await fiddupay.wallets.generate('SOL');
await fiddupay.wallets.configureAddress('SOL', 'address');
```

## 🔒 Security Enhancements

### **Maintained 10/10 Security Score**
- ✅ **XSS Prevention** & CSRF Protection
- ✅ **SQL Injection Protection** with parameterized queries
- ✅ **Advanced Rate Limiting** (60 req/min, burst 100/10s)
- ✅ **Real-time Threat Detection** with automated responses
- ✅ **Account Lockout Protection** after failed attempts
- ✅ **Role-based Access Control** with proper boundaries

### **Authentication Improvements**
- **Enhanced API key validation** with environment detection
- **Session-based admin authentication** for better security
- **Proper authentication boundaries** between merchant/admin/public endpoints

## 📊 Supported Features

### **Payment Processing**
- **5 Blockchain Networks**: Solana, Ethereum, BSC, Polygon, Arbitrum
- **10 Cryptocurrencies**: SOL, ETH, BNB, MATIC, ARB + USDT on each network
- **Real-time Processing** with instant confirmations
- **Automatic Forwarding** to merchant wallets

### **Daily Volume Management**
- **Non-KYC Merchants**: $1,000 USD daily limit
- **KYC Verified**: Unlimited volume
- **Real-time Tracking** across all transaction types

## 🧪 Comprehensive Testing

### **Testing Completed**
- ✅ **All 45+ merchant endpoints** tested and verified
- ✅ **Admin endpoints** with session authentication tested
- ✅ **SDK integration** tested against live backend
- ✅ **Frontend integration** verified with new API structure
- ✅ **Postman collections** updated and validated
- ✅ **OpenAPI specification** updated to v2.3.6

## 🚀 Quick Start

### **Installation**
```bash
npm install @fiddupay/fiddupay-node@2.3.6
```

### **Usage**
```javascript
import { FidduPay } from '@fiddupay/fiddupay-node';

const fiddupay = new FidduPay('sk_your_api_key');

// Create a payment
const payment = await fiddupay.payments.create({
  amount: '100.00',
  currency: 'USD',
  crypto_type: 'SOL',
  description: 'Order #123'
});
```

## 📞 Support & Resources

- 📖 **Documentation**: https://docs.fiddupay.com
- 💬 **Support**: support@fiddupay.com
- 🐛 **Issues**: https://github.com/fiddupay/fiddupay-node/issues
- 📋 **Migration Guide**: See MIGRATION_GUIDE_v2.3.6.md

## 🔮 What's Next

- **Q1 2026**: Mobile SDK and GraphQL API
- **Q2 2026**: Multi-signature wallets and advanced analytics
- **Q3 2026**: Enterprise features and white-label solutions

---

**🎉 Thank you for using FidduPay! This release represents a significant step forward in our API maturity and developer experience.**

**© 2026 TechyTro Software - FidduPay v2.3.6**
EOF
)

    # Create the release
    gh release create $VERSION \
        --title "$RELEASE_NAME" \
        --notes "$RELEASE_NOTES" \
        --target $RELEASE_BRANCH \
        --latest

    echo "✅ GitHub release created successfully!"
}

# Function to provide manual instructions
provide_manual_instructions() {
    echo ""
    echo "📋 Manual Release Creation Instructions:"
    echo ""
    echo "1. Go to: https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\([^.]*\).*/\1/')/releases/new"
    echo ""
    echo "2. Use these details:"
    echo "   - Tag version: $VERSION"
    echo "   - Release title: $RELEASE_NAME"
    echo "   - Target: $RELEASE_BRANCH"
    echo "   - Mark as latest release: ✅"
    echo ""
    echo "3. Copy the release notes from: RELEASE_NOTES_v2.3.6.md"
    echo "4. Copy the migration guide from: MIGRATION_GUIDE_v2.3.6.md"
    echo ""
    echo "📝 Key points to highlight in the release:"
    echo "   ✨ API Centralization with /api/v1/merchant/ prefix"
    echo "   🔒 Enhanced security with 10/10 score maintained"
    echo "   📦 Zero code changes required for SDK users"
    echo "   🛠️ Comprehensive migration guide included"
    echo "   🧪 All 45+ endpoints tested and verified"
}

# Main execution
if [ "$USE_GH_CLI" = true ]; then
    # Check authentication
    if ! gh auth status &> /dev/null; then
        echo "❌ Not authenticated with GitHub CLI."
        echo "   Run: gh auth login"
        echo ""
        provide_manual_instructions
        exit 1
    fi
    
    create_release_with_cli
else
    provide_manual_instructions
fi

echo ""
echo "📋 Post-Release Checklist:"
echo "   1. ✅ Verify release on GitHub"
echo "   2. 📦 Update npm package (if needed)"
echo "   3. 📢 Notify users about the release"
echo "   4. 📊 Monitor for any issues"
echo "   5. 📖 Update documentation links"
echo ""
echo "🔗 Release will be available at:"
echo "   https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\([^.]*\).*/\1/')/releases/tag/$VERSION"
echo ""
echo "🎉 FidduPay SDK v2.3.6 release process completed!"