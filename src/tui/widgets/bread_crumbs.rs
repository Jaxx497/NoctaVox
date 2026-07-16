use crate::{
    theme::fade_color,
    ui_state::{Mode, Pane, Root, UiState},
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{StatefulWidget, Widget},
};
use unicode_width::UnicodeWidthStr;

pub struct BreadCrumbs;

const CRUMB_SEP: &str = "  ";

impl StatefulWidget for BreadCrumbs {
    type State = UiState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !matches!(state.get_mode(), Mode::Queue | Mode::Library) {
            return;
        }

        let theme = state.theme.get_display_theme(true);
        let top_level = state.get_selected_root();

        let bc_highlight = fade_color(theme.dark, theme.accent, 0.85);
        let dimmed = fade_color(theme.dark, theme.text_muted, 0.75);

        let right_label = match top_level {
            Root::Library => state.nav.get_album_sort().to_string(),
            Root::Playlist => format!("{} {}", state.playlists.len(), state.theme.icons().upcoming),
        };

        let spans = match state.get_pane() {
            Pane::SideBar => {
                let padding =
                    area.width
                        .saturating_sub(top_level.label().width() as u16)
                        .saturating_sub(right_label.width() as u16) as usize;

                vec![
                    Span::from(top_level.label()).fg(bc_highlight).underlined(),
                    Span::raw(" ".repeat(padding)),
                    Span::from(right_label).fg(dimmed),
                ]
            }

            Pane::TrackList => match state.get_mode() {
                Mode::Library => {
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
                        Vec::from([
                            Span::from(artist.to_string()).fg(bc_highlight).underlined(),
                            Span::from(" [All tracks]").fg(theme.text_muted),
                        ])
                    } else {
                        Vec::new()
                    };

                    let mut spans = Vec::from([Span::from(top_level.label()).fg(theme.text_muted)]);
                    if !crumb.is_empty() {
                        spans.push(Span::from(CRUMB_SEP).fg(theme.text_muted));
                        spans.extend(crumb);
                    }
                    spans
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
