use crate::ui_state::{
    ParsedBar, ParsedOscillo, ParsedSpectrum, ParsedWaveform, ProgressGradient, ThemeImport,
    theme::theme_utils::{parse_borders, parse_display},
};
use anyhow::{Result, anyhow};
use ratatui::{
    style::Color,
    symbols::Marker,
    widgets::{BorderType, Borders},
};
use std::{path::Path, rc::Rc, sync::Arc};

#[derive(Clone)]
pub struct ThemeConfig {
    pub name: String,
    pub is_dark: bool,

    // Surface Colors
    pub surface_global: Color,   // Global bg
    pub surface_active: Color,   // Focused pane
    pub surface_inactive: Color, // Inactive pane
    pub surface_error: Color,    // Error popup bg

    // Text colors
    pub text_primary: Color,      // Focused text
    pub text_secondary: Color,    // Accented text
    pub text_secondary_in: Color, // Accented text
    pub text_muted: Color,        // Inactive/quiet text
    pub text_selection: Color,    // Text inside of selection bar

    // Border colors
    pub border_active: Color,   // Border highlight
    pub border_inactive: Color, // Border Inactive

    // Selection colors
    pub accent: Color,          // Selection Bar color
    pub accent_inactive: Color, // Selection inactive

    // Border configuration
    pub border_display: Borders,
    pub border_type: BorderType,

    // Progress Displays
    pub bar: ParsedBar,
    pub waveform: ParsedWaveform,
    pub spectrum: ParsedSpectrum,
    pub oscillo: ParsedOscillo,

    pub progress_style: Marker,

    pub decorator: Rc<String>,
}

impl ThemeConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file_str = std::fs::read_to_string(&path.as_ref())?;
        let config = toml::from_str::<ThemeImport>(&file_str)?;
        let mut theme = Self::try_from(&config)?;

        theme.name = path
            .as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or(anyhow!("Could not identify theme name"))?
            .to_string();

        Ok(theme)
    }
}

impl TryFrom<&ThemeImport> for ThemeConfig {
    type Error = anyhow::Error;

    fn try_from(config: &ThemeImport) -> anyhow::Result<Self> {
        let colors = &config.colors;
        let progress = config.progress.as_ref();

        let accent = *colors.accent;
        let pcolor = progress
            .and_then(|p| p.color.as_ref())
            .map(|raw| ProgressGradient::from_raw(raw))
            .transpose()?
            .unwrap_or(ProgressGradient::Static(accent));

        let speed = progress.and_then(|p| p.speed);

        Ok(ThemeConfig {
            name: String::new(),

            surface_global: *colors.surface_global,
            surface_active: *colors.surface_active,
            surface_inactive: *colors.surface_inactive,
            surface_error: *colors.surface_error,

            text_primary: *colors.text_primary,
            text_secondary: *colors.text_secondary,
            text_secondary_in: *colors.text_secondary_in,
            text_selection: *colors.text_selection,
            text_muted: *colors.text_muted,

            border_active: *colors.border_active,
            border_inactive: *colors.border_inactive,

            accent: accent,
            accent_inactive: *colors.accent_inactive,

            border_display: parse_borders(
                config
                    .borders
                    .as_ref()
                    .and_then(|b| b.display)
                    .unwrap_or(true),
            ),
            border_type: config
                .borders
                .as_ref()
                .and_then(|b| b.style)
                .unwrap_or(BorderType::Rounded),

            progress_style: parse_display(progress.and_then(|p| p.style.as_deref())),

            bar: ParsedBar::parse(progress.and_then(|p| p.bar.as_ref()), &pcolor, speed)?,
            oscillo: ParsedOscillo::parse(
                progress.and_then(|p| p.oscilloscope.as_ref()),
                &pcolor,
                speed,
            )?,
            spectrum: ParsedSpectrum::parse(
                progress.and_then(|p| p.spectrum.as_ref()),
                &pcolor,
                speed,
            )?,
            waveform: ParsedWaveform::parse(
                progress.and_then(|p| p.waveform.as_ref()),
                &pcolor,
                speed,
            )?,

            decorator: Rc::from(
                config
                    .extras
                    .as_ref()
                    .and_then(|e| e.decorator.as_deref())
                    .unwrap_or("✧")
                    .to_owned(),
            ),

            is_dark: config
                .extras
                .as_ref()
                .and_then(|e| e.is_dark)
                .unwrap_or(true),
        })
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        use super::*;

        ThemeConfig {
            name: String::from("Noctavox_Alpha"),
            is_dark: true,

            surface_global: DARK_GRAY_FADED,
            surface_active: DARK_GRAY,
            surface_inactive: DARK_GRAY_FADED,
            surface_error: GOOD_RED_DARK,

            text_primary: DARK_WHITE,
            text_muted: MID_GRAY,
            text_selection: DARK_GRAY,
            text_secondary: GOOD_RED,
            text_secondary_in: GOOD_RED_DARK,

            border_active: GOLD,
            border_inactive: DARK_GRAY_FADED,

            accent: GOLD,
            accent_inactive: GOLD_FADED,

            border_display: Borders::ALL,
            border_type: BorderType::Rounded,

            progress_style: Marker::Braille,

            bar: ParsedBar::parse(None, &ProgressGradient::Static(GOLD), None)
                .expect("If you see this, what have you done?"),

            oscillo: ParsedOscillo {
                color: ProgressGradient::Gradient(Arc::from([
                    DARK_WHITE,
                    GOOD_RED_DARK,
                    DARK_GRAY,
                ])),
                speed: NONWAVEFORM_SPEED,
            },

            spectrum: ParsedSpectrum {
                colors: ProgressGradient::Gradient(Arc::from([
                    DARK_WHITE,
                    GOOD_RED_DARK,
                    DARK_GRAY,
                ])),
                mirror: SPECTRUM_MIRROR,
                decay: SPECTRUM_DECAY,
                speed: NONWAVEFORM_SPEED,
            },

            waveform: ParsedWaveform {
                active_color: ProgressGradient::Gradient(Arc::from([
                    DARK_WHITE,
                    GOOD_RED_DARK,
                    DARK_GRAY,
                ])),
                inactive_color: InactiveGradient::Dimmed,
                speed: WAVEFORM_SPEED / 10.0,
            },

            decorator: Rc::from("✧".to_string()),
        }
    }
}
