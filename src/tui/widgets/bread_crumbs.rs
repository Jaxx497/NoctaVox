use crate::ui_state::{LibraryView, Mode, Pane, UiState, fade_color};
use ratatui::{
    style::Stylize,
    text::{Line, Span},
    widgets::{StatefulWidget, Widget},
};
use unicode_width::UnicodeWidthStr;

pub struct BreadCrumbs;

impl StatefulWidget for BreadCrumbs {
    type State = UiState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        if !matches!(state.get_mode(), Mode::Queue | Mode::Library(_)) {
            return;
        }

        let theme = state.theme_manager.get_display_theme(true);
        let top_level = state.get_sidebar_view();
        let sidebar = top_level.to_string();

        let bc_highlight = fade_color(theme.dark, theme.accent, 0.85);
        let dimmed = fade_color(theme.dark, theme.text_muted, 0.75);

        let right_label = match top_level {
            LibraryView::Albums => state.get_album_sort_string(),
            LibraryView::Playlists => format!("{} 󰲸", state.playlists.len()),
        };

        let spans = match state.get_pane() {
            Pane::SideBar => {
                let padding =
                    area.width
                        .saturating_sub(sidebar.width() as u16)
                        .saturating_sub(right_label.width() as u16) as usize;

                vec![
                    Span::from(&sidebar).fg(bc_highlight),
                    Span::raw(" ".repeat(padding)),
                    Span::from(right_label).fg(dimmed),
                ]
            }
            Pane::TrackList => match state.get_mode() {
                Mode::Library(LibraryView::Albums) => {
                    // match top_level {
                    // LibraryView::Albums => {
                    let Some(album) = state.get_selected_album() else {
                        return;
                    };
                    Vec::from([
                        Span::from(format!("{top_level}  ")).fg(theme.text_muted),
                        Span::from(format!("{}", album.title)).fg(bc_highlight),
                        Span::from(format!(" [{}]", album.artist)).fg(theme.text_muted),
                    ])
                }
                Mode::Library(LibraryView::Playlists) => {
                    let Some(playlist) = state.get_selected_playlist() else {
                        return;
                    };
                    Vec::from([
                        Span::from(format!("{top_level}  ")).fg(theme.text_muted),
                        Span::from(format!("{}", playlist.name)).fg(bc_highlight),
                    ])
                }
                Mode::Queue => {
                    let queue_len = state.playback.queue_len();
                    Vec::from([
                        Span::from(format!("Queue ")).fg(theme.text_muted),
                        Span::from(format!("({queue_len})")).fg(theme.text_muted),
                    ])
                }
                _ => return,
            },
            _ => return,
        };

        Line::from(spans).render(area, buf);
    }
}
