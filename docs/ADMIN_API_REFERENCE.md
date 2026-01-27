# FidduPay Admin API Reference (INTERNAL)

**⚠️ INTERNAL DOCUMENTATION - NOT FOR PUBLIC DISTRIBUTION**

This document contains admin-only endpoints for system administration and should not be shared publicly.

## Admin Authentication

Admin users use session tokens instead of API keys:
```
Authorization: Bearer admin_session_192
```

## Admin Endpoints

### Dashboard
```http
GET /api/v1/admin/dashboard
Authorization: Bearer {admin_session_token}
```

### Merchant Management
```http
GET /api/v1/admin/merchants
GET /api/v1/admin/merchants/{merchant_id}
POST /api/v1/admin/merchants/{merchant_id}/suspend
POST /api/v1/admin/merchants/{merchant_id}/activate
DELETE /api/v1/admin/merchants/{merchant_id}/delete
```

### Security Management
```http
GET /api/v1/admin/security/events
GET /api/v1/admin/security/alerts
POST /api/v1/admin/security/alerts/{alert_id}/acknowledge
GET /api/v1/admin/security/settings
PUT /api/v1/admin/security/settings
```

### System Configuration
```http
GET /api/v1/admin/config/environment
PUT /api/v1/admin/config/environment
GET /api/v1/admin/config/fees
PUT /api/v1/admin/config/fees
GET /api/v1/admin/config/limits
PUT /api/v1/admin/config/limits
```

### Payment Management
```http
GET /api/v1/admin/payments
GET /api/v1/admin/payments/{payment_id}
POST /api/v1/admin/payments/{payment_id}/force-confirm
POST /api/v1/admin/payments/{payment_id}/force-fail
```

### Withdrawal Management
```http
GET /api/v1/admin/withdrawals
POST /api/v1/admin/withdrawals/{withdrawal_id}/approve
POST /api/v1/admin/withdrawals/{withdrawal_id}/reject
```

### Analytics & Reporting
```http
GET /api/v1/admin/analytics/platform
GET /api/v1/admin/analytics/revenue
GET /api/v1/admin/reports/transactions
GET /api/v1/admin/reports/merchants
```

### Wallet Management
```http
GET /api/v1/admin/wallets/hot
GET /api/v1/admin/wallets/cold
GET /api/v1/admin/wallets/balances
POST /api/v1/admin/wallets/transfer
```

### User Management
```http
GET /api/v1/admin/users
POST /api/v1/admin/users
DELETE /api/v1/admin/users/{user_id}
PUT /api/v1/admin/users/{user_id}/permissions
```

### System Maintenance
```http
GET /api/v1/admin/system/health
GET /api/v1/admin/system/logs
POST /api/v1/admin/system/backup
POST /api/v1/admin/system/maintenance
```

## Daily Volume Configuration

Admins can view and configure daily volume limits:

```http
GET /api/v1/admin/config/limits
```

Response includes:
```json
{
  "daily_volume_limit_non_kyc_usd": "1000.00",
  "rate_limit_requests_per_minute": 60
}
```

## Security Notes

- Admin endpoints require elevated privileges
- All admin actions are logged and audited
- Session tokens expire after 24 hours
- IP restrictions apply to admin access
