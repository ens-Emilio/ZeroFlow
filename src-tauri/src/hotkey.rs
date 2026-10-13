// Minimal hotkey: X11 via x11rb; Wayland via global-shortcuts-portal (ashpd).
// Tauri ships a global-shortcut plugin that does exactly this (plugin chooses backend).
// We'll just wire the command; the plugin provides the registration.
// In KDE Wayland, KGlobalAccel is used instead; the plugin may fall back silently.
// shortcut: plugin registration via setup() in lib.rs; fallback native KGlobalAccel later.

use tauri::{Emitter, Manager};

use crate::capture;

/// Copies a JPEG (base64) to the system clipboard via wl-copy (Wayland).
#[tauri::command]
pub fn copy_image(jpeg_base64: String) -> Result<bool, String> {
    use base64::Engine;
    use std::io::Write;
    use std::process::{Command, Stdio};
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(jpeg_base64.trim())
        .map_err(|e| format!("base64 decode: {e}"))?;
    let child = Command::new("wl-copy")
        .arg("-t")
        .arg("image/jpeg")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("wl-copy spawn: {e} (is wl-clipboard installed?)"))?;
    {
        let mut stdin = child.stdin.unwrap();
        stdin.write_all(&bytes).map_err(|e| format!("wl-copy write: {e}"))?;
    }
    Ok(true)
}

/// Saves a JPEG to a user-chosen path (native file dialog) and returns the path.
#[tauri::command]
pub async fn save_image(jpeg_base64: String) -> Result<Option<String>, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(jpeg_base64.trim())
        .map_err(|e| format!("base64 decode: {e}"))?;
    let Some(path) = rfd::AsyncFileDialog::new()
        .set_file_name("zeroflow-selection.jpg")
        .add_filter("JPEG image", &["jpg", "jpeg"])
        .save_file()
        .await
    else {
        return Ok(None);
    };
    let path = path.path().to_path_buf();
    std::fs::write(&path, &bytes).map_err(|e| format!("write {path:?}: {e}"))?;
    Ok(Some(path.display().to_string()))
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

/// Frontend telemetry: console.log does not reach stdout on WebKitGTK+KWin.
#[tauri::command]
pub fn debug_log(message: String) {
    eprintln!("[zf-fe] {message}");
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
