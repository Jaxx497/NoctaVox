use crate::{TAP_BUFFER_CAPACITY, ui_state::UiState};

#[derive(Default)]
pub enum ProgressDisplay {
    Waveform,
    Oscilloscope,
    ProgressBar,
    #[default]
    Spectrum,
}

impl ProgressDisplay {
    pub fn next(&self) -> Self {
        match self {
            Self::ProgressBar => Self::Waveform,
            Self::Waveform => Self::Oscilloscope,
            Self::Oscilloscope => Self::Spectrum,
            Self::Spectrum => Self::ProgressBar,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "waveform" => Self::Waveform,
            "oscilloscope" => Self::Oscilloscope,
            "progress_bar" => Self::ProgressBar,
            _ => Self::Spectrum,
        }
    }
}

impl std::fmt::Display for ProgressDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgressDisplay::Waveform => write!(f, "waveform"),
            ProgressDisplay::Spectrum => write!(f, "spectrum"),
            ProgressDisplay::ProgressBar => write!(f, "progress_bar"),
            ProgressDisplay::Oscilloscope => write!(f, "oscilloscope"),
        }
    }
}

impl UiState {
    pub fn next_progress_display(&mut self) {
        self.progress_display = self.progress_display.next()
    }

    pub fn is_progress_display(&self) -> bool {
        self.metrics.is_active() || !self.queue_is_empty()
    }

    pub fn get_progress_display(&self) -> &ProgressDisplay {
        &self.progress_display
    }

    pub fn set_progress_display(&mut self, display: ProgressDisplay) {
        self.progress_display = display
    }

    pub fn fill_tap(&mut self) {
        let channels = self.metrics.channels();

        let latest = self.tap.latest(TAP_BUFFER_CAPACITY * channels);
        for frame in latest.chunks_exact(channels) {
            let mono = frame.iter().copied().sum::<f32>() / channels as f32;
            self.sample_tap.push_back(mono);
        }

        let overflow = self.sample_tap.len().saturating_sub(TAP_BUFFER_CAPACITY);
        self.sample_tap.drain(..overflow);
    }

    pub fn update_spectrum(&mut self) {
        if self.sample_tap.is_empty() {
            return;
        } else {
            let samples = self.sample_tap.make_contiguous();
            let sample_rate = self.metrics.sample_rate();
            self.spectrum.update(samples, sample_rate);
        }
    }
}
