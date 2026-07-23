use crate::{
    tui::widgets::tracklist::{
        TRAD_ROW_STRIDE, create_empty_block, create_standard_table, get_padding,
        row::{RowCtx, build_row},
        scroll_offset,
    },
    ui_state::{LayoutStyle, Mode, Pane, UiState},
};
use ratatui::widgets::{Row, StatefulWidget, TableState, Widget};

pub struct SongTable;
impl StatefulWidget for SongTable {
    type State = UiState;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let focus = matches!(state.get_pane(), Pane::TrackList | Pane::Search);
        let theme = state.theme.get_display_theme(focus);
        let songs = state.get_legal_songs();
        if songs.is_empty() && matches!(state.get_mode(), Mode::Library | Mode::Queue) {
            return Widget::render(create_empty_block(&theme, ""), area, buf);
        }

        let ctx = RowCtx::new(state);

        let total = songs.len();
        let padding = get_padding(state, theme, area);
        let borders = if theme.has_borders() { 2 } else { 0 };
        let row_height = match (ctx.layout, &ctx.mode) {
            (LayoutStyle::Traditional, Mode::Library | Mode::Queue) => TRAD_ROW_STRIDE,
            _ => 1,
        } as usize;

        let capacity = (area
            .height
            .saturating_sub(borders + padding.top + padding.bottom)
            .max(1) as usize)
            .div_ceil(row_height)
            .max(1);
        let sel = state.nav.table_pos.selected().unwrap_or(0);
        let offset = scroll_offset(total, capacity, sel, state.nav.table_pos.offset());
        let end = (offset + capacity).min(total);

        let rows: Vec<Row> = songs[offset..end]
            .iter()
            .enumerate()
            .map(|(i, song)| build_row(&ctx, &song, i + offset))
            .collect();

        let table = create_standard_table(rows, state, theme, area);
        state.nav.table_pos.select(Some(sel));
        *state.nav.table_pos.offset_mut() = offset;
        let mut local = TableState::default().with_selected(Some(sel.saturating_sub(offset)));
        StatefulWidget::render(table, area, buf, &mut local);
    }
}
