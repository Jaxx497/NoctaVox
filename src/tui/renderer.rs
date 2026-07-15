use super::{LayoutTraditional, Progress, SearchBar, SideBar, widgets::SongTable};
use crate::{
    UiState,
    tui::{
        layout_minimal::LayoutMinimal,
        render_bg,
        widgets::{BreadCrumbs, BufferLine, PopupManager},
    },
    ui_state::{LayoutStyle, Mode, Pane},
};
use ratatui::{Frame, layout::Rect, widgets::StatefulWidget};

pub fn render(f: &mut Frame, state: &mut UiState) {
    let area = f.area();

    if matches!(state.get_mode(), Mode::Fullscreen) {
        let bf_area = get_bufferline_area(area);
        Progress.render(area, f.buffer_mut(), state);
        BufferLine.render(bf_area, f.buffer_mut(), state);

        return;
    }

    match state.layout {
        LayoutStyle::Traditional => render_traditional(area, f, state),
        LayoutStyle::Minimal => render_minimal(area, f, state),
    };

    if state.popup.is_open() {
        PopupManager.render(f.area(), f.buffer_mut(), state);
    }
}

fn render_minimal(area: Rect, f: &mut Frame, state: &mut UiState) {
    let layout = LayoutMinimal::new(area, state);

    render_bg(state, f);
    let bf_area = get_bufferline_area(area);
    let bc_area = get_breadcrumbs_area(layout.content);

    BreadCrumbs.render(bc_area, f.buffer_mut(), state);
    SearchBar.render(layout.search_bar, f.buffer_mut(), state);

    match state.get_pane() == Pane::SideBar {
        true => SideBar.render(layout.content, f.buffer_mut(), state),
        false => SongTable.render(layout.content, f.buffer_mut(), state),
    }
    Progress.render(layout.widget, f.buffer_mut(), state);
    BufferLine.render(bf_area, f.buffer_mut(), state);
}

fn render_traditional(area: Rect, f: &mut Frame, state: &mut UiState) {
    let layout = LayoutTraditional::new(area, state);

    let bf_area = get_bufferline_area(area);
    render_bg(state, f);

    SearchBar.render(layout.search_bar, f.buffer_mut(), state);
    SideBar.render(layout.sidebar, f.buffer_mut(), state);
    SongTable.render(layout.song_window, f.buffer_mut(), state);

    if state.metrics.is_active() || state.library_refresh.is_some() {
        Progress.render(layout.widget, f.buffer_mut(), state);
        BufferLine.render(bf_area, f.buffer_mut(), state);
    }
}

fn get_breadcrumbs_area(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y.saturating_sub(1),
        height: 1,
        ..area
    }
}

fn get_bufferline_area(area: Rect) -> Rect {
    Rect {
        y: area.bottom().saturating_sub(1),
        height: 1,
        ..area
    }
}
