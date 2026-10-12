// Tray + notifications. Uses tray-icon (StatusNotifierItem) and notify-rust.

use notify_rust::Notification;
use tauri::{Manager, Window};

#[tauri::command]
pub fn show_toast(message: String, is_error: bool) -> Result<(), String> {
    let mut n = Notification::new();
    n.summary("ZeroFlow")
        .body(&message)
        .icon(if is_error { "dialog-error" } else { "dialog-information" });
    if let Err(e) = n.show() {
        return Err(e.to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn open_settings(window: Window) {
    if let Ok(settings) = window.get_window("settings") {
        let _ = settings.show();
        let _ = settings.set_focus();
    }
}

#[tauri::command]
pub fn open_music(window: Window) {
    if let Ok(music) = window.get_window("music") {
        let _ = music.show();
        let _ = music.set_focus();
    }
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}