use crate::{
    DurationStyle,
    library::SongInfo,
    theme::fade_color,
    tui::widgets::tracklist::{
        CellFactory, TRAD_ROW_HEIGHT, TRAD_ROW_MARGIN, TRAD_ROW_STRIDE, create_empty_block,
        create_standard_table, get_padding, scroll_offset,
    },
    ui_state::{LayoutStyle, Pane, UiState},
};
use ratatui::{
    style::Stylize,
    text::{Line, Span, Text},
    widgets::{Cell, Row, StatefulWidget, TableState, Widget},
};

pub struct GenericView;
impl StatefulWidget for GenericView {
    type State = UiState;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let focus = matches!(state.get_pane(), Pane::TrackList);
        let theme = &state.theme.get_display_theme(focus);

        let album = state.get_selected_album();
        let songs = state.get_legal_songs();

        if songs.is_empty() {
            return create_empty_block(theme, "").render(area, buf);
        }

        let total = songs.len();
        let padding = get_padding(state, theme, area);
        let borders = if theme.has_borders() { 2 } else { 0 };

        let row_height = match state.layout {
            LayoutStyle::Traditional => TRAD_ROW_STRIDE,
            _ => 1,
        } as usize;

        let h = (area
            .height
            .saturating_sub(borders + padding.top + padding.bottom)
            .max(1) as usize)
            .div_ceil(row_height)
            .max(1);

        // Scroll-off margin, in songs, kept above/below the selection.
        let sel = state.nav.table_pos.selected().unwrap_or(0);
        let offset = scroll_offset(total, h, sel, state.nav.table_pos.offset());
        let end = (offset + h).min(total);

        let rows = songs[offset..end]
            .iter()
            .enumerate()
            .map(|(i, song)| {
                let idx = i + offset;
                let is_m_selected = state.get_multi_select_indices().contains(&idx);

                match state.layout {
                    LayoutStyle::Traditional => {
                        let ms = is_m_selected;
                        let c_title = match ms {
                            true => theme.text_selected,
                            false => theme.text_primary,
                        };
                        let c_artist = match ms {
                            true => theme.text_selected,
                            false => fade_color(theme.dark, theme.text_secondary, 0.8),
                        };
                        let c_muted = match ms {
                            true => theme.text_selected,
                            false => theme.text_muted,
                        };
                        let c_num = match ms {
                            true => theme.text_selected,
                            false => fade_color(theme.dark, theme.accent, 0.8),
                        };

                        // Left column: status + title + format, artist beneath.
                        let mut title_line = Vec::new();
                        match CellFactory::status_icon(song, state, ms) {
                            Some(icon) => {
                                title_line.push(Span::raw(" "));
                                title_line.push(icon);
                                title_line.push(Span::raw("  "));
                            }
                            None => title_line.push(Span::raw("    ")),
                        }
                        title_line.push(Span::raw(song.get_title().to_string()).fg(c_title).bold());
                        title_line.push(Span::raw("  "));
                        title_line.push(Span::raw(song.filetype.as_str_label()).fg(c_muted));

                        // Artist line carries the track/disc number right beside it.
                        let number = CellFactory::track_disc_super(song, idx, album.is_some());
                        let artist_line = Line::from(vec![
                            Span::raw("    "),
                            Span::raw(number).fg(c_num),
                            Span::raw(format!(" {} ", state.theme.icons().decorator)).fg(c_muted),
                            Span::raw(song.get_artist().to_string()).fg(c_artist),
                        ]);

                        let left =
                            Cell::from(Text::from(vec![Line::from(title_line), artist_line]));

                        // Right column: only the duration, right-aligned to the edge.
                        let right = Cell::from(
                            Line::from(song.get_duration_str(DurationStyle::Clean))
                                .fg(c_muted)
                                .right_aligned(),
                        );

                        let row = Row::new([left, right])
                            .height(TRAD_ROW_HEIGHT)
                            .bottom_margin(TRAD_ROW_MARGIN);

                        match ms {
                            true => row.bg(state.theme.active.accent_inactive),
                            false => row,
                        }
                    }
                    LayoutStyle::Minimal => {
                        let symbol = CellFactory::status_cell(song, state, is_m_selected);
                        let title = CellFactory::title_cell(theme, song.get_title(), is_m_selected);
                        let duration = CellFactory::duration_cell(
                            theme,
                            song,
                            DurationStyle::Clean,
                            is_m_selected,
                        );

                        match is_m_selected {
                            true => Row::new([title, symbol, duration])
                                .fg(theme.text_selected)
                                .bg(state.theme.active.accent_inactive),
                            false => Row::new([title, symbol, duration]),
                        }
                    }
                }
            })
            .collect::<Vec<Row>>();

        let table = create_standard_table(rows, state, theme, area);

        state.nav.table_pos.select(Some(sel));
        *state.nav.table_pos.offset_mut() = offset;
        let mut local = TableState::default().with_selected(Some(sel - offset));
        StatefulWidget::render(table, area, buf, &mut local);
    }
}
