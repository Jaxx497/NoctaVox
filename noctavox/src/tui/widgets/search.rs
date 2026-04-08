use crate::ui_state::{LayoutStyle, Pane, UiState, fade_color};
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
        let theme = &state.theme_manager.get_display_theme(focus);
        let (border_display, border_type, border_style, highlight, bg) = {
            (
                theme.border_display,
                theme.border_type,
                theme.border,
                theme.accent,
                fade_color(theme.dark, theme.bg, 0.8),
            )
        };

        let x = match state.layout {
            LayoutStyle::Traditional => 1,
            LayoutStyle::Minimal => 0,
        };

        let (pd_v, pd_h) = match theme.border_display {
            Borders::NONE => (1 + x, 3),
            _ => (1, 2),
        };

        let search = state.get_search_widget();
        search.set_block(
            Block::bordered()
                .borders(border_display)
                .border_type(border_type)
                .border_style(border_style)
                .padding(Padding {
                    left: pd_h,
                    right: 0,
                    top: pd_v,
                    bottom: 0,
                })
                .fg(highlight)
                .bg(bg),
        );

        search.render(area, buf);
    }
}
