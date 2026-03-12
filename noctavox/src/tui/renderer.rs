use super::{AppLayout, Progress, SearchBar, SideBar, widgets::SongTable};
use crate::{
    UiState,
    tui::{
        render_bg,
        widgets::{BufferLine, PopupManager},
    },
    ui_state::Mode,
};
use ratatui::{Frame, layout::Rect, widgets::StatefulWidget};

pub fn render(f: &mut Frame, state: &mut UiState) {
    let area = f.area();
    if matches!(state.get_mode(), Mode::Fullscreen) {
        let bf_area = get_bf_area(area);

        Progress.render(area, f.buffer_mut(), state);
        BufferLine.render(bf_area, f.buffer_mut(), state);

        return;
    }

    let layout = AppLayout::new(area, state);
    let bf_area = get_bf_area(layout.display_widget);
    render_bg(state, f);

    SearchBar.render(layout.search_bar, f.buffer_mut(), state);
    SideBar.render(layout.sidebar, f.buffer_mut(), state);
    SongTable.render(layout.song_window, f.buffer_mut(), state);
    Progress.render(layout.display_widget, f.buffer_mut(), state);
    BufferLine.render(bf_area, f.buffer_mut(), state);

    if state.popup.is_open() {
        PopupManager.render(f.area(), f.buffer_mut(), state);
    }
}

fn get_bf_area(area: Rect) -> Rect {
    Rect {
        y: area.bottom().saturating_sub(1),
        height: 1,
        ..area
    }
}
