mod handler;

pub use handler::SideBarHandler;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, Padding},
};

const KILL_WIDTH_ALBUM: u16 = 42;
const KILL_WIDTH_PLAYLIST: u16 = 25;
const PADDING_L: u16 = 1;
const PADDING_R: u16 = 2;

use crate::ui_state::{LayoutStyle, Pane, Root, UiState};

pub fn create_standard_list<'a>(
    list_items: Vec<ListItem<'a>>,
    state: &UiState,
    area: Rect,
) -> List<'a> {
    let focus = matches!(&state.get_pane(), Pane::SideBar);
    let layout = &state.layout;
    let theme = state.theme.get_display_theme(focus);

    let title = state
        .selected_row()
        .map(|row| {
            let root = row.root();
            let count = match root {
                Root::Library => state.albums.len(),
                Root::Playlist => state.playlists.len(),
            };
            Line::from(format!(" ⟪ {} {} ⟫ ", count, root.label()))
        })
        .unwrap_or_default()
        .left_aligned()
        .fg(theme.accent);

    let sorting_title = match state.get_selected_root() {
        Root::Library => Line::from(format!(" {:<10}", state.nav.get_album_sort().to_string()))
            .right_aligned()
            .fg(theme.text_secondary),
        _ => Line::default(),
    };

    let keymaps = if focus {
        match state.get_selected_root() {
            Root::Library => Line::from(" [q]ueue "),
            Root::Playlist => {
                let decorator = &state.theme.icons().decorator;
                let playlist_keymaps =
                    format!(" [q]ueue {decorator} [c]reate {decorator} [x] delete ");
                match area.width as usize + 4 < playlist_keymaps.len() {
                    true => Line::default(),
                    false => Line::from(playlist_keymaps),
                }
            }
        }
    } else {
        Line::default()
    };

    let block = match layout {
        LayoutStyle::Traditional => Block::bordered()
            .borders(theme.border_display)
            .border_type(theme.border_type)
            .border_style(theme.border)
            .bg(theme.bg)
            .title_top(title)
            .title_top(sorting_title)
            .title_bottom(keymaps.centered().fg(theme.text_muted))
            .padding(get_padding(layout, theme.border_display)),
        LayoutStyle::Minimal => Block::bordered()
            .borders(theme.border_display)
            .border_type(theme.border_type)
            .border_style(theme.border)
            .bg(theme.bg_global)
            .padding(get_padding(layout, theme.border_display)),
    };

    List::new(list_items)
        .block(block)
        .highlight_style(Style::new().fg(theme.text_selected).bg(theme.accent))
        .scroll_padding((area.height as f32 * 0.25) as usize)
        .highlight_spacing(HighlightSpacing::Always)
}

fn get_padding(layout: &LayoutStyle, borders: Borders) -> Padding {
    let borders = borders != Borders::NONE;
    let v_pad = if borders { 0 } else { 1 };
    match layout {
        LayoutStyle::Traditional => Padding {
            left: PADDING_L,
            right: PADDING_R,
            top: 1,
            bottom: 0,
        },
        LayoutStyle::Minimal => Padding {
            left: if borders { PADDING_L } else { 2 },
            right: if borders { PADDING_R } else { 3 },
            top: v_pad,
            bottom: v_pad,
        },
    }
}
