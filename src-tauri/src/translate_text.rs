// LibreTranslate text translation: translates OCR'd text, free and
// self-hostable. Endpoint configurable via settings/LIBRETRANSLATE_URL.
use serde_json::json;

#[tauri::command]
pub async fn translate_text(text: String, target_lang: String) -> Result<serde_json::Value, String> {
    let endpoint = std::env::var("LIBRETRANSLATE_URL")
        .unwrap_or_else(|_| "https://libretranslate.com/translate".into());
    let text_trim = text.trim().to_string();
    if text_trim.is_empty() {
        return Err("nothing to translate".into());
    }
    if target_lang.trim().is_empty() {
        return Err("target language is empty".into());
    }

    let client = reqwest::Client::new();
    let resp = client
        .post(&endpoint)
        .json(&json!({
            "q": text_trim,
            "source": "auto",
            "target": target_lang,
            "format": "text",
        }))
        .send()
        .await
        .map_err(|e| format!("LibreTranslate request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("LibreTranslate returned {}: {}", status, &body[..body.len().min(200)]));
    }

    let parsed: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("bad LibreTranslate response: {e}"))?;
    let translated = parsed
        .get("translatedText")
        .and_then(|v| v.as_str())
        .ok_or("LibreTranslate response missing translatedText")?
        .to_string();
    Ok(json!({ "translatedText": translated }))
}
