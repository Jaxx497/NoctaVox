use crate::{
    THEME_DIR,
    theme::{DisplayTheme, ThemeConfig, ThemeIcons, fade_color},
};

pub struct ThemeManager {
    pub active: ThemeConfig,
    pub cached_focused: DisplayTheme,
    pub cached_unfocused: DisplayTheme,

    pub theme_lib: Vec<ThemeConfig>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let theme_lib = Self::collect_themes();
        let active = theme_lib.first().cloned().unwrap_or_default();

        let cached_focused = Self::set_display_theme(&active, true);
        let cached_unfocused = Self::set_display_theme(&active, false);

        ThemeManager {
            active,
            theme_lib,
            cached_focused,
            cached_unfocused,
        }
    }

    pub fn get_display_theme(&self, focus: bool) -> &DisplayTheme {
        match focus {
            true => &self.cached_focused,
            false => &self.cached_unfocused,
        }
    }

    pub fn get_themes(&self) -> &[ThemeConfig] {
        &self.theme_lib
    }

    pub fn icons(&self) -> &ThemeIcons {
        &self.active.icons
    }

    pub fn update_themes(&mut self) {
        let themes = Self::collect_themes();
        self.theme_lib = themes
    }

    pub fn find_theme_by_name(&self, name: &str) -> Option<&ThemeConfig> {
        self.theme_lib.iter().find(|t| t.name == name)
    }

    pub fn get_current_theme_index(&self) -> Option<usize> {
        self.theme_lib
            .iter()
            .position(|t| t.name == self.active.name)
    }

    pub fn get_theme_at_index(&self, idx: usize) -> Option<ThemeConfig> {
        self.theme_lib.get(idx).cloned()
    }

    fn collect_themes() -> Vec<ThemeConfig> {
        let mut themes = vec![];
        let theme_path = &*THEME_DIR;

        if let Ok(entries) = theme_path.read_dir() {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    if let Ok(theme) = ThemeConfig::load_from_file(&path) {
                        themes.push(theme);
                    }
                }
            }
        }
        themes.sort_by(|a, b| a.name.cmp(&b.name));
        themes
    }

    pub(crate) fn set_display_theme(theme: &ThemeConfig, focused: bool) -> DisplayTheme {
        let is_dark = theme.is_dark;

        let progress_style = theme.progress_style;

        let progress_bar = theme.bar.clone();
        let oscilloscope = theme.oscillo.clone();
        let spectrum = theme.spectrum.clone();
        let waveform = theme.waveform.clone();

        match focused {
            true => DisplayTheme {
                dark: theme.is_dark,
                bg: theme.surface_active,
                bg_global: theme.surface_global,
                bg_error: theme.surface_error,

                text_primary: theme.text_primary,
                text_secondary: theme.text_secondary,
                text_muted: theme.text_muted,
                text_selected: theme.text_selection,

                accent: theme.accent,
                border: theme.border_active,
                border_display: theme.border_display,
                border_type: theme.border_type,

                progress_style,

                progress_bar,
                oscilloscope,
                spectrum,
                waveform,
            },

            false => DisplayTheme {
                dark: theme.is_dark,
                bg: theme.surface_inactive,
                bg_global: theme.surface_global,
                bg_error: theme.surface_error,

                text_primary: theme.text_muted,
                text_secondary: theme.text_secondary_in,
                text_muted: fade_color(is_dark, theme.text_muted, 0.7),
                text_selected: theme.text_selection,

                accent: theme.accent_inactive,
                border: theme.border_inactive,
                border_display: theme.border_display,
                border_type: theme.border_type,

                progress_style,

                progress_bar,
                oscilloscope,
                spectrum,
                waveform,
            },
        }
    }
}
