# Non-Custodial Payment Architecture

## Overview

FidduPay now operates as a **non-custodial** payment gateway, meaning merchants control their own cryptocurrency wallets and private keys.

## How It Works

### 1. Merchant Setup
- Merchant registers and provides their wallet addresses for each cryptocurrency
- Merchant controls private keys (FidduPay never has access)
- Wallet addresses stored in `merchant_wallets` table

### 2. Payment Flow
```
Customer → Pays directly to → Merchant's Wallet
                ↓
         FidduPay tracks payment
                ↓
         Fee collected separately
```

### 3. Key Differences from Custodial

| Aspect | Custodial (Old) | Non-Custodial (New) |
|--------|----------------|-------------------|
| **Wallet Control** | FidduPay controls | Merchant controls |
| **Private Keys** | FidduPay holds | Merchant holds |
| **Payment Flow** | Customer → FidduPay → Merchant | Customer → Merchant directly |
| **Withdrawals** | Merchant requests withdrawal | No withdrawals needed |
| **Gas Fees** | FidduPay pays | Merchant handles |
| **Regulatory Risk** | High (money transmitter) | Low (payment processor) |

## Implementation Details

### Payment Creation
```rust
// Customer pays directly to merchant's wallet
let deposit_address = merchant_wallet_address;

// No intermediate addresses or forwarding needed
// Payment tracked but funds go directly to merchant
```

### Fee Collection
- **Customer pays**: Base amount + Fee
- **Merchant receives**: Full amount (base + fee)  
- **FidduPay collects fee**: Through separate mechanism (invoice/settlement)

### Wallet Management
```sql
-- Merchants provide their own wallet addresses
INSERT INTO merchant_wallets (merchant_id, crypto_type, address) 
VALUES (1, 'USDT_ETH', '0x742d35Cc6634C0532925a3b8D4C9db96590c6C87');
```

## Benefits

### For Merchants
- ✅ **Full control** over their funds
- ✅ **No withdrawal delays** - funds arrive directly
- ✅ **No gas fee worries** for FidduPay
- ✅ **Better security** - private keys never shared

### For FidduPay
- ✅ **Reduced regulatory risk** - not a money transmitter
- ✅ **No custody liability** - don't hold customer funds
- ✅ **Simplified architecture** - no withdrawal system needed
- ✅ **Lower operational costs** - no gas fees to pay

### For Customers
- ✅ **Direct payments** to merchants
- ✅ **Transparent** - can verify merchant receives funds
- ✅ **Standard crypto transactions** - familiar UX

## API Endpoints

### Create Non-Custodial Payment
```http
POST /api/v1/merchants/{merchant_id}/payments/non-custodial
```

### Get Payment Status
```http
GET /api/v1/payments/{payment_id}/non-custodial
```

## Migration Strategy

1. **New payments** use non-custodial processor by default
2. **Existing custodial payments** continue to work
3. **Merchants** gradually migrate to providing wallet addresses
4. **Old custodial system** can be deprecated over time

## Security Considerations

- **Merchant responsibility**: Must secure their private keys
- **Address validation**: Verify wallet addresses are correct
- **Network matching**: Ensure address matches cryptocurrency network
- **Backup recommendations**: Advise merchants on key backup

## Network Fee Handling

### USDT-ETH Example
- **Customer pays**: 100 USDT + 1.50 USDT fee = 101.50 USDT
- **Merchant receives**: 101.50 USDT in their wallet
- **Gas fees**: Paid by customer (standard ETH transaction)
- **FidduPay fee**: Collected through separate billing/settlement

This architecture is **safer, simpler, and more compliant** than custodial solutions! 🎯
