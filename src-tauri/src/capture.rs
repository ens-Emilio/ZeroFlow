// Screen capture for Phase 0: xdg-desktop-portal Screenshot (one-shot).
// shortcut: no persistent PipeWire session yet; portal screenshot works on every
// DE and gives us the frozen-frame backdrop CircleFlow uses. Swap for a persistent
// ScreenCast session if the consent prompt/latency proves annoying.

use base64::Engine;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

static LAST_FRAME: Mutex<Option<DynamicImage>> = Mutex::new(None);

/// Takes a full-screen screenshot via the portal and returns it as a PNG data URL.
/// The frame is also cached in memory for crop_selection.
#[tauri::command]
pub async fn grab_frame(interactive: bool) -> Result<String, String> {
    let proxy = ashpd::desktop::screenshot::ScreenshotProxy::new()
        .await
        .map_err(|e| format!("portal connect failed: {e}"))?;
    let response = proxy
        .screenshot(None, ashpd::desktop::screenshot::ScreenshotOptions::default().set_modal(false).set_interactive(interactive))
        .await
        .map_err(|e| format!("portal screenshot failed: {e}"))?
        .response()
        .map_err(|e| format!("portal screenshot failed: {e}"))?;

    let uri = response.uri().to_string();
    let path = uri.strip_prefix("file://").unwrap_or(&uri).to_string();

    let bytes = std::fs::read(&path).map_err(|e| format!("read screenshot {path}: {e}"))?;
    let img = image::load_from_memory(&bytes).map_err(|e| format!("decode screenshot: {e}"))?;

    let mut png = Vec::new();
    img.to_rgba8()
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|e| format!("encode png: {e}"))?;

    *LAST_FRAME.lock().unwrap() = Some(img);

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
    Ok(format!("data:image/png;base64,{b64}"))
}

/// Crops the cached frame to the lasso polygon (keeps pixels inside the polygon),
/// JPEG-encodes, returns base64 (no data: prefix).
#[tauri::command]
pub async fn crop_selection(polygon: Vec<Point>) -> Result<String, String> {
    let guard = LAST_FRAME.lock().unwrap();
    let frame = guard.as_ref().ok_or("no frame grabbed yet")?;

    let src = frame.to_rgba8();
    let (w, h) = src.dimensions();
    let mut out = image::RgbaImage::new(w, h);
    for (x, y, pixel) in src.enumerate_pixels() {
        if point_in_polygon(x as f64 + 0.5, y as f64 + 0.5, &polygon) {
            out.put_pixel(x, y, *pixel);
        }
    }

    let mut jpeg = Vec::new();
    DynamicImage::ImageRgba8(out)
        .to_rgb8()
        .write_to(&mut std::io::Cursor::new(&mut jpeg), image::ImageFormat::Jpeg)
        .map_err(|e| format!("encode jpeg: {e}"))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&jpeg))
}

fn point_in_polygon(px: f64, py: f64, polygon: &[Point]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let a = &polygon[i];
        let b = &polygon[j];
        let intersects =
            (a.y > py) != (b.y > py) && px < (b.x - a.x) * (py - a.y) / (b.y - a.y) + a.x;
        if intersects {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_in_polygon_square() {
        let square = [
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
            Point { x: 0.0, y: 10.0 },
        ];
        assert!(point_in_polygon(5.0, 5.0, &square));
        assert!(!point_in_polygon(15.0, 5.0, &square));
        assert!(!point_in_polygon(0.0, 0.0, &[]));
    }
}
