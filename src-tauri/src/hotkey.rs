// Minimal hotkey: X11 via x11rb; Wayland via global-shortcuts-portal (ashpd).
// Tauri ships a global-shortcut plugin that does exactly this (plugin chooses backend).
// We'll just wire the command; the plugin provides the registration.
// In KDE Wayland, KGlobalAccel is used instead; the plugin may fall back silently.
// shortcut: plugin registration via setup() in lib.rs; fallback native KGlobalAccel later.

use tauri::{Emitter, Manager};

use crate::capture;

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
pub async fn toggle_overlay(window: tauri::Window) {
    if let Some(overlay) = window.get_webview_window("overlay") {
        if overlay.is_visible().unwrap_or(false) {
            let _ = overlay.hide();
        } else {
            show_overlay(&window).await;
        }
    }
}

#[tauri::command]
pub async fn start_overlay(window: tauri::Window) {
    show_overlay(&window).await;
}

/// Destroys the overlay window. On this stack hide() leaves a dead handle,
/// so closing is the only reliable way out.
#[tauri::command]
pub fn close_overlay(app: tauri::AppHandle) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.close();
    }
}

pub async fn show_overlay(window: &tauri::Window) {
    if let Some(overlay) = window.get_webview_window("overlay") {
        // Wayland: only show(); window is maximized via config.
        let _ = overlay.show();
        // Grab screenshot and emit to frontend for backdrop.
        // shortcut: interactive=false skips the picker where possible.
        if let Ok(data_url) = capture::grab_frame(false).await {
            let _ = overlay.emit("overlay-frame", data_url);
        }
    }
}
