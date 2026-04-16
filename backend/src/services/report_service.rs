use crate::error::ServiceError;
use chrono::{DateTime, Utc};
use csv::Writer;
use genpdf::{elements, fonts, style, Element};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::{FromRow, PgPool};

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
        if sandbox_mode.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND sandbox_mode = ${}", param_count));
        }
        if blockchain.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND network = ${}", param_count));
        }
        if status.is_some() {
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

    pub async fn generate_csv(&self, data: Vec<PaymentReportRow>) -> Result<Vec<u8>, ServiceError> {
        let mut wtr = Writer::from_writer(vec![]);

        for row in data {
            wtr.serialize(row).map_err(|e| {
                ServiceError::InternalError(format!("CSV serialization failed: {}", e))
            })?;
        }

        let inner = wtr
            .into_inner()
            .map_err(|e| ServiceError::InternalError(format!("CSV generation failed: {}", e)))?;
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

        let font_data = fonts::FontData::new(font_bytes, None).map_err(|e| {
            ServiceError::InternalError(format!("Failed to create font data: {}", e))
        })?;
        let font_family = fonts::FontFamily {
            regular: font_data.clone(),
            bold: font_data.clone(),
            italic: font_data.clone(),
            bold_italic: font_data.clone(),
        };

        let mut doc = genpdf::Document::new(font_family);
        doc.set_title("FidduPay Transaction Report");

        // Brand Color (FidduPay Blue-ish)
        let brand_color = style::Color::Rgb(63, 81, 181);

        // Decoration
        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(15);
        doc.set_page_decorator(decorator);

        // --- Header Section ---
        doc.push(
            elements::Text::new("FidduPay").styled(
                style::Style::new()
                    .bold()
                    .with_font_size(24)
                    .with_color(brand_color),
            ),
        );
        doc.push(
            elements::Text::new("Decentralized Payment Gateway").styled(
                style::Style::new()
                    .with_font_size(10)
                    .with_color(style::Color::Rgb(100, 100, 100)),
            ),
        );
        doc.push(elements::Break::new(1.0));

        // Accent Line
        doc.push(elements::PaddedElement::new(
            elements::Break::new(0.1),
            genpdf::Margins::trbl(0, 0, 1, 0),
        ));

        doc.push(
            elements::Paragraph::new("Transaction Report")
                .styled(style::Style::new().bold().with_font_size(16)),
        );

        let mut meta_table = elements::TableLayout::new(vec![1, 3]);
        let _ = meta_table
            .row()
            .element(elements::Text::new("Merchant:").styled(style::Style::new().bold()))
            .element(elements::Text::new(merchant_name))
            .push();
        let _ = meta_table
            .row()
            .element(elements::Text::new("Period:").styled(style::Style::new().bold()))
            .element(elements::Text::new(format!(
                "{} to {}",
                start_date.format("%Y-%m-%d"),
                end_date.format("%Y-%m-%d")
            )))
            .push();
        doc.push(meta_table);
        doc.push(elements::Break::new(1.5));

        // --- Summary Section ---
        let total_count = data.len();
        let total_amount_usd: Decimal = data.iter().map(|d| d.amount_usd).sum();
        let total_fees_usd: Decimal = data.iter().map(|d| d.fee_amount_usd).sum();

        let mut summary_box = elements::LinearLayout::vertical();
        summary_box.push(
            elements::Text::new("Financial Summary").styled(
                style::Style::new()
                    .bold()
                    .with_font_size(14)
                    .with_color(brand_color),
            ),
        );
        summary_box.push(elements::Break::new(0.5));
        summary_box.push(elements::Text::new(format!(
            "Total Transactions: {}",
            total_count
        )));
        summary_box.push(elements::Text::new(format!(
            "Total Volume:       ${:.2} USD",
            total_amount_usd
        )));
        summary_box.push(elements::Text::new(format!(
            "Total Fees:         ${:.2} USD",
            total_fees_usd
        )));

        doc.push(elements::PaddedElement::new(
            summary_box,
            genpdf::Margins::trbl(5, 10, 5, 10),
        ));
        doc.push(elements::Break::new(2.0));

        // --- Table Section ---
        // Adjusted widths: Date(2), Payment ID(5), Crypto(4), Amount(2), Status(2)
        let mut table = elements::TableLayout::new(vec![2, 5, 4, 3, 2]);
        table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));

        // Helper for truncation
        let truncate_id = |id: &str| -> String {
            if id.len() > 20 {
                format!("{}...", &id[..17])
            } else {
                id.to_string()
            }
        };

        // Header Row
        // Note: genpdf doesn't support cell background colors easily in 0.2 without custom decorators,
        // so we'll just use bold text and standard borders for now but with better spacing.

        let _ = table
            .row()
            .element(elements::PaddedElement::new(
                elements::Text::new("Date").styled(style::Style::new().bold()),
                2,
            ))
            .element(elements::PaddedElement::new(
                elements::Text::new("Payment ID").styled(style::Style::new().bold()),
                2,
            ))
            .element(elements::PaddedElement::new(
                elements::Text::new("Crypto / Network").styled(style::Style::new().bold()),
                2,
            ))
            .element(elements::PaddedElement::new(
                elements::Text::new("Amount USD").styled(style::Style::new().bold()),
                2,
            ))
            .element(elements::PaddedElement::new(
                elements::Text::new("Status").styled(style::Style::new().bold()),
                2,
            ))
            .push();

        // Data Rows
        let row_style = style::Style::new().with_font_size(10);
        for row in data {
            let status_color = if row.status == "CONFIRMED" {
                style::Color::Rgb(46, 125, 50) // Material Green 700
            } else if row.status == "FAILED" {
                style::Color::Rgb(198, 40, 40) // Material Red 800
            } else {
                style::Color::Rgb(0, 0, 0)
            };

            let _ = table
                .row()
                .element(elements::PaddedElement::new(
                    elements::Text::new(row.created_at.format("%Y-%m-%d").to_string())
                        .styled(row_style),
                    2,
                ))
                .element(elements::PaddedElement::new(
                    elements::Text::new(truncate_id(&row.payment_id)).styled(row_style),
                    2,
                ))
                .element(elements::PaddedElement::new(
                    elements::Text::new(format!(
                        "{} on {}",
                        row.crypto_type.as_deref().unwrap_or("N/A"),
                        row.network.as_deref().unwrap_or("N/A")
                    ))
                    .styled(row_style),
                    2,
                ))
                .element(elements::PaddedElement::new(
                    elements::Text::new(format!("${:.2}", row.amount_usd)).styled(row_style),
                    2,
                ))
                .element(elements::PaddedElement::new(
                    elements::Text::new(row.status.to_string())
                        .styled(row_style.with_color(status_color)),
                    2,
                ))
                .push();
        }

        doc.push(table);

        // Footer
        doc.push(elements::Break::new(2.0));
        doc.push(
            elements::Paragraph::new("Generated by FidduPay - Decentralized Payment Gateway")
                .styled(style::Color::Rgb(128, 128, 128)),
        );

        let mut buffer = Vec::new();
        doc.render(&mut buffer)
            .map_err(|e| ServiceError::InternalError(format!("PDF rendering failed: {}", e)))?;

        Ok(buffer)
    }
}
