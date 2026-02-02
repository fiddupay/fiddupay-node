// Invoice Model
// Database model for invoices

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invoice {
    pub id: i32,
    pub invoice_id: String,
    pub merchant_id: i64,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub status: String,
    pub items: serde_json::Value,
    pub subtotal: Decimal,
    pub tax: Decimal,
    pub total: Decimal,
    pub currency: String,
    pub payment_id: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceRequest {
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub items: serde_json::Value,
    pub tax: Option<Decimal>,
    pub currency: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub notes: Option<String>,
}
