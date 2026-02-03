use crate::{FFMPEG_AVAILABLE, OSCILLO_BUFFER_CAPACITY, player::PlaybackState, ui_state::UiState};

#[derive(Clone, Default, PartialEq, Eq)]
pub enum ProgressDisplay {
    Waveform,
    Oscilloscope,
    #[default]
    ProgressBar,
}

impl ProgressDisplay {
    pub fn from_str(s: &str) -> Self {
        match s {
            "oscilloscope" => Self::Oscilloscope,
            "waveform" => Self::Waveform,
            _ => Self::ProgressBar,
        }
    }
}

impl std::fmt::Display for ProgressDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgressDisplay::Waveform => write!(f, "waveform"),
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

    pub fn fill_oscillo(&mut self) {
        self.metrics
            .drain_into(&mut self.oscillo, OSCILLO_BUFFER_CAPACITY);
    }

    pub fn get_oscillo_samples(&self) -> Vec<f32> {
        Vec::from(self.oscillo.clone())
    }
}
