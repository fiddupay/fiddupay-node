use crate::error::ServiceError;
use crate::models::notification::{MerchantNotification, NotificationListResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub struct NotificationService {
    db_pool: PgPool,
}

impl NotificationService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Create a new notification for a merchant
    pub async fn create_notification(
        &self,
        merchant_id: i64,
        title: &str,
        message: &str,
        notif_type: &str,
        event_type: &str,
        sandbox_mode: bool,
    ) -> Result<MerchantNotification, ServiceError> {
        let notification = sqlx::query_as::<_, MerchantNotification>(
            r#"
            INSERT INTO merchant_notifications (merchant_id, title, message, notification_type, event_type, sandbox_mode)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, merchant_id, title, message, notification_type, event_type, is_read, sandbox_mode, created_at, expires_at
            "#
        )
        .bind(merchant_id)
        .bind(title)
        .bind(message)
        .bind(notif_type)
        .bind(event_type)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(notification)
    }

    /// List notifications for a merchant with unread count
    pub async fn list_notifications(
        &self,
        merchant_id: i64,
        limit: i64,
        offset: i64,
        sandbox_mode: bool,
    ) -> Result<NotificationListResponse, ServiceError> {
        let notifications = sqlx::query_as::<_, MerchantNotification>(
            r#"
            SELECT id, merchant_id, title, message, notification_type, event_type, is_read, sandbox_mode, created_at, expires_at
            FROM merchant_notifications
            WHERE merchant_id = $1 AND sandbox_mode = $3
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $4
            "#
        )
        .bind(merchant_id)
        .bind(limit)
        .bind(sandbox_mode)
        .bind(offset)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let unread_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM merchant_notifications WHERE merchant_id = $1 AND is_read = false AND sandbox_mode = $2"
        )
        .bind(merchant_id)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM merchant_notifications WHERE merchant_id = $1 AND sandbox_mode = $2"
        )
        .bind(merchant_id)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(NotificationListResponse {
            notifications,
            total,
            unread_count,
        })
    }

    /// Mark notifications as read (either specific ID or all)
    pub async fn mark_as_read(
        &self,
        merchant_id: i64,
        notification_id: Option<Uuid>,
        sandbox_mode: bool,
    ) -> Result<u64, ServiceError> {
        let query = match notification_id {
            Some(id) => {
                sqlx::query("UPDATE merchant_notifications SET is_read = true WHERE merchant_id = $1 AND id = $2 AND sandbox_mode = $3")
                    .bind(merchant_id)
                    .bind(id)
                    .bind(sandbox_mode)
            }
            None => {
                sqlx::query("UPDATE merchant_notifications SET is_read = true WHERE merchant_id = $1 AND sandbox_mode = $2 AND is_read = false")
                    .bind(merchant_id)
                    .bind(sandbox_mode)
            }
        };

        let result = query.execute(&self.db_pool)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Delete notifications (either specific ID or all)
    pub async fn delete_notifications(
        &self,
        merchant_id: i64,
        notification_id: Option<Uuid>,
        sandbox_mode: bool,
    ) -> Result<u64, ServiceError> {
        let query = match notification_id {
            Some(id) => {
                sqlx::query("DELETE FROM merchant_notifications WHERE merchant_id = $1 AND id = $2 AND sandbox_mode = $3")
                    .bind(merchant_id)
                    .bind(id)
                    .bind(sandbox_mode)
            }
            None => {
                sqlx::query("DELETE FROM merchant_notifications WHERE merchant_id = $1 AND sandbox_mode = $2")
                    .bind(merchant_id)
                    .bind(sandbox_mode)
            }
        };

        let result = query.execute(&self.db_pool)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
