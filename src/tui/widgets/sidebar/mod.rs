mod album_sidebar;
mod handler;
mod playlist_sidebar;

pub use album_sidebar::SideBarAlbum;
pub use handler::SideBarHandler;
pub use playlist_sidebar::SideBarPlaylist;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, Padding},
};

const KILL_WIDTH_ALBUM: u16 = 40;
const KILL_WIDTH_PLAYLIST: u16 = 25;
const PADDING_L: u16 = 3;
const PADDING_R: u16 = 2;

use crate::ui_state::{LayoutStyle, LibraryView, Pane, UiState};

pub fn create_standard_list<'a>(
    list_items: Vec<ListItem<'a>>,
    sorting_title: Option<Line<'a>>,
    state: &UiState,
    area: Rect,
) -> List<'a> {
    let focus = matches!(&state.get_pane(), Pane::SideBar);
    let theme = state.theme_manager.get_display_theme(focus);

    let (sidebar_type, count) = state.get_sidebar_details();

    let title = Line::from(format!(" ⟪ {} {} ⟫ ", count, sidebar_type))
        .left_aligned()
        .fg(theme.accent);

    let keymaps = if state.get_pane() == Pane::SideBar {
        match state.display_state.sidebar_view {
            LibraryView::Albums => Line::from(" [q] Queue Album ")
                .centered()
                .fg(theme.text_muted),
            LibraryView::Playlists => {
                let playlist_keymaps = " [c]reate 󰲸 | [^D]elete 󰐓 ";
                match area.width as usize + 2 < playlist_keymaps.len() {
                    true => Line::default(),
                    false => Line::from(playlist_keymaps).centered().fg(theme.text_muted),
                }
            }
        }
    } else {
        Line::default()
    };

    let block = match state.get_layout() {
        LayoutStyle::Traditional => Block::bordered()
            .borders(theme.border_display)
            .border_type(theme.border_type)
            .border_style(theme.border)
            .bg(theme.bg)
            .title_top(title)
            .title_top(sorting_title.unwrap_or_default())
            .title_bottom(Line::from(keymaps).centered().fg(theme.text_muted))
            .padding(get_padding(state.get_layout(), theme.border_display)),
        LayoutStyle::Minimal => Block::bordered()
            .borders(theme.border_display)
            .border_type(theme.border_type)
            .border_style(theme.border)
            .bg(theme.bg_global)
            .padding(get_padding(state.get_layout(), theme.border_display)),
    };

    List::new(list_items)
        .block(block)
        .highlight_style(Style::new().fg(theme.text_selected).bg(theme.accent))
        .scroll_padding((area.height as f32 * 0.15) as usize)
        .highlight_spacing(HighlightSpacing::Always)
}

fn get_padding(layout: &LayoutStyle, borders: Borders) -> Padding {
    let v_pad = match borders {
        Borders::NONE => 0,
        _ => 1,
    };
    match layout {
        LayoutStyle::Traditional => Padding {
            left: PADDING_L,
            right: PADDING_R,
            top: 1,
            bottom: v_pad,
        },
        LayoutStyle::Minimal => Padding {
            left: PADDING_L,
            right: PADDING_R,
            top: match borders {
                Borders::NONE => 1,
                _ => 0,
            },
            bottom: 0,
        },
    }
}
