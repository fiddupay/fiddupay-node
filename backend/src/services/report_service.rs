use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, FromRow};
use serde::Serialize;
use csv::Writer;
use genpdf::{elements, fonts, style};
use crate::error::ServiceError;

#[derive(Debug, FromRow, Serialize)]
pub struct PaymentReportRow {
    pub payment_id: String,
    pub status: String,
    pub amount: Option<Decimal>,
    pub amount_usd: Decimal,
    pub crypto_type: Option<String>,
    pub network: Option<String>,
    pub transaction_hash: Option<String>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub fee_amount: Option<Decimal>,
    pub fee_amount_usd: Decimal,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

pub struct ReportService {
    db_pool: PgPool,
}

impl ReportService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn get_payment_data(
        &self,
        merchant_id: i64,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        blockchain: Option<String>,
        status: Option<String>,
        sandbox_mode: Option<bool>,
    ) -> Result<Vec<PaymentReportRow>, ServiceError> {
        let mut query = String::from(
            r#"
            SELECT 
                payment_id, status, amount, amount_usd, crypto_type, network,
                transaction_hash, from_address, to_address, fee_amount, fee_amount_usd,
                description, created_at, confirmed_at
            FROM payment_transactions
            WHERE merchant_id = $1
                AND created_at >= $2
                AND created_at <= $3
            "#,
        );

        let mut param_count = 3;
        if let Some(_) = sandbox_mode {
            param_count += 1;
            query.push_str(&format!(" AND sandbox_mode = ${}", param_count));
        }
        if let Some(_) = blockchain {
            param_count += 1;
            query.push_str(&format!(" AND network = ${}", param_count));
        }
        if let Some(_) = status {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut query_builder = sqlx::query_as::<_, PaymentReportRow>(&query)
            .bind(merchant_id)
            .bind(start_date)
            .bind(end_date);

        if let Some(sb) = sandbox_mode {
            query_builder = query_builder.bind(sb);
        }
        if let Some(bc) = blockchain {
            query_builder = query_builder.bind(bc);
        }
        if let Some(st) = status {
            query_builder = query_builder.bind(st);
        }

        let rows = query_builder.fetch_all(&self.db_pool).await?;
        Ok(rows)
    }

    pub async fn generate_csv(
        &self,
        data: Vec<PaymentReportRow>,
    ) -> Result<Vec<u8>, ServiceError> {
        let mut wtr = Writer::from_writer(vec![]);
        
        for row in data {
            wtr.serialize(row).map_err(|e| ServiceError::InternalError(format!("CSV serialization failed: {}", e)))?;
        }
        
        let inner = wtr.into_inner().map_err(|e| ServiceError::InternalError(format!("CSV generation failed: {}", e)))?;
        Ok(inner)
    }

    pub async fn generate_pdf(
        &self,
        merchant_name: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        data: Vec<PaymentReportRow>,
    ) -> Result<Vec<u8>, ServiceError> {
        // 1. Load Font (Attempt typical Linux system paths)
        let font_path = [
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/LiberationSans-Regular.ttf",
        ].iter().find(|p| std::path::Path::new(p).exists())
        .ok_or_else(|| ServiceError::InternalError("No TrueType font found on server. Please install liberation-fonts or dejavu-fonts.".to_string()))?;

        let font_bytes = std::fs::read(font_path)
            .map_err(|e| ServiceError::InternalError(format!("Failed to read font file: {}", e)))?;
        
        let font_data = fonts::FontData::new(font_bytes, None);
        let font_family = fonts::FontFamily {
            regular: font_data.clone(),
            bold: font_data.clone(),
            italic: font_data.clone(),
            bold_italic: font_data.clone(),
        };

        let mut doc = genpdf::Document::new(font_family);
        doc.set_title("FidduPay Transaction Report");

        // Decoration
        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(10);
        doc.set_page_decorator(decorator);

        // Header
        doc.push(elements::Text::new("FidduPay Transaction Report")
            .styled(style::Style::new().bold().with_font_size(18)));
        doc.push(elements::Text::new(format!("Merchant: {}", merchant_name)));
        doc.push(elements::Text::new(format!("Period: {} - {}", 
            start_date.format("%Y-%m-%d"), 
            end_date.format("%Y-%m-%d")
        )));
        doc.push(elements::Break::new(1.5));

        // Summary Calculations
        let total_count = data.len();
        let total_amount_usd: Decimal = data.iter().map(|d| d.amount_usd).sum();
        let total_fees_usd: Decimal = data.iter().map(|d| d.fee_amount_usd).sum();

        doc.push(elements::Paragraph::new("Summary")
            .styled(style::Style::new().bold().with_font_size(14)));
        doc.push(elements::Text::new(format!("Total Transactions: {}", total_count)));
        doc.push(elements::Text::new(format!("Total Volume (USD): ${:.2}", total_amount_usd)));
        doc.push(elements::Text::new(format!("Total Fees (USD):   ${:.2}", total_fees_usd)));
        doc.push(elements::Break::new(1.5));

        // Table
        let mut table = elements::TableLayout::new(vec![2, 4, 3, 2, 2]);
        table.set_cell_decorator(elements::FrameCellDecorator::new());

        // Header Row
        table.push_row(elements::TableRow::new(vec![
            Box::new(elements::Text::new("Date").styled(style::Style::new().bold())),
            Box::new(elements::Text::new("Payment ID").styled(style::Style::new().bold())),
            Box::new(elements::Text::new("Crypto").styled(style::Style::new().bold())),
            Box::new(elements::Text::new("Amount USD").styled(style::Style::new().bold())),
            Box::new(elements::Text::new("Status").styled(style::Style::new().bold())),
        ]));

        // Data Rows
        for row in data {
            table.push_row(elements::TableRow::new(vec![
                Box::new(elements::Text::new(row.created_at.format("%Y-%m-%d").to_string())),
                Box::new(elements::Text::new(&row.payment_id)),
                Box::new(elements::Text::new(format!("{} ({})", 
                    row.crypto_type.as_deref().unwrap_or("N/A"),
                    row.network.as_deref().unwrap_or("N/A")
                ))),
                Box::new(elements::Text::new(format!("${:.2}", row.amount_usd))),
                Box::new(elements::Text::new(row.status.to_string())),
            ]));
        }

        doc.push(table);

        // Footer
        doc.push(elements::Break::new(2.0));
        doc.push(elements::Paragraph::new("Generated by FidduPay - Decentralized Payment Gateway")
            .styled(style::Color::Rgb(128, 128, 128)));

        let mut buffer = Vec::new();
        doc.render_to(&mut buffer)
            .map_err(|e| ServiceError::InternalError(format!("PDF rendering failed: {}", e)))?;

        Ok(buffer)
    }
}
