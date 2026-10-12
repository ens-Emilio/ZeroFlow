// Screen capture via xdg-desktop-portal (ScreenCast interface) + PipeWire.
// ZeroFlow keeps a persistent portal session so the hotkey can capture instantly,
// matching CircleFlow's instant-overlay UX on Windows.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Starts (or reuses) a portal ScreenCast session.
/// TODO(F0): implement with ashpd: request session, select all monitors,
/// persist the PipeWire stream node so `frame()` can capture at any time.
#[tauri::command]
pub async fn start_capture_session() -> Result<(), String> {
    // Phase 0 prototype: validate this works across GNOME/KDE/Hyprland/X11.
    Err("capture session not implemented yet (Phase 0)".into())
}

/// Grabs a single frame from the persistent session and returns PNG bytes (base64).
#[tauri::command]
pub async fn grab_frame() -> Result<String, String> {
    // Phase 0: pipewire-rs or ashpd's screencast stream -> image capture.
    Err("frame grab not implemented yet (Phase 0)".into())
}

/// Crops the latest frame to the lasso polygon.
/// Polygon fill on CPU (imageproc), JPEG encode, base64 out.
#[tauri::command]
pub async fn crop_selection(
    frame_png_base64: String,
    bounds: SelectionBounds,
    polygon: Vec<Point>,
) -> Result<String, String> {
    let _ = (frame_png_base64, bounds, polygon);
    Err("crop_selection not implemented yet (Phase 1)".into())
}
