use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// ZeroFlow settings, stored as JSON under $XDG_CONFIG_HOME/zeroflow/settings.json.
/// Mirrors CircleFlow's AppSettings record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub search_provider_id: String,
    pub hidden_search_provider_ids: Vec<String>,
    pub text_search_engine_id: String,
    pub text_search_in_built_in_browser: bool,
    pub hotkey_gesture: String,
    pub ignore_hotkey_in_fullscreen: bool,
    pub ui_sounds: bool,
    pub scan_qr_codes: bool,
    pub overlay_zoom: bool,
    pub browser_data_cleanup_days: u32,
    pub save_music_history: bool,
    pub music_history_retention_days: u32,
    pub max_long_side_px: u32,
    pub padding_px: u32,
    pub hide_delay_milliseconds: u32,
    pub lasso_min_diagonal_px: u32,
    pub ocr_language_tag: String,
    pub app_language_tag: String,
    pub onboarding_completed: bool,
    pub target_lang: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            search_provider_id: "googlelens".into(),
            hidden_search_provider_ids: vec!["yandeximages".into()],
            text_search_engine_id: "matchimagesearch".into(),
            text_search_in_built_in_browser: false,
            hotkey_gesture: "Ctrl+Alt+Space".into(),
            ignore_hotkey_in_fullscreen: true,
            ui_sounds: true,
            scan_qr_codes: true,
            overlay_zoom: true,
            browser_data_cleanup_days: 28,
            save_music_history: true,
            music_history_retention_days: 28,
            max_long_side_px: 1600,
            padding_px: 8,
            hide_delay_milliseconds: 60,
            lasso_min_diagonal_px: 12,
            ocr_language_tag: String::new(),
            app_language_tag: String::new(),
            onboarding_completed: false,
            target_lang: String::from("pt-BR"),
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zeroflow")
}

pub fn settings_path() -> PathBuf {
    config_dir().join("settings.json")
}

pub fn load() -> Settings {
    match fs::read_to_string(settings_path()) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

pub fn save(settings: &Settings) -> std::io::Result<()> {
    fs::create_dir_all(config_dir())?;
    let text = serde_json::to_string_pretty(settings).map_err(|e| std::io::Error::other(e))?;
    // shortcut: no atomic write yet, swap for tempfile+rename if settings grow
    fs::write(settings_path(), text)
}
