// On-screen image translation. Port of CTS/Translation/*.
// Google image translation is a signed HTTP protocol (GoogleImageTranslationSigner/
// Protocol) — a direct Rust port, no WebView involved.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedOverlay {
    /// Detected text blocks with translations, in screenshot coordinates.
    pub blocks: Vec<TranslatedBlock>,
    pub source_language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedBlock {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub source_text: String,
    pub translated_text: String,
}

#[tauri::command]
pub async fn translate_selection(
    jpeg_base64: String,
    target_lang: String,
) -> Result<serde_json::Value, String> {
    let _ = (jpeg_base64, target_lang);
    // Phase 3: port the signer (it hashes with a fixed salt — port the exact
    // scheme from GoogleImageTranslationSigner.cs) + protocol request.
    Err("translation not implemented yet (Phase 3)".into())
}
