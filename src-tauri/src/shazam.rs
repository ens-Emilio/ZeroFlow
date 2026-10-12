// Shazam music recognition. Port of CTS/MusicRecognition/Shazam/*.
// The original: HTTP/1.1 only, Pixel user-agent, throttled requests, GPS spoof.
// Audio: PipeWire monitor stream -> mono 16 kHz -> Shazam endpoint.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShazamTrack {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub cover_url: Option<String>,
    pub shazam_url: Option<String>,
    pub apple_music_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionResult {
    pub track: Option<ShazamTrack>,
    pub timestamp: u64,
}

/// Pixel user-agent, matching the original PixelUserAgentProvider.
const USER_AGENT: &str =
    "Dalvik/2.1.0 (Linux; U; Android 13; Pixel 7 Build/TQ3A.230903.001)";

// shortcut: original signs requests with a salted hash; port the exact scheme from
// ShazamClient.cs in Phase 3, when audio capture is ready to test end-to-end.

#[tauri::command]
pub async fn start_music_recognition() -> Result<(), String> {
    // Phase 3: PipeWire monitor -> 16kHz mono -> 8s sample -> POST to Shazam endpoint.
    Err("music recognition not implemented yet (Phase 3)".into())
}

#[tauri::command]
pub async fn stop_music_recognition() -> Result<(), String> {
    Ok(())
}
