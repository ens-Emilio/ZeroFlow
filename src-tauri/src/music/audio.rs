
//! Audio capture for music recognition — spawns `parec` (PulseAudio/PipeWire compat)
//! recording the default monitor at 48 kHz, then downsamples to 16 kHz mono.
//! No cpal: avoids blocking the tokio runtime and works headless.

use std::io::Read;
use std::process::{Child, Command, Stdio};

fn spawn_parec(duration_secs: u32) -> Result<Child, String> {
    // Record from the default monitor (what is currently playing) for `duration_secs`.
    // PipeWire exposes monitors as sources; @DEFAULT_MONITOR@ resolves correctly.
    let _samples = duration_secs as usize * 48000;
    // parec with --raw --format=s16le --rate=48000 --channels=2 --device=@DEFAULT_MONITOR@
    // writes raw PCM to stdout; we cap bytes to avoid hanging.
    Command::new("parec")
        .args([
            "--raw",
            "--format=s16le",
            "--rate=48000",
            "--channels=2",
            "--device=@DEFAULT_MONITOR@",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("parec spawn: {e} (is pipewire-pulse installed?)"))
}

#[allow(dead_code)]
fn read_pcm(child: &mut Child, max_bytes: usize) -> Result<Vec<i16>, String> {
    let mut out = Vec::new();
    let stdout = child.stdout.take().ok_or("parec no stdout")?;
    let mut limited = stdout.take(max_bytes as u64);
    let mut buf = vec![0u8; 8192];
    loop {
        let n = limited.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 { break; }
        out.extend_from_slice(&buf[..n]);
    }
    // Scale: raw s16le -> Vec<i16>
    let mut samples = Vec::with_capacity(out.len() / 2);
    for chunk in out.chunks_exact(2) {
        samples.push(i16::from_le_bytes([chunk[0], chunk[1]]));
    }
    Ok(samples)
}

/// Capture `duration_secs` seconds of audio from the default monitor,
/// return mono f32 samples at 16 kHz (downsampled from 48 kHz stereo s16le).
pub fn capture_monitor(duration_secs: u32) -> Result<Vec<f32>, String> {
    let max_bytes = duration_secs as usize * 48000 * 2 * 2; // 48k * 2ch * 2bytes
    let mut child = spawn_parec(duration_secs)?;
    // Read in a thread and kill parec after duration_secs so pipe closes.
    let stdout = child.stdout.take();
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut out = Vec::new();
        if let Some(mut s) = stdout {
            let mut buf = vec![0u8; 8192];
            loop {
                match s.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        out.extend_from_slice(&buf[..n]);
                        if out.len() >= max_bytes { break; }
                    }
                }
            }
        }
        let _ = tx.send(out);
    });
    std::thread::sleep(std::time::Duration::from_secs(duration_secs as u64));
    let _ = child.kill();
    let _ = child.wait();
    let out = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap_or_default();
    if out.is_empty() { return Err("no audio captured (is anything playing?)".into()); }
    let mut s16 = Vec::with_capacity(out.len() / 2);
    for c in out.chunks_exact(2) { s16.push(i16::from_le_bytes([c[0], c[1]])); }
    Ok(stereo_s16_48k_to_mono_f32_16k(&s16))
}

fn stereo_s16_48k_to_mono_f32_16k(src: &[i16]) -> Vec<f32> {
    // src is interleaved stereo s16 @ 48 kHz. Average to mono, then downsample 3:1 to 16 kHz.
    let frames = src.len() / 2;
    let mono: Vec<f32> = (0..frames).map(|i| {
        let l = src[i * 2] as f32 / 32768.0;
        let r = src[i * 2 + 1] as f32 / 32768.0;
        (l + r) * 0.5
    }).collect();
    // Simple box downsample 48k -> 16k: average every 3 samples
    let out_len = mono.len() / 3;
    (0..out_len).map(|i| {
        let a = mono[i * 3];
        let b = mono[i * 3 + 1];
        let c = mono[i * 3 + 2];
        (a + b + c) / 3.0
    }).collect()
}

pub fn rms(samples: &[f32]) -> f64 {
    if samples.is_empty() { return 0.0; }
    let sum: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum / samples.len() as f64).sqrt()
}
