// Input Validation Middleware
// Provides comprehensive input validation for all API endpoints

use axum::{
    extract::{Json, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::borrow::Cow;
use validator::{Validate, ValidationErrors};

/// Validation middleware for JSON payloads
pub async fn validation_middleware<T>(Json(payload): Json<T>) -> Result<Json<T>, ValidationError>
where
    T: Validate,
{
    match payload.validate() {
        Ok(_) => Ok(Json(payload)),
        Err(errors) => Err(ValidationError::from(errors)),
    }
}

/// Custom validation error type
#[derive(Debug)]
pub struct ValidationError {
    pub errors: ValidationErrors,
}

impl From<ValidationErrors> for ValidationError {
    fn from(errors: ValidationErrors) -> Self {
        Self { errors }
    }
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        let error_messages: Vec<String> = self
            .errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();

        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Validation failed",
                "details": error_messages
            })),
        )
            .into_response()
    }
}

/// Request size limiting middleware
pub async fn request_size_middleware(
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    const MAX_REQUEST_SIZE: usize = 1024 * 1024; // 1MB

    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > MAX_REQUEST_SIZE {
                    return Err((
                        StatusCode::PAYLOAD_TOO_LARGE,
                        Json(json!({
                            "error": "Request too large",
                            "max_size": MAX_REQUEST_SIZE
                        })),
                    ));
                }
            }
        }
    }

    Ok(next.run(request).await)
}

/// Security headers middleware
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Prevent XSS attacks
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());

    // HTTPS enforcement
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );

    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
            .parse()
            .unwrap(),
    );

    // Referrer policy
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    response
}

/// Password strength validation using hardcoded security policies
pub fn validate_password_strength(password: &str) -> Result<(), validator::ValidationError> {
    const MIN_LENGTH: usize = 8;

    // Length check
    if password.len() < MIN_LENGTH {
        let mut err = validator::ValidationError::new("password_too_short");
        err.message = Some(Cow::from(format!(
            "Password must be at least {} characters long",
            MIN_LENGTH
        )));
        return Err(err);
    }

    // Uppercase check
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(
            validator::ValidationError::new("password_no_uppercase").with_message(Cow::from(
                "Password must contain at least one uppercase letter",
            )),
        );
    }

    // Lowercase check
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(
            validator::ValidationError::new("password_no_lowercase").with_message(Cow::from(
                "Password must contain at least one lowercase letter",
            )),
        );
    }

    // Numbers check
    if !password.chars().any(|c| c.is_numeric()) {
        return Err(validator::ValidationError::new("password_no_number")
            .with_message(Cow::from("Password must contain at least one number")));
    }

    // Symbols check (Encouraged but not strictly forced in simple mode,
    // but I'll keep it for high security)
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(
            validator::ValidationError::new("password_no_special").with_message(Cow::from(
                "Password must contain at least one special character",
            )),
        );
    }

    // Common patterns
    if password.to_lowercase().contains("password")
        || password.to_lowercase().contains("123456")
        || password.to_lowercase().contains("qwerty")
    {
        return Err(validator::ValidationError::new("password_too_common")
            .with_message(Cow::from("Password is too common or easy to guess")));
    }

    Ok(())
}

/// Email domain validation
pub fn validate_business_email(email: &str) -> Result<(), validator::ValidationError> {
    // Block common disposable email domains
    let disposable_domains = [
        "10minutemail.com",
        "tempmail.org",
        "guerrillamail.com",
        "mailinator.com",
        "yopmail.com",
        "temp-mail.org",
    ];

    if let Some(domain) = email.split('@').nth(1) {
        if disposable_domains.contains(&domain.to_lowercase().as_str()) {
            return Err(validator::ValidationError::new(
                "Disposable email addresses not allowed",
            ));
        }
    }

    Ok(())
}

/// Positive amount validation (for Decimal)
pub fn validate_positive_amount(
    amount: &rust_decimal::Decimal,
) -> Result<(), validator::ValidationError> {
    if *amount <= rust_decimal::Decimal::ZERO {
        return Err(validator::ValidationError::new(
            "Amount must be greater than zero",
        ));
    }
    Ok(())
}

/// URL validation for webhooks
pub fn validate_webhook_url(url: &str) -> Result<(), validator::ValidationError> {
    use url::Url;

    let parsed =
        Url::parse(url).map_err(|_| validator::ValidationError::new("Invalid URL format"))?;

    // Must be HTTPS
    if parsed.scheme() != "https" {
        return Err(validator::ValidationError::new(
            "Webhook URL must use HTTPS",
        ));
    }

    // Check for private/localhost IPs to prevent SSRF
    if let Some(host) = parsed.host_str() {
        if is_private_or_localhost(host) {
            return Err(validator::ValidationError::new(
                "Private IP addresses not allowed",
            ));
        }
    }

    Ok(())
}

/// Check if host is private IP or localhost
fn is_private_or_localhost(host: &str) -> bool {
    use std::net::IpAddr;

    // Check for localhost names
    if host == "localhost" || host == "127.0.0.1" || host == "::1" {
        return true;
    }

    // Parse as IP and check if private
    if let Ok(ip) = host.parse::<IpAddr>() {
        match ip {
            IpAddr::V4(ipv4) => {
                ipv4.is_private()
                    || ipv4.is_loopback()
                    || ipv4.is_link_local()
                    || ipv4.is_unspecified()
            }
            IpAddr::V6(ipv6) => {
                // Adjust to standard methods where available, keeping fallback compatibility
                ipv6.is_loopback() ||
                ipv6.segments()[0] & 0xfe00 == 0xfc00 || // Unique local (fc00::/7)
                (ipv6.segments()[0] & 0xffc0) == 0xfe80 // Link-local (fe80::/10)
            }
        }
    } else {
        false
    }
}
