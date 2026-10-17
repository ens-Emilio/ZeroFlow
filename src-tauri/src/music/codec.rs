// SPDX-License-Identifier: GPL-3.0-or-later
// Port of ShazamSignatureCodec.cs (SongRec).

use base64::Engine;
use super::fingerprint::ShazamSignature;

const MAGIC1: u32 = 0xCAFE2580;
const MAGIC2: u32 = 0x94119C00;
const CONTENTS_TAG: u32 = 0x40000000;
const FREQ_BAND_TAG: u32 = 0x60030040;
const HEADER_SIZE: usize = 48;
const DATA_URI_PREFIX: &str = "data:audio/vnd.shazam.sig;base64,";

pub fn encode_to_uri(sig: &ShazamSignature) -> String {
    format!("{}{}", DATA_URI_PREFIX, base64::engine::general_purpose::STANDARD.encode(encode_to_binary(sig)))
}

pub fn encode_to_binary(sig: &ShazamSignature) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&MAGIC1.to_le_bytes()); buf.extend_from_slice(&0u32.to_le_bytes()); buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&MAGIC2.to_le_bytes()); buf.extend_from_slice(&0u32.to_le_bytes()); buf.extend_from_slice(&0u32.to_le_bytes()); buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&(3u32 << 27).to_le_bytes()); buf.extend_from_slice(&0u32.to_le_bytes()); buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&((sig.number_samples as f32 + sig.sample_rate_hz() as f32 * 0.24) as u32).to_le_bytes());
    buf.extend_from_slice(&((15u32 << 19) + 0x40000u32).to_le_bytes());
    buf.extend_from_slice(&CONTENTS_TAG.to_le_bytes()); buf.extend_from_slice(&0u32.to_le_bytes());
    for band in 0..4 {
        let peaks = &sig.peaks_by_band[band];
        if peaks.is_empty() { continue; }
        let mut pb = Vec::new();
        let mut prev: u32 = 0;
        for p in peaks {
            if p.fft_pass < prev { panic!("peaks must be ordered"); }
            let delta = p.fft_pass - prev;
            if delta >= 255 { pb.push(255u8); pb.extend_from_slice(&p.fft_pass.to_le_bytes()); prev = p.fft_pass; }
            let d2 = (p.fft_pass - prev) as u8;
            pb.push(d2);
            pb.extend_from_slice(&p.magnitude.to_le_bytes());
            pb.extend_from_slice(&p.bin.to_le_bytes());
            prev = p.fft_pass;
        }
        buf.extend_from_slice(&(FREQ_BAND_TAG + band as u32).to_le_bytes());
        buf.extend_from_slice(&(pb.len() as u32).to_le_bytes());
        buf.extend_from_slice(&pb);
        while buf.len() % 4 != 0 { buf.push(0); }
    }
    let size_without_header = (buf.len() - HEADER_SIZE) as u32;
    buf[8..12].copy_from_slice(&size_without_header.to_le_bytes());
    buf[HEADER_SIZE + 4..HEADER_SIZE + 8].copy_from_slice(&size_without_header.to_le_bytes());
    let crc = crc32fast::hash(&buf[8..]);
    buf[4..8].copy_from_slice(&crc.to_le_bytes());
    buf
}
