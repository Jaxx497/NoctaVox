use crate::{
    DurationStyle, get_readable_duration, ui_state::UiState, visualization::ProgressDisplay,
};
use ratatui::{
    layout::Rect,
    style::Stylize,
    text::Text,
    widgets::{StatefulWidget, Widget},
};

pub struct Timer;
impl StatefulWidget for Timer {
    type State = UiState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let y_pos = match state.viz.get_progress_display() {
            ProgressDisplay::ProgressBar | ProgressDisplay::Waveform => {
                area.y + ((area.height.saturating_sub(1)) / 2)
            }
            _ => area.y + 1,
        };

        let text_color = state.theme_manager.active.text_muted;

        let elapsed = state.metrics.position();
        let elapsed_str = get_readable_duration(elapsed, crate::DurationStyle::Compact);
        let elapsed_str_len = elapsed_str.len() as u16;

        let dur_str = state.metrics.duration();
        let duration_str = get_readable_duration(dur_str, DurationStyle::Compact);

        let dur_str_len = duration_str.len() as u16;

        let x_elapsed_pos = 2 + (8_u16.saturating_sub(elapsed_str_len) / 2);
        let x_dur_pos =
            (area.x + area.width.saturating_sub(10)) + (10_u16.saturating_sub(dur_str_len) / 2);

        Text::from(elapsed_str)
            .fg(text_color)
            .render(Rect::new(x_elapsed_pos, y_pos, elapsed_str_len, 1), buf);

        Text::from(duration_str)
            .fg(text_color)
            .render(Rect::new(x_dur_pos, y_pos, dur_str_len, 1), buf);
    }
}
