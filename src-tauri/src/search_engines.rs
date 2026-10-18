// Search engine URL builders. Pure functions; no network here.


#[tauri::command]
pub fn open_search(query: String, engine: String) -> Result<(), String> {
    let url = match engine.as_str() {
        "google" => format!("https://www.google.com/search?q={}", urlencoding::encode(&query)),
        "duckduckgo" => format!("https://duckduckgo.com/?q={}", urlencoding::encode(&query)),
        "kagi" => format!("https://kagi.com/search?q={}", urlencoding::encode(&query)),
        "qwant" => format!("https://www.qwant.com/?q={}", urlencoding::encode(&query)),
        "startpage" => format!("https://www.startpage.com/sp/search?q={}", urlencoding::encode(&query)),
        "bing" => format!("https://www.bing.com/search?q={}", urlencoding::encode(&query)),
        _ => format!("https://duckduckgo.com/?q={}", urlencoding::encode(&query)),
    };
    // Reuse the existing open_url command via tauri's shell
    std::process::Command::new("xdg-open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("xdg-open failed: {e}"))?;
    Ok(())
}