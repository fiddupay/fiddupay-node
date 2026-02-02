use serde::Serialize;
use crate::error::ServiceError;
use tracing::{info, warn, error};
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use lettre::message::header::ContentType;

#[derive(Debug, Serialize)]
pub struct EmailTemplate {
    pub to: String,
    pub subject: String,
    pub body: String,
}

pub struct EmailService {
    enabled: bool,
    from_email: String,
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
}

impl EmailService {
    pub fn new(
        enabled: bool,
        from_email: String,
        smtp_host: Option<String>,
        smtp_port: Option<u16>,
        smtp_username: Option<String>,
        smtp_password: Option<String>,
    ) -> Self {
        Self {
            enabled,
            from_email,
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
        }
    }

    pub async fn send_payment_confirmed(&self, to: &str, payment_id: &str, amount: &str, crypto: &str) -> Result<(), ServiceError> {
        if !self.enabled {
            info!(" Email disabled - would send payment confirmation to {}", to);
            return Ok(());
        }

        let subject = format!("Payment Confirmed - {}", payment_id);
        let body = format!(
            "Your payment has been confirmed!\n\n\
             Payment ID: {}\n\
             Amount: {} {}\n\n\
             Thank you for your payment.",
            payment_id, amount, crypto
        );

        self.send_email(to, &subject, &body).await
    }

    pub async fn send_withdrawal_completed(&self, to: &str, withdrawal_id: &str, amount: &str, crypto: &str) -> Result<(), ServiceError> {
        if !self.enabled {
            info!(" Email disabled - would send withdrawal notification to {}", to);
            return Ok(());
        }

        let subject = format!("Withdrawal Completed - {}", withdrawal_id);
        let body = format!(
            "Your withdrawal has been completed!\n\n\
             Withdrawal ID: {}\n\
             Amount: {} {}\n\n\
             Funds have been sent to your wallet.",
            withdrawal_id, amount, crypto
        );

        self.send_email(to, &subject, &body).await
    }

    pub async fn send_invoice(&self, to: &str, invoice_id: &str, total: &str, due_date: &str) -> Result<(), ServiceError> {
        if !self.enabled {
            info!(" Email disabled - would send invoice to {}", to);
            return Ok(());
        }

        let subject = format!("Invoice {} - Due {}", invoice_id, due_date);
        let body = format!(
            "You have received an invoice.\n\n\
             Invoice ID: {}\n\
             Total: ${}\n\
             Due Date: {}\n\n\
             Please pay at your earliest convenience.",
            invoice_id, total, due_date
        );

        self.send_email(to, &subject, &body).await
    }

    pub async fn send_2fa_enabled(&self, to: &str) -> Result<(), ServiceError> {
        if !self.enabled {
            info!(" Email disabled - would send 2FA notification to {}", to);
            return Ok(());
        }

        let subject = "Two-Factor Authentication Enabled";
        let body = "Two-factor authentication has been enabled on your account.\n\n\
                    If you did not make this change, please contact support immediately.";

        self.send_email(to, subject, body).await
    }

    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), ServiceError> {
        if !self.enabled {
            return Ok(());
        }

        // Check if SMTP is configured
        let smtp_host = self.smtp_host.as_ref()
            .ok_or_else(|| ServiceError::InternalError("SMTP host not configured".to_string()))?;
        let smtp_username = self.smtp_username.as_ref()
            .ok_or_else(|| ServiceError::InternalError("SMTP username not configured".to_string()))?;
        let smtp_password = self.smtp_password.as_ref()
            .ok_or_else(|| ServiceError::InternalError("SMTP password not configured".to_string()))?;

        // Build email
        let email = Message::builder()
            .from(self.from_email.parse()
                .map_err(|e| ServiceError::InternalError(format!("Invalid from email: {}", e)))?)
            .to(to.parse()
                .map_err(|e| ServiceError::InternalError(format!("Invalid to email: {}", e)))?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| ServiceError::InternalError(format!("Email build failed: {}", e)))?;

        // Create SMTP transport
        let creds = Credentials::new(smtp_username.clone(), smtp_password.clone());
        
        let mailer = SmtpTransport::relay(smtp_host)
            .map_err(|e| ServiceError::InternalError(format!("SMTP relay failed: {}", e)))?
            .credentials(creds)
            .build();

        // Send email
        match mailer.send(&email) {
            Ok(_) => {
                info!(" Email sent: {} -> {} | {}", self.from_email, to, subject);
                Ok(())
            }
            Err(e) => {
                error!(" Email send failed: {}", e);
                Err(ServiceError::InternalError(format!("Email send failed: {}", e)))
            }
        }
    }
}
