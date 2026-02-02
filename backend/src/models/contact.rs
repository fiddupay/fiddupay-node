// Contact Message Model
// Database model for contact form submissions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContactMessage {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateContactMessageRequest {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContactStatusRequest {
    pub status: String,
}
