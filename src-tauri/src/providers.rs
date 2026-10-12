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
    // Phase 1: multipart POST to https://trace.moe/api/search with the cropped JPEG.
    let _ = jpeg_base64;
    Err("trace.moe not implemented yet (Phase 1)".into())
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
