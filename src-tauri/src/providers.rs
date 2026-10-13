// Visual search providers: trace.moe, Yandex, Pinterest, Google Lens.
// Ports of CTS/Search/*Provider.cs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PreparedSearch {
    /// Opens a URL in a browser (external or built-in).
    Url { url: String },
    /// Runs an HTTP flow and returns structured results.
    Results { results: serde_json::Value },
}

/// trace.moe — pure REST, easiest provider. Port of TraceMoeProvider.cs.
#[tauri::command]
pub async fn search_tracemoe(jpeg_base64: String) -> Result<serde_json::Value, String> {
    search_tracemoe_inner(jpeg_base64).await
}

pub async fn search_tracemoe_inner(jpeg_base64: String) -> Result<serde_json::Value, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(jpeg_base64.trim())
        .map_err(|e| format!("base64 decode: {e}"))?;

    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name("image.jpg")
        .mime_str("image/jpeg")
        .map_err(|e| e.to_string())?;
    let form = reqwest::multipart::Form::new().part("image", part);

    let resp = reqwest::Client::new()
        .post("https://api.trace.moe/search")
        .header("trace.moe-api-key", std::env::var("TRACE_MOE_API_KEY").unwrap_or_default())
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("trace.moe request: {e}"))?;

    let status = resp.status();
    let raw = resp
        .text()
        .await
        .map_err(|e| format!("trace.moe read body: {e}"))?;
    eprintln!("zeroflow: trace.moe HTTP {status}, body[:400] {}", &raw[..raw.len().min(400)]);
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("trace.moe response not JSON: {e}"))?;
    eprintln!("zeroflow: trace.moe HTTP {status}");
    // Best match summary for debugging until the result UI exists.
    if let Some(docs) = value.get("result").and_then(|r| r.as_array()) {
        if let Some(best) = docs.first() {
            let sim = best.get("similarity").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let name = best
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let anilist = best
                .get("anilist")
                .and_then(|a| a.as_u64())
                .map(|i| i.to_string())
                .unwrap_or_default();
            eprintln!("zeroflow: best match {name} ({:.0}%) anilist {anilist}", sim * 100.0);
        } else {
            eprintln!("zeroflow: trace.moe no match");
        }
    } else {
        eprintln!("zeroflow: trace.moe unexpected body: {value}");
    }
    if !status.is_success() {
        return Err(format!("trace.moe HTTP {status}"));
    }
    Ok(value)
}

/// Kicks off a trace.moe search on a detached task: the overlay window may be
/// destroyed right after this returns, so the search must not live in the
/// webview's JS context.
#[tauri::command]
pub async fn submit_selection(app: tauri::AppHandle, jpeg_base64: String) -> Result<(), String> {
    tauri::async_runtime::spawn(async move {
        match search_tracemoe_inner(jpeg_base64).await {
            Ok(value) => {
                let title = match value.get("result").and_then(|r| r.as_array()) {
                    Some(docs) if !docs.is_empty() => {
                        let best = &docs[0];
                        let sim = best.get("similarity").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let name = best.get("filename").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let ep = best.get("episode").and_then(|v| v.as_u64()).unwrap_or(0);
                        format!("{name} ep {ep} ({:.0}%)", sim * 100.0)
                    }
                    _ => "No trace.moe match found".to_string(),
                };
                notify(&app, "ZeroFlow", &title);
            }
            Err(e) => {
                eprintln!("zeroflow: trace.moe failed: {e}");
                notify(&app, "ZeroFlow", "trace.moe failed");
            }
        }
    });
    Ok(())
}

fn notify(app: &tauri::AppHandle, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show();
}

/// Yandex Images — requires session cookies + upload flow. Port of YandexImagesProvider.cs.
#[tauri::command]
pub async fn search_yandex(jpeg_base64: String) -> Result<serde_json::Value, String> {
    let _ = jpeg_base64;
    Err("Yandex not implemented yet (Phase 2)".into())
}

/// Pinterest — HTTP flow. Port of PinterestProvider.cs.
#[tauri::command]
pub async fn search_pinterest(jpeg_base64: String) -> Result<serde_json::Value, String> {
    let _ = jpeg_base64;
    Err("Pinterest not implemented yet (Phase 2)".into())
}

/// Google Lens — the F0 risk item. Phase 2 decides between:
/// (a) xdg-open with a session URL, (b) built-in WebKitGTK window with the
/// original's JS injection scripts, (c) CEF if WebKit is blocked.
#[tauri::command]
pub async fn open_lens(jpeg_base64: String) -> Result<serde_json::Value, String> {
    let _ = jpeg_base64;
    Err("Lens not implemented yet (Phase 2, see PLAN F0)".into())
}
