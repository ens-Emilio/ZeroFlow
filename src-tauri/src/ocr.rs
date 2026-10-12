// OCR via Tesseract (leptess). Port of CTS/TextRecognition/*.
// The original uses Windows.Media.Ocr; here we wrap Tesseract and produce the
// same OcrDocument shape: words with bounding boxes, so the TS hit-tester
// (OcrTextHitTester port) works unchanged.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrWord {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrDocument {
    pub words: Vec<OcrWord>,
    pub language_tag: String,
}

#[tauri::command]
pub async fn ocr(png_base64: String, lang_tag: String) -> Result<OcrDocument, String> {
    let _ = (png_base64, lang_tag);
    // Phase 3: decode PNG, leptess with lang_tag (tesseract traineddata must be installed).
    Err("OCR not implemented yet (Phase 3)".into())
}
