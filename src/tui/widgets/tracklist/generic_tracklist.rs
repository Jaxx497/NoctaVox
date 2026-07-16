use crate::{
    DurationStyle,
    library::SongInfo,
    tui::widgets::tracklist::{
        CellFactory, create_empty_block, create_standard_table, get_padding, get_title,
    },
    ui_state::{LayoutStyle, Pane, UiState},
};
use ratatui::{
    style::Stylize,
    widgets::{Borders, Row, StatefulWidget, TableState, Widget},
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
        let borders = match theme.border_display {
            Borders::NONE => 0,
            _ => 2,
        };
        let h = area
            .height
            .saturating_sub(borders + padding.top + padding.bottom)
            .max(1) as usize;

        let pad = ((area.height as f32 * 0.30) as usize).min(h.saturating_sub(1) / 2);
        let sel = state.nav.table_pos.selected().unwrap_or(0).min(total - 1);
        let mut offset = state.nav.table_pos.offset();
        if sel < offset + pad {
            offset = sel.saturating_sub(pad);
        }
        if sel + pad >= offset + h {
            offset = sel + pad + 1 - h;
        }
        offset = offset.min(total.saturating_sub(h));
        let end = (offset + h).min(total);

        let rows = songs[offset..end]
            .iter()
            .enumerate()
            .map(|(i, song)| {
                let idx = i + offset;
                let is_m_selected = state.get_multi_select_indices().contains(&idx);

                let index = match album {
                    Some(_) => CellFactory::track_disc_cell(theme, song, idx, is_m_selected),
                    None => CellFactory::index_cell(theme, &state.layout, idx, is_m_selected),
                };
                let symbol = CellFactory::status_cell(song, state, is_m_selected);
                let title = CellFactory::title_cell(theme, song.get_title(), is_m_selected);
                let artist = CellFactory::artist_cell(theme, song, is_m_selected);
                let filetype = CellFactory::filetype_cell(theme, song, is_m_selected);
                let duration =
                    CellFactory::duration_cell(theme, song, DurationStyle::Clean, is_m_selected);

                match state.layout {
                    LayoutStyle::Traditional => match is_m_selected {
                        true => Row::new([index, symbol, title, artist, filetype, duration])
                            .fg(theme.text_selected)
                            .bg(state.theme.active.accent_inactive),
                        false => Row::new([index, symbol, title, artist, filetype, duration]),
                    },
                    LayoutStyle::Minimal => match is_m_selected {
                        true => Row::new([index, symbol, title, duration])
                            .fg(theme.text_selected)
                            .bg(state.theme.active.accent_inactive),
                        false => Row::new([index, symbol, title, duration]),
                    },
                }
            })
            .collect::<Vec<Row>>();

        let title = get_title(state, album, area);
        let table = create_standard_table(rows, title, state, theme, area);

        state.nav.table_pos.select(Some(sel));
        *state.nav.table_pos.offset_mut() = offset;
        let mut local = TableState::default().with_selected(Some(sel - offset));
        StatefulWidget::render(table, area, buf, &mut local);
    }
}
