use crate::ui_state::{UiState, fade_color};
use ratatui::{
    style::{Color, Stylize},
    widgets::{
        Block, StatefulWidget, Widget,
        canvas::{Canvas, Line},
    },
};

const LEFT_MARG: u16 = 0;
const RIGHT_MARG: u16 = 0;

pub struct SpectrumAnalyzer;

impl StatefulWidget for SpectrumAnalyzer {
    type State = UiState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        if state.player_is_active() && !state.is_paused() {
            state.update_spectrum();
        }

        let theme = state.theme_manager.get_display_theme(true);
        let elapsed = state.get_playback_elapsed_f32();

        let canvas_width = area.width.saturating_sub(LEFT_MARG + RIGHT_MARG).max(1) as usize;
        let pixel_width = canvas_width * 2;

        state.spectrum.remap_display(canvas_width);

        let display = &state.spectrum.display_bins;
        if display.is_empty() {
            return;
        }

        let is_mirrored = theme.spectrum.mirror;

        let y_min = match is_mirrored {
            true => -1.05,
            false => 0.05,
        };

        Canvas::default()
            .x_bounds([0.00, pixel_width as f64])
            .y_bounds([y_min, 1.05]) // The 0.05 prevents overflow
            .marker(theme.progress_style)
            .paint(|ctx| {
                for (i, &mag) in display.iter().enumerate() {
                    let progress = i as f32 / canvas_width as f32;
                    let base =
                        theme
                            .spectrum
                            .colors
                            .color_at(progress, elapsed, theme.spectrum.speed);
                    let color = fade_color(theme.dark, base, mag.clamp(0.25, 1.0));

                    for x in [i * 2, i * 2 + 1] {
                        ctx.draw(&spectrum_line(x as f64, mag as f64, is_mirrored, color))
                    }
                }
            })
            .background_color(theme.bg_global)
            .block(Block::new().bg(theme.bg_global))
            .render(area, buf)
    }
}

fn spectrum_line(x: f64, mag: f64, mirrored: bool, color: Color) -> Line {
    Line {
        x1: x,
        y1: match mirrored {
            true => -mag,
            false => 0.0,
        },
        x2: x,
        y2: mag,
        color,
    }
}
