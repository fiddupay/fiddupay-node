use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MerchantNotification {
    pub id: Uuid,
    pub merchant_id: i64,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub event_type: String,
    pub is_read: bool,
    pub sandbox_mode: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub event_type: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationListResponse {
    pub notifications: Vec<MerchantNotification>,
    pub total: i64,
    pub unread_count: i64,
}
