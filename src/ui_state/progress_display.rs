use crate::{FFMPEG_AVAILABLE, TAP_BUFFER_CAPACITY, player::PlaybackState, ui_state::UiState};

#[derive(Clone, Default, PartialEq, Eq)]
pub enum ProgressDisplay {
    Waveform,
    Oscilloscope,
    Spectrum,
    #[default]
    ProgressBar,
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
            "spectrum" => Self::Spectrum,
            "waveform" => Self::Waveform,
            "oscilloscope" => Self::Oscilloscope,
            _ => Self::ProgressBar,
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
        self.metrics.get_state() != PlaybackState::Stopped || !self.queue_is_empty()
    }

    pub fn get_progress_display(&self) -> &ProgressDisplay {
        &self.progress_display
    }

    pub fn set_progress_display(&mut self, display: ProgressDisplay) {
        self.progress_display = match display {
            ProgressDisplay::Waveform => match *FFMPEG_AVAILABLE {
                true => display,
                false => ProgressDisplay::default(),
            },
            _ => display,
        }
    }

    pub fn fill_tap(&mut self) {
        self.metrics
            .drain_into(&mut self.sample_tap, TAP_BUFFER_CAPACITY);
    }

    pub fn update_spectrum(&mut self) {
        if self.sample_tap.is_empty() {
            return;
        } else {
            let samples = self.sample_tap.make_contiguous();
            let channels = self.metrics.channels();
            let sample_rate = self.metrics.sample_rate();
            self.spectrum.update(samples, channels, sample_rate);
        }
    }
}
