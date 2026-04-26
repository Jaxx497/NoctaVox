use crate::ui_state::{LibraryView, Mode, Pane, ProgressDisplay, UiState};
use ratatui::layout::{Constraint, Layout, Rect};

pub struct LayoutMinimal {
    pub search_bar: Rect,
    pub content: Rect,
    pub widget: Rect,
}

const MIN_WINDOW_WIDTH: u16 = 50;
const WIDGET_PERCENTAGE: f32 = 0.12;

impl LayoutMinimal {
    pub fn new(area: Rect, state: &mut UiState) -> Self {
        let is_progress_display = state.is_progress_display();

        let search_height = match state.get_mode() {
            Mode::Search => match state.borders_enabled() {
                true => 5,
                false => 3,
            },
            _ => 0,
        };

        let widget_h = match is_progress_display {
            false => 0,
            true => match (state.get_progress_display(), area.height > 20) {
                (ProgressDisplay::ProgressBar, _) | (_, false) => 3,
                _ => (area.height as f32 * WIDGET_PERCENTAGE).ceil() as u16,
            },
        };

        let main_area = {
            let max_width = area.width.saturating_sub(2).max(MIN_WINDOW_WIDTH);
            let width = (area.width / 2).clamp(MIN_WINDOW_WIDTH, max_width);
            area.centered_horizontally(Constraint::Length(width))
        };

        let item_count = match state.get_pane() {
            Pane::SideBar => match state.get_sidebar_view() {
                LibraryView::Albums => state.albums.len(),
                LibraryView::Playlists => state.playlists.len(),
            },
            _ => state.get_legal_songs().len(),
        };

        let max_h = (area.height as f64 * 0.5).ceil() as u16;
        let block_h = (get_block_height(item_count, area) as u16 + search_height).min(max_h);

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
            Layout::vertical([Constraint::Length(search_height), Constraint::Fill(1)])
                .areas(upper_block);

        let widget = widget_spacing.centered_horizontally(Constraint::Percentage(80));

        LayoutMinimal {
            search_bar,
            content: song_window,
            widget,
        }
    }
}

fn get_block_height(len: usize, area: Rect) -> usize {
    (len + 2).clamp(3, (area.height as f64 * 0.5).ceil() as usize)
}

// Vertically Centered:
//
//   Constraint::Percentage(20),
//   Constraint::Length(x as u16),
//   Constraint::Length(1),
//   Constraint::Length(widget_h),
//   Constraint::Fill(1),
