// Tray + notifications. Uses tray-icon (StatusNotifierItem) and notify-rust.
// The tray itself is built in main (Phase 1); for now only the toast command lives here.

use notify_rust::Notification;

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
