use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicHistoryEntry {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub cover_url: Option<String>,
    pub shazam_url: Option<String>,
    pub timestamp: u64,
}

fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("zeroflow")
}
fn history_path() -> PathBuf { config_dir().join("music_history.json") }

pub fn load_history() -> Vec<MusicHistoryEntry> {
    fs::read_to_string(history_path()).ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn push_history(entry: MusicHistoryEntry) {
    let mut list = load_history();
    list.insert(0, entry);
    let retention_days = crate::settings::load().music_history_retention_days as u64;
    if retention_days > 0 {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let cutoff = now.saturating_sub(retention_days * 86400);
        list.retain(|e| e.timestamp >= cutoff);
    }
    list.truncate(100);
    let _ = fs::create_dir_all(config_dir());
    if let Ok(text) = serde_json::to_string_pretty(&list) {
        let _ = fs::write(history_path(), text);
    }
}

#[tauri::command]
pub fn get_music_history() -> Vec<MusicHistoryEntry> { load_history() }

#[tauri::command]
pub fn clear_music_history() -> Result<(), String> {
    let p = history_path();
    if p.exists() { fs::remove_file(&p).map_err(|e| e.to_string())?; }
    Ok(())
}
