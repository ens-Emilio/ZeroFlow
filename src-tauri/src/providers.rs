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

fn notify(app: &tauri::AppHandle, _title: &str, body: &str) {
    // The Tauri notification plugin on Linux (DBus) is flaky w/ KDE.
    // Direct DBus call works (tested): gdbus call ... Notify returns ID.
    use std::process::Command;
    let title = "ZeroFlow";
    let app_name = "zeroflow";
    let icon = "";
    let urgency = 1; // normal
    let timeout = 5000; // ms
    // org.freedesktop.Notifications.Notify signature:
    // (app_name, replaces_id, icon, summary, body, actions[], hints{}, timeout)
    // We use hints with "urgency" as byte (1).
    let hints = "{'urgency': <byte 1>}";
    let timeout = "5000";
    let output = Command::new("gdbus")
        .args([
            "call", "--session",
            "--dest", "org.freedesktop.Notifications",
            "--object-path", "/org/freedesktop/Notifications",
            "--method", "org.freedesktop.Notifications.Notify",
            app_name, "0", icon, title, body,
            "[]", hints, timeout
        ])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            eprintln!("zeroflow: notify OK: {}", String::from_utf8_lossy(&o.stdout).trim());
        }
        Ok(o) => {
            eprintln!("zeroflow: notify FAIL: {}", String::from_utf8_lossy(&o.stderr).trim());
        }
        Err(e) => eprintln!("zeroflow: notify spawn: {e}"),
    }
}

/// Yandex Images — requires session cookies + upload flow. Port of YandexImagesProvider.cs.
#[tauri::command]
pub async fn search_yandex(jpeg_base64: String) -> Result<serde_json::Value, String> {
    // Reverse image search needs a public URL: upload the crop to a free
    // anonymous host, then open Yandex with that URL.
    let jpeg = base64::decode(jpeg_base64.trim())
        .map_err(|e| format!("invalid base64: {e}"))?;
    let url = upload_to_host(&jpeg).await?;
    let search = format!(
        "https://yandex.com/images/search?rpt=imageview&url={}",
        urlencoding::encode(&url)
    );
    let _ = crate::hotkey::open_url(search.clone());
    Ok(serde_json::json!({ "url": search }))
}

/// Pinterest — HTTP flow. Port of PinterestProvider.cs.
#[tauri::command]
pub async fn search_pinterest(jpeg_base64: String) -> Result<serde_json::Value, String> {
    let jpeg = base64::decode(jpeg_base64.trim())
        .map_err(|e| format!("invalid base64: {e}"))?;
    let url = upload_to_host(&jpeg).await?;
    let search = format!(
        "https://www.pinterest.com/visual-search/?image_url={}",
        urlencoding::encode(&url)
    );
    let _ = crate::hotkey::open_url(search.clone());
    Ok(serde_json::json!({ "url": search }))
}

/// Uploads bytes to a free anonymous file host and returns the public URL.
/// 0x0.st: no account, no key, files expire after ~30 days of inactivity.
async fn upload_to_host(jpeg: &[u8]) -> Result<String, String> {
    let client = reqwest::Client::new();
    let part = reqwest::multipart::Part::bytes(jpeg.to_vec())
        .mime_str("image/jpeg")
        .map_err(|e| format!("mime: {e}"))?
        .file_name("upload.jpg");
    let form = reqwest::multipart::Form::new().part("file", part);
    let resp = client
        .post("https://0x0.st")
        .header("User-Agent", "ZeroFlow/0.1 (https://github.com/ens-Emilio/ZeroFlow)")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("upload failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("upload host returned {}", resp.status()));
    }
    let url = resp.text().await.map_err(|e| format!("upload read: {e}"))?;
    let url = url.trim().to_string();
    if !url.starts_with("https://") {
        return Err(format!("upload host returned unexpected body: {}", &url[..url.len().min(100)]));
    }
    Ok(url)
}

/// Google Lens — the F0 risk item. Phase 2 decides between:
/// (a) xdg-open with a session URL, (b) built-in WebKitGTK window with the
/// original's JS injection scripts, (c) CEF if WebKit is blocked.
#[tauri::command]
pub async fn open_lens(jpeg_base64: String) -> Result<serde_json::Value, String> {
    eprintln!("zeroflow: open_lens invoked, {} b64 chars", jpeg_base64.len());
    // Lens expects the raw JPEG bytes, not the base64 string.
    let jpeg = match base64::decode(jpeg_base64.trim()) {
        Ok(b) => b,
        Err(e) => return Err(format!("invalid base64: {e}")),
    };

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("client build: {e}"))?;

    // Same upload as CircleFlow's GoogleLensBrowserOperation: a single
    // encoded_image field with the raw JPEG, posted to v3/upload.
    let form = reqwest::multipart::Form::new().part(
        "encoded_image",
        reqwest::multipart::Part::bytes(jpeg)
            .mime_str("image/jpeg")
            .map_err(|e| format!("mime: {e}"))?
            .file_name("upload.jpg"),
    );

    let resp = client
        .post("https://lens.google.com/v3/upload")
        .header("Origin", "https://lens.google.com")
        .header("Referer", "https://lens.google.com/")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Lens request failed: {e}"))?;

    eprintln!("zeroflow: lens upload status {}", resp.status());

    // Lens answers with a redirect to the results page.
    let mut url = None;
    if let Some(loc) = resp.headers().get("location") {
        if let Ok(loc_str) = loc.to_str() {
            if loc_str.starts_with('/') {
                url = Some(format!("https://lens.google.com{}", loc_str));
            } else {
                url = Some(loc_str.to_string());
            }
        }
    }
    if let Some(url) = url {
        eprintln!("zeroflow: lens results url: {}", url);
        let _ = crate::hotkey::open_url(url.clone());
        return Ok(serde_json::json!({ "url": url }));
    }

    let body_text = match resp.text().await {
        Ok(t) => t,
        Err(e) => return Err(format!("failed to read response: {e}")),
    };
    eprintln!("zeroflow: lens body[:300]: {}", &body_text[..body_text.len().min(300)]);

    use scraper::{Html, Selector};
    let document = Html::parse_document(&body_text);
    let link_sel = Selector::parse("a[href]").map_err(|e| format!("bad selector: {e}"))?;
    for link in document.select(&link_sel) {
        if let Some(href) = link.value().attr("href") {
            if href.starts_with("https://lens.google.com/search")
                || href.contains("udm=26")
                || href.contains("tbm=isch")
            {
                let _ = crate::hotkey::open_url(href.to_string());
                return Ok(serde_json::json!({ "url": href }));
            }
        }
    }

    Err("Google Lens did not return a results page".into())
}
