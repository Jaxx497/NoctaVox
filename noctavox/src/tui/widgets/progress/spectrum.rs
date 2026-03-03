use crate::ui_state::UiState;
use ratatui::{
    style::Stylize,
    widgets::{
        Block, Padding, StatefulWidget, Widget,
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
        state.update_spectrum();

        let theme = state.theme_manager.get_display_theme(true);
        let elapsed = state.get_playback_elapsed_f32();

        let canvas_width = area.width.saturating_sub(LEFT_MARG + RIGHT_MARG).max(1) as usize;
        let pixel_width = canvas_width * 2;

        state.spectrum.remap_display(canvas_width);

        let display = &state.spectrum.display_bins;
        if display.is_empty() {
            return;
        }

        Canvas::default()
            .x_bounds([0.00, pixel_width as f64])
            .y_bounds([-1.0, 1.05])
            .marker(theme.oscilloscope_style)
            .paint(|ctx| {
                for (i, &mag) in display.iter().enumerate() {
                    let left = (i * 2) as f64;
                    let right = (i * 2 + 1) as f64;
                    let progress = i as f32 / canvas_width as f32;
                    let color = theme.get_focused_color(progress, elapsed);

                    ctx.draw(&Line {
                        x1: left,
                        y1: -mag as f64,
                        x2: left,
                        y2: mag as f64,
                        color,
                    });
                    ctx.draw(&Line {
                        x1: right,
                        y1: -mag as f64,
                        x2: right,
                        y2: mag as f64,
                        color,
                    });
                }
            })
            .background_color(theme.bg_global)
            .block(Block::new().bg(theme.bg_global).padding(Padding {
                left: LEFT_MARG,
                right: RIGHT_MARG,
                top: 0,
                bottom: 0,
            }))
            .render(area, buf);
    }
}
