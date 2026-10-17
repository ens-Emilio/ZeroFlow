// SPDX-License-Identifier: GPL-3.0-or-later
// Port of CTS/MusicRecognition/Fingerprinting/ShazamSignatureGenerator.cs (SongRec).

use num_complex::Complex32;
use rustfft::FftPlanner;

const FFT_SIZE: usize = 2048;
const FFT_RESULT_SIZE: usize = FFT_SIZE / 2 + 1;
const HOP_SIZE: usize = 128;
const HISTORY_SIZE: usize = 256;

const SPREAD_OFFSETS: [usize; 3] = [1, 3, 6];
const FREQUENCY_OFFSETS: [i32; 8] = [-10, -7, -4, -3, 1, 2, 5, 8];
const TIME_OFFSETS: [i32; 14] = [-53, -45, 165, 172, 179, 186, 193, 200, 214, 221, 228, 235, 242, 249];

fn hanning_window() -> [f32; FFT_SIZE] {
    let mut w = [0.0f32; FFT_SIZE];
    for i in 0..FFT_SIZE {
        w[i] = (0.5 - 0.5 * ((2.0 * std::f64::consts::PI * (i + 1) as f64) / 2049.0).cos()) as f32;
    }
    w
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrequencyBand { Hz250To520 = 0, Hz520To1450 = 1, Hz1450To3500 = 2, Hz3500To5500 = 3 }

#[derive(Debug, Clone)]
pub struct FrequencyPeak { pub fft_pass: u32, pub magnitude: u16, pub bin: u16 }

pub struct ShazamSignature {
    pub number_samples: usize,
    pub peaks_by_band: [Vec<FrequencyPeak>; 4],
}
impl ShazamSignature {
    pub fn new(n: usize) -> Self { Self { number_samples: n, peaks_by_band: [Vec::new(), Vec::new(), Vec::new(), Vec::new()] } }
    pub fn peak_count(&self) -> usize { self.peaks_by_band.iter().map(|v| v.len()).sum() }
    pub fn duration_ms(&self) -> u32 { (self.number_samples as u32 * 1000) / 16000 }
    pub fn sample_rate_hz(&self) -> u32 { 16000 }
}

fn band_for(freq_hz: f32) -> Option<FrequencyBand> {
    match freq_hz {
        f if (250.0..520.0).contains(&f) => Some(FrequencyBand::Hz250To520),
        f if (520.0..1450.0).contains(&f) => Some(FrequencyBand::Hz520To1450),
        f if (1450.0..3500.0).contains(&f) => Some(FrequencyBand::Hz1450To3500),
        f if (3500.0..=5500.0).contains(&f) => Some(FrequencyBand::Hz3500To5500),
        _ => None,
    }
}
fn calc_magnitude(v: f32) -> f32 { v.ln().max(1.0 / 64.0) * 1477.3 + 6144.0 }

pub struct ShazamSignatureGenerator {
    sample_ring: [i16; FFT_SIZE],
    fft_outputs: [[f32; FFT_RESULT_SIZE]; HISTORY_SIZE],
    spread_outputs: [[f32; FFT_RESULT_SIZE]; HISTORY_SIZE],
    fft_buffer: Vec<Complex32>,
    sample_ring_index: usize,
    fft_output_index: usize,
    spread_output_index: usize,
    spread_done: u32,
    signature: ShazamSignature,
}

impl ShazamSignatureGenerator {
    pub fn generate(mono_16k: &[f32]) -> ShazamSignature {
        let mut g = Self {
            sample_ring: [0; FFT_SIZE],
            fft_outputs: [[0.0; FFT_RESULT_SIZE]; HISTORY_SIZE],
            spread_outputs: [[0.0; FFT_RESULT_SIZE]; HISTORY_SIZE],
            fft_buffer: vec![Complex32::new(0.0, 0.0); FFT_SIZE],
            sample_ring_index: 0,
            fft_output_index: 0,
            spread_output_index: 0,
            spread_done: 0,
            signature: ShazamSignature::new(mono_16k.len()),
        };
        let window = hanning_window();
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        let chunks = mono_16k.len() / HOP_SIZE;
        let mut chunk = [0i16; HOP_SIZE];
        for ci in 0..chunks {
            let src = &mono_16k[ci * HOP_SIZE..(ci + 1) * HOP_SIZE];
            for (i, &s) in src.iter().enumerate() {
                chunk[i] = (s * 32768.0).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            }
            g.do_fft(&chunk, &window, fft.as_ref());
            g.do_peak_spreading();
            g.spread_done += 1;
            if g.spread_done >= 46 { g.do_peak_recognition(); }
        }
        g.signature
    }

    fn do_fft(&mut self, chunk: &[i16; HOP_SIZE], window: &[f32; FFT_SIZE], fft: &dyn rustfft::Fft<f32>) {
        for i in 0..HOP_SIZE {
            self.sample_ring[(self.sample_ring_index + i) & (FFT_SIZE - 1)] = chunk[i];
        }
        self.sample_ring_index = (self.sample_ring_index + HOP_SIZE) & (FFT_SIZE - 1);
        for i in 0..FFT_SIZE {
            let s = self.sample_ring[(i + self.sample_ring_index) & (FFT_SIZE - 1)] as f32;
            self.fft_buffer[i] = Complex32::new(s * window[i], 0.0);
        }
        fft.process(&mut self.fft_buffer);
        let out = &mut self.fft_outputs[self.fft_output_index];
        for i in 0..FFT_RESULT_SIZE {
            let v = self.fft_buffer[i];
            let mag = (v.re * v.re + v.im * v.im) / ((1 << 17) as f32);
            out[i] = mag.max(0.0000000001);
        }
        self.fft_output_index = (self.fft_output_index + 1) & (HISTORY_SIZE - 1);
    }

    fn do_peak_spreading(&mut self) {
        let fft_idx = (self.fft_output_index + HISTORY_SIZE - 1) & (HISTORY_SIZE - 1);
        let spread_idx = self.spread_output_index;
        self.spread_outputs[spread_idx].copy_from_slice(&self.fft_outputs[fft_idx]);
        {
            let s = &mut self.spread_outputs[spread_idx];
            for pos in 0..=1022 { s[pos] = s[pos].max(s[pos + 1].max(s[pos + 2])); }
        }
        let copy = self.spread_outputs[spread_idx];
        for pos in 0..FFT_RESULT_SIZE {
            for &off in &SPREAD_OFFSETS {
                let prev = (spread_idx + HISTORY_SIZE - off) & (HISTORY_SIZE - 1);
                let v = copy[pos];
                if v > self.spread_outputs[prev][pos] { self.spread_outputs[prev][pos] = v; }
            }
        }
        self.spread_output_index = (self.spread_output_index + 1) & (HISTORY_SIZE - 1);
    }

    fn do_peak_recognition(&mut self) {
        let fft_m46 = (self.fft_output_index + HISTORY_SIZE - 46) & (HISTORY_SIZE - 1);
        let spread_m49 = (self.spread_output_index + HISTORY_SIZE - 49) & (HISTORY_SIZE - 1);
        let fft = self.fft_outputs[fft_m46];
        let spread = self.spread_outputs[spread_m49];
        for bin in 10..=1014 {
            if fft[bin] < 1.0 / 64.0 || fft[bin] < spread[bin - 1] { continue; }
            let mut max_n: f32 = 0.0;
            for &off in &FREQUENCY_OFFSETS {
                let idx = (bin as i32 + off) as usize;
                if idx < FFT_RESULT_SIZE { max_n = max_n.max(spread[idx]); }
            }
            if fft[bin] <= max_n { continue; }
            for &off in &TIME_OFFSETS {
                let idx = ((self.spread_output_index as i32 + off).rem_euclid(HISTORY_SIZE as i32)) as usize;
                max_n = max_n.max(self.spread_outputs[idx][bin - 1]);
            }
            if fft[bin] <= max_n { continue; }
            let mag = calc_magnitude(fft[bin]);
            let before = calc_magnitude(fft[bin - 1]);
            let after = calc_magnitude(fft[bin + 1]);
            let var1 = mag * 2.0 - before - after;
            if var1 <= 0.0 { continue; }
            let corrected = bin as i32 * 64 + ((after - before) * 32.0 / var1) as i32;
            if !(0..=65535).contains(&corrected) { continue; }
            let bin_u = corrected as u16;
            let freq = bin_u as f32 * (16000.0 / 2.0 / 1024.0 / 64.0);
            let Some(band) = band_for(freq) else { continue; };
            self.signature.peaks_by_band[band as usize].push(FrequencyPeak {
                fft_pass: self.spread_done - 46,
                magnitude: (mag as i32).clamp(0, 65535) as u16,
                bin: bin_u,
            });
        }
    }
}
