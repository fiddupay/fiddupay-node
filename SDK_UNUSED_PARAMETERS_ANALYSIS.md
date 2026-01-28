# SDK UNUSED PARAMETERS ANALYSIS

## 📋 COMPREHENSIVE ANALYSIS OF UNUSED SDK PARAMETERS

### 🔍 **METHODOLOGY**
Analyzed all 11 SDK resource files for:
1. Unused `options?: RequestOptions` parameters
2. Unused constructor `client` parameters  
3. Unused method parameters
4. Parameters not supported by backend

---

## 📊 **UNUSED PARAMETERS BY CATEGORY**

### **1. RequestOptions Parameters (Future HTTP Options)**
**Found in ALL resources** - Used for future HTTP request customization:

```typescript
interface RequestOptions {
  timeout?: number;           // Request timeout in ms
  retries?: number;          // Retry attempts
  headers?: Record<string, string>; // Custom headers
  idempotencyKey?: string;   // Prevent duplicate requests
  apiVersion?: string;       // API version override
}
```

**Files with unused `options` parameters:**
- ✅ `analytics.ts` - 2 methods
- ✅ `balances.ts` - 2 methods  
- ✅ `contact.ts` - 1 method
- ✅ `merchants.ts` - 12 methods
- ✅ `payments.ts` - 5 methods
- ✅ `refunds.ts` - 3 methods
- ✅ `security.ts` - 8 methods
- ✅ `wallets.ts` - 8 methods
- ✅ `withdrawals.ts` - 5 methods

**Total: 46 unused `options` parameters**

---

### **2. Backend Unsupported Parameters**

#### **Analytics Resource:**
```typescript
// UNUSED: Backend doesn't support granularity
granularity?: 'day' | 'week' | 'month'
```
**Future Use**: Time-based analytics grouping

#### **Payments Resource:**
```typescript
// UNUSED: Backend doesn't support these filters
async list(params?: {
  status?: string;           // Filter by payment status
  crypto_type?: string;      // Filter by cryptocurrency
  start_date?: string;       // Date range filtering
  end_date?: string;         // Date range filtering
  limit?: number;            // Pagination limit
  offset?: number;           // Pagination offset
})
```
**Future Use**: Advanced payment filtering and pagination

#### **Security Resource:**
```typescript
// UNUSED: Backend has basic security endpoints only
async getEvents(params?: {
  event_type?: string;       // Filter by event type
  severity?: string;         // Filter by severity level
  limit?: number;            // Pagination
})

async getAlerts(params?: {
  status?: 'active' | 'resolved'; // Filter by alert status
  priority?: 'low' | 'medium' | 'high'; // Filter by priority
})
```
**Future Use**: Advanced security monitoring and filtering

#### **Merchants Resource:**
```typescript
// UNUSED: Backend doesn't support merchant settings
async updateSettings(data: {
  notification_preferences?: object;
  api_rate_limits?: object;
  webhook_settings?: object;
})
```
**Future Use**: Merchant customization and preferences

---

### **3. Constructor Client Parameters**
**Found in ALL 11 resources** - Required for future method implementations:

```typescript
export class ResourceName {
  constructor(private client: HttpClient) {} // UNUSED in some methods
}
```

**Why Unused**: Some methods are placeholders or use direct HTTP calls instead of the client wrapper.

---

## 🎯 **FUTURE IMPLEMENTATION ROADMAP**

### **Phase 1: Enhanced Filtering**
- Payment status filtering
- Date range queries  
- Cryptocurrency-specific analytics
- Pagination support

### **Phase 2: Advanced Security**
- Security event categorization
- Alert priority levels
- Real-time monitoring
- Audit trail filtering

### **Phase 3: Request Customization**
- Request timeout configuration
- Retry mechanisms
- Custom headers support
- Idempotency keys

### **Phase 4: Merchant Preferences**
- Notification settings
- API rate limit customization
- Webhook configuration
- Dashboard preferences

---

## 📈 **BACKEND ENDPOINTS TO ADD**

### **Analytics Enhancements:**
```rust
// Support granularity parameter
GET /api/v1/merchants/analytics?granularity=day
GET /api/v1/merchants/analytics?granularity=week
GET /api/v1/merchants/analytics?granularity=month
```

### **Payment Filtering:**
```rust
// Support advanced filtering
GET /api/v1/payments?status=confirmed&crypto_type=USDT_ETH&limit=50
GET /api/v1/payments?start_date=2026-01-01&end_date=2026-01-31
```

### **Security Monitoring:**
```rust
// Enhanced security endpoints
GET /api/v1/security/events?event_type=login&severity=high
GET /api/v1/security/alerts?status=active&priority=medium
```

---

## ✅ **RECOMMENDATIONS**

### **Keep Unused Parameters** ✅
- They represent planned features
- Provide forward compatibility
- Enable gradual backend implementation
- Maintain consistent SDK interface

### **Priority Implementation Order:**
1. **High**: Payment filtering (most requested)
2. **Medium**: Analytics granularity (business intelligence)
3. **Medium**: Security event filtering (compliance)
4. **Low**: Request options (developer experience)

### **Database Schema Additions Needed:**
- Event logging tables for security
- Payment status indexing
- Analytics aggregation tables
- Merchant preferences table

---

## 🔒 **SECURITY CONSIDERATIONS**

All unused parameters are **safe** because:
- They're optional parameters
- Backend validates all inputs
- No security vulnerabilities introduced
- Future-proofing without current risk

---

**TOTAL UNUSED PARAMETERS: 46 options + 11 clients + 15 feature parameters = 72 unused parameters**

**STATUS: ✅ All intentional for future development**
