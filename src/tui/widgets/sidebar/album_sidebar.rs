use crate::{
    theme::fade_color,
    tui::widgets::sidebar::{KILL_WIDTH_ALBUM, PADDING_L, PADDING_R, create_standard_list},
    ui_state::{AlbumSort, LayoutStyle, Pane, UiState},
};
use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Borders, ListItem, ListState, StatefulWidget},
};
use unicode_width::UnicodeWidthStr;

pub struct SideBarAlbum;
impl StatefulWidget for SideBarAlbum {
    type State = UiState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let focus = matches!(&state.get_pane(), Pane::SideBar);
        let theme = &state.theme.get_display_theme(focus);

        let albums = &state.albums;
        let album_sort = state.nav.get_album_sort();
        let sort_label = format!(" {:<10}", album_sort.to_string());

        let selected_album_idx = state.nav.album_pos.selected();
        let selected_artist = state.get_selected_album().map(|a| a.get_album_artist());

        let mut list_items = Vec::with_capacity(albums.len());
        let mut current_artist = None;
        let mut current_display_idx = 0;
        let mut selected_display_idx = None;

        for (idx, album) in albums.iter().enumerate() {
            // Add header if artist changed (only for Artist sort)
            if album_sort == AlbumSort::Artist
                && current_artist.as_ref() != Some(&album.get_album_artist())
            {
                let artist_str = album.get_album_artist();
                let is_selected_artist = selected_artist == Some(artist_str);

                let header_style = match is_selected_artist {
                    true => Style::default().fg(theme.text_secondary).underlined(),
                    false => Style::default().fg(theme.text_secondary),
                };

                list_items.push(ListItem::new(Span::from(artist_str).style(header_style)));

                current_artist = Some(artist_str);
                current_display_idx += 1;
            }

            let mut year = album.year.map_or("????".to_string(), |y| format!("{y}"));
            let year_color = match album_sort {
                AlbumSort::Artist => theme.text_muted,
                _ => theme.text_secondary,
            };

            let indent_len = match album_sort.eq(&AlbumSort::Artist) {
                true => 2,
                false => 0,
            };

            let is_selected = selected_album_idx == Some(idx);
            if is_selected {
                selected_display_idx = Some(current_display_idx);
            }
            let decorator = &state.theme.icons().decorator;

            let album_title = match album.title.is_empty() {
                true => album.get_album_artist().to_string() + " [Unknown Album]",
                false => album.title.to_string(),
            };

            match state.layout {
                LayoutStyle::Minimal => {
                    let padding = PADDING_L
                        + PADDING_R
                        + match theme.border_display {
                            Borders::NONE => 0,
                            _ => 2,
                        };

                    if area.width < KILL_WIDTH_ALBUM {
                        year.clear();
                    }
                    list_items.push(ListItem::new(Line::from_iter([
                        Span::from(" ".repeat(indent_len)),
                        Span::from(album_title.to_string()).fg(theme.text_primary),
                        Span::from(" ".repeat(area.width.saturating_sub(
                            (album_title.width() + year.width()) as u16
                                + padding
                                + indent_len as u16,
                        ) as usize)),
                        Span::from(year).fg(fade_color(theme.dark, theme.accent, 0.7)),
                    ])))
                }

                LayoutStyle::Traditional => list_items.push(ListItem::new(Line::from_iter([
                    Span::from(format!("{}{: >4} ", " ".repeat(indent_len), year)).fg(year_color),
                    Span::from(format!("{decorator} ")).fg(theme.text_muted),
                    Span::from(album_title).fg(theme.text_primary),
                ]))),
            };

            current_display_idx += 1;
        }

        // Temp state for rendering with display index
        let mut render_state = ListState::default();
        render_state.select(selected_display_idx);

        // Sync offset to ensure selection is visible
        if let Some(idx) = selected_display_idx {
            let current_offset = state.nav.album_pos.offset();
            let visible_height = area.height.saturating_sub(4) as usize;

            if idx < current_offset {
                *render_state.offset_mut() = idx;
            } else if idx >= current_offset + visible_height {
                *render_state.offset_mut() = idx.saturating_sub(visible_height.saturating_sub(1));
            } else {
                *render_state.offset_mut() = current_offset;
            }
        }

        let sorting_title = Some(
            Line::from(sort_label)
                .right_aligned()
                .fg(theme.text_secondary),
        );

        create_standard_list(list_items, sorting_title, state, area).render(
            area,
            buf,
            &mut render_state,
        );

        // Sync offset back
        *state.nav.album_pos.offset_mut() = render_state.offset();
    }
}
