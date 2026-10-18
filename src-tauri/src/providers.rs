// Visual search providers: trace.moe, Yandex, Pinterest, Google Lens.
// Ports of reference/CTS/Search/*Provider.cs.
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

    let resp = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("trace.moe client: {e}"))?
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

/// Yandex Images - requires session cookies + upload flow. Port of YandexImagesProvider.cs.
#[tauri::command]
pub async fn search_yandex(jpeg_base64: String) -> Result<serde_json::Value, String> {
    use base64::Engine;
    let jpeg = base64::engine::general_purpose::STANDARD
        .decode(jpeg_base64.trim())
        .map_err(|e| format!("invalid base64: {e}"))?;
    let url = upload_to_host(&jpeg).await?;
    let search = format!(
        "https://yandex.com/images/search?rpt=imageview&url={}",
        urlencoding::encode(&url)
    );
    let _ = crate::hotkey::open_url(search.clone());
    Ok(serde_json::json!({ "url": search }))
}

/// Pinterest - HTTP flow. Port of PinterestProvider.cs.
#[tauri::command]
pub async fn search_pinterest(jpeg_base64: String) -> Result<serde_json::Value, String> {
    use base64::Engine;
    let jpeg = base64::engine::general_purpose::STANDARD
        .decode(jpeg_base64.trim())
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
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("upload client: {e}"))?;
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

/// Google Lens - the F0 risk item. The Linux port opens the returned URL through xdg-open.
#[tauri::command]
pub async fn open_lens(jpeg_base64: String) -> Result<serde_json::Value, String> {
    use base64::Engine;
    eprintln!("zeroflow: open_lens invoked, {} b64 chars", jpeg_base64.len());
    let jpeg = base64::engine::general_purpose::STANDARD
        .decode(jpeg_base64.trim())
        .map_err(|e| format!("invalid base64: {e}"))?;

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("client build: {e}"))?;
    // Warmup: fetch lens homepage to prime cookies / TLS (improves 303 rate)
    let warmup_cookies = match client.get("https://lens.google.com/").send().await {
        Ok(r) => r.headers().get_all("set-cookie").iter().filter_map(|v| v.to_str().ok()).map(|s| s.split(';').next().unwrap_or("").to_string()).collect::<Vec<_>>().join("; "),
        Err(_) => String::new(),
    };

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
        .header("Cookie", warmup_cookies.clone())
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
