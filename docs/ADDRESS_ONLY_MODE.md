# Address-Only Mode Implementation (Phase 1)

## Overview
Address-only mode with auto-forwarding for native currencies only. Gateway receives payments, deducts processing fees, and automatically forwards the remainder to merchant addresses.

## Supported Currencies (Phase 1)
- ✅ **ETH** (Ethereum)
- ✅ **BNB** (Binance Smart Chain)  
- ✅ **MATIC** (Polygon)
- ✅ **ARB** (Arbitrum)
- ✅ **SOL** (Solana)

❌ **USDT variants excluded** (requires gas fees for forwarding)

## Flow

### 1. Payment Request Creation
```
Merchant requests payment → Gateway generates unique deposit address → Customer pays to deposit address
```

### 2. Auto-Forwarding Process
```
Payment received → Processing fee deducted → Remaining amount forwarded to merchant address
```

### 3. Fee Structure
- **Processing Fee**: 0.75% (deducted from payment)
- **Gas Fees**: Paid by gateway (minimal for native currencies)
- **Merchant Receives**: Requested amount - processing fee - gas fee

## Database Schema

### `address_only_payments`
- Tracks each payment request
- Links gateway deposit address to merchant destination
- Records fee calculations and status

### `address_only_forwarding_txs`  
- Tracks forwarding transactions
- Records gas fees and transaction hashes
- Monitors forwarding status

## API Endpoints

### `POST /api/address-only/create`
Create new address-only payment request

### `GET /api/address-only/status?payment_id=xxx`
Check payment and forwarding status

### `GET /api/address-only/supported-currencies`
List supported native currencies

## Implementation Status

✅ **Core Service**: `AddressOnlyService` with payment creation and auto-forwarding
✅ **Database Schema**: Tables for payments and forwarding transactions  
✅ **API Endpoints**: Payment creation and status checking
⚠️ **TODO**: Actual blockchain address generation and transaction sending
⚠️ **TODO**: Payment monitoring and webhook notifications

## Next Steps

1. Implement real address generation per network
2. Add blockchain transaction sending
3. Set up payment monitoring service
4. Add webhook notifications for status updates
5. **Phase 2**: Add USDT support with gas fee management
