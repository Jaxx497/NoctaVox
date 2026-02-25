use crate::ui_state::UiState;
use ratatui::{
    style::Stylize,
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
        let theme = state.theme_manager.get_display_theme(true);
        let elapsed = state.get_playback_elapsed_f32();
        let samples = state.sample_tap.make_contiguous();
        let channels = state.metrics.channels();
        let sample_rate = state.metrics.sample_rate();

        if samples.is_empty() {
            return;
        }

        let is_inactive = state.metrics.is_paused() || state.metrics.is_stopped();
        if is_inactive {
            for bin in state.spectrum.bins.iter_mut() {
                *bin *= 0.92;
            }
        } else {
            if !samples.is_empty() {
                state.spectrum.update(samples, channels, sample_rate);
            }
        }

        let bins = &state.spectrum.bins;

        if bins.is_empty() {
            return;
        }

        let num_bins = bins.len();
        let canvas_width = area.width.saturating_sub(2).max(1) as usize;
        let pixel_width = canvas_width * 2;

        let display: Vec<f32> = (0..canvas_width)
            .map(|i| {
                let t = i as f32 / (canvas_width - 1).max(1) as f32;
                let src = t * (num_bins - 1) as f32;
                let lo = src.floor() as usize;
                let hi = (lo + 1).min(num_bins - 1);
                let frac = src - lo as f32;
                bins[lo] * (1.0 - frac) + bins[hi] * frac
            })
            .collect();

        Canvas::default()
            .x_bounds([0.02, pixel_width as f64])
            .y_bounds([0.0, 1.05])
            .marker(theme.oscilloscope_style)
            .paint(|ctx| {
                for (i, &mag) in display.iter().enumerate() {
                    let left = (i * 2) as f64;
                    let right = (i * 2 + 1) as f64;

                    let progress = i as f32 / samples.len() as f32;

                    let time = elapsed / 4.0; // Slow down gradient scroll substantially
                    let color = theme.get_focused_color(progress, time);

                    ctx.draw(&Line {
                        x1: left,
                        y1: 0.0,
                        x2: left,
                        y2: mag as f64,
                        color,
                    });
                    ctx.draw(&Line {
                        x1: right,
                        y1: 0.0,
                        x2: right,
                        y2: mag as f64,
                        color,
                    });
                }
            })
            .background_color(theme.bg_global)
            .block(Block::new().bg(theme.bg_global).padding(Padding {
                left: 1,
                right: 1,
                top: 0,
                bottom: 0,
            }))
            .render(area, buf);
    }
}
