use crate::ui_state::{LibraryView, Mode, Pane, ProgressDisplay, UiState};
use ratatui::layout::{Constraint, Layout, Rect};

pub struct LayoutMinimal {
    pub search_bar: Rect,
    pub content: Rect,
    pub widget: Rect,
}

impl LayoutMinimal {
    pub fn new(area: Rect, state: &mut UiState) -> Self {
        let is_progress_display = state.is_progress_display();

        let search_height = match state.get_mode() == Mode::Search {
            true => match state.borders_enabled() {
                true => 5,
                false => 3,
            },
            false => 0,
        };

        let widget_h = match is_progress_display {
            false => 0,
            true => match (state.get_progress_display(), area.height > 20) {
                (ProgressDisplay::ProgressBar, _) | (_, false) => 3,
                _ => (area.height as f32 * 0.12).ceil() as u16,
            },
        };

        let [_lpadding, main_area, _rpadding] = Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Fill(1),
            Constraint::Percentage(20),
        ])
        .areas(area);

        let block_h = match state.get_pane() {
            Pane::TrackList => get_block_height(state.get_legal_songs().len(), area),
            _ => match state.get_sidebar_view() {
                LibraryView::Albums => get_block_height(state.albums.len(), area),
                LibraryView::Playlists => get_block_height(state.playlists.len(), area),
            },
        } as u16;

        let [_upper_pad, upper_block, _, widget_spacing, _bottom_pad] = Layout::vertical([
            Constraint::Percentage(20),
            Constraint::Length(block_h),
            Constraint::Min(1),
            Constraint::Length(widget_h),
            Constraint::Percentage(match is_progress_display {
                true => 10,
                false => 15,
            }),
        ])
        .areas(main_area);

        let [search_bar, song_window] =
            Layout::vertical([Constraint::Length(search_height), Constraint::Fill(100)])
                .areas(upper_block);

        let [_, widget, _] = Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Fill(1),
            Constraint::Percentage(10),
        ])
        .areas(widget_spacing);

        LayoutMinimal {
            search_bar,
            content: song_window,
            widget,
        }
    }
}

fn get_block_height(len: usize, area: Rect) -> usize {
    (len + 4).clamp(3, (area.height as f64 * 0.5).ceil() as usize)
}

// Vertically Centered:
//
//   Constraint::Percentage(20),
//   Constraint::Length(x as u16),
//   Constraint::Length(1),
//   Constraint::Length(widget_h),
//   Constraint::Fill(1),
