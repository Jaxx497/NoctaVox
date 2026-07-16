use crate::{
    DurationStyle,
    library::SongInfo,
    theme::fade_color,
    tui::widgets::tracklist::{CellFactory, create_standard_table, get_padding},
    ui_state::{LayoutStyle, MatchField, Pane, UiState},
};
use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{StatefulWidget, *},
};

pub struct SearchResults;
impl StatefulWidget for SearchResults {
    type State = UiState;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let focus = matches!(state.get_pane(), Pane::TrackList | Pane::Search);
        let theme = &state.theme.get_display_theme(focus);

        let songs = state.get_legal_songs().to_owned();
        let total = songs.len();
        let search_len = state.search.len();

        let title = match state.layout {
            LayoutStyle::Traditional => match search_len > 1 {
                true => format!(" Search Results: {} Songs ", total),
                false => format!(" Total: {} Songs ", total),
            },
            LayoutStyle::Minimal => String::new(),
        };

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
        let sel = state.nav.table_pos.selected().unwrap_or(0);
        let mut offset = state.nav.table_pos.offset();
        if sel < offset + pad {
            offset = sel.saturating_sub(pad);
        }
        if sel + pad >= offset + h {
            offset = sel + pad + 1 - h;
        }
        offset = offset.min(total.saturating_sub(h));
        let end = (offset + h).min(total);

        let inactive = fade_color(theme.dark, theme.text_primary, 0.6);
        let rows = songs[offset..end]
            .iter()
            .enumerate()
            .map(|(i, song)| {
                let idx = i + offset;
                let idx = Cell::from((idx + 1).to_string()).fg(inactive);
                let symbol = CellFactory::status_cell(song, state, false);
                let mut title_col = Cell::from(song.get_title()).fg(inactive);
                let mut artist_col = Cell::from(song.get_artist()).fg(inactive);
                let mut album_col = Cell::from(song.get_album()).fg(inactive);
                let dur_col = Cell::from(
                    Line::from(song.get_duration_str(DurationStyle::Compact)).right_aligned(),
                )
                .fg(inactive);

                if let Some(field) = state.get_match_fields(song.id) {
                    match field {
                        MatchField::Title => title_col = title_col.fg(theme.text_secondary),
                        MatchField::Artist => artist_col = artist_col.fg(theme.text_secondary),
                        MatchField::Album => album_col = album_col.fg(theme.text_secondary),
                    }
                }

                match state.layout {
                    LayoutStyle::Traditional => {
                        Row::new([idx, symbol, title_col, artist_col, album_col, dur_col])
                    }
                    LayoutStyle::Minimal => Row::new([title_col, artist_col, album_col]),
                }
            })
            .collect::<Vec<Row>>();

        let table = create_standard_table(rows, title.into(), state, theme, area);

        state.nav.table_pos.select(Some(sel));
        *state.nav.table_pos.offset_mut() = offset;
        let mut local = TableState::default().with_selected(Some(sel - offset));

        StatefulWidget::render(table, area, buf, &mut local);
    }
}
