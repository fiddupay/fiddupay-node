# Delora Integration — Impact Analysis for FidduPay

## Current FidduPay Model (Before Delora)

```
Customer Wallet → FidduPay Deposit Address → FidduPay detects deposit
                                                  ↓
                                         Atomic Ledger Transfer
                                                  ↓
                                         Merchant Balance (FidduPay ledger)
                                                  ↓
                                         Merchant sweeps to external wallet
```

- FidduPay **controls the deposit wallets** — it generates addresses, tracks them, detects incoming transactions
- Funds sit in FidduPay-controlled wallets until merchant sweeps
- Customer **must** send the exact crypto type the merchant configured (USDT on Polygon only)
- FidduPay does **internal ledger credits** — not actual on-chain transfers to merchant until sweep

---

## How Delora Changes This

Delora is **NOT** an asset custodian, wallet provider, or storage system. It's a pure **execution/routing infrastructure** — an API that returns `calldata` which your backend or customer's wallet signs and broadcasts. Funds flow directly from the customer's wallet to the merchant's wallet. Delora itself never holds funds.

The integration shifts FidduPay from a **custodial wallet-holding** model to an **execution+tracking** model for cross-chain payments:

```
Before Delora:
Customer → [FidduPay Wallet] → [Ledger] → Merchant sweeps

After Delora:
Customer → signs one tx → [Delora Router Contract] → [Bridge] → [DEX] → Merchant Wallet
                ↑
           FidduPay tracks this entire flow
```

---

## What Delora Improves / Upgrades

| Area | Before | With Delora |
|------|--------|-------------|
| **Token flexibility** | Customer must hold the exact token merchant wants (13 crypto types across 5 chains) | Customer can pay with any token on any EVM/SVM chain Delora supports |
| **Checkout UX** | If customer doesn't have the right token, they leave FidduPay → go to DEX/bridge → swap → come back → pay | Single flow: pick your token, sign one tx, done |
| **Merchant reach** | Merchant limited to customers who hold their exact configured token | Merchant can accept payments from any chain, any token |
| **Speed of settlement** | Fast — internal ledger credit is instant once blockchain confirms deposit | Slower for cross-chain (bridges take minutes to hours), but same-chain swaps are instant |
| **FidduPay custody** | Holds funds in deposit wallets until sweep | For cross-chain: funds route directly customer→merchant. No FidduPay custody needed |
| **Revenue** | Platform fee from merchant (0.75%) | Platform fee + integrator fee from every cross-chain swap (0.5%) |
| **Infrastructure** | Must maintain wallet generation, RPC nodes, blockchain monitors for 5 chains | Delora handles routing; FidduPay tracks status via its own DB/cache |
| **New token addition** | Requires new CryptoType variant, new wallet config, new RPC, new monitor | Token is automatically available if Delora supports it |
| **Same-chain payments** | Works fine — no change needed | Delora is not used for same-chain same-token — existing flow remains |

---

## What Delora Does NOT Help With

| Limitation | Why |
|------------|-----|
| **Same-chain same-token payments** | No benefit — current FidduPay flow is already optimal. Delora is bypassed in this case |
| **Asset storage** | Delora is NOT a wallet or custodian. It never holds user funds. FidduPay's existing deposit wallets and balance ledger are unchanged for direct payments |
| **Withdrawal processing** | Existing withdrawal system is untouched — Delora only handles the incoming payment side |
| **Bitcoin** | Delora doesn't support Bitcoin. BTC payments continue through the existing flow |
| **P2P trades** | Delora could optionally route cross-chain escrow funding, but the core P2P escrow mechanics are unchanged |
| **Merchant balances / ledger** | For cross-chain payments routed through Delora, the payment goes directly to the merchant's wallet. FidduPay needs to detect it on the destination chain to credit the internal ledger (same as verification flow) |
| **Gas fees** | Customer pays gas on the origin chain. If the bridge requires gas on destination chain, the customer still needs native tokens there. Delora doesn't sponsor gas |

---

## How It Affects "Storing" Crypto Assets

**Delora does NOT store assets.** It is not a wallet, vault, or custodian. Here's how assets are handled:

1. **Customer assets**: Never leave their wallet until they sign the transaction. The swap happens atomically on-chain.
2. **FidduPay deposit wallets**: Unchanged for same-chain payments. For cross-chain, the deposit wallet is bypassed — funds route directly to merchant's wallet.
3. **Merchant receiving**: The merchant wallet is the destination address on the target chain. Delora's contracts route the swapped/bridged assets there directly. FidduPay detects arrival via its existing blockchain monitors.
4. **One-time payment vs recurring storage**: Every Delora transaction is a **one-time payment**. There is no "Delora balance" or "Delora account" where funds sit. If a customer pays 10 times cross-chain, 10 separate on-chain transactions happen — just like existing FidduPay payments.

---

## How Delora Supports Payment Processing (End-to-End)

### Customer Pays Merchant

1. Customer visits FidduPay checkout for a merchant invoice
2. Customer has SOL on Solana, merchant wants USDT on Polygon
3. Customer selects "Pay with SOL on Solana" from cross-chain picker
4. FidduPay backend calls `GET /v1/quotes` with `receiverAddress=merchant_polygon_wallet`, `integrator=fiddupay&fee=0.005`
5. Delora returns calldata pointing to the **Delora Diamond Router** on Solana
6. Customer signs the transaction via their wallet
7. On-chain: SOL → Delora Router → Bridge Facet → swap to USDT → Bridge to Polygon → USDT arrives at merchant's Polygon wallet
8. FidduPay's blockchain monitor detects the incoming USDT on Polygon at the merchant's address
9. FidduPay credits the merchant's internal balance ledger
10. CrossChainPayment status updates from `tx_submitted` → `tx_confirmed` → `bridge_pending` → `bridge_complete` → `completed`

### Merchant Receives Money

- **Same-chain**: via existing FidduPay flow (no change) — funds arrive at deposit wallet, detected, ledger credited
- **Cross-chain**: merchant wallet receives funds directly on the destination chain from Delora's settlement. FidduPay bridge monitor polls for completion. Once confirmed, existing `confirm_payment()` logic credits the merchant's balance ledger.

---

## Revenue Model Change

| Stream | Before Delora | After Delora |
|--------|--------------|--------------|
| Platform fee from merchant | 0.75% on every payment | 0.75% on same-chain, same for cross-chain |
| Integrator fee from Delora | N/A | 0.5% on every cross-chain swap — goes directly to FidduPay's configured treasury wallet |
| **Total on cross-chain** | 0.75% | **1.25%** (0.75% platform + 0.5% integrator) |

---

## Architecture Impact Summary

| System Component | Impact |
|-----------------|--------|
| **Deposit wallet generation** | Unchanged for same-chain. Bypassed for cross-chain |
| **Blockchain monitors** | Must now also detect deposits on destination chains for cross-chain completions |
| **Ledger/balance system** | Unchanged — same `confirm_payment()` is called regardless of path |
| **Merchant sweep** | Unchanged for same-chain. For cross-chain, funds are already at merchant's external wallet — no sweep needed |
| **Payment verifier** | `confirm_payment()` is called by bridge monitor for cross-chain completions. Same three-layer double-credit protection applies |
| **Database** | New `cross_chain_payments` table tracks quote→registration→bridge→complete lifecycle |
| **Customer UX** | New currency picker on payment page, single-signature flow for cross-chain |
| **Security** | Delora routes through audited Diamond contracts with encoded constraints. Calldata validated before exposing to frontend |

---

## Delora Protocol Architecture

**Delora Diamond** (EIP-2535 Multi-facet Proxy):
- Single entry point that delegates calls to specialized Facet Modules
- Modular — new liquidity providers added as separate facets without redeploying
- Upgradeable — business logic evolves without changing the main contract address
- Deterministic — all routes follow strict execution rules encoded in the returned transaction payload

**Facet Contracts:**
- Bridge Facets: Connect to supported cross-chain bridges (Across, Relay, Mayan, Symbiosis, etc.)
- GenericSwap Facet: Route same-chain swaps to the best available DEX (OpenOcean, Jupiter, OKX, etc.)

**Helper Contracts:**
- DiamondCutFacet: Add, replace, or remove facet logic
- DiamondLoupeFacet: Query available functions and facet addresses
- OwnershipFacet: Manage contract ownership and permissions
- WithdrawFacet: Enable authorized funds withdrawal

**Security model:**
- Routes validated against supported adapters, encoded constraints, allowlists, and execution requirements
- Execution designed to fail if encoded constraints are not met
- Calldata returned as unsigned transaction payload — integrators sign and broadcast from their own wallet or backend
- Integrator fees on EVM accumulated in Fee Pool contract, claimable by configured wallet
- Integrator fees on Solana transferred directly to configured wallet during execution

---

## Partnership

- Register at [portal.delora.build](https://portal.delora.build) for API key (higher rate limits: 200 req/min vs 200/2hr)
- Contact: team@delora.build or Telegram @whoisfedelya
- Configure EVM and Solana fee collection wallets in the Partner Portal
- Set custom integrator string (e.g., "fiddupay") for fee attribution
- Monitor volumes, routes, and accumulated fees through the Partner Portal analytics
