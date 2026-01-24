// Refund Models
// Data structures for refund operations

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    pub refund_id: String,
    pub payment_id: String,
    pub amount: Decimal,
    pub amount_usd: Decimal,
    pub status: String,
    pub reason: Option<String>,
    pub transaction_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_refund_response_creation() {
        let refund = RefundResponse {
            refund_id: "ref_abc123".to_string(),
            payment_id: "pay_xyz789".to_string(),
            amount: Decimal::new(100, 0),
            amount_usd: Decimal::new(10000, 2),
            status: "pending".to_string(),
            reason: Some("Customer requested refund".to_string()),
            transaction_hash: None,
            created_at: Utc::now(),
            completed_at: None,
        };

        assert_eq!(refund.refund_id, "ref_abc123");
        assert_eq!(refund.payment_id, "pay_xyz789");
        assert_eq!(refund.amount, Decimal::new(100, 0));
        assert_eq!(refund.amount_usd, Decimal::new(10000, 2));
        assert_eq!(refund.status, "pending");
        assert_eq!(refund.reason, Some("Customer requested refund".to_string()));
        assert!(refund.transaction_hash.is_none());
        assert!(refund.completed_at.is_none());
    }

    #[test]
    fn test_refund_response_completed() {
        let refund = RefundResponse {
            refund_id: "ref_completed".to_string(),
            payment_id: "pay_original".to_string(),
            amount: Decimal::new(50, 0),
            amount_usd: Decimal::new(5000, 2),
            status: "completed".to_string(),
            reason: Some("Duplicate payment".to_string()),
            transaction_hash: Some("0xrefund123".to_string()),
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };

        assert_eq!(refund.status, "completed");
        assert_eq!(refund.transaction_hash, Some("0xrefund123".to_string()));
        assert!(refund.completed_at.is_some());
    }

    #[test]
    fn test_refund_response_serialization() {
        let refund = RefundResponse {
            refund_id: "ref_test".to_string(),
            payment_id: "pay_test".to_string(),
            amount: Decimal::new(75, 0),
            amount_usd: Decimal::new(7500, 2),
            status: "pending".to_string(),
            reason: Some("Test refund".to_string()),
            transaction_hash: None,
            created_at: Utc::now(),
            completed_at: None,
        };

        let json = serde_json::to_string(&refund).unwrap();
        assert!(json.contains("ref_test"));
        assert!(json.contains("pay_test"));
        assert!(json.contains("pending"));

        let deserialized: RefundResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.refund_id, refund.refund_id);
        assert_eq!(deserialized.payment_id, refund.payment_id);
        assert_eq!(deserialized.amount, refund.amount);
        assert_eq!(deserialized.status, refund.status);
    }

    #[test]
    fn test_refund_response_without_reason() {
        let refund = RefundResponse {
            refund_id: "ref_no_reason".to_string(),
            payment_id: "pay_test".to_string(),
            amount: Decimal::new(100, 0),
            amount_usd: Decimal::new(10000, 2),
            status: "pending".to_string(),
            reason: None,
            transaction_hash: None,
            created_at: Utc::now(),
            completed_at: None,
        };

        assert!(refund.reason.is_none());
    }

    #[test]
    fn test_refund_response_status_types() {
        let statuses = vec!["pending", "completed", "failed"];

        for status in statuses {
            let refund = RefundResponse {
                refund_id: format!("ref_{}", status),
                payment_id: "pay_test".to_string(),
                amount: Decimal::new(100, 0),
                amount_usd: Decimal::new(10000, 2),
                status: status.to_string(),
                reason: None,
                transaction_hash: None,
                created_at: Utc::now(),
                completed_at: None,
            };

            assert_eq!(refund.status, status);
        }
    }

    #[test]
    fn test_refund_response_partial_refund() {
        // Test partial refund (50% of original payment)
        let original_amount = Decimal::new(100, 0);
        let refund_amount = Decimal::new(50, 0);

        let refund = RefundResponse {
            refund_id: "ref_partial".to_string(),
            payment_id: "pay_original".to_string(),
            amount: refund_amount,
            amount_usd: Decimal::new(5000, 2),
            status: "completed".to_string(),
            reason: Some("Partial refund requested".to_string()),
            transaction_hash: Some("0xpartial".to_string()),
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };

        assert!(refund.amount < original_amount);
        assert_eq!(refund.amount, Decimal::new(50, 0));
    }

    #[test]
    fn test_refund_response_full_refund() {
        // Test full refund (100% of original payment)
        let original_amount = Decimal::new(100, 0);

        let refund = RefundResponse {
            refund_id: "ref_full".to_string(),
            payment_id: "pay_original".to_string(),
            amount: original_amount,
            amount_usd: Decimal::new(10000, 2),
            status: "completed".to_string(),
            reason: Some("Full refund".to_string()),
            transaction_hash: Some("0xfull".to_string()),
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };

        assert_eq!(refund.amount, original_amount);
    }

    #[test]
    fn test_refund_response_failed() {
        let refund = RefundResponse {
            refund_id: "ref_failed".to_string(),
            payment_id: "pay_test".to_string(),
            amount: Decimal::new(100, 0),
            amount_usd: Decimal::new(10000, 2),
            status: "failed".to_string(),
            reason: Some("Insufficient funds".to_string()),
            transaction_hash: None,
            created_at: Utc::now(),
            completed_at: None,
        };

        assert_eq!(refund.status, "failed");
        assert!(refund.transaction_hash.is_none());
        assert!(refund.completed_at.is_none());
    }

    #[test]
    fn test_refund_response_with_different_crypto_amounts() {
        // Test refund with different crypto amounts (e.g., SOL vs USDT)
        let refunds = vec![
            (Decimal::new(1, 0), Decimal::new(10000, 2)),    // 1 SOL = $100
            (Decimal::new(100, 0), Decimal::new(10000, 2)),  // 100 USDT = $100
            (Decimal::new(50, 0), Decimal::new(5000, 2)),    // 50 USDT = $50
        ];

        for (amount, amount_usd) in refunds {
            let refund = RefundResponse {
                refund_id: "ref_test".to_string(),
                payment_id: "pay_test".to_string(),
                amount,
                amount_usd,
                status: "pending".to_string(),
                reason: None,
                transaction_hash: None,
                created_at: Utc::now(),
                completed_at: None,
            };

            assert_eq!(refund.amount, amount);
            assert_eq!(refund.amount_usd, amount_usd);
        }
    }
}
