use crate::ui_state::{LayoutStyle, UiState};
use ratatui::{
    style::Stylize,
    widgets::{Block, LineGauge, Padding, StatefulWidget, Widget},
};

pub struct ProgressBar;

impl StatefulWidget for ProgressBar {
    type State = UiState;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let theme = state.theme.get_display_theme(true);

        let elapsed = state.metrics.position().as_secs_f32();
        let duration = state.metrics.duration().as_secs_f32();

        // Prevent crash
        let ratio = (elapsed / duration).min(0.9999);

        let pos = if theme.progress_bar.speed == 0.0 {
            0.0
        } else {
            ratio
        };

        let fg = theme
            .progress_bar
            .active_color
            .color_at(pos, elapsed, theme.progress_bar.speed);
        let bg = theme.progress_bar.inactive_color.color_at(
            ratio,
            elapsed,
            theme.progress_bar.speed,
            theme.dark,
            super::DEFAULT_AMP,
            &theme.progress_bar.active_color,
        );

        let w_pad = match state.layout {
            LayoutStyle::Traditional => 10,
            LayoutStyle::Minimal => 0,
        };

        let guage = LineGauge::default()
            .block(Block::new().bg(theme.bg_global).padding(Padding {
                left: w_pad,
                right: w_pad + 1,
                top: (area.height / 2),
                bottom: 0,
            }))
            .filled_style(fg)
            .unfilled_style(bg)
            .filled_symbol(&theme.progress_bar.played_symbol)
            .unfilled_symbol(&theme.progress_bar.unplayed_symbol)
            .label("")
            .ratio(ratio as f64);

        guage.render(area, buf);
    }
}
