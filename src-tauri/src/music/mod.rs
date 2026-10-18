// SPDX-License-Identifier: GPL-3.0-or-later
// Port of the SongRec-based Shazam flow from reference/CTS/MusicRecognition.

pub mod audio;
pub mod client;
pub mod codec;
pub mod fingerprint;
pub mod user_agent;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ShazamRecognition {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub track_key: Option<String>,
    pub cover_art_url: Option<String>,
    pub share_url: Option<String>,
}

/// Simulated geolocation matching the original ShazamLocationProvider.
pub struct ShazamLocation {
    pub altitude: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
}

impl ShazamLocation {
    pub fn sao_paulo() -> Self {
        Self {
            altitude: 760.0,
            latitude: -23.5505,
            longitude: -46.6333,
            timezone: "America/Sao_Paulo".into(),
        }
    }
}
