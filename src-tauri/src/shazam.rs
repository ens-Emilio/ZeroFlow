// Shazam music recognition — pipe: parec monitor @48k stereo -> 16k mono
// -> ShazamSignatureGenerator -> ShazamSignatureCodec -> amp.shazam.com.
// Port of CTS/MusicRecognition/* (SongRec).

use serde::{Deserialize, Serialize};

// We avoid redefining ShazamRecognition; use crate::music::* where possible.
use crate::music::audio as music_audio;
use crate::music::client as music_client;
use crate::music::fingerprint::ShazamSignatureGenerator;
use crate::music::ShazamLocation;

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
#[allow(dead_code)]
pub struct RecognitionResult {
    pub track: Option<ShazamTrack>,
    pub timestamp: u64,
}

#[tauri::command]
pub async fn start_music_recognition() -> Result<serde_json::Value, String> {
    // Single 12s capture like CTS ProgressiveMusicRecognizer; slice at 4/8/12 for Shazam attempts.
    let full = tokio::task::spawn_blocking(|| music_audio::capture_monitor(12))
        .await
        .map_err(|e| e.to_string())??;
    eprintln!("zeroflow: music capture 12s samples={} rms={:.4}", full.len(), music_audio::rms(&full));
    let mut had_usable = false;
    for secs in [4u32, 8, 12] {
        let need = (secs as usize) * 16000;
        let samples = if full.len() >= need { &full[full.len() - need..] } else { &full[..] };
        let rms = music_audio::rms(samples);
        eprintln!("zeroflow: music attempt {secs}s rms={rms:.4} samples={}", samples.len());
        if rms < 0.0001 { continue; }
        let sig = ShazamSignatureGenerator::generate(samples);
        eprintln!("zeroflow: music fingerprint peaks={} duration_ms={}", sig.peak_count(), sig.duration_ms());
        if sig.peak_count() == 0 { continue; }
        had_usable = true;
        let loc = ShazamLocation::sao_paulo();
        match music_client::recognize(&sig, &loc).await {
            Ok(Some(rec)) => {
                eprintln!("zeroflow: music matched {} - {}", rec.title, rec.artist);
                let track = ShazamTrack {
                    title: rec.title.clone(),
                    artist: rec.artist.clone(),
                    album: rec.album.clone(),
                    cover_url: rec.cover_art_url.clone(),
                    shazam_url: rec.share_url.clone(),
                    apple_music_url: None,
                };
                return Ok(serde_json::json!({
                    "status": "matched",
                    "track": track,
                    "recognition": rec
                }));
            }
            Ok(None) => {
                eprintln!("zeroflow: music no match at {secs}s, trying longer capture");
                continue;
            }
            Err(e) if e.contains("rate-limited") => {
                return Ok(serde_json::json!({ "status": "rate_limited", "error": e }));
            }
            Err(e) => {
                eprintln!("zeroflow: music shazam error at {secs}s: {e}");
                if secs == 12 {
                    return Ok(serde_json::json!({ "status": "service_error", "error": e }));
                }
                continue;
            }
        }
    }
    if had_usable {
        Ok(serde_json::json!({ "status": "no_match" }))
    } else {
        Ok(serde_json::json!({ "status": "no_audio", "error": "no usable audio captured" }))
    }
}

#[tauri::command]
pub async fn stop_music_recognition() -> Result<(), String> { Ok(()) }
