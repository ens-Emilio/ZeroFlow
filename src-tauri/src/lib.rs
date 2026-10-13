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
use std::str::FromStr;
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
        .setup(|app| {
            // Register the toggle hotkey at runtime; Wayland goes through the
            // global-shortcuts portal, X11 grabs directly.
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
            let handle = app.handle().clone();
            let shortcut = Shortcut::from_str("Ctrl+Alt+Z")
                .map_err(|e| format!("bad shortcut: {e}"))?;
            let gs = app.global_shortcut();
            // Wayland/KWin: never call show/hide from the handler; the frontend
            // toggles visibility via CSS so no window protocol ops ever run.
            gs.on_shortcut(shortcut, move |_app, _sc, event| {
                if event.state == ShortcutState::Pressed {
                    let handle = handle.clone();
                    // Spawn async: grab the frame BEFORE showing the overlay so
                    // the screenshot does not contain the overlay itself.
                    tauri::async_runtime::spawn(async move {
                        use tauri::Emitter;
                        // Hide path: destroy the window (close) instead of hide;
                        // on this stack hide() leaves a dead handle that makes
                        // get_webview_window return None on later toggles.
                        if let Some(overlay) = handle.get_webview_window("overlay") {
                            if overlay.is_visible().unwrap_or(false) {
                                let _ = overlay.close();
                                let _ = handle.emit("toggle-overlay", String::new());
                                return;
                            }
                        }
                        let url = match crate::capture::grab_frame_inner(false).await {
                            Ok(url) => url,
                            Err(e) => {
                                eprintln!("zeroflow: grab_frame failed: {e}");
                                return;
                            }
                        };
                        if let Some(overlay) = handle.get_webview_window("overlay") {
                            let _ = overlay.close();
                        }
                        let overlay = tauri::webview::WebviewWindowBuilder::new(
                            &handle,
                            "overlay",
                            tauri::WebviewUrl::App("/".into()),
                        )
                        .title("ZeroFlow Overlay")
                        .decorations(false)
                        .transparent(true)
                        .always_on_top(true)
                        .skip_taskbar(true)
                        .resizable(false)
                        .fullscreen(true)
                        .build();
                        if let Ok(w) = &overlay {
                            let _ = w.set_focus();
                            let _ = handle.emit("toggle-overlay", url);
                        }
                        if let Err(e) = overlay {
                            eprintln!("zeroflow: overlay create ERR: {e}");
                        }
                    });
                }
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            hotkey::toggle_overlay,
            hotkey::start_overlay,
            hotkey::close_overlay,
            hotkey::copy_image,
            hotkey::save_image,
            hotkey::open_settings,
            hotkey::open_music,
            hotkey::quit_app,
            capture::grab_frame,
            capture::get_pending_frame,
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
