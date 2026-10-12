// Minimal hotkey: X11 via x11rb; Wayland via global-shortcuts-portal (ashpd).
// Tauri ships a global-shortcut plugin that does exactly this (plugin chooses backend).
// We'll just wire the command; the plugin provides the registration.

use tauri::{Manager, Window};

#[tauri::command]
pub fn toggle_overlay(window: Window) {
    if let Ok(overlay) = window.get_window("overlay") {
        // simple toggle: if visible -> hide, else show + grab pointer/focus.
        let _ = if overlay.is_visible().unwrap_or(false) {
            overlay.hide()
        } else {
            overlay.show().and_then(|_| {
                // bring to front, grab input, ensure topmost
                overlay.set_focus().and_then(|_| overlay.set_always_on_top(true))
            })
        };
    }
}
