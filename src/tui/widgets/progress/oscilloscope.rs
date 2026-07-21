use crate::{theme::DisplayTheme, ui_state::UiState};
use ratatui::{
    style::Stylize,
    widgets::{
        Block, Padding, StatefulWidget, Widget,
        canvas::{Canvas, Context, Line},
    },
};

const OSCILLO_LIMITER: usize = 512;

pub struct Oscilloscope;
impl StatefulWidget for Oscilloscope {
    type State = UiState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let theme = state.theme.get_display_theme(true);
        let elapsed = state.metrics.position().as_secs_f32();

        let samples = state.viz.display_tap();

        let n = OSCILLO_LIMITER.min(samples.len());
        let samples = &samples[samples.len() - n..];

        if samples.is_empty() {
            return;
        }

        let v_marg = match area.height > 20 {
            true => ((area.height as f32) * 0.25) as u16,
            false => 0,
        };

        Canvas::default()
            .x_bounds([0.0, samples.len() as f64])
            .y_bounds([-1.0, 1.0])
            .marker(theme.progress_style)
            .paint(|ctx| {
                draw_oscilloscope(ctx, samples, elapsed, theme);
            })
            .background_color(theme.bg_global)
            .block(Block::new().bg(theme.bg_global).padding(Padding {
                left: 1,
                right: 1,
                top: v_marg,
                bottom: v_marg,
            }))
            .render(area, buf);
    }
}

fn draw_oscilloscope(ctx: &mut Context, samples: &[f32], time: f32, theme: &DisplayTheme) {
    let peak = samples
        .iter()
        .map(|s| s.abs())
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(1.0);

    let scale = if peak > 1.0 { 1.0 / peak } else { 1.0 };

    for (i, window) in samples.windows(2).enumerate() {
        let x1 = i as f64;
        let y1 = (window[0] * scale) as f64;
        let x2 = (i + 1) as f64;
        let y2 = (window[1] * scale) as f64;

        let progress = i as f32 / samples.len() as f32;

        let time = time / 4.0; // Slow down gradient scroll substantially
        let color = theme
            .oscilloscope
            .color
            .color_at(progress, time, theme.oscilloscope.speed);

        ctx.draw(&Line {
            x1,
            y1,
            x2,
            y2,
            color,
        });
    }
}
