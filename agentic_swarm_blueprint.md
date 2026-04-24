# Blueprint: FidduPay Agentic Verification Swarm

## Overview
The **FidduPay Agentic Swarm** is a multi-agent system designed to replace static KYC checks with dynamic, real-time trust intelligence. The swarm consists of specialized "Agents" that verify specific facets of a merchant's identity and reputation, coordinated by a central **Swarm Consensus Engine**.

## 1. Identity Agent (The Sentinel)
**Primary Responsibility:** Verify legal identity and prevent impersonation.

### Integration: SmileID
- **NIN/BVN Authority Check:** Real-time lookup against government databases.
- **Biometric Liveness:** 3D Face Map matching against the ID document (SmartSelfie™).
- **Document Verification:** OCR and authenticity check on International Passports or Driver's Licenses.

### Output Signals
- `id_verified`: Boolean
- `liveness_score`: 0-100
- `authority_match`: Boolean

---

## 2. Social Signal Agent (The Oracle)
**Primary Responsibility:** Verify the digital presence and social proof of the business.

### Data Sources
- **Twitter/X API:** Check account age, follower count, and activity frequency.
- **LinkedIn API:** Verify professional credentials and business page existence.
- **Custom Scraping/Public APIs:** Search for brand mentions and website domain age (Whois).

### Logic
- **Bot Detection:** Filter out fake accounts using activity pattern analysis.
- **Engagement Quality:** High volume of real interactions increases the trust signal.

### Output Signals
- `social_reach`: Integer
- `domain_authority`: 0-100
- `account_age_verified`: Boolean

---

## 3. Reputation & History Agent (The Archivist)
**Primary Responsibility:** Analyze financial behavior and risk patterns.

### History Check Parameters
- **On-chain History:**
    - Scan merchant settlement addresses for links to high-risk protocols or blacklisted mixers (OFAC compliance).
    - Analyze previous transaction volume on relevant chains (EVM, Solana).
- **Platform Tenure:**
    - Time elapsed since the merchant's first successful transaction on FidduPay.
    - Ratio of successful payments vs. chargebacks/refunds.
- **Dispute Rate:**
    - Number of customer complaints or payment disputes flagged by the system.

### Output Signals
- `risk_score`: 0-100 (Inverse of trust)
- `tenure_days`: Integer
- `success_ratio`: Decimal (e.g., 0.99)

---

## 4. Swarm Consensus Engine (The Intelligence Layer)
**Primary Responsibility:** Aggregate signals and determine the final Tier.

### Decision Matrix
1. **Tier 1 (Silver):** Sentinel (Identity Agent) must return `id_verified = true`.
2. **Tier 2 (Gold):** Tier 1 status + Oracle (Social Agent) must verify at least 2 social signals.
3. **Tier 3 (Institutional/Elite):** Tier 2 status + Archivist (History Agent) must show >90 days tenure and <1% dispute rate.

### Dynamic Trust Score Calculation
The Trust Score is a weighted average of all agent outputs:
- **Identity:** 50%
- **Reputation/History:** 30%
- **Social Signals:** 20%

---

## Technical Architecture
- **Environment:** Rust-based microservices with asynchronous worker pools.
- **Communication:** Internal event bus (Redis/RabbitMQ) for agent coordination.
- **Persistence:** Signals cached in PostgreSQL with audit trails for every verification attempt.
