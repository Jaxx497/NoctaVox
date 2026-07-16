use crate::{
    theme::{DisplayTheme, fade_color},
    truncate_at_last_space,
    tui::widgets::sidebar::{
        KILL_WIDTH_ALBUM, KILL_WIDTH_PLAYLIST, PADDING_L, PADDING_R, create_standard_list,
    },
    ui_state::{AlbumSort, LayoutStyle, Pane, RowKind, SidebarRow, UiState},
};
use ratatui::{
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{Borders, ListItem, StatefulWidget},
};
use unicode_width::UnicodeWidthStr;

pub struct SideBarHandler;
impl StatefulWidget for SideBarHandler {
    type State = UiState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let focus = matches!(&state.get_pane(), Pane::SideBar);
        let theme = state.theme.get_display_theme(focus);

        let mut items = Vec::with_capacity(state.nav.sidebar.rows.len());
        for row in &state.nav.sidebar.rows {
            items.push(render_row(row, state, area, theme));
        }

        let list = create_standard_list(items, state, area);
        StatefulWidget::render(list, area, buf, &mut state.nav.sidebar.pos);
    }
}

fn render_row(
    row: &SidebarRow,
    state: &UiState,
    area: Rect,
    theme: &DisplayTheme,
) -> ListItem<'static> {
    let indent = " ".repeat((row.depth as usize * 2).saturating_sub(1));

    let glyph = match row.collapse_key() {
        Some(k) => match state.nav.sidebar.collapsed.contains(&k) {
            true => state.theme.icons().collapsed.to_string(),
            false => state.theme.icons().expanded.to_string(),
        },
        None => " ".to_string(),
    };
    let prefix = format!("{indent}{glyph} ");
    let prefix_w = prefix.width() as u16;

    let padding = PADDING_L
        + PADDING_R
        + match theme.border_display {
            Borders::NONE => 0,
            _ => 2,
        };

    match &row.kind {
        RowKind::Category(root) => {
            let label = root.label().to_uppercase();

            ListItem::new(Line::from_iter([
                Span::from(prefix).fg(theme.text_muted),
                Span::from(label).underlined().fg(theme.accent).bold(),
            ]))
        }

        RowKind::Artist { name, .. } => {
            let inside = state
                .get_selected_album()
                .is_some_and(|a| a.artist == *name);
            let name_span = match inside {
                true => Span::from(name.to_string())
                    .fg(theme.text_secondary)
                    .underlined(),
                false => Span::from(name.to_string()).fg(theme.text_secondary),
            };

            ListItem::new(Line::from_iter([
                Span::from(prefix).fg(theme.text_muted),
                name_span,
            ]))
        }

        RowKind::Album(id) => {
            let Some(album) = state.library().albums.get(id) else {
                return ListItem::new("");
            };

            let mut year = album.year.map_or("????".to_string(), |y| format!("{y}"));
            let album_title = match album.title.is_empty() {
                true => album.get_album_artist().to_string() + " [Unknown Album]",
                false => album.title.to_string(),
            };

            match state.layout {
                LayoutStyle::Minimal => {
                    if area.width < KILL_WIDTH_ALBUM {
                        year.clear();
                    }

                    let gap = area.width.saturating_sub(
                        (album_title.width() + year.width()) as u16 + padding + prefix_w,
                    ) as usize;

                    ListItem::new(Line::from_iter([
                        Span::from(prefix),
                        Span::from(album_title).fg(theme.text_primary),
                        Span::from(" ".repeat(gap)),
                        Span::from(year).fg(fade_color(theme.dark, theme.accent, 0.7)),
                    ]))
                }

                LayoutStyle::Traditional => {
                    let year_color = match state.nav.get_album_sort() {
                        AlbumSort::Artist => theme.text_muted,
                        _ => theme.text_secondary,
                    };
                    let decorator = &state.theme.icons().decorator;

                    ListItem::new(Line::from_iter([
                        Span::from(prefix),
                        Span::from(format!("{year: >4} ")).fg(year_color),
                        Span::from(format!("{decorator} ")).fg(theme.text_muted),
                        Span::from(album_title).fg(theme.text_primary),
                    ]))
                }
            }
        }

        RowKind::Playlist(id) => {
            let Some(playlist) = state.playlists.get(id) else {
                return ListItem::new("");
            };

            let song_count = playlist.len();
            let count_str = match area.width > KILL_WIDTH_PLAYLIST {
                false => String::new(),
                true => match state.layout {
                    LayoutStyle::Traditional => format!("({song_count})"),
                    LayoutStyle::Minimal => format!("{song_count}"),
                },
            };
            let count_w = count_str.width() as u16;

            let max_name_width =
                area.width.saturating_sub(count_w + padding + prefix_w + 1) as usize;
            let name = match playlist.name.width() > max_name_width {
                true => truncate_at_last_space(&playlist.name, max_name_width),
                false => playlist.name.to_string(),
            };

            let gap = area
                .width
                .saturating_sub(padding)
                .saturating_sub(prefix_w)
                .saturating_sub(name.width() as u16)
                .saturating_sub(count_w) as usize;

            ListItem::new(Line::from_iter([
                Span::from(prefix),
                Span::from(name).fg(theme.text_secondary),
                Span::from(" ".repeat(gap)),
                Span::from(count_str).fg(theme.text_muted),
            ]))
        }
    }
}
