# Phase 1: Compliance-Ready Architecture (Technical Hardening)

This plan outlines the technical changes required to move FidduPay from a "High-Risk" state to a "Compliance-Ready" state, specifically addressing the Nigerian CBN, NDPC, and global regulatory requirements.

## User Review Required

> [!IMPORTANT]
> These changes involve modifying the database schema and registration flows. You will need to update your frontend to collect the additional mandatory data (NIN, BVN, Social Media handles).

> [!WARNING]
> Collecting BVN and NIN requires integration with a verified government API or a BaaS partner. This plan only adds the *infrastructure* to store and manage this data securely.

## Proposed Changes

---

### 1. Backend Data Model Refactoring (Rust)

#### [MODIFY] [merchant.rs](file:///c:/Users/USER/Desktop/TechyTro/crypto-payment-gateway/backend/src/models/merchant.rs)
Add mandatory compliance fields:
- `bvn_hash`: Encrypted/Hashed BVN for identity deduplication.
- `nin_hash`: Encrypted/Hashed NIN.
- `social_media_handles`: Array/String of handles (Mandatory since May 2024).
- `kyc_tier`: Enum (T1, T2, T3) to control transaction limits.
- `dpo_contact`: Optional field for large merchants.

#### [MODIFY] [p2p.rs](file:///c:/Users/USER/Desktop/TechyTro/crypto-payment-gateway/backend/src/models/p2p.rs)
Ensure P2P profiles have the same tiered KYC structure to mitigate "Scam Report" risks.

#### [NEW] [compliance.rs](file:///c:/Users/USER/Desktop/TechyTro/crypto-payment-gateway/backend/src/models/compliance.rs)
Create a new model for:
- `PrivacyPolicyAgreement`: Tracks `user_id`, `version`, `timestamp`, and `ip_address` for audit trails.
- `SarReport`: Suspicious Activity Report metadata for internal compliance reviews.

---

### 2. Implementation of "The 72-Hour Pipeline"

#### [NEW] [compliance_service.rs](file:///c:/Users/USER/Desktop/TechyTro/crypto-payment-gateway/backend/src/services/compliance_service.rs)
A dedicated service to:
- Handle KYC verification logic.
- Monitor transactions for "Anomalies" (Large spikes, rapid P2P trades).
- Trigger "Breach Alerts" if sensitive data fields are accessed irregularly.

---

### 3. Frontend Registration & Settings

#### [MODIFY] [Registration Flows]
Update React components to include the new fields:
- NIN/BVN input (with security warnings).
- Social media handle input.
- Clear fee disclosure before transaction finalization.

---

## Verification Plan

### Automated Tests
- `cargo test`: Ensure new model fields don't break existing balance logic.
- Mock KYC verification flow in a sandbox environment.

### Manual Verification
- Verify that a user cannot exceed "Tier 1" limits without providing a NIN.
- Verify that the Privacy Policy acceptance is correctly logged in the database.
- Inspect database to ensure BVN/NIN are stored securely (not in plaintext).

---

## Open Questions

1. **BaaS Partner**: Do you have a preference for a verification API (e.g., Smile ID, Dojah, or a bank partner)?
2. **Data Residency**: For global expansion, should we plan for multi-region data storage (e.g., keeping Nigerian data in Africa, EU data in the EU as per GDPR/NDPA)?
