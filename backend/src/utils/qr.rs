use base64::{engine::general_purpose, Engine as _};
use qrcode::QrCode;

pub fn generate_qr_code(data: &str) -> Result<String, String> {
    let code = QrCode::new(data).map_err(|e| format!("QR code generation failed: {}", e))?;

    // Get dimensions
    let width = code.width();
    let scale = 8; // 8 pixels per module
    let image_size = (width * scale) as u32;

    // Create a new Grayscale image buffer
    let mut img = image::ImageBuffer::new(image_size, image_size);

    // Draw the modules manually
    for y in 0..width {
        for x in 0..width {
            let color = if code[(x, y)] == qrcode::Color::Dark {
                image::Luma([0u8])
            } else {
                image::Luma([255u8])
            };

            // Fill the scaled block
            for iy in 0..scale {
                for ix in 0..scale {
                    img.put_pixel(
                        (x * scale) as u32 + ix as u32,
                        (y * scale) as u32 + iy as u32,
                        color,
                    );
                }
            }
        }
    }

    // Encode the image as PNG format in memory
    let mut bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut bytes);

    image::DynamicImage::ImageLuma8(img)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image as PNG: {}", e))?;

    // Convert to base64 string
    Ok(general_purpose::STANDARD.encode(bytes))
}
