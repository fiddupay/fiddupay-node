use sqlx::PgPool;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};
use nanoid::nanoid;
use crate::error::ServiceError;

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub description: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceRequest {
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub items: Vec<InvoiceItem>,
    pub tax: Option<Decimal>,
    pub due_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Invoice {
    pub invoice_id: String,
    pub merchant_id: i64,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub status: String,
    pub items: Vec<InvoiceItem>,
    pub subtotal: Decimal,
    pub tax: Decimal,
    pub total: Decimal,
    pub payment_id: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

pub struct InvoiceService {
    pool: PgPool,
}

impl InvoiceService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_invoice(&self, merchant_id: i64, req: CreateInvoiceRequest) -> Result<Invoice, ServiceError> {
        // Calculate totals
        let subtotal: Decimal = req.items.iter().map(|item| item.amount).sum();
        let tax = req.tax.unwrap_or(Decimal::ZERO);
        let total = subtotal + tax;

        let invoice_id = format!("inv_{}", nanoid!(12));
        let items_json = serde_json::to_value(&req.items)?;

        sqlx::query!(
            r#"INSERT INTO invoices 
               (invoice_id, merchant_id, customer_email, customer_name, items, subtotal, tax, total, due_date, notes)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
            invoice_id, merchant_id, req.customer_email, req.customer_name,
            items_json, subtotal, tax, total, req.due_date, req.notes
        )
        .execute(&self.pool)
        .await?;

        self.get_invoice(merchant_id, &invoice_id).await
    }

    pub async fn get_invoice(&self, merchant_id: i64, invoice_id: &str) -> Result<Invoice, ServiceError> {
        let record = sqlx::query!(
            r#"SELECT invoice_id, merchant_id, customer_email, customer_name, status, items, 
                      subtotal, tax, total, payment_id, due_date, notes, created_at, paid_at
               FROM invoices WHERE invoice_id = $1 AND merchant_id = $2"#,
            invoice_id, merchant_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Invoice not found".to_string()))?;

        let items: Vec<InvoiceItem> = serde_json::from_value(record.items)?;

        Ok(Invoice {
            invoice_id: record.invoice_id,
            merchant_id: record.merchant_id,
            customer_email: record.customer_email,
            customer_name: record.customer_name,
            status: record.status,
            items,
            subtotal: record.subtotal,
            tax: record.tax,
            total: record.total,
            payment_id: record.payment_id,
            due_date: record.due_date,
            notes: record.notes,
            created_at: record.created_at,
            paid_at: record.paid_at,
        })
    }

    pub async fn list_invoices(&self, merchant_id: i64, limit: i64) -> Result<Vec<Invoice>, ServiceError> {
        let records = sqlx::query!(
            r#"SELECT invoice_id, merchant_id, customer_email, customer_name, status, items,
                      subtotal, tax, total, payment_id, due_date, notes, created_at, paid_at
               FROM invoices WHERE merchant_id = $1 ORDER BY created_at DESC LIMIT $2"#,
            merchant_id, limit
        )
        .fetch_all(&self.pool)
        .await?;

        records.into_iter().map(|r| {
            let items: Vec<InvoiceItem> = serde_json::from_value(r.items)?;
            Ok(Invoice {
                invoice_id: r.invoice_id,
                merchant_id: r.merchant_id,
                customer_email: r.customer_email,
                customer_name: r.customer_name,
                status: r.status,
                items,
                subtotal: r.subtotal,
                tax: r.tax,
                total: r.total,
                payment_id: r.payment_id,
                due_date: r.due_date,
                notes: r.notes,
                created_at: r.created_at,
                paid_at: r.paid_at,
            })
        }).collect()
    }

    pub async fn mark_as_paid(&self, invoice_id: &str, payment_id: &str) -> Result<(), ServiceError> {
        sqlx::query!(
            "UPDATE invoices SET status = 'PAID', payment_id = $2, paid_at = NOW() WHERE invoice_id = $1",
            invoice_id, payment_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
