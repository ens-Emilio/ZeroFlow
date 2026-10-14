// QR code scanning from the captured frame. Port of CTS/QrCodes/QrCodeScanner.cs (ZXing.Net -> rxing).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrContent {
    /// Raw decoded text (URL or other payload).
    pub text: String,
    /// Bounds within the screenshot, CSS px.
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[tauri::command]
pub async fn scan_qr(png_base64: String) -> Result<Option<QrContent>, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(png_base64.trim())
        .map_err(|e| format!("base64 decode: {e}"))?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("decode image: {e}"))?;

    // The crop already isolates the QR; decode the cropped region directly.
    // HybridBinarizer handles uneven lighting better than GlobalHistogram.
    let source = rxing::BufferedImageLuminanceSource::new(img);
    let binarizer = rxing::common::HybridBinarizer::new(source);
    let mut bitmap = rxing::BinaryBitmap::new(binarizer);
    let mut reader = rxing::qrcode::QRCodeReader::new();
    use rxing::Reader as _;
    match reader.decode(&mut bitmap) {
        Ok(result) => {
            let text = result.getText().to_string();
            eprintln!("zeroflow: scan_qr found {text}");
            let points = result.getPoints();
            let (x, y) = (
                points.first().map(|p| p.x as f64).unwrap_or_default(),
                points.first().map(|p| p.y as f64).unwrap_or_default(),
            );
            let (max_x, max_y) = points
                .iter()
                .fold((0.0f64, 0.0f64), |(mx, my), p| {
                    (mx.max(p.x as f64), my.max(p.y as f64))
                });
            Ok(Some(QrContent {
                text,
                x,
                y,
                width: max_x - x,
                height: max_y - y,
            }))
        }
        Err(e) => {
            eprintln!("zeroflow: scan_qr no code: {e:?}");
            Ok(None)
        }
    }
}
