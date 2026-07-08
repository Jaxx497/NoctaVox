use crate::{
    DurationStyle,
    library::SongInfo,
    tui::widgets::tracklist::{CellFactory, create_standard_table, get_title},
    ui_state::{LayoutStyle, Pane, UiState},
};
use ratatui::{
    style::Stylize,
    widgets::{Row, StatefulWidget},
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
        let songs = state.get_legal_songs();

        let rows = songs
            .iter()
            .enumerate()
            .map(|(idx, song)| {
                let is_m_selected = state.get_multi_select_indices().contains(&idx);

                let index = CellFactory::index_cell(&theme, &state.layout, idx, is_m_selected);
                let symbol = CellFactory::status_cell(song, state, is_m_selected);
                let title = CellFactory::title_cell(&theme, song.get_title(), is_m_selected);
                let artist = CellFactory::artist_cell(&theme, song, is_m_selected);
                let filetype = CellFactory::filetype_cell(&theme, song, is_m_selected);
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

        let title = get_title(state, area);

        let table = create_standard_table(rows, title, state, theme, area);
        StatefulWidget::render(table, area, buf, &mut state.nav.table_pos);
    }
}
