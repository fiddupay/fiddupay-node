# FidduPay Fee Structure & Implementation (2026)

## Fee Types & Who Pays

### 1. Network Gas Fees (User Pays)
**Blockchain Requirement**: Users MUST pay network transaction fees
- **Ethereum**: Base fee + Priority fee (EIP-1559)
- **BSC**: Gas price × Gas limit
- **Polygon**: Base fee + Priority fee
- **Arbitrum**: Gas price × Gas limit (L2 optimized)
- **Solana**: Base fee (5000 lamports) + Prioritization fee

### 2. Gateway Processing Fees (Merchant Pays)
**FidduPay Revenue Model**: 0.5% - 2% processing fee
- Deducted from merchant's received amount
- Configurable per merchant tier
- Optional "user pays" mode available

## Implementation Structure

```rust
pub struct TransactionFee {
    pub network_fee: NetworkFee,     // User pays (blockchain)
    pub processing_fee: ProcessingFee, // Merchant pays (gateway)
}

pub struct NetworkFee {
    pub base_fee: Option<Decimal>,    // EIP-1559 base fee
    pub priority_fee: Option<Decimal>, // Priority/tip fee
    pub gas_limit: u64,              // Estimated gas units
    pub total_native: Decimal,       // Total in native currency
}

pub struct ProcessingFee {
    pub rate: Decimal,               // 0.005 = 0.5%
    pub fixed_amount: Option<Decimal>, // Optional fixed fee
    pub paid_by: FeePayer,           // Merchant or User
    pub total_usd: Decimal,          // Total processing fee
}
```

## Fee Collection Flow

1. **Payment Request**: User initiates payment
2. **Gas Estimation**: System estimates network fees using 2026 RPC methods
3. **User Payment**: User pays `payment_amount + network_fee`
4. **Network Processing**: Blockchain processes transaction
5. **Gateway Processing**: FidduPay deducts processing fee
6. **Merchant Settlement**: Merchant receives `payment_amount - processing_fee`

## 2026 RPC Methods Used

### Ethereum & EVM Chains
- `eth_feeHistory` - EIP-1559 fee history for base + priority fees
- `eth_estimateGas` - Gas limit estimation for transactions
- `eth_gasPrice` - Legacy gas price (BSC, Arbitrum)

### Solana
- `getRecentPrioritizationFees` - Recent prioritization fees from 150 blocks
- Uses median/75th/90th percentile for dynamic fee adjustment

## Working RPC Endpoints (2026)

```env
# Ethereum Mainnet
ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY

# Polygon
POLYGON_RPC_URL=https://polygon-rpc.com

# BSC
BSC_RPC_URL=https://bsc-dataseed.binance.org

# Arbitrum
ARBITRUM_RPC_URL=https://arb1.arbitrum.io/rpc

# Solana
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
```

## Real-time Updates

WebSocket subscriptions for live gas price monitoring:
- **Ethereum**: `eth_subscribe` to `newHeads` for base fee updates
- **Solana**: `slotSubscribe` for slot updates, then fetch prioritization fees
- **Other EVM**: Polling every 15 seconds for gas price updates

## Fee Transparency

Users see:
- Network fee (required by blockchain)
- Processing fee (if user pays mode enabled)
- Total payment amount

Merchants see:
- Payment received
- Processing fee deducted
- Net settlement amount
