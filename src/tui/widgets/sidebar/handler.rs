use crate::{
    library::Album,
    theme::{Bar, DisplayTheme, animation_time, fade_color},
    truncate_at_last_space,
    tui::widgets::sidebar::{
        KILL_WIDTH_ALBUM, KILL_WIDTH_PLAYLIST, create_sidebar_block, create_standard_list,
        get_padding,
    },
    ui_state::{AlbumSort, LayoutStyle, Pane, RowKind, SidebarRow, UiState},
};
use ratatui::{
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{ListItem, StatefulWidget},
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

        let block = create_sidebar_block(state, theme, area);
        let inner = block.inner(area);
        let accent = theme.accent;

        let list = create_standard_list(items, theme, area, block);
        StatefulWidget::render(list, area, buf, &mut state.nav.sidebar.pos);

        if focus && let Some(rect) = selected_row_rect(state, inner) {
            Bar::SELECTION.paint(buf, rect, accent, animation_time());
        }
    }
}

fn selected_row_rect(state: &UiState, inner: Rect) -> Option<Rect> {
    let rows = &state.nav.sidebar.rows;
    let selected = state.nav.sidebar.pos.selected()?;
    let offset = state.nav.sidebar.pos.offset();

    let above: u16 = rows
        .get(offset..selected)?
        .iter()
        .map(|row| state.row_height(row))
        .sum();

    let y = inner.y + above;
    let height = state
        .row_height(rows.get(selected)?)
        .min(inner.bottom().saturating_sub(y));

    Some(Rect { y, height, ..inner })
}

fn render_row(
    row: &SidebarRow,
    state: &UiState,
    area: Rect,
    theme: &DisplayTheme,
) -> ListItem<'static> {
    let icons = state.theme.icons();
    let key = row.collapse_key();
    let folded = key
        .as_ref()
        .is_some_and(|k| state.nav.sidebar.collapsed.contains(k));

    let glyph: &str = match (key.is_some(), folded) {
        (false, _) => " ",
        (true, true) => &icons.collapsed,
        (true, false) => &icons.expanded,
    };
    let prefix = row_prefix(row.depth, glyph);
    let prefix_w = prefix.width() as u16;

    let p = get_padding(&state.layout, theme.border_display);
    let padding = p.left + p.right + (if theme.has_borders() { 2 } else { 0 });

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

        RowKind::LoneAlbum { artist, id } => {
            let Some(album) = state.library().albums.get(id) else {
                return ListItem::new("");
            };

            let artist_line = Line::from_iter([
                Span::from(prefix).fg(theme.text_muted),
                Span::from(artist.to_string()).fg(theme.text_secondary),
            ]);

            match folded {
                true => ListItem::new(artist_line),
                false => ListItem::new(vec![
                    artist_line,
                    album_line(
                        album,
                        row_prefix(row.depth + 1, " "),
                        padding,
                        area,
                        state,
                        theme,
                    ),
                ]),
            }
        }

        RowKind::Album(id) => {
            let Some(album) = state.library().albums.get(id) else {
                return ListItem::new("");
            };

            ListItem::new(album_line(album, prefix, padding, area, state, theme))
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

fn row_prefix(depth: u8, glyph: &str) -> String {
    let indent = " ".repeat((depth as usize * 2).saturating_sub(1));
    format!("{indent}{glyph} ")
}

fn album_line(
    album: &Album,
    prefix: String,
    padding: u16,
    area: Rect,
    state: &UiState,
    theme: &DisplayTheme,
) -> Line<'static> {
    let prefix_w = prefix.width() as u16;

    let mut year = album.year.map_or("????".to_string(), |y| format!("{y}"));
    let mut album_title = match album.title.is_empty() {
        true => album.get_album_artist().to_string() + " [Unknown Album]",
        false => album.title.to_string(),
    };

    match area.width < KILL_WIDTH_ALBUM {
        true => year.clear(),
        false => {
            album_title = truncate_at_last_space(
                &album_title,
                area.width.saturating_sub(prefix_w + padding + 5) as usize,
            );
        }
    }

    let gap = area
        .width
        .saturating_sub((album_title.width() + year.width()) as u16 + padding + prefix_w)
        as usize;

    match state.nav.get_album_sort() {
        AlbumSort::Title => Line::from_iter([
            Span::from(prefix),
            Span::from(year).fg(theme.text_muted),
            Span::from(format!(" {} ", state.theme.icons().decorator)).fg(theme.text_muted),
            Span::from(album_title).fg(theme.text_secondary),
        ]),
        AlbumSort::Year => Line::from_iter([
            Span::from(prefix),
            Span::from(year).fg(theme.text_secondary),
            Span::from(format!(" {} ", state.theme.icons().decorator)).fg(theme.text_muted),
            Span::from(album_title).fg(fade_color(theme.dark, theme.text_primary, 0.8)),
        ]),
        AlbumSort::Artist => Line::from_iter([
            Span::from(prefix),
            Span::from(album_title).fg(theme.text_primary),
            Span::from(" ".repeat(gap)),
            Span::from(year).fg(fade_color(theme.dark, theme.accent, 0.7)),
        ]),
    }
}
