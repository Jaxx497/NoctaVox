use crate::{
    theme::{Bar, animation_time},
    tui::widgets::tracklist::{
        TRAD_ROW_HEIGHT, TRAD_ROW_STRIDE, create_empty_block, create_standard_table,
        create_table_block, get_padding,
        row::{RowCtx, build_row},
        scroll_offset,
    },
    ui_state::{LayoutStyle, Mode, Pane, UiState},
};
use ratatui::{
    layout::Rect,
    widgets::{Row, StatefulWidget, TableState, Widget},
};

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
        let tracklist = matches!(&ctx.mode, Mode::Library | Mode::Queue);
        let trad_rows = tracklist && matches!(ctx.layout, LayoutStyle::Traditional);
        let (row_height, bar_height) = match trad_rows {
            true => (TRAD_ROW_STRIDE, TRAD_ROW_HEIGHT),
            false => (1, 1),
        };

        let capacity = (area
            .height
            .saturating_sub(borders + padding.top + padding.bottom)
            .max(1) as usize)
            .div_ceil(row_height as usize)
            .max(1);
        let sel = state.nav.table_pos.selected().unwrap_or(0);
        let offset = scroll_offset(total, capacity, sel, state.nav.table_pos.offset());
        let end = (offset + capacity).min(total);

        let rows: Vec<Row> = songs[offset..end]
            .iter()
            .enumerate()
            .map(|(i, song)| build_row(&ctx, &song, i + offset))
            .collect();

        let block = create_table_block(state, theme, area);
        let rows_area = block.inner(area);

        let table = create_standard_table(rows, state, theme, block);
        state.nav.table_pos.select(Some(sel));
        *state.nav.table_pos.offset_mut() = offset;
        let mut local = TableState::default().with_selected(Some(sel.saturating_sub(offset)));
        StatefulWidget::render(table, area, buf, &mut local);

        if tracklist && matches!(state.get_pane(), Pane::TrackList) {
            let above = sel.saturating_sub(offset).saturating_sub(local.offset()) as u16;
            let top = rows_area.y + above * row_height;
            let height = (top + bar_height)
                .min(rows_area.bottom())
                .saturating_sub(top);

            Bar::SELECTION.paint(
                buf,
                Rect {
                    y: top,
                    height,
                    ..rows_area
                },
                theme.accent,
                animation_time(),
            );
        }
    }
}
