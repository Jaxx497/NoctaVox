use crate::{
    ui_state::{Mode, UiState},
    visualization::ProgressDisplay,
};
use ratatui::layout::{Constraint, Layout, Rect};

pub struct LayoutTraditional {
    pub sidebar: Rect,
    pub search_bar: Rect,
    pub song_window: Rect,
    pub widget: Rect,
}

impl LayoutTraditional {
    pub fn new(area: Rect, state: &mut UiState) -> Self {
        let prog_height = match state.metrics.is_active() {
            false => 0,
            true => match (state.viz.get_progress_display(), area.height > 20) {
                (ProgressDisplay::ProgressBar, _) | (_, false) => 3,
                _ => {
                    let h = (area.height as f32 * 0.15).ceil() as u16;
                    h | 1
                }
            },
        };

        let refresh_no_media = match state.library_refresh.is_some() && !state.metrics.is_active() {
            true => 1,
            false => 0,
        };

        let search_height = match state.get_mode() == Mode::Search {
            true => 5,
            false => 0,
        };

        let [upper_block, widget, _] = Layout::vertical([
            Constraint::Min(16),
            Constraint::Length(prog_height),
            Constraint::Length(refresh_no_media),
        ])
        .areas(area);

        let [sidebar, upper_block] = Layout::horizontal([
            Constraint::Percentage(state.nav.sidebar.width),
            Constraint::Fill(1),
        ])
        .areas(upper_block);

        let [search_bar, song_window] =
            Layout::vertical([Constraint::Length(search_height), Constraint::Fill(100)])
                .areas(upper_block);

        LayoutTraditional {
            sidebar,
            search_bar,
            song_window,
            widget,
        }
    }
}
