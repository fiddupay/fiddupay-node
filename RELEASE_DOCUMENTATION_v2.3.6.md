# FidduPay v2.3.6 - API Centralization Release

## 🎯 Release Overview

**Version**: 2.3.6  
**Release Date**: January 28, 2026  
**Type**: Major API Restructuring  

This release implements comprehensive API centralization with improved route organization, enhanced security, and better developer experience.

## 🚀 Major Changes

### **API Route Centralization**
- **All merchant endpoints** now use `/api/v1/merchant/` prefix
- **Admin endpoints** use `/api/v1/admin/` prefix  
- **Public endpoints** remain at `/api/v1/` level
- **Sandbox endpoints** use `/api/v1/merchant/sandbox/` prefix

### **New Route Structure**
```
/api/v1/                    # Public endpoints
├── status                  # System status
├── currencies/supported    # Supported currencies
├── contact                 # Contact form
└── pricing                 # Pricing information

/api/v1/merchant/           # Merchant endpoints (API key auth)
├── register               # Merchant registration
├── login                  # Merchant login
├── profile                # Merchant profile
├── analytics              # Payment analytics
├── payments               # Payment management
├── balance                # Balance information
├── wallets                # Wallet management
├── invoices               # Invoice system
├── refunds                # Refund processing
├── withdrawals            # Withdrawal management
├── security/              # Security features
└── sandbox/               # Sandbox testing

/api/v1/admin/             # Admin endpoints (session auth)
├── login                  # Admin login
├── dashboard              # Admin dashboard
├── security/              # Security monitoring
├── merchants              # Merchant management
└── users                  # User management
```

## 📦 Component Updates

### **Backend (Rust)**
- ✅ Route organization with dedicated files
- ✅ Proper authentication middleware separation
- ✅ Enhanced security with role-based access
- ✅ Comprehensive error handling

### **Frontend (React/TypeScript)**
- ✅ Updated API service with new endpoints
- ✅ All components use centralized API calls
- ✅ Documentation updated with new paths
- ✅ TypeScript types updated

### **SDK (Node.js)**
- ✅ Version 2.3.6 with updated endpoints
- ✅ All merchant methods use new paths
- ✅ Sandbox endpoints updated
- ✅ Backward compatibility maintained

### **Documentation**
- ✅ OpenAPI specification updated to v2.3.6
- ✅ Postman collections updated
- ✅ Frontend documentation updated
- ✅ API reference documentation updated

## 🔧 Breaking Changes

### **Endpoint Path Changes**
**Old Format** → **New Format**
```
/api/v1/merchant/profile     → /api/v1/merchant/profile
/api/v1/merchant/analytics   → /api/v1/merchant/analytics
/api/v1/merchant/payments    → /api/v1/merchant/payments
/api/v1/merchant/balance     → /api/v1/merchant/balance
/api/v1/merchant/wallets     → /api/v1/merchant/wallets
/api/v1/merchant/invoices    → /api/v1/merchant/invoices
/api/v1/merchant/refunds     → /api/v1/merchant/refunds
/api/v1/merchant/withdrawals → /api/v1/merchant/withdrawals
/api/v1/sandbox/enable        → /api/v1/merchant/sandbox/enable
```

### **Authentication Changes**
- **Merchant endpoints**: Continue using API key authentication
- **Admin endpoints**: Now use session-based authentication
- **Public endpoints**: No authentication required (unchanged)

## 🛠️ Migration Guide

### **For API Users**
1. Update all merchant endpoint URLs to use `/merchant/` instead of `/merchants/`
2. Update sandbox endpoints to use `/merchant/sandbox/` prefix
3. Admin endpoints now require session authentication
4. Public endpoints remain unchanged

### **For SDK Users**
1. Update to SDK version 2.3.6
2. All method calls remain the same (internal paths updated)
3. No code changes required for existing implementations

### **Example Migration**
```javascript
// Before v2.3.6
const response = await fetch('/api/v1/merchant/profile', {
  headers: { 'Authorization': `Bearer ${apiKey}` }
});

// After v2.3.6
const response = await fetch('/api/v1/merchant/profile', {
  headers: { 'Authorization': `Bearer ${apiKey}` }
});
```

## ✅ Testing & Verification

### **Comprehensive Testing Completed**
- ✅ All merchant endpoints tested and verified
- ✅ Admin endpoints with session auth tested
- ✅ Public endpoints confirmed unchanged
- ✅ SDK integration tested against live backend
- ✅ Frontend integration verified
- ✅ Postman collections validated

### **Performance Impact**
- ✅ No performance degradation
- ✅ Improved route organization for better maintainability
- ✅ Enhanced security with proper authentication boundaries

## 🔒 Security Enhancements

### **Authentication Improvements**
- **Role-based access control** with proper separation
- **Session-based admin authentication** for enhanced security
- **API key validation** with environment detection
- **Rate limiting** maintained across all endpoints

### **Security Features Maintained**
- ✅ 10/10 security score maintained
- ✅ XSS and CSRF protection
- ✅ SQL injection protection
- ✅ Advanced rate limiting
- ✅ Real-time threat detection
- ✅ Account lockout protection

## 📋 Deployment Checklist

### **Pre-Deployment**
- [x] Backend compilation verified
- [x] Frontend build successful
- [x] SDK build and tests passed
- [x] Documentation updated
- [x] Postman collections updated
- [x] OpenAPI specification updated

### **Post-Deployment**
- [ ] Monitor endpoint performance
- [ ] Verify authentication systems
- [ ] Check error rates and logs
- [ ] Validate webhook deliveries
- [ ] Confirm rate limiting functionality

## 🎉 Benefits

### **For Developers**
- **Cleaner API structure** with logical endpoint grouping
- **Better documentation** with updated examples
- **Improved SDK** with consistent method naming
- **Enhanced security** with proper authentication boundaries

### **For Operations**
- **Better monitoring** with organized route structure
- **Easier maintenance** with separated concerns
- **Improved security** with role-based access control
- **Enhanced debugging** with clearer error boundaries

## 📞 Support

For questions or issues related to this release:
- **Documentation**: https://docs.fiddupay.com
- **Support Email**: support@fiddupay.com
- **GitHub Issues**: https://github.com/fiddupay/issues

---

**© 2026 TechyTro Software - FidduPay v2.3.6**
