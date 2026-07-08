use crate::theme::{ParsedBar, ParsedOscillo, ParsedSpectrum, ParsedWaveform};
use ratatui::{
    style::Color,
    symbols::Marker,
    widgets::{BorderType, Borders},
};

pub struct DisplayTheme {
    pub dark: bool,
    pub bg: Color,
    pub bg_global: Color,
    pub bg_error: Color,

    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_selected: Color,

    pub accent: Color,

    pub border: Color,
    pub border_display: Borders,
    pub border_type: BorderType,

    pub progress_style: Marker,

    pub progress_bar: ParsedBar,
    pub waveform: ParsedWaveform,
    pub spectrum: ParsedSpectrum,
    pub oscilloscope: ParsedOscillo,
}

impl DisplayTheme {
    pub fn has_borders(&self) -> bool {
        self.border_display != Borders::NONE
    }
}
