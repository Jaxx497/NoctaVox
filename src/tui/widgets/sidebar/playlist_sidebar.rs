use ratatui::{
    layout::Alignment,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, ListItem, Padding, Paragraph, StatefulWidget, Widget, Wrap},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    truncate_at_last_space,
    tui::widgets::sidebar::{KILL_WIDTH_PLAYLIST, PADDING_L, PADDING_R, create_standard_list},
    ui_state::{LayoutStyle, Pane, UiState},
};

pub struct SideBarPlaylist;
impl StatefulWidget for SideBarPlaylist {
    type State = UiState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let focus = matches!(&state.get_pane(), Pane::SideBar);
        let theme = state.theme_manager.get_display_theme(focus);
        let playlists = &state.playlists;

        if playlists.is_empty() {
            Widget::render(
                Paragraph::new("Create some playlists!\n\nPress [c] to get started!")
                    .block(
                        Block::new()
                            .borders(Borders::NONE)
                            .padding(Padding {
                                left: 2,
                                right: 2,
                                top: 5,
                                bottom: 0,
                            })
                            .bg(theme.bg),
                    )
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
                    .fg(theme.text_primary),
                area,
                buf,
            );
            return;
        }

        let padding = PADDING_L
            + PADDING_R
            + match theme.border_display {
                Borders::NONE => 0,
                _ => 2,
            };

        let list_items = playlists
            .iter()
            .map(|p| {
                let song_count = p.get_tracklist().len();
                let count_str = match area.width > KILL_WIDTH_PLAYLIST {
                    false => "".into(),
                    true => match state.layout {
                        LayoutStyle::Traditional => format!("({song_count})"),
                        LayoutStyle::Minimal => format!("{song_count}"),
                    },
                };

                let count_width = count_str.len() as u16;

                let max_name_width = area.width.saturating_sub(count_width + padding + 1) as usize;

                let name = match p.name.width() > max_name_width {
                    true => truncate_at_last_space(&p.name, max_name_width),
                    false => p.name.to_string(),
                };

                let n = area
                    .width
                    .saturating_sub(padding)
                    .saturating_sub(name.width() as u16)
                    .saturating_sub(count_width) as usize;

                ListItem::new(Line::from_iter([
                    Span::from(name).fg(theme.text_secondary),
                    Span::from(" ".repeat(n)),
                    Span::from(count_str).fg(theme.text_muted),
                ]))
            })
            .collect();

        StatefulWidget::render(
            create_standard_list(list_items, None, state, area),
            area,
            buf,
            &mut state.display_state.playlist_pos,
        );
    }
}
