# Swarm Intelligence & Automated KYC Blueprints

To further evolve the FidduPay Trust Layer, we propose the integration of automated agents and formal identity verification providers.

## 1. SmileID Integration (Identity Agent)

The **Identity Agent** will automate the transition from Tier 1 (Self-Reported) to Tier 1 (Verified).

### Implementation Flow:
1. **User Action**: Clicks "Verify Identity" in `VerificationTab`.
2. **Agent Trigger**: Triggers a SmileID "Biometric KYC" or "ID Validation" flow.
3. **Backend Hook**: `smileid_webhook_handler` receives validation status.
4. **Trust Update**: On success, `kyc_tier` is bumped to 1, and the trust score baseline moves from 20% (Email) to 60% (ID).

```rust
// Proposed Backend Bridge
pub async fn verify_nin_with_smileid(nin: &str, real_name: &str) -> Result<bool, Error> {
    // Call SmileID/Identity API
    // Return true if NIN matches Real Name and DOB
}
```

## 2. The "Watcher" Agent (Compliance Monitoring)

The **Watcher Agent** is a background worker that monitors merchant behavior to adjust trust scores dynamically.

### Monitoring Signals:
- **Refund Squall**: Multiple refunds in a short window triggers a score penalty.
- **Whale Transactions**: Sudden spikes in volume for Tier 0 merchants trigger a "Step-up Verification" request.
- **Dispute Resolution**: Fast resolution of P2P disputes provides a "Social Trust" boost.

## 3. Social Signal scraping (Intelligence Agent)

Instead of manual verification, an **Intelligence Agent** can periodically verify that the provided Social Handles (X/LinkedIn) are active and belong to the business name, providing an automated "Social Badge."

> [!TIP]
> **Priority Path**: Start with SmileID for Nigerian NIN/BVN validation as it directly impacts transaction limits and merchant payouts.
