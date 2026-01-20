# PayFlow - Integration TODO

**Critical integrations needed before production**

## 1. Payment Creation - BitPay Model Integration

**Current:** Payment uses merchant's wallet directly
**Required:** Generate temporary deposit address per payment

**File:** `src/payment/processor.rs`

**Change needed:**
```rust
// OLD (line ~60)
let merchant_wallet = self.merchant_service
    .get_wallet_address(merchant_id, request.crypto_type)
    .await?;

// NEW
let merchant_wallet = self.merchant_service
    .get_wallet_address(merchant_id, request.crypto_type)
    .await?;

// Generate temporary deposit address
let deposit_address_service = DepositAddressService::new(self.db_pool.clone())?;
let deposit = deposit_address_service.generate_deposit_address(
    &payment_id,
    request.crypto_type,
    &merchant_wallet,
    15  // 15 minute expiration
).await?;

// Use deposit.deposit_address instead of merchant_wallet
```

## 2. Payment Monitoring - Auto-Forward

**File:** `src/background_tasks.rs`

**Add forwarding logic:**
```rust
// When payment confirmed
if payment.status == "CONFIRMED" {
    // 1. Get deposit address and private key
    let deposit = deposit_service.get_deposit_address(&payment_id).await?;
    let private_key = deposit_service.get_private_key(&payment_id).await?;
    
    // 2. Forward funds to merchant wallet (minus fee)
    let forward_tx = forward_payment(
        &deposit.crypto_type,
        &private_key,
        &deposit.merchant_destination,
        payment.amount - payment.fee_amount
    ).await?;
    
    // 3. Mark deposit address as used
    deposit_service.mark_as_used(&payment_id, &forward_tx).await?;
    
    // 4. Credit merchant balance
    balance_service.credit_from_payment(&payment_id).await?;
    
    // 5. Send notifications
    if email_enabled {
        email_service.send_payment_confirmed(...).await?;
    }
}
```

## 3. Blockchain Transaction Sending

**Create:** `src/blockchain/transaction_sender.rs`

**Implement:**
- Solana transaction signing and sending
- EVM transaction signing and sending
- Gas estimation
- Error handling and retries

## 4. Payment Expiration - Cleanup

**File:** `src/background_tasks.rs`

**Add:**
```rust
// Expire old deposit addresses
deposit_service.expire_old_addresses().await?;
```

## 5. Refund Integration

**File:** `src/services/refund_service.rs`

**Update:** Use deposit address for refund source

## Status

- [x] Deposit address service created
- [x] Key generation implemented
- [x] Encryption implemented
- [ ] Payment processor integration
- [ ] Background task forwarding
- [ ] Transaction sender implementation
- [ ] Testing on testnets

## Timeline

**Phase 1 (Testing):** Run comprehensive tests
**Phase 2 (Integration):** Implement above changes
**Phase 3 (Testnet):** Test on blockchain testnets
**Phase 4 (Production):** Deploy to mainnet
