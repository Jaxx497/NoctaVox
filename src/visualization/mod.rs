use std::{collections::VecDeque, sync::Arc};

use voxio::{TapHandle, Vox};

mod progress_display;
mod spectrum;
mod waveform;

pub use progress_display::ProgressDisplay;
pub use spectrum::SpectrumState;
pub use waveform::WaveformManager;

pub const TAP_BUFFER_CAPACITY: usize = 2048;

pub struct Visualizer {
    metrics: Arc<Vox>,
    tap: TapHandle,
    display_tap: VecDeque<f32>,
    spectrum: SpectrumState,
    waveform: WaveformManager,
    mode: ProgressDisplay,
}

impl Visualizer {
    pub fn new(metrics: Arc<Vox>, tap: TapHandle) -> Self {
        Visualizer {
            metrics,
            tap,
            display_tap: VecDeque::with_capacity(TAP_BUFFER_CAPACITY),
            spectrum: SpectrumState::default(),
            waveform: WaveformManager::new(),
            mode: ProgressDisplay::Spectrum,
        }
    }

    pub fn spectrum(&self) -> &SpectrumState {
        &self.spectrum
    }

    pub fn spectrum_mut(&mut self) -> &mut SpectrumState {
        &mut self.spectrum
    }

    pub fn display_tap(&mut self) -> &[f32] {
        self.display_tap.make_contiguous()
    }

    pub fn flush_tap(&mut self) {
        self.display_tap.clear();
        self.tap.latest(usize::MAX);
        self.spectrum_mut().reset();
    }

    pub fn fill_tap(&mut self) {
        let channels = self.metrics.channels();

        let latest = self.tap.latest(TAP_BUFFER_CAPACITY * channels);
        for frame in latest.chunks_exact(channels) {
            let mono = frame.iter().copied().sum::<f32>() / channels as f32;
            self.display_tap.push_back(mono);
        }

        let overflow = self.display_tap.len().saturating_sub(TAP_BUFFER_CAPACITY);
        self.display_tap.drain(..overflow);
    }

    pub fn update_spectrum(&mut self) {
        if !self.display_tap.is_empty() {
            let samples = self.display_tap.make_contiguous();
            let sample_rate = self.metrics.sample_rate();
            self.spectrum.update(samples, sample_rate);
        }
    }
}
