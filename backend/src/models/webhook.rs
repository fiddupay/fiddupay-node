// Webhook Models
// Data structures for webhook notifications

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::payment::models::PaymentStatus;

/// Webhook payload sent to merchant endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event_type: String,  // "payment.confirmed", "payment.expired", "refund.completed"
    pub payment_id: String,
    pub merchant_id: i64,
    pub status: PaymentStatus,
    pub amount: Decimal,
    pub crypto_type: String,
    pub transaction_hash: Option<String>,
    pub timestamp: i64,
}

/// Webhook delivery record for tracking delivery attempts
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookDelivery {
    pub id: i64,
    pub merchant_id: i64,
    pub payment_id: i64,
    pub event_type: String,  // "payment.confirmed", "payment.expired", "refund.completed"
    pub url: String,
    pub payload: serde_json::Value,
    pub status: String,  // "pending", "delivered", "failed"
    pub attempts: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub created_at: DateTime<Utc>,
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_webhook_payload_creation() {
        let payload = WebhookPayload {
            event_type: "payment.confirmed".to_string(),
            payment_id: "pay_abc123".to_string(),
            merchant_id: 1i64,
            status: PaymentStatus::Confirmed,
            amount: Decimal::new(100, 0),
            crypto_type: "USDT_BEP20".to_string(),
            transaction_hash: Some("0xabc123".to_string()),
            timestamp: 1234567890,
        };

        assert_eq!(payload.event_type, "payment.confirmed");
        assert_eq!(payload.payment_id, "pay_abc123");
        assert_eq!(payload.merchant_id, 1);
        assert_eq!(payload.status, PaymentStatus::Confirmed);
        assert_eq!(payload.amount, Decimal::new(100, 0));
        assert_eq!(payload.crypto_type, "USDT_BEP20");
        assert_eq!(payload.transaction_hash, Some("0xabc123".to_string()));
        assert_eq!(payload.timestamp, 1234567890);
    }

    #[test]
    fn test_webhook_payload_serialization() {
        let payload = WebhookPayload {
            event_type: "payment.expired".to_string(),
            payment_id: "pay_xyz789".to_string(),
            merchant_id: 1i64,
            status: PaymentStatus::Failed,
            amount: Decimal::new(50, 0),
            crypto_type: "SOL".to_string(),
            transaction_hash: None,
            timestamp: 9876543210,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("payment.expired"));
        assert!(json.contains("pay_xyz789"));
        assert!(json.contains("Failed"));

        let deserialized: WebhookPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event_type, payload.event_type);
        assert_eq!(deserialized.payment_id, payload.payment_id);
        assert_eq!(deserialized.merchant_id, payload.merchant_id);
        assert_eq!(deserialized.status, payload.status);
    }

    #[test]
    fn test_webhook_delivery_creation() {
        let delivery = WebhookDelivery {
            id: 1,
            merchant_id: 1i64,
            payment_id: 100,
            event_type: "payment.confirmed".to_string(),
            url: "https://merchant.example.com/webhook".to_string(),
            payload: serde_json::json!({
                "event_type": "payment.confirmed",
                "payment_id": "pay_abc123"
            }),
            status: "pending".to_string(),
            attempts: 0,
            last_attempt_at: None,
            next_retry_at: Some(Utc::now()),
            response_status: None,
            response_body: None,
            created_at: Utc::now(),
        };

        assert_eq!(delivery.id, 1);
        assert_eq!(delivery.merchant_id, 1);
        assert_eq!(delivery.payment_id, 100);
        assert_eq!(delivery.event_type, "payment.confirmed");
        assert_eq!(delivery.url, "https://merchant.example.com/webhook");
        assert_eq!(delivery.status, "pending");
        assert_eq!(delivery.attempts, 0);
        assert!(delivery.last_attempt_at.is_none());
        assert!(delivery.next_retry_at.is_some());
        assert!(delivery.response_status.is_none());
        assert!(delivery.response_body.is_none());
    }

    #[test]
    fn test_webhook_delivery_with_attempts() {
        let delivery = WebhookDelivery {
            id: 2,
            merchant_id: 1i64,
            payment_id: 200,
            event_type: "payment.expired".to_string(),
            url: "https://merchant.example.com/webhook".to_string(),
            payload: serde_json::json!({"event_type": "payment.expired"}),
            status: "failed".to_string(),
            attempts: 5,
            last_attempt_at: Some(Utc::now()),
            next_retry_at: None,
            response_status: Some(500),
            response_body: Some("Internal Server Error".to_string()),
            created_at: Utc::now(),
        };

        assert_eq!(delivery.status, "failed");
        assert_eq!(delivery.attempts, 5);
        assert!(delivery.last_attempt_at.is_some());
        assert!(delivery.next_retry_at.is_none());
        assert_eq!(delivery.response_status, Some(500));
        assert_eq!(delivery.response_body, Some("Internal Server Error".to_string()));
    }

    #[test]
    fn test_webhook_delivery_serialization() {
        let delivery = WebhookDelivery {
            id: 3,
            merchant_id: 1i64,
            payment_id: 300,
            event_type: "refund.completed".to_string(),
            url: "https://merchant.example.com/webhook".to_string(),
            payload: serde_json::json!({"refund_id": "ref_123"}),
            status: "delivered".to_string(),
            attempts: 1,
            last_attempt_at: Some(Utc::now()),
            next_retry_at: None,
            response_status: Some(200),
            response_body: Some("OK".to_string()),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&delivery).unwrap();
        assert!(json.contains("refund.completed"));
        assert!(json.contains("delivered"));
        assert!(json.contains("ref_123"));

        let deserialized: WebhookDelivery = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, delivery.id);
        assert_eq!(deserialized.merchant_id, delivery.merchant_id);
        assert_eq!(deserialized.event_type, delivery.event_type);
        assert_eq!(deserialized.status, delivery.status);
    }

    #[test]
    fn test_webhook_payload_event_types() {
        let event_types = vec![
            "payment.confirmed",
            "payment.expired",
            "payment.failed",
            "refund.completed",
            "refund.failed",
        ];

        for event_type in event_types {
            let payload = WebhookPayload {
                event_type: event_type.to_string(),
                payment_id: "pay_test".to_string(),
                merchant_id: 1i64,
                status: PaymentStatus::Pending,
                amount: Decimal::new(100, 0),
                crypto_type: "SOL".to_string(),
                transaction_hash: None,
                timestamp: 1234567890,
            };

            assert_eq!(payload.event_type, event_type);
        }
    }

    #[test]
    fn test_webhook_delivery_status_transitions() {
        let statuses = vec!["pending", "delivered", "failed"];

        for status in statuses {
            let delivery = WebhookDelivery {
                id: 1,
                merchant_id: 1i64,
                payment_id: 1,
                event_type: "payment.confirmed".to_string(),
                url: "https://example.com/webhook".to_string(),
                payload: serde_json::json!({}),
                status: status.to_string(),
                attempts: 0,
                last_attempt_at: None,
                next_retry_at: None,
                response_status: None,
                response_body: None,
                created_at: Utc::now(),
            };

            assert_eq!(delivery.status, status);
        }
    }

    #[test]
    fn test_webhook_payload_without_transaction_hash() {
        let payload = WebhookPayload {
            event_type: "payment.expired".to_string(),
            payment_id: "pay_expired".to_string(),
            merchant_id: 1i64,
            status: PaymentStatus::Failed,
            amount: Decimal::new(100, 0),
            crypto_type: "USDT_POLYGON".to_string(),
            transaction_hash: None,
            timestamp: 1234567890,
        };

        assert!(payload.transaction_hash.is_none());
    }

    #[test]
    fn test_webhook_delivery_retry_tracking() {
        // Simulate retry attempts
        let mut delivery = WebhookDelivery {
            id: 1,
            merchant_id: 1i64,
            payment_id: 1,
            event_type: "payment.confirmed".to_string(),
            url: "https://example.com/webhook".to_string(),
            payload: serde_json::json!({}),
            status: "pending".to_string(),
            attempts: 0,
            last_attempt_at: None,
            next_retry_at: Some(Utc::now()),
            response_status: None,
            response_body: None,
            created_at: Utc::now(),
        };

        // First attempt
        delivery.attempts = 1;
        delivery.last_attempt_at = Some(Utc::now());
        delivery.response_status = Some(500);
        assert_eq!(delivery.attempts, 1);
        assert!(delivery.last_attempt_at.is_some());

        // Second attempt
        delivery.attempts = 2;
        delivery.last_attempt_at = Some(Utc::now());
        assert_eq!(delivery.attempts, 2);

        // Final attempt (5th)
        delivery.attempts = 5;
        delivery.status = "failed".to_string();
        delivery.next_retry_at = None;
        assert_eq!(delivery.attempts, 5);
        assert_eq!(delivery.status, "failed");
        assert!(delivery.next_retry_at.is_none());
    }
}
