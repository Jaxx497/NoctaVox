use crate::{
    theme::fade_color,
    ui_state::{LibraryView, Mode, Pane, UiState},
};
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

        let theme = state.theme.get_display_theme(true);
        let top_level = state.nav.get_sidebar_view();
        let sidebar = top_level.to_string();

        let bc_highlight = fade_color(theme.dark, theme.accent, 0.85);
        let dimmed = fade_color(theme.dark, theme.text_muted, 0.75);

        let right_label = match top_level {
            LibraryView::Omni => String::default(),
            LibraryView::Albums => state.nav.get_album_sort().to_string(),
            LibraryView::Playlists => format!("{} 󰲸", state.playlists.len()),
        };

        let spans = match state.get_pane() {
            Pane::SideBar => {
                let padding =
                    area.width
                        .saturating_sub(sidebar.width() as u16)
                        .saturating_sub(right_label.width() as u16) as usize;

                vec![
                    Span::from(&sidebar).fg(bc_highlight).underlined(),
                    Span::raw(" ".repeat(padding)),
                    Span::from(right_label).fg(dimmed),
                ]
            }
            Pane::TrackList => match state.get_mode() {
                Mode::Library(LibraryView::Omni) => {
                    let crumb = if let Some(album) = state.get_selected_album() {
                        Vec::from([
                            Span::from(album.title.as_ref())
                                .fg(bc_highlight)
                                .underlined(),
                            Span::from(" [").fg(theme.text_muted),
                            Span::from(album.get_album_artist()).fg(theme.text_muted),
                            Span::from("]").fg(theme.text_muted),
                        ])
                    } else if let Some(playlist) = state.get_selected_playlist() {
                        Vec::from([Span::from(playlist.name.clone()).fg(bc_highlight)])
                    } else if let Some(artist) = state.get_selected_group_label() {
                        Vec::from([Span::from(artist.to_string()).fg(bc_highlight).underlined()])
                    } else {
                        return;
                    };

                    let mut spans = Vec::from([
                        Span::from(top_level.to_str()).fg(theme.text_muted),
                        Span::from("  ").fg(theme.text_muted),
                    ]);
                    spans.extend(crumb);
                    spans
                }
                Mode::Library(LibraryView::Albums) => {
                    let Some(album) = state.get_selected_album() else {
                        return;
                    };
                    Vec::from([
                        Span::from(top_level.to_str()).fg(theme.text_muted),
                        Span::from("  ").fg(theme.text_muted),
                        Span::from(album.title.as_ref())
                            .fg(bc_highlight)
                            .underlined(),
                        Span::from(" [").fg(theme.text_muted),
                        Span::from(album.get_album_artist()).fg(theme.text_muted),
                        Span::from("]").fg(theme.text_muted),
                    ])
                }
                Mode::Library(LibraryView::Playlists) => {
                    let Some(playlist) = state.get_selected_playlist() else {
                        return;
                    };
                    Vec::from([
                        Span::from(top_level.to_str()).fg(theme.text_muted),
                        Span::from("  ").fg(theme.text_muted),
                        Span::from(&playlist.name).fg(bc_highlight),
                    ])
                }
                Mode::Queue => {
                    let queue_len = state.playback.queue_len();
                    Vec::from([
                        Span::from("Queue (").fg(theme.text_muted),
                        Span::from(queue_len.to_string()).fg(theme.text_muted),
                        Span::from(" tracks)").fg(theme.text_muted),
                    ])
                }
                _ => return,
            },
            _ => return,
        };

        Line::from(spans).render(area, buf);
    }
}
