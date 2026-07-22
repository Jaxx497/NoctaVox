use crate::{
    theme::fade_color,
    ui_state::{LayoutStyle, UiState},
};
use ratatui::{
    style::{Color, Stylize},
    widgets::{
        Block, Padding, StatefulWidget, Widget,
        canvas::{Canvas, Line},
    },
};

pub struct SpectrumAnalyzer;

impl StatefulWidget for SpectrumAnalyzer {
    type State = UiState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        if state.metrics.is_active() && !state.metrics.is_paused() {
            state.viz.update_spectrum();
        }

        let theme = state.theme.get_display_theme(true);
        let elapsed = state.metrics.position().as_secs_f32();

        let canvas_width = area.width.max(1) as usize;
        let pixel_width = canvas_width * 2;

        let spectrum = state.viz.spectrum_mut();
        spectrum.remap_display(canvas_width);
        let display = spectrum.get_display_bins();

        if display.is_empty() {
            return;
        }

        let padding = if let LayoutStyle::Traditional = state.layout {
            Padding {
                left: 10,
                right: 10,
                top: 1,
                bottom: 1,
            }
        } else {
            Padding::default()
        };

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
            .block(Block::new().bg(theme.bg_global).padding(padding))
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
