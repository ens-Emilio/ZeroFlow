// Minimal hotkey: X11 via x11rb; Wayland via global-shortcuts-portal (ashpd).
// Tauri ships a global-shortcut plugin that does exactly this (plugin chooses backend).
// We'll just wire the command; the plugin provides the registration.

use tauri::Manager;

/// Copies a JPEG (base64) to the system clipboard via the xdg clipboard (wl-copy / xclip fallback).
#[tauri::command]
pub fn copy_image(jpeg_base64: String) -> Result<bool, String> {
    let _ = jpeg_base64;
    // Phase 1: pipe to wl-copy (Wayland) or xclip (X11).
    Err("clipboard image copy not implemented yet (Phase 1)".into())
}

/// Saves a JPEG to a user-chosen path (native file dialog) and returns the path.
#[tauri::command]
pub fn save_image(jpeg_base64: String) -> Result<Option<String>, String> {
    let _ = jpeg_base64;
    // Phase 1: rfd file dialog -> fs write.
    Err("save image not implemented yet (Phase 1)".into())
}

#[tauri::command]
pub fn open_settings(window: tauri::Window) {
    if let Some(settings) = window.get_webview_window("settings") {
        let _ = settings.show();
        let _ = settings.set_focus();
    }
}

#[tauri::command]
pub fn open_music(window: tauri::Window) {
    if let Some(music) = window.get_webview_window("music") {
        let _ = music.show();
        let _ = music.set_focus();
    }
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

// Shows/hides the overlay window. Shared by hotkey toggle and frontend Esc handling.
#[tauri::command]
pub fn toggle_overlay(window: tauri::Window) {
    if let Some(overlay) = window.get_webview_window("overlay") {
        if overlay.is_visible().unwrap_or(false) {
            let _ = overlay.hide();
        } else {
            show_overlay(&window);
        }
    }
}

#[tauri::command]
pub fn start_overlay(window: tauri::Window) {
    show_overlay(&window);
}

#[tauri::command]
pub fn hide_overlay(window: tauri::Window) {
    if let Some(overlay) = window.get_webview_window("overlay") {
        let _ = overlay.hide();
    }
}

fn show_overlay(window: &tauri::Window) {
    if let Some(overlay) = window.get_webview_window("overlay") {
        let _ = overlay
            .show()
            .and_then(|_| overlay.set_always_on_top(true))
            .and_then(|_| overlay.set_focus());
    }
}
