use anyhow::{Result, anyhow};
use crossbeam_channel::Receiver;
use std::{sync::Arc, thread};
use voxio::{BinMetric, Waveform, WaveformOptions};

use crate::{
    key_handler::Incrementor,
    library::{SimpleSong, SongDatabase},
    ui_state::UiState,
};

const WF_BIN_LEN: usize = 500;
const SMOOTHNESS_STEP: f32 = 0.5;

pub struct WaveformManager {
    bins: Option<Vec<f32>>,
    smoothed_view: Vec<f32>,
    smoothing_factor: f32,
    reciever: Option<Receiver<Result<Waveform>>>,
}

impl WaveformManager {
    pub fn new() -> Self {
        WaveformManager {
            bins: None,
            smoothed_view: Vec::with_capacity(WF_BIN_LEN),
            smoothing_factor: 1.0,
            reciever: None,
        }
    }

    pub fn request(&mut self, song: &SimpleSong) {
        // If a waveform exists in db, use it
        if let Ok(cached) = song.get_waveform_db() {
            self.bins = Some(cached);
            self.apply_smoothing();
            return;
        }

        if let Ok(path) = song.get_path() {
            let (tx, rx) = crossbeam_channel::bounded(1);

            thread::spawn(move || {
                if let Ok(mut wf) = Waveform::generate(
                    &path,
                    &WaveformOptions {
                        bins: WF_BIN_LEN,
                        metric: BinMetric::Rms,
                        highpass_hz: Some(350.0),
                        treble_db: 9.0,
                    },
                ) {
                    wf.local_normalize(0.8, 20);
                    wf.normalize();
                    wf.contrast(1.5);
                    let _ = tx.send(Ok(wf));
                } else {
                    let _ = tx.send(Err(anyhow!("Invalid waveform")));
                }
            });

            self.reciever = Some(rx)
        }
    }

    pub fn reciever(&self) -> Option<&Receiver<Result<Waveform>>> {
        self.reciever.as_ref()
    }

    pub fn complete(&mut self, result: Result<Waveform>, song: Option<&Arc<SimpleSong>>) {
        match result {
            Ok(waveform) => {
                if let Some(s) = song {
                    let _ = s.set_waveform_db(&waveform.bins);
                    let _ = s.update_duration_db(waveform.duration);
                }
                self.bins = Some(waveform.bins);
                self.apply_smoothing();
            }
            Err(_) => self.bins = None,
        }
        self.reciever = None;
    }
}

impl WaveformManager {
    pub fn clear(&mut self) {
        self.reciever = None;
        self.smoothed_view.clear();
        self.bins = None;
    }

    pub fn apply_smoothing(&mut self) {
        if let Some(bins) = &mut self.bins {
            self.smoothed_view = smooth_waveform(bins, self.smoothing_factor);
        }
    }

    pub fn increment_smoothness(&mut self, direction: Incrementor) {
        match direction {
            Incrementor::Up => {
                if self.smoothing_factor < 3.9 {
                    self.smoothing_factor += SMOOTHNESS_STEP;
                    self.apply_smoothing();
                }
            }
            Incrementor::Down => {
                if self.smoothing_factor > 0.1 {
                    self.smoothing_factor -= SMOOTHNESS_STEP;
                    self.apply_smoothing();
                }
            }
        }
    }

    fn get_waveform_visual(&self) -> &[f32] {
        self.smoothed_view.as_slice()
    }
}

impl UiState {
    pub fn request_waveform(&mut self, song: &SimpleSong) {
        self.waveform.request(song);
    }

    pub fn handle_wf_result(&mut self, result: Result<Waveform>, song: Option<&Arc<SimpleSong>>) {
        self.waveform.complete(result, song);
    }

    pub fn wf_reciever(&self) -> Option<&Receiver<Result<Waveform>>> {
        self.waveform.reciever()
    }

    pub fn clear_waveform(&mut self) {
        self.waveform.clear();
    }

    pub fn waveform_is_valid(&self) -> bool {
        self.waveform.bins.is_some()
    }

    pub fn get_waveform_as_slice(&self) -> &[f32] {
        self.waveform.get_waveform_visual()
    }

    pub fn get_smoothing_factor(&self) -> f32 {
        self.waveform.smoothing_factor
    }

    pub fn set_smoothing_factor(&mut self, sf: f32) {
        self.waveform.smoothing_factor = sf
    }

    pub fn increment_wf_smoothness(&mut self, direction: Incrementor) {
        self.waveform.increment_smoothness(direction);
    }
}

/// Apply a smoothing filter to the waveform with float smoothing factor
pub fn smooth_waveform(waveform: &[f32], smoothing_factor: f32) -> Vec<f32> {
    if waveform.len() <= (smoothing_factor.ceil() as usize * 2 + 1) {
        return waveform.to_vec();
    }

    let range = smoothing_factor.ceil() as isize;

    waveform
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let mut sum = 0.0;
            let mut total_weight = 0.0;

            // Calculate weighted average of surrounding points
            for offset in -range..=range {
                let idx = i as isize + offset;
                if idx >= 0 && idx < waveform.len() as isize {
                    // Weight calculation - based on distance and the smoothing factor
                    // Points beyond the float smoothing factor get reduced weight
                    let distance = offset.abs() as f32;
                    let weight = if distance <= smoothing_factor {
                        // Full weight for points within the smooth factor
                        1.0
                    } else {
                        // Partial weight for the fractional part
                        1.0 - (distance - smoothing_factor)
                    };

                    if weight > 0.0 {
                        sum += waveform[idx as usize] * weight;
                        total_weight += weight;
                    }
                }
            }

            if total_weight > 0.0 {
                sum / total_weight
            } else {
                waveform[i]
            }
        })
        .collect()
}
