// Input Validation Middleware
// Provides comprehensive input validation for all API endpoints

use axum::{
    extract::{Json, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use validator::{Validate, ValidationErrors};

/// Validation middleware for JSON payloads
pub async fn validation_middleware<T>(
    Json(payload): Json<T>,
) -> Result<Json<T>, ValidationError>
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
                    format!("{}: {}", field, error.message.as_ref().unwrap_or(&"Invalid value".into()))
                })
            })
            .collect();

        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Validation failed",
                "details": error_messages
            }))
        ).into_response()
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
                        }))
                    ));
                }
            }
        }
    }

    Ok(next.run(request).await)
}

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Prevent XSS attacks
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    
    // HTTPS enforcement
    headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
    
    // Content Security Policy
    headers.insert("Content-Security-Policy", 
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".parse().unwrap());
    
    // Referrer policy
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    
    response
}

/// Password strength validation
pub fn validate_password_strength(password: &str) -> Result<(), validator::ValidationError> {
    let mut score = 0;
    
    // Length check
    if password.len() >= 8 { score += 1; }
    if password.len() >= 12 { score += 1; }
    
    // Character variety
    if password.chars().any(|c| c.is_lowercase()) { score += 1; }
    if password.chars().any(|c| c.is_uppercase()) { score += 1; }
    if password.chars().any(|c| c.is_numeric()) { score += 1; }
    if password.chars().any(|c| !c.is_alphanumeric()) { score += 1; }
    
    // Common patterns
    if !password.to_lowercase().contains("password") &&
       !password.to_lowercase().contains("123456") &&
       !password.to_lowercase().contains("qwerty") {
        score += 1;
    }
    
    if score >= 5 {
        Ok(())
    } else {
        Err(validator::ValidationError::new("Password too weak. Must be at least 8 characters with uppercase, lowercase, numbers, and symbols"))
    }
}

/// Email domain validation
pub fn validate_business_email(email: &str) -> Result<(), validator::ValidationError> {
    // Block common disposable email domains
    let disposable_domains = [
        "10minutemail.com", "tempmail.org", "guerrillamail.com",
        "mailinator.com", "yopmail.com", "temp-mail.org"
    ];
    
    if let Some(domain) = email.split('@').nth(1) {
        if disposable_domains.contains(&domain.to_lowercase().as_str()) {
            return Err(validator::ValidationError::new("Disposable email addresses not allowed"));
        }
    }
    
    Ok(())
}

/// URL validation for webhooks
pub fn validate_webhook_url(url: &str) -> Result<(), validator::ValidationError> {
    use url::Url;
    
    let parsed = Url::parse(url)
        .map_err(|_| validator::ValidationError::new("Invalid URL format"))?;
    
    // Must be HTTPS
    if parsed.scheme() != "https" {
        return Err(validator::ValidationError::new("Webhook URL must use HTTPS"));
    }
    
    // Check for private/localhost IPs to prevent SSRF
    if let Some(host) = parsed.host_str() {
        if is_private_or_localhost(host) {
            return Err(validator::ValidationError::new("Private IP addresses not allowed"));
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
                let octets = ipv4.octets();
                // Private IPv4 ranges
                matches!(octets[0], 10) ||
                (octets[0] == 172 && (16..=31).contains(&octets[1])) ||
                (octets[0] == 192 && octets[1] == 168) ||
                octets[0] == 127 // Loopback
            }
            IpAddr::V6(ipv6) => {
                // Private IPv6 ranges and loopback
                ipv6.is_loopback() || 
                ipv6.segments()[0] == 0xfc00 || // Unique local
                ipv6.segments()[0] == 0xfd00    // Unique local
            }
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation() {
        assert!(validate_password_strength("Password123!").is_ok());
        assert!(validate_password_strength("weak").is_err());
        assert!(validate_password_strength("password123").is_err());
    }

    #[test]
    fn test_webhook_url_validation() {
        assert!(validate_webhook_url("https://example.com/webhook").is_ok());
        assert!(validate_webhook_url("http://example.com/webhook").is_err());
        assert!(validate_webhook_url("https://localhost/webhook").is_err());
        assert!(validate_webhook_url("https://192.168.1.1/webhook").is_err());
    }

    #[test]
    fn test_private_ip_detection() {
        assert!(is_private_or_localhost("localhost"));
        assert!(is_private_or_localhost("127.0.0.1"));
        assert!(is_private_or_localhost("192.168.1.1"));
        assert!(is_private_or_localhost("10.0.0.1"));
        assert!(!is_private_or_localhost("8.8.8.8"));
        assert!(!is_private_or_localhost("example.com"));
    }
}
