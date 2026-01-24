use qrcode::QrCode;
use base64::{Engine as _, engine::general_purpose};

pub fn generate_qr_code(data: &str) -> Result<String, String> {
    let code = QrCode::new(data)
        .map_err(|e| format!("QR code generation failed: {}", e))?;
    
    let image = code.render::<char>()
        .min_dimensions(200, 200)
        .max_dimensions(400, 400)
        .build();
    
    // Convert to simple base64 string representation
    Ok(general_purpose::STANDARD.encode(image.as_bytes()))
}
