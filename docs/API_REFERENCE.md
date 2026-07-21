# FidduPay API Reference v2.5.6
> [!IMPORTANT]
> **Security Model (v2.5.6)**: All high-risk operations including Merchant Withdrawals and Customer-scoped actions (`/sweep` and `/withdraw`) now require the **Merchant's Transaction PIN** in the request body. Customer-specific PINs have been deprecated and removed. Note: `/pay-merchant` is customer-initiated and does not require a PIN in this model.

Official technical specification for the FidduPay cryptocurrency payment gateway API.

## Base URLs
- **Production**: `https://api.fiddupay.com`
- **Sandbox**: `http://localhost:8080` (Development)

## Authentication
All requests must include a Bearer token in the `Authorization` header:
`Authorization: Bearer <your_api_key>`

---

## Merchant API

### Core Operations
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/merchants/profile` | Get merchant profile & KYC status |
| GET | `/api/v1/merchants/status` | Get merchant readiness / health |
| GET | `/api/v1/merchants/balance` | Get current account balance |
| GET | `/api/v1/merchants/balance/history` | Get balance change history |
| GET | `/api/v1/merchants/settings` | Get all merchant settings |
| PATCH | `/api/v1/merchants/settings` | Update unified merchant settings |
| GET | `/api/v1/merchants/fee-setting` | Get fee payment configuration |
| POST | `/api/v1/merchants/api-keys/generate` | Generate secret API key |
| POST | `/api/v1/merchants/api-keys/rotate` | Rotate secret API key |
| POST | `/api/v1/merchants/webhook/test` | Trigger a test webhook event |
| POST | `/api/v1/merchants/environment/switch` | Switch environment (Sandbox <-> Live) |

### Security & Monitoring
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/merchants/security/settings` | Get merchant security specs |
| PUT | `/api/v1/merchants/security/settings` | Update merchant security settings |
| POST | `/api/v1/merchants/security/password` | Update account password |
| POST | `/api/v1/merchants/security/transaction-pin` | Set merchant transaction PIN |
| POST | `/api/v1/merchants/security/transaction-pin/verify` | Verify transaction PIN |
| POST | `/api/v1/merchants/security/wallets/lock` | Toggle master wallet lock |
| POST | `/api/v1/merchants/security/customers/wallets/lock` | Toggle customer wallet lock |
| GET | `/api/v1/merchants/security/alerts` | Get security alerts |
| POST | `/api/v1/merchants/security/alerts/:id/acknowledge` | Acknowledge a security alert |
| GET | `/api/v1/merchants/security/events` | List security-related events |
| GET | `/api/v1/merchants/security/balance-alerts` | List low-balance alerts |
| POST | `/api/v1/merchants/security/balance-alerts/:id/resolve` | Resolve a balance alert |
| GET | `/api/v1/merchants/security/gas-check` | Check gas levels for all wallets |
| GET | `/api/v1/merchants/ip-whitelist` | List allowed IP CIDRs |

### Customer Management
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/merchants/customers` | Register a new customer |
| GET | `/api/v1/merchants/customers` | List all registered customers |
| GET | `/api/v1/merchants/customers/summary` | Get customer directory summary stats |
| POST | `/api/v1/merchants/customers/bulk-provision` | Bulk provision customer wallets |
| POST | `/api/v1/merchants/customers/verify-wallets` | Verify & auto-repair customer wallets |
| GET | `/api/v1/merchants/customers/lookup-address/:address` | Reverse lookup wallet address to customer |
| GET | `/api/v1/merchants/customers/wallets-audit` | Full audit of active vs historical customer wallets |
| POST | `/api/v1/merchants/customers/:external_id/wallets` | Provision customer wallets |
| GET | `/api/v1/merchants/customers/:external_id/wallets` | Get customer wallets |
| GET | `/api/v1/merchants/customers/:external_id/balances` | Get customer wallet balances |
| POST | `/api/v1/merchants/customers/:external_id/sweep` | Sweep funds to master balance |
| POST | `/api/v1/merchants/customers/:external_id/withdraw` | Withdraw from customer wallet |
| POST | `/api/v1/merchants/customers/:external_id/pay-merchant` | Internal payment from customer to merchant |
| PATCH | `/api/v1/merchants/customers/:external_id/permissions` | Update customer permissions |
| GET | `/api/v1/merchants/customers/:external_id/deposit-address/:crypto` | Get specific deposit address |
| GET | `/api/v1/merchants/customers/:external_id/transactions` | List customer transactions |
| PATCH | `/api/v1/merchants/customers/:external_id/status` | Update customer active status |
| POST | `/api/v1/merchants/customers/:external_id/deactivate` | Deactivate customer deposits |

### Payment Operations
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/merchants/payments` | Create a new payment |
| GET | `/api/v1/merchants/payments` | List payments with filtering |
| GET | `/api/v1/merchants/payments/:id` | Retrieve payment details |
| POST | `/api/v1/merchants/payments/:id/cancel` | Cancel a pending payment |
| POST | `/api/v1/merchants/payments/:id/verify` | Manually trigger on-chain verification |
| POST | `/api/v1/merchants/payments/:id/select` | Finalize asset selection on multi-checkout |

### Refund Management
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/merchants/refunds` | Create a refund |
| GET | `/api/v1/merchants/refunds` | List all refunds |
| GET | `/api/v1/merchants/refunds/:id` | Get refund details |
| POST | `/api/v1/merchants/refunds/:id/complete`| Complete/Approve a refund |

### Wallet Management
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/merchants/wallets` | Unified wallet setup (generate/import/address) |
| GET | `/api/v1/merchants/wallets` | Get wallet configurations |
| GET | `/api/v1/merchants/wallets/balances` | Get on-chain wallet balances |
| DELETE | `/api/v1/merchants/wallets/:crypto_type` | Revoke a wallet |
| GET | `/api/v1/merchants/wallets/gas-check` | Check gas requirements |
| GET | `/api/v1/merchants/wallets/gas-estimates` | Get gas fee estimates |
| GET | `/api/v1/merchants/wallets/withdrawal-capability/:crypto` | Check withdrawal capability |

### Withdrawal Management
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/merchants/withdrawals` | Create a withdrawal request |
| GET | `/api/v1/merchants/withdrawals` | List withdrawals |
| GET | `/api/v1/merchants/withdrawals/:id` | Get withdrawal details |
| POST | `/api/v1/merchants/withdrawals/:id/cancel` | Cancel a withdrawal |
| POST | `/api/v1/merchants/withdrawals/:id/process`| Process withdrawal on-chain |

### Invoice Management
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/merchants/invoices` | Create a new invoice |
| GET | `/api/v1/merchants/invoices` | List invoices with filtering |
| GET | `/api/v1/merchants/invoices/:invoice_id` | Get invoice details |

### Analytics & Audit Logs
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/merchants/analytics` | Get general volume/stats summary |
| GET | `/api/v1/merchants/analytics/export` | Export analytics data (CSV) |
| GET | `/api/v1/merchants/transactions` | List unified transactions history |
| GET | `/api/v1/merchants/audit-logs` | Retrieve merchant activity audit logs |

### Address-Only Operations (WIP - Experimental)
> [!WARNING]
> Address-Only Mode (Forwarding Mode) is currently in **Beta**. Endpoints in this section are subject to change and should not be used in critical production flows yet.

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/merchants/address-only/create` | Create a native-only deposit address |
| GET | `/api/v1/merchants/address-only/status` | Get status by payment ID |
| GET | `/api/v1/merchants/address-only/currencies` | List supported native currencies |
| GET | `/api/v1/merchants/address-only/stats` | Get merchant address-only stats |
| GET | `/api/v1/merchants/address-only/fee-setting` | Get current fee payment setting |
| PUT | `/api/v1/merchants/address-only/fee-setting` | Update who pays the fee |

---

## Error Responses (Mode Enforcement)
Starting with v2.5.0, the gateway strictly enforces settlement modes & fully atomic transactional integrity:
- **403 Forbidden**: "Standard payments are not available in Forwarding mode."
- **403 Forbidden**: "Address-Only payments are not available in Managed mode."
- **Unified EVM Wallet**: All EVM networks (ETH, BSC, Polygon, Arbitrum) share a single private key and address. Updating one syncs them all.
- **Bitcoin Support**: Native Bitcoin (SegWit) is supported for both Managed and Forwarding modes.

---

## SDK Integration
For a more convenient integration, use our official Node.js SDK:
[Node.js SDK Guide](NODE_SDK.md)

## Postman Collection
A complete Postman collection is available for testing:
[FidduPay Merchant API Postman Collection](../fiddupay-node-sdk/postman/FidduPay-Merchant-API.postman_collection.json)
