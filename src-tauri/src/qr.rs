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
    let _ = png_base64;
    // Phase 1: rxing barcode scan over the captured frame; return first QR found.
    Err("QR scan not implemented yet (Phase 1)".into())
}
