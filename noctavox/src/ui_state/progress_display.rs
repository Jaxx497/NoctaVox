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
    pub fn display_progress(&self) -> bool {
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
}
