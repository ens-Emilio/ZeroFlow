// Port of ShazamClient.cs (SongRec) — amp.shazam.com v5 endpoint, HTTP/1.1 only.
use super::{ShazamLocation, ShazamRecognition};
use super::fingerprint::ShazamSignature;
use super::codec::encode_to_uri;

const SHAZAM_URL: &str = "https://amp.shazam.com/discovery/v5/en/US/android/-/tag";

fn random_ids() -> (String, String) {
    (uuid::Uuid::new_v4().to_string().to_uppercase(), uuid::Uuid::new_v4().to_string())
}

pub async fn recognize(sig: &ShazamSignature, location: &ShazamLocation) -> Result<Option<ShazamRecognition>, String> {
    let (id1, id2) = random_ids();
    let url = format!("{}/{}/{}?sync=true&webv3=true&sampling=true&connected=&shazamapiversion=v3&sharehub=true&video=v3", SHAZAM_URL, id1, id2);
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
    let ua = super::user_agent::pick();
    let body = serde_json::json!({
        "geolocation": { "altitude": location.altitude, "latitude": location.latitude, "longitude": location.longitude },
        "signature": { "samplems": sig.duration_ms(), "timestamp": ts, "uri": encode_to_uri(sig) },
        "timestamp": ts,
        "timezone": location.timezone
    });
    // Force HTTP/1.1 (Shazam requires it)
    let client = reqwest::Client::builder().http1_only().timeout(std::time::Duration::from_secs(8)).connect_timeout(std::time::Duration::from_secs(5)).build().map_err(|e| e.to_string())?;
    let resp = client.post(&url)
        .header("User-Agent", ua)
        .header("Content-Language", "en_US")
        .json(&body)
        .send().await.map_err(|e| e.to_string())?;
    if resp.status() == 429 {
        let retry_after = resp.headers().get("retry-after").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
        return Err(format!("rate-limited{}", retry_after.map(|v| format!(" retry-after={v}")).unwrap_or_default()));
    }
    if !resp.status().is_success() { return Err(format!("shazam http {}", resp.status())); }
    let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let Some(track) = v.get("track") else { return Ok(None); };
    let title = track.get("title").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let artist = track.get("subtitle").and_then(|x| x.as_str()).unwrap_or("").to_string();
    if title.is_empty() || artist.is_empty() { return Ok(None); }
    let cover = track.get("images").and_then(|x| x.get("coverart")).and_then(|x| x.as_str()).map(|s| s.to_string());
    let share = track.get("share").and_then(|x| x.get("href")).and_then(|x| x.as_str()).map(|s| s.to_string());
    let track_key = track.get("key").and_then(|x| x.as_str()).map(|s| s.to_string());
    // album is inside sections[].metadata where title=="Album"
    let mut album: Option<String> = None;
    if let Some(sections) = track.get("sections").and_then(|x| x.as_array()) {
        for sec in sections {
            if sec.get("type").and_then(|x| x.as_str()) != Some("SONG") { continue; }
            if let Some(meta) = sec.get("metadata").and_then(|x| x.as_array()) {
                for item in meta { if item.get("title").and_then(|x| x.as_str()) == Some("Album") { album = item.get("text").and_then(|x| x.as_str()).map(|s| s.to_string()); break; } }
            }
        }
    }
    let genre = track.get("genres").and_then(|x| x.get("primary")).and_then(|x| x.as_str()).map(|s| s.to_string());
    Ok(Some(ShazamRecognition { title, artist, album, genre, track_key, cover_art_url: cover, share_url: share }))
}
