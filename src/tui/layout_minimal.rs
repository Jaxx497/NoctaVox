use crate::{
    ui_state::{Mode, Pane, UiState},
    visualization::ProgressDisplay,
};
use ratatui::layout::{Constraint, Layout, Rect};

pub struct LayoutMinimal {
    pub search_bar: Rect,
    pub bread_crumbs: Rect,
    pub content: Rect,
    pub widget: Rect,
}

const MIN_WINDOW_WIDTH: u16 = 50;
const TOP_WIN_CAP_PERCENTAGE: f32 = 15.0;
const WIDGET_PERCENTAGE: f32 = 0.10;
const FULLY_CONDENSED: u16 = 20;

impl LayoutMinimal {
    pub fn new(area: Rect, state: &mut UiState) -> Self {
        let is_progress_display = state.metrics.is_active();
        let widget_pad = match area.height > FULLY_CONDENSED {
            true => 2,
            false => 1,
        };

        let (search_height, bc) = match state.get_mode() {
            Mode::Search => match state.borders_enabled() {
                true => (5, 0),
                false => (3, 0),
            },
            _ => (0, 1),
        };

        let widget_h = match is_progress_display {
            false => 0,
            true => match (state.viz.get_progress_display(), area.height > 20) {
                (ProgressDisplay::ProgressBar, _) | (_, false) => 3,
                _ => (area.height as f32 * WIDGET_PERCENTAGE).ceil() as u16,
            },
        };

        let main_area = {
            let max_width = area.width.saturating_sub(4).max(MIN_WINDOW_WIDTH);
            let width = (area.width / 2).clamp(MIN_WINDOW_WIDTH, max_width);
            area.centered_horizontally(Constraint::Length(width))
        };

        let item_count = match (state.get_pane(), &state.popup.cached) {
            (Pane::SideBar, _) | (Pane::Popup, Pane::SideBar) => state.nav.sidebar.rows.len(),
            _ => state.get_legal_songs().len(),
        };

        let upper_pct = ((area.height.saturating_sub(20) as f32 / 12.0) * TOP_WIN_CAP_PERCENTAGE)
            .clamp(0.0, TOP_WIN_CAP_PERCENTAGE) as u16;

        let widget_per = match (state.metrics.is_active(), state.viz.get_progress_display()) {
            (false, _) => 7,
            (true, ProgressDisplay::ProgressBar) => 7,
            _ => 10,
        };

        let [_upper_pad, middle, _bottom_pad] = Layout::vertical([
            Constraint::Percentage(upper_pct),
            Constraint::Fill(1),
            Constraint::Percentage(widget_per),
        ])
        .areas(main_area);

        let available = middle
            .height
            .saturating_sub(bc + widget_pad as u16 + widget_h) as usize;
        let block_h = (item_count + widget_pad + search_height as usize).min(available) as u16;

        let [bread_crumbs, upper_block, _gap, widget_spacing] = Layout::vertical([
            Constraint::Length(bc),
            Constraint::Length(block_h),
            Constraint::Min(widget_pad as u16),
            Constraint::Length(widget_h),
        ])
        .areas(middle);

        let [search_bar, song_window] =
            Layout::vertical([Constraint::Length(search_height), Constraint::Fill(1)])
                .areas(upper_block);

        let widget = widget_spacing.centered_horizontally(Constraint::Percentage(80));

        LayoutMinimal {
            search_bar,
            bread_crumbs,
            content: song_window,
            widget,
        }
    }
}
