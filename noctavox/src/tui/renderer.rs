use super::{AppLayout, Progress, SearchBar, SideBar, widgets::SongTable};
use crate::{
    UiState,
    tui::{
        render_bg,
        widgets::{BufferLine, PopupManager},
    },
    ui_state::Mode,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::StatefulWidget,
};

pub fn render(f: &mut Frame, state: &mut UiState) {
    if matches!(state.get_mode(), Mode::Fullscreen) {
        let [progress, bufferline] = get_fullscreen_layout(f.area());

        Progress.render(progress, f.buffer_mut(), state);
        BufferLine.render(bufferline, f.buffer_mut(), state);

        return;
    }

    let layout = AppLayout::new(f.area(), state);
    render_bg(state, f);

    let bf_area = Rect {
        y: layout.progress_bar.bottom().saturating_sub(1),
        height: 1,
        ..layout.progress_bar
    };

    SearchBar.render(layout.search_bar, f.buffer_mut(), state);
    SideBar.render(layout.sidebar, f.buffer_mut(), state);
    SongTable.render(layout.song_window, f.buffer_mut(), state);
    Progress.render(layout.progress_bar, f.buffer_mut(), state);
    BufferLine.render(bf_area, f.buffer_mut(), state);

    if state.popup.is_open() {
        PopupManager.render(f.area(), f.buffer_mut(), state);
    }
}

fn get_fullscreen_layout(area: Rect) -> [Rect; 2] {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(99), Constraint::Length(1)])
        .areas::<2>(area)
}
