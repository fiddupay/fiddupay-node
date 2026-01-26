# FidduPay Fee Collection by Wallet Mode

## The 3 Wallet Modes & Fee Collection

### Mode 1: Address-Only (Merchant Provides Wallet Address)
**Scenario**: Merchant only submits their existing wallet address

**Fee Collection Method**:
- ❌ **Cannot collect processing fees automatically**
- ✅ **Network gas fees**: User pays (blockchain requirement)
- 🔄 **Processing fees**: Must be collected upfront or via separate mechanism

**Implementation**:
```rust
// Customer pays: payment_amount + network_gas_fee
// Funds go directly to merchant's wallet
// Platform cannot deduct processing fees from received funds
// Solution: Collect processing fee upfront from customer
let customer_total = payment_amount + processing_fee + network_gas_fee;
```

**Limitations**:
- No automatic fee deduction
- Requires trust-based or prepaid fee model
- Merchant receives full crypto amount

---

### Mode 2: Gateway-Generated Wallet (Platform Controls Keys)
**Scenario**: FidduPay generates wallet and holds private keys

**Fee Collection Method**:
- ✅ **Full control over funds**
- ✅ **Automatic processing fee deduction**
- ✅ **Network gas fees**: Deducted from merchant balance

**Implementation**:
```rust
// 1. Customer pays to gateway-controlled wallet
let customer_payment = payment_amount + network_gas_fee;

// 2. Gateway receives funds and deducts processing fee
let processing_fee = payment_amount * fee_rate; // 0.5-2%
let merchant_settlement = payment_amount - processing_fee;

// 3. Merchant can withdraw net amount
// Gateway keeps processing_fee as revenue
```

**Advantages**:
- Automatic fee collection
- Clean settlement process
- Platform revenue guaranteed

---

### Mode 3: Imported Private Key (Merchant Imports Their Key)
**Scenario**: Merchant imports their existing private key to the platform

**Fee Collection Method**:
- ✅ **Platform has key access for fee deduction**
- ✅ **Automatic processing fee collection**
- ⚠️ **Security risk**: Platform holds merchant's private key

**Implementation**:
```rust
// Similar to Mode 2, but using merchant's imported key
// 1. Customer pays to merchant's wallet (platform controls)
// 2. Platform deducts processing fee before settlement
// 3. Merchant gets net amount minus fees
```

**Security Considerations**:
- Platform has full access to merchant's wallet
- Requires high trust from merchant
- Key encryption and secure storage critical

---

## Fee Collection Comparison

| Wallet Mode | Processing Fee Collection | Network Gas Fee | Platform Control |
|-------------|--------------------------|-----------------|------------------|
| **Address-Only** | ❌ Manual/Upfront | User pays | None |
| **Gateway-Generated** | ✅ Automatic | Platform manages | Full |
| **Imported Key** | ✅ Automatic | Platform manages | Full |

## Recommended Implementation

### For Address-Only Mode:
```rust
pub struct AddressOnlyFeeModel {
    pub upfront_fee: bool,          // Collect fee from customer upfront
    pub subscription_model: bool,   // Monthly/yearly platform fee
    pub trust_based: bool,          // Bill merchant later
}
```

### For Gateway-Controlled Modes:
```rust
pub struct GatewayFeeModel {
    pub automatic_deduction: bool,  // Deduct from received payments
    pub settlement_net: bool,       // Settle net amount to merchant
    pub fee_transparency: bool,     // Show fee breakdown to merchant
}
```

## Current Implementation Status

Based on the codebase, FidduPay currently:
- ✅ Supports all 3 wallet modes
- ✅ Has fee calculation logic
- ⚠️ **Missing**: Automatic fee deduction for address-only mode
- ⚠️ **Missing**: Clear fee collection workflow per mode

## Recommended Solution

1. **Address-Only**: Implement upfront fee collection from customer
2. **Gateway-Generated**: Current automatic deduction works
3. **Imported Key**: Secure key storage + automatic deduction

The key insight is that **processing fees can only be automatically collected when the platform controls the receiving wallet**.
