use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum, windows::hann_window};

use crate::TAP_BUFFER_CAPACITY;

pub struct SpectrumState {
    pub bins: Vec<f32>,
    pub display_bins: Vec<f32>,
    pub decay_factor: f32,
    bands: Vec<(f32, f32)>,
    band_peaks: Vec<f32>,
    sample_rate: u32,
    last_display_width: usize,
    bins_dirty: bool,
}

impl SpectrumState {
    pub fn update(&mut self, samples: &[f32], channels: u8, sample_rate: u32) {
        if channels == 0 || sample_rate == 0 {
            return;
        }

        let fft_size = TAP_BUFFER_CAPACITY / channels as usize;

        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            let freq_resolution = sample_rate as f32 / fft_size as f32;
            self.bands.clear();
            let mut freq = 20.0_f32;
            while freq < 20000.0 {
                let next = (freq * 1.05).max(freq + freq_resolution);
                self.bands.push((freq, next.min(20000.0)));
                freq = next;
            }
            let n = self.bands.len();
            self.band_peaks.resize(n, 1e-3);
            self.bins.resize(n, 0.0);
        }

        let mono: Vec<f32> = samples
            .chunks_exact(channels as usize)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect();

        if mono.len() < fft_size {
            for bin in self.bins.iter_mut() {
                *bin *= self.decay_factor;
            }
            return;
        }

        let start = mono.len() - fft_size;
        let windowed = hann_window(&mono[start..]);

        let spectrum = match samples_fft_to_spectrum(
            &windowed,
            self.sample_rate,
            FrequencyLimit::Range(20.0, 20000.0),
            None,
        ) {
            Ok(s) => s,
            Err(_) => {
                for bin in self.bins.iter_mut() {
                    *bin *= self.decay_factor;
                }
                return;
            }
        };

        let mut data_iter = spectrum.data().iter().peekable();

        for i in 0..self.bands.len() {
            let (lo, hi) = self.bands[i];
            let mut sum = 0.0_f32;
            let mut count = 0_usize;

            while let Some(&(f, m)) = data_iter.peek() {
                let freq_val = f.val();
                if freq_val < lo {
                    data_iter.next();
                } else if freq_val < hi {
                    sum += m.val();
                    count += 1;
                    data_iter.next();
                } else {
                    break;
                }
            }

            let mag = if count > 0 { sum / count as f32 } else { 0.0 };
            let normalized = mag / (fft_size as f32 / 2.0);

            // Per-band auto-gain: instant attack, slow release
            if normalized > self.band_peaks[i] {
                self.band_peaks[i] = normalized;
            } else {
                self.band_peaks[i] = (self.band_peaks[i] * 0.99).max(1e-3);
            }

            let relative = (normalized / self.band_peaks[i]).clamp(0.0, 1.0);

            if relative > self.bins[i] {
                self.bins[i] = relative;
            } else {
                self.bins[i] *= self.decay_factor;
            }
        }

        self.bins_dirty = true;
    }

    pub fn decay(&mut self) {
        for bin in self.bins.iter_mut() {
            *bin *= 0.92;
        }
        self.bins_dirty = true;
    }

    pub fn remap_display(&mut self, width: usize) {
        if self.bins.is_empty() || (!self.bins_dirty && self.last_display_width == width) {
            return;
        }
        let num_bins = self.bins.len();
        self.display_bins = (0..width)
            .map(|i| {
                let t = i as f32 / (width - 1).max(1) as f32;
                let src = t * (num_bins - 1) as f32;
                let lo = src.floor() as usize;
                let hi = (lo + 1).min(num_bins - 1);
                let frac = src - lo as f32;
                self.bins[lo] * (1.0 - frac) + self.bins[hi] * frac
            })
            .collect();
        self.last_display_width = width;
        self.bins_dirty = false;
    }
}

impl Default for SpectrumState {
    fn default() -> Self {
        SpectrumState {
            bins: Vec::new(),
            display_bins: Vec::new(),
            band_peaks: Vec::new(),
            bands: Vec::new(),
            decay_factor: 0.85,
            sample_rate: 0,
            last_display_width: 0,
            bins_dirty: false,
        }
    }
}
