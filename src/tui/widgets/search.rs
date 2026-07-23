use crate::ui_state::{LayoutStyle, Pane, UiState};
use ratatui::{
    style::Stylize,
    widgets::{Block, Borders, Padding, StatefulWidget, Widget},
};

pub struct SearchBar;

impl StatefulWidget for SearchBar {
    type State = UiState;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let focus = matches!(&state.get_pane(), Pane::Search);
        let layout = &state.layout;
        let theme = &state.theme.get_display_theme(focus);
        let (border_display, border_type, border_style, highlight, bg) = {
            (
                theme.border_display,
                theme.border_type,
                theme.border,
                theme.accent,
                match layout {
                    LayoutStyle::Minimal => theme.bg,
                    LayoutStyle::Traditional => theme.bg,
                },
            )
        };

        let (pd_y, pd_x) = match theme.border_display {
            Borders::NONE => (
                match layout {
                    LayoutStyle::Traditional => 2,
                    LayoutStyle::Minimal => 1,
                },
                3,
            ),
            _ => (1, 2),
        };

        let search = state.search.get_widget_mut();
        search.set_block(
            Block::bordered()
                .borders(border_display)
                .border_type(border_type)
                .border_style(border_style)
                .padding(Padding {
                    left: pd_x,
                    right: 0,
                    top: pd_y,
                    bottom: 0,
                })
                .fg(highlight)
                .bg(bg),
        );

        search.render(area, buf);
    }
}
