# FidduPay API Reference v2.4.6

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
| GET | `/api/v1/merchants/balance` | Get current account balance |
| GET | `/api/v1/merchants/balance/history` | Get balance change history (v2.4.6) |
| PATCH | `/api/v1/merchants/settings` | Update unified merchant settings |
| POST | `/api/v1/merchants/api-keys/rotate` | Rotate secret API key |

### Security & Monitoring (v2.4.6)
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/merchants/security/lock-master` | Toggle master wallet lock |
| POST | `/api/v1/merchants/security/lock-customers` | Toggle customer wallet lock |
| GET | `/api/v1/merchants/security/alerts` | Get security alerts |
| POST | `/api/v1/merchants/security/alerts/:id/acknowledge` | Acknowledge a security alert |
| GET | `/api/v1/merchants/security/events` | List security-related events |
| GET | `/api/v1/merchants/security/gas-check` | Check gas levels for all wallets |

### Customer Management (v2.4.6)
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/merchants/customers` | Register a new customer |
| GET | `/api/v1/merchants/customers` | List all registered customers |
| POST | `/api/v1/merchants/customers/:external_id/wallets` | Provision customer wallets |
| GET | `/api/v1/merchants/customers/:external_id/balances` | Get customer wallet balances |
| POST | `/api/v1/merchants/customers/:external_id/sweep` | Sweep funds to master balance |
| POST | `/api/v1/merchants/customers/:external_id/pay-merchant` | internal payment from customer to merchant |
| PATCH | `/api/v1/merchants/customers/:external_id/permissions` | Update customer permissions |
| GET | `/api/v1/merchants/customers/:external_id/deposit-address/:crypto` | Get specific deposit address |
| GET | `/api/v1/merchants/customers/:external_id/transactions` | List customer transactions |

### Payment Operations
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/merchants/payments` | Create a new payment |
| GET | `/api/v1/merchants/payments/:id` | Retrieve payment details |
| GET | `/api/v1/merchants/payments` | List payments with filtering |
| POST | `/api/v1/merchants/payments/:id/cancel` | Cancel a pending payment |

---

## SDK Integration
For a more convenient integration, use our official Node.js SDK:
[Node.js SDK Guide](NODE_SDK.md)

## Postman Collection
A complete Postman collection is available for testing:
[FidduPay Merchant API Postman Collection](../fiddupay-node-sdk/postman/FidduPay-Merchant-API.postman_collection.json)
