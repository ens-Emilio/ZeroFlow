mod capture;
mod hotkey;
mod ocr;
mod providers;
mod qr;
mod settings;
mod shazam;
mod translate;
mod tray;

use settings::Settings;
use tauri::Manager;

#[tauri::command]
fn get_settings() -> Settings {
    settings::load()
}

#[tauri::command]
fn set_settings(patch: serde_json::Value) -> Result<(), String> {
    // Merge patch into current settings, then persist.
    let current = serde_json::to_value(settings::load()).map_err(|e| e.to_string())?;
    let merged = json_merge(&current, patch);
    let new_settings: Settings = serde_json::from_value(merged).map_err(|e| e.to_string())?;
    // shortcut: shallow merge only; nested objects get replaced wholesale
    settings::save(&new_settings).map_err(|e| e.to_string())
}

fn json_merge(base: &serde_json::Value, patch: serde_json::Value) -> serde_json::Value {
    match (base, patch) {
        (serde_json::Value::Object(b), serde_json::Value::Object(p)) => {
            let mut out = b.clone();
            for (k, v) in p {
                out.insert(k.clone(), v);
            }
            serde_json::Value::Object(out)
        }
        (_, patch) => patch,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Focus the overlay window on a second launch attempt.
            if let Some(overlay) = app.get_webview_window("overlay") {
                let _ = overlay.show();
                let _ = overlay.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            hotkey::toggle_overlay,
            hotkey::start_overlay,
            hotkey::hide_overlay,
            hotkey::copy_image,
            hotkey::save_image,
            hotkey::open_settings,
            hotkey::open_music,
            hotkey::quit_app,
            capture::grab_frame,
            capture::crop_selection,
            providers::search_tracemoe,
            providers::search_yandex,
            providers::search_pinterest,
            providers::open_lens,
            ocr::ocr,
            qr::scan_qr,
            shazam::start_music_recognition,
            shazam::stop_music_recognition,
            translate::translate_selection,
            tray::show_toast,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ZeroFlow");
}
