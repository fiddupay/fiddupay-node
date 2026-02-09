use qrcode::QrCode;
use base64::{Engine as _, engine::general_purpose};

pub fn generate_qr_code(data: &str) -> Result<String, String> {
    let code = QrCode::new(data)
        .map_err(|e| format!("QR code generation failed: {}", e))?;
    
    // Render the QR code into an image buffer
    let image = code.render::<image::Luma<u8>>()
        .min_dimensions(200, 200)
        .max_dimensions(400, 400)
        .build();
    
    // Encode the image as PNG format in memory
    let mut bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut bytes);
    
    image.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image as PNG: {}", e))?;
    
    // Convert to base64 string
    Ok(general_purpose::STANDARD.encode(bytes))
}
