// Shazam music recognition — detached + guards so it never wedges the compositor.
// Port of reference/CTS/MusicRecognition/* (SongRec).

use serde::{Deserialize, Serialize};
use crate::music::audio as music_audio;
use crate::music::client as music_client;
use crate::music::fingerprint::ShazamSignatureGenerator;
use crate::music::ShazamLocation;
use crate::music_history::{push_history, MusicHistoryEntry};

use std::sync::atomic::{AtomicBool, Ordering};

static MUSIC_CANCELLED: AtomicBool = AtomicBool::new(false);

async fn wait_for_cancellation() {
    while !MUSIC_CANCELLED.load(Ordering::Relaxed) {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

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
pub async fn start_music_recognition(app: tauri::AppHandle) -> Result<(), String> {
    MUSIC_CANCELLED.store(false, Ordering::Relaxed);
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        use tauri::Emitter;
        let inner = async {
            let full = tokio::task::spawn_blocking(|| music_audio::capture_monitor(12, &MUSIC_CANCELLED))
                .await.map_err(|e| e.to_string())??;
            eprintln!("zeroflow: music capture 12s samples={} rms={:.4}", full.len(), music_audio::rms(&full));
            let mut had_usable = false;
            for secs in [4u32, 8, 12] {
                let need = (secs as usize) * 16000;
                let samples: Vec<f32> = if full.len() >= need { full[full.len() - need..].to_vec() } else { full.clone() };
                let rms = music_audio::rms(&samples);
                eprintln!("zeroflow: music attempt {secs}s rms={rms:.4} samples={}", samples.len());
                if rms < 0.0001 { continue; }
                if MUSIC_CANCELLED.load(Ordering::Relaxed) { return Err("music recognition cancelled".into()); }
                let sig = tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    tokio::task::spawn_blocking({
                        let samples = samples.clone();
                        move || ShazamSignatureGenerator::generate(&samples)
                    }),
                ).await.map_err(|_| "fingerprint timed out (10s)".to_string())?.map_err(|e| e.to_string())?;
                if MUSIC_CANCELLED.load(Ordering::Relaxed) { return Err("music recognition cancelled".into()); }
                eprintln!("zeroflow: music fingerprint peaks={} duration_ms={}", sig.peak_count(), sig.duration_ms());
                if sig.peak_count() == 0 { continue; }
                had_usable = true;
                let loc = ShazamLocation::sao_paulo();
                let rec = match tokio::select! {
                    _ = wait_for_cancellation() => return Err("music recognition cancelled".into()),
                    result = tokio::time::timeout(std::time::Duration::from_secs(8), music_client::recognize(&sig, &loc)) => result,
                } {
                    Ok(r) => r,
                    Err(_) => { eprintln!("zeroflow: music shazam network timeout at {secs}s"); if secs == 12 { return Ok(serde_json::json!({ "status": "service_error", "error": "Shazam network timeout" })); } else { continue; } }
                };
                match rec {
                    Ok(Some(rec)) => {
                        eprintln!("zeroflow: music matched {} - {}", rec.title, rec.artist);
                        if crate::settings::load().save_music_history {
                            push_history(MusicHistoryEntry { title: rec.title.clone(), artist: rec.artist.clone(), album: rec.album.clone(), cover_url: rec.cover_art_url.clone(), shazam_url: rec.share_url.clone(), timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() });
                        }
                        let track = ShazamTrack { title: rec.title.clone(), artist: rec.artist.clone(), album: rec.album.clone(), cover_url: rec.cover_art_url.clone(), shazam_url: rec.share_url.clone(), apple_music_url: None };
                        return Ok(serde_json::json!({ "status": "matched", "track": track, "recognition": rec }));
                    }
                    Ok(None) => { eprintln!("zeroflow: music no match at {secs}s, trying longer capture"); continue; }
                    Err(e) if e.contains("rate-limited") => return Ok(serde_json::json!({ "status": "rate_limited", "error": e })),
                    Err(e) => { eprintln!("zeroflow: music shazam error at {secs}s: {e}"); if secs == 12 { return Ok(serde_json::json!({ "status": "service_error", "error": e })); } continue; }
                }
            }
            if !had_usable { return Ok(serde_json::json!({ "status": "no_audio", "error": "silence (is music playing on the monitor?)" })); }
            Ok(serde_json::json!({ "status": "no_match", "error": "no Shazam match — try louder / closer to speaker" }))
        };
        let res: Result<serde_json::Value, String> = match tokio::time::timeout(std::time::Duration::from_secs(35), inner).await {
            Ok(r) => r,
            Err(_) => Err("music overall timeout (35s) — try again".into()),
        };
        let payload = match res {
            Ok(v) => v,
            Err(e) if e == "music recognition cancelled" => serde_json::json!({ "status": "cancelled" }),
            Err(e) => serde_json::json!({ "status": "service_error", "error": e }),
        };
        eprintln!("zeroflow: music done -> {}", payload);
        let _ = app2.emit("zeroflow://music-result", payload);
    });
    Ok(())
}

#[tauri::command]
pub async fn stop_music_recognition() -> Result<(), String> {
    MUSIC_CANCELLED.store(true, Ordering::Relaxed);
    Ok(())
}
