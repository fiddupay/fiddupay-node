# P2P Exchange Implementation Plan

Adding a Peer-to-Peer (P2P) crypto exchange is a massive feature that essentially introduces a brand new product to the FidduPay ecosystem. It requires strict isolation from the Merchant checkout flow to maintain security (specifically Escrow integrity) and usability.

## Recommended UI Architecture

**DO NOT add P2P to the existing Merchant Dashboard.** 
The Merchant Dashboard is built for businesses accepting payments via APIs, Webhooks, and Checkout pages. A P2P user is a retail trader. Mixing these two fundamentally different user experiences into a single dashboard will result in a bloated, confusing interface.

**Recommendation: A Separate P2P Frontend Application**
- **Merchant App:** `merchant.fiddupay.com` (Your current React app)
- **P2P App:** `p2p.fiddupay.com` (A new, separate React app)
- **Shared Backend:** Both and connect to the identical Rust backend, but authenticate differently and hit different route groups.

*Note: Users can use the same login credentials for both, but their session tokens will dictate which application they are currently using.*

## High-Level Implementation Phases

We will break this down into four distinct, manageable phases.

### Phase 1: Foundation (Database & Roles)
Before building any UI or trading logic, we need to adapt the database to support new user types and internal ledger transfers without breaking the existing merchant logic.
- [ ] **Create `users` table:** A generalized table for login credentials. Both Merchants and P2P Traders will link to this table.
- [ ] **Create `p2p_profiles` table:** Stores KYC status, nickname, completion rate, and trading statistics for P2P users.
- [ ] **Create `p2p_wallets` table:** Strictly custodial hot wallets assigned to P2P users for depositing/withdrawing. They *cannot* export keys.
- [ ] **Create Internal Ledger:** SQL logic for instantly moving balances between custodial wallets without hitting the blockchain (the core of escrow and P2P).

### Phase 2: Core Trading Engine (Ads & Escrow)
The heart of the P2P exchange. This logic runs entirely off-chain on the Rust backend.
- [ ] **Create `p2p_ads` table:** Stores buy/sell orders (Asset, Fiat Currency, Price, Limits, Payment Methods).
- [ ] **Create `p2p_trades` table:** Tracks active trades between a Maker (Ad creator) and Taker (Ad responder).
- [ ] **Implement Escrow Lock Logic:** When a trade starts, immediately deduct the crypto amount from the seller's `p2p_balances` and hold it in a locked state attached to the `p2p_trades` ID.
- [ ] **Implement Release/Cancel Logic:** Move locked funds to the buyer on release, or back to the seller on cancellation.

### Phase 3: Communication & Disputes
P2P trading requires real-time communication between the buyer and seller to confirm bank transfers.
- [ ] **Implement Order Chat:** A WebSocket or polling-based chat system tied specifically to an active `p2p_trades` ID.
- [ ] **Dispute System:** Allow either party to trigger an "Appeal." Lock the trade status so admins can review chat logs and request bank statements before manually releasing funds.

### Phase 4: Frontend Development
Building the user-facing application for retail traders.
- [ ] **Marketplace Page:** A live board of open Buy/Sell ads filterable by Fiat, Amount, Payment Method, and Crypto.
- [ ] **Ad Creation Modal:** Interface for P2P Vendors to list their assets for sale or set buy targets.
- [ ] **Trading Room Page:** The dedicated interface for an active trade, showing the escrow status, fiat payment details, countdown timer, and live chat.
- [ ] **P2P Wallet Dashboard:** Interface for depositing external crypto into FidduPay, and moving funds internally between a user's Merchant Account and their P2P Account.

## Next Steps

To begin, we should focus *exclusively* on **Phase 1: Foundation**. 
Once the database schema is updated to support distinct, non-merchant user roles, we can begin building the backend routes for the P2P engine.

Shall we begin drafting the exact SQL migrations for Phase 1?
