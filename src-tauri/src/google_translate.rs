// Google Image Translation protocol: port of CTS/Translation/GoogleImageTranslationProtocol.cs.
// Handles signing challenge and request/response format for
// https://translate.google.com/ (visual image translation).

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use regex::Regex;
use serde_json::json;

/// Builds the signed RPC request body for the translate endpoint.
pub fn create_request(png: &[u8], target_lang: &str) -> String {
    // Matches CTS: new object[] { new object?[] { new[] { png_b64, "image/png" }, "auto", target }, null, "generic" }
    let png_b64 = STANDARD.encode(png);
    let inner = json!([png_b64, "image/png", "auto", target_lang]);
    let outer = json!([inner, null, "generic"]);
    json!([outer]).to_string()
}

/// Extracts the challenge from the initialization HTML.
/// Looks for: "mnsUbf":"<value>" where <value> is a base64 string that starts with "%.@."
pub fn read_challenge(html: &str) -> Result<(String, String, String), String> {
    // The page embeds the challenge as a JSON-escaped string value:
    //   "mnsUbf":"%.@.[\"program\",...]"
    // Matches CTS ReadChallenge: deserialize the captured value as a JSON
    // string first, strip the "%.@." prefix, then parse the remainder as a
    // JSON array (fields[0] program, fields[5][5] interpreter, fields[7] state).
    let re = Regex::new(r#""mnsUbf"\s*:\s*("(?:\\.|[^"\\])*")"#)
        .map_err(|e| format!("regex compile: {e}"))?;
    let caps = re.captures(html)
        .ok_or_else(|| "challenge not found in HTML (mnsUbf missing)".to_string())?;
    let encoded: String = serde_json::from_str(caps.get(1).unwrap().as_str())
        .map_err(|e| format!("challenge json string decode: {e}"))?;
    if !encoded.starts_with("%.@.") {
        return Err("unexpected challenge encoding (no %.@. prefix)".to_string());
    }
    let fields: serde_json::Value = serde_json::from_str(&format!("[{}", &encoded[4..]))
        .map_err(|e| format!("challenge fields parse: {e}"))?;
    let arr = fields.as_array().ok_or("challenge fields not an array")?;
    let program = arr.first()
        .and_then(|v| v.as_str())
        .ok_or("challenge program missing")?
        .to_string();
    let interpreter = arr.get(5)
        .and_then(|v| v.as_array())
        .and_then(|a| a.get(5))
        .and_then(|v| v.as_str())
        .ok_or("challenge interpreter missing")?
        .to_string();
    let state = arr.get(7)
        .and_then(|v| v.as_str())
        .ok_or("challenge state missing")?
        .to_string();
    Ok((program, interpreter, state))
}

/// Decodes the base64 PNG from the translate response.
/// Expects JSON: [[ "wrb.fr", "WqWDPb", "<base64 png>" ]]
pub fn read_response(png_b64: &str) -> Result<Vec<u8>, String> {
    let data: serde_json::Value = serde_json::from_str(png_b64)
        .map_err(|e| format!("json parse response: {e}"))?;
    let inner = data
        .get(0)
        .and_then(|v| v.as_array())
        .ok_or_else(|| "response not an array".to_string())?;
    let png_b64_inner = inner
        .get(2)
        .and_then(|v| v.as_str())
        .ok_or_else(|| "response missing third element".to_string())?;
    let png_bytes = STANDARD.decode(png_b64_inner)
        .map_err(|e| format!("png base64 decode: {e}"))?;
    // Confirm it's a PNG by checking the header
    if png_b64_inner.len() < 44 || &png_bytes[0..8] != &[137, 80, 78, 71, 13, 10, 26, 10] {
        return Err(format!(
            "decoded data is not a PNG (header check failed): {} bytes",
            png_bytes.len()
        ));
    }
    Ok(png_bytes)
}
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Mini HTTP server that listens for a single POST /token and returns the body.
/// Returns the received token as String when done.
async fn start_token_server(port: u16) -> Result<String, String> {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .map_err(|e| format!("token server bind: {e}"))?;
    // Accept one connection
    let (mut socket, _) = listener
        .accept()
        .await
        .map_err(|e| format!("token server accept: {e}"))?;
    // Read request line and headers until \r\n\r\n
    let mut buf = [0; 1024];
    let n = socket
        .read(&mut buf)
        .await
        .map_err(|e| format!("token server read: {e}"))?;
    let req = std::str::from_utf8(&buf[..n])
        .map_err(|e| format!("invalid utf-8: {e}"))?;
    // Very crude: assume it's a POST with body after \r\n\r\n
    // We just extract everything after the first \r\n\r\n as the body
    if let Some(body) = req.split("\r\n\r\n").nth(1) {
        // Respond with 200 OK
        let resp = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
        socket
            .write_all(resp.as_bytes())
            .await
            .map_err(|e| format!("token server write: {e}"))?;
        Ok(body.to_string())
    } else {
        Err("malformed token request".to_string())
    }
}

/// Translates an image in-situ with Google's image translation:
/// 1. GET the init page to obtain the signing challenge
/// 2. Run the challenge in a hidden webview (via initialization script)
///    which posts the signed token back to a local HTTP server
/// 3. POST the image to batchexecute with the signed header
/// 4. Decode the returned PNG (the image with translated text)
#[tauri::command]
pub async fn translate_image(
    app: tauri::AppHandle,
    jpeg_base64: String,
    target_lang: String,
) -> Result<serde_json::Value, String> {
    eprintln!("zeroflow: translate_image invoked, {} b64 chars, target {}", jpeg_base64.len(), target_lang);

    let jpeg = STANDARD.decode(jpeg_base64.trim())
        .map_err(|e| format!("invalid base64: {e}"))?;

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("client build: {e}"))?;

    // 1. Fetch the initialization page and extract the challenge.
    let init_url = format!("https://translate.google.com/?sl=auto&tl={}&op=images", urlencoding::encode(&target_lang));
    let html = client
        .get(&init_url)
        .send()
        .await
        .map_err(|e| format!("init page fetch: {e}"))?
        .text()
        .await
        .map_err(|e| format!("init page read: {e}"))?;
    eprintln!("zeroflow: translate init page {} bytes", html.len());

    let (program, interpreter, _state) = read_challenge(&html)
        .map_err(|e| format!("challenge: {e}"))?;
    eprintln!("zeroflow: translate challenge ok, program {} bytes, interpreter {} bytes", program.len(), interpreter.len());

    // The image must be sent as PNG; convert the JPEG crop.
    let img = image::load_from_memory(&jpeg)
        .map_err(|e| format!("image decode: {e}"))?;
    let mut png_bytes = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        img.write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| format!("png encode: {e}"))?;
    }
    let request_body = create_request(&png_bytes, &target_lang);
    eprintln!("zeroflow: translate request body {} bytes", request_body.len());

    // 2. Hidden webview + local token server.
    let port = pick_free_port()?;
    let token_rx = {
        let (tx, rx) = tokio::sync::oneshot::channel::<String>();
        tokio::spawn(async move {
            match start_token_server(port).await {
                Ok(token) => { let _ = tx.send(token); }
                Err(e) => { let _ = tx.send(format!("__error__{e}")); }
            }
        });
        rx
    };

    // initialization script: runs after the bscframe page loads; waits for
    // the challenge interpreter, generates the token and POSTs it to us.
    let init_script = format!(
        r#"(async () => {{
            try {{
                {interpreter}
                window.ctsGenerator = new window.ridgeslice.gc({program_json});
                const timeout = setTimeout(() => reject(new Error('timeout')), 15000);
                window.ctsGenerator.ply(token => {{
                    clearTimeout(timeout);
                    fetch('http://127.0.0.1:{port}/token', {{
                        method: 'POST',
                        body: token,
                    }});
                }}, true, {{mgGpzd: {request_json}}});
            }} catch (e) {{
                fetch('http://127.0.0.1:{port}/token', {{
                    method: 'POST',
                    body: '__error__' + e,
                }});
            }}
        }})();"#,
        interpreter = interpreter,
        program_json = serde_json::to_string(&program).map_err(|e| format!("program json: {e}"))?,
        port = port,
        request_json = serde_json::to_string(&request_body).map_err(|e| format!("request json: {e}"))?,
    );

    // Create the hidden translator webview.
    let translator = tauri::webview::WebviewWindowBuilder::new(
        &app,
        "translator",
        tauri::WebviewUrl::External("https://translate.google.com/_/TranslateWebserverUi/bscframe".parse().map_err(|e| format!("bad url: {e}"))?),
    )
    .visible(false)
    .skip_taskbar(true)
    .resizable(false)
    .inner_size(64.0, 64.0)
    .initialization_script(init_script)
    .build()
    .map_err(|e| format!("translator window: {e}"))?;

    // Wait for the token (or an error marker).
    let token = tokio::time::timeout(std::time::Duration::from_secs(20), token_rx)
        .await
        .map_err(|e| format!("token wait: {e}"))
        .and_then(|r| r.map_err(|e| format!("token channel: {e}")))
        .map_err(|_| "token generation timed out".to_string())?;
    let _ = translator.close();
    if token.starts_with("__error__") {
        return Err(token.trim_start_matches("__error__").to_string());
    }
    eprintln!("zeroflow: translate token received, {} chars", token.len());

    // 3. POST to batchexecute with the signed header.
    let endpoint = "https://translate.google.com/_/TranslateWebserverUi/data/batchexecute?rpcids=WqWDPb&rt=c";
    let resp = client
        .post(endpoint)
        .header("X-Goog-BatchExecute-Bgr", token)
        .form(&[("f.req", request_body.as_str())])
        .send()
        .await
        .map_err(|e| format!("batchexecute: {e}"))?;

    let status = resp.status();
    eprintln!("zeroflow: translate batchexecute status {}", status);
    if !status.is_success() {
        return Err(format!("batchexecute returned {}", status));
    }

    // 4. Parse the response: find the record line starting with [[ and decode.
    let body = resp.text().await.map_err(|e| format!("batchexecute read: {e}"))?;
    for line in body.lines() {
        if !line.starts_with("[[") {
            continue;
        }
        let parsed: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(records) = parsed.as_array() {
            for record in records {
                let arr = match record.as_array() { Some(a) => a, None => continue };
                if arr.len() < 3 { continue; }
                if arr[0].as_str() != Some("wrb.fr") { continue; }
                if arr[1].as_str() != Some("WqWDPb") { continue; }
                let payload_str = match arr[2].as_str() { Some(s) => s, None => continue };
                let payload: serde_json::Value = match serde_json::from_str(payload_str) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                // payload = [[b64, mime], detected, translated, ...]
                let b64 = payload
                    .get(0)
                    .and_then(|v| v.get(0))
                    .and_then(|v| v.as_str())
                    .ok_or("payload missing image data")?;
                let png = STANDARD.decode(b64)
                    .map_err(|e| format!("image decode: {e}"))?;
                let b64_out = STANDARD.encode(&png);
                eprintln!("zeroflow: translate got image, {} bytes png", png.len());
                return Ok(serde_json::json!({ "translatedPngBase64": b64_out }));
            }
        }
    }

    Err("translation RPC absent from response".into())
}

fn pick_free_port() -> Result<u16, String> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("port bind: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("local addr: {e}"))?
        .port();
    drop(listener);
    Ok(port)
}
