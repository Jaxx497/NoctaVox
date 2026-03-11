use crate::ui_state::{
    InactiveGradient, ProgressGradient,
    theme::{
        BAR_SYMBOL_PLAYED, BAR_SYMBOL_UNPLAYED, NONWAVEFORM_SPEED, SPECTRUM_DECAY, SPECTRUM_MIRROR,
        WAVEFORM_SPEED,
        import::{OscilloScheme, ProgressBarScheme, SpectrumScheme, WaveformScheme},
    },
};
use anyhow::Result;

#[derive(Clone)]
pub struct ParsedBar {
    pub active_color: ProgressGradient,
    pub inactive_color: InactiveGradient,
    pub speed: f32,

    pub played_symbol: String,
    pub unplayed_symbol: String,
}

impl ParsedBar {
    pub(crate) fn parse(
        p: Option<&ProgressBarScheme>,
        c: &ProgressGradient,
        s: Option<f32>,
    ) -> Result<Self> {
        match p {
            Some(bar) => Ok(ParsedBar {
                active_color: match bar.color.as_ref() {
                    Some(raw) => ProgressGradient::from_raw(raw)?,
                    None => c.clone(),
                },
                inactive_color: match bar.color_unplayed.as_ref() {
                    Some(raw) => InactiveGradient::from_raw(raw)?,
                    None => InactiveGradient::Dimmed,
                },
                speed: p.and_then(|b| b.speed).or(s).unwrap_or(NONWAVEFORM_SPEED) / 10.0,
                played_symbol: bar
                    .symbol_played
                    .as_deref()
                    .unwrap_or(BAR_SYMBOL_PLAYED)
                    .to_string(),
                unplayed_symbol: bar
                    .symbol_unplayed
                    .as_deref()
                    .unwrap_or(BAR_SYMBOL_UNPLAYED)
                    .to_string(),
            }),
            None => Ok(ParsedBar {
                active_color: c.clone(),
                inactive_color: InactiveGradient::Dimmed,
                played_symbol: BAR_SYMBOL_PLAYED.to_string(),
                unplayed_symbol: BAR_SYMBOL_UNPLAYED.to_string(),
                speed: s.unwrap_or(NONWAVEFORM_SPEED) / 20.0,
            }),
        }
    }
}

#[derive(Clone)]
pub struct ParsedOscillo {
    pub color: ProgressGradient,
    pub speed: f32,
}

impl ParsedOscillo {
    pub(crate) fn parse(
        p: Option<&OscilloScheme>,
        c: &ProgressGradient,
        s: Option<f32>,
    ) -> Result<Self> {
        match p {
            Some(oscillo) => Ok(ParsedOscillo {
                color: match oscillo.color.as_ref() {
                    Some(raw) => ProgressGradient::from_raw(raw)?,
                    None => c.clone(),
                },
                speed: p.and_then(|o| o.speed).or(s).unwrap_or(NONWAVEFORM_SPEED) / 10.0,
            }),
            None => Ok(ParsedOscillo {
                color: c.clone(),
                speed: s.unwrap_or(NONWAVEFORM_SPEED) / 10.0,
            }),
        }
    }
}

#[derive(Clone)]
pub struct ParsedSpectrum {
    pub colors: ProgressGradient,
    pub mirror: bool,
    pub decay: f32,
    pub speed: f32,
}

impl ParsedSpectrum {
    pub(crate) fn parse(
        p: Option<&SpectrumScheme>,
        c: &ProgressGradient,
        s: Option<f32>,
    ) -> Result<Self> {
        match p {
            Some(spectrum) => Ok(ParsedSpectrum {
                colors: match spectrum.color.as_ref() {
                    Some(raw) => ProgressGradient::from_raw(raw)?,
                    None => c.clone(),
                },
                mirror: spectrum.mirror.unwrap_or(SPECTRUM_MIRROR),
                decay: spectrum.decay.unwrap_or(SPECTRUM_DECAY).clamp(0.7, 0.97),
                speed: p.and_then(|w| w.speed).or(s).unwrap_or(NONWAVEFORM_SPEED) / 10.0,
            }),

            None => Ok(ParsedSpectrum {
                colors: c.clone(),
                mirror: SPECTRUM_MIRROR,
                decay: SPECTRUM_DECAY,
                speed: s.unwrap_or(NONWAVEFORM_SPEED) / 10.0,
            }),
        }
    }
}

#[derive(Clone)]
pub struct ParsedWaveform {
    pub active_color: ProgressGradient,
    pub inactive_color: InactiveGradient,
    pub speed: f32,
}

impl ParsedWaveform {
    pub(crate) fn parse(
        p: Option<&WaveformScheme>,
        c: &ProgressGradient,
        s: Option<f32>,
    ) -> Result<Self> {
        match p {
            Some(wf) => Ok(ParsedWaveform {
                active_color: match wf.color.as_ref() {
                    Some(raw) => ProgressGradient::from_raw(raw)?,
                    None => c.clone(),
                },
                inactive_color: match wf.color_unplayed.as_ref() {
                    Some(raw) => InactiveGradient::from_raw(raw)?,
                    None => InactiveGradient::Dimmed,
                },
                speed: p.and_then(|w| w.speed).or(s).unwrap_or(WAVEFORM_SPEED) / 10.0,
            }),
            None => Ok(ParsedWaveform {
                active_color: c.clone(),
                inactive_color: InactiveGradient::Dimmed,
                speed: s.unwrap_or(WAVEFORM_SPEED) / 10.0,
            }),
        }
    }
}
