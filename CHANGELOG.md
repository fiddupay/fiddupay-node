# Changelog

## [2.3.6] - 2026-01-28

### 🚀 **API Centralization Release**

#### ✨ **Major Features**
- **API Centralization**: All merchant endpoints now use `/api/v1/merchant/` prefix
- **Enhanced Security**: Role-based access control with proper authentication boundaries
- **Organized Structure**: Admin endpoints under `/api/v1/admin/`, sandbox under `/api/v1/merchant/sandbox/`
- **Improved Developer Experience**: Better endpoint organization and clearer documentation

#### 🔧 **Breaking Changes**
- **Endpoint Path Updates**: All merchant endpoints centralized under `/api/v1/merchant/`
- **Admin Authentication**: Admin endpoints now use session-based authentication
- **Sandbox Endpoints**: Moved to `/api/v1/merchant/sandbox/` prefix
- **Security Endpoints**: Moved to `/api/v1/merchant/security/` prefix

#### 📦 **SDK Improvements**
- **Automatic Path Updates**: All internal endpoint paths updated automatically
- **Backward Compatibility**: Method signatures remain unchanged - no code changes required
- **Enhanced TypeScript**: Improved type definitions for all endpoints
- **Comprehensive Testing**: All 45+ merchant endpoints tested and verified

#### 🛠️ **Migration**
- **Easy Update**: Simply update to v2.3.6 - no code changes needed
- **Method Compatibility**: All existing method calls work unchanged
- **Response Formats**: No changes to response data structures
- **Error Handling**: Same error codes and message formats

#### ✅ **Verified Features**
- All merchant profile and authentication methods
- Complete payment management (create, list, get, verify)
- Balance and analytics endpoints
- Wallet management (generate, import, configure)
- Security monitoring and alerts
- Refund and withdrawal processing
- Sandbox testing capabilities
- IP whitelisting and audit logs

#### 🔒 **Security Enhancements**
- **10/10 Security Score Maintained**: All security protections intact
- **Enhanced Authentication**: Proper role-based access control
- **Rate Limiting**: Maintained across all endpoint categories
- **Threat Detection**: Real-time monitoring with automated responses

#### 📚 **Documentation Updates**
- **API Reference**: Updated with new endpoint structure
- **SDK Guide**: Complete method documentation
- **Migration Guide**: Step-by-step upgrade instructions
- **Postman Collections**: Updated with new endpoint paths
- **OpenAPI Specification**: Updated to v2.3.6

## [2.3.0] - 2026-01-27

### ✅ Added
- Daily volume limit support for non-KYC merchants
- KYC status checking in merchant profile
- Real-time daily volume remaining calculations
- Complete API coverage for all 45+ merchant endpoints
- Security monitoring and alert management
- Wallet management (generate, import, configure)
- Withdrawal management with full CRUD operations
- Enhanced TypeScript types and interfaces

### ✅ Enhanced
- MerchantProfile interface now includes `kyc_verified` and `daily_volume_remaining`
- Improved error handling across all API methods
- Better documentation with daily volume examples
- Updated response types for all endpoints

### ✅ API Coverage
- Authentication & Profile Management
- Payment Processing (create, list, verify)
- Refund Management
- Analytics & Reporting
- Balance Management
- Security Monitoring
- Audit Logging
- Sandbox Testing
- IP Whitelisting
- Webhook Management

## [2.2.0] - Previous Release
- Basic payment functionality
- Core API integration
- TypeScript support

## [2.1.0] - Previous Release
- Initial SDK release
- Basic merchant operations
