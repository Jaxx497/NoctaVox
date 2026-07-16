use crate::{
    tui::widgets::tracklist::{GenericView, SearchResults},
    ui_state::{Mode, UiState},
};
use ratatui::widgets::StatefulWidget;

pub struct SongTable;
impl StatefulWidget for SongTable {
    type State = UiState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        match state.get_mode() {
            Mode::Library | Mode::Queue => GenericView.render(area, buf, state),
            _ => SearchResults.render(area, buf, state),
        }
    }
}
