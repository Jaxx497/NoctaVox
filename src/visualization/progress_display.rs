use crate::visualization::Visualizer;

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

impl Visualizer {
    pub fn next_progress_display(&mut self) {
        self.mode = self.mode.next()
    }

    pub fn get_progress_display(&self) -> &ProgressDisplay {
        &self.mode
    }

    pub fn set_progress_display(&mut self, display: ProgressDisplay) {
        self.mode = display
    }
}
