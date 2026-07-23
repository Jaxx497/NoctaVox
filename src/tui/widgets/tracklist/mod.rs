mod row;
mod song_table;

pub use song_table::SongTable;

use crate::{
    DurationStyle, get_readable_duration,
    theme::DisplayTheme,
    truncate_at_last_space,
    ui_state::{LayoutStyle, Mode, Pane, UiState},
};
use ratatui::{
    layout::{Constraint, Flex, HorizontalAlignment, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Row, Table},
};

// max len of `XXm XXs` or `XXh XXm` format
const DURATION_SPACING: u16 = 7;
const COLUMN_SPACING: u16 = 2;
const SCROLL_PAD: f32 = 0.25;

pub(super) const TRAD_ROW_HEIGHT: u16 = 2;
pub(super) const TRAD_ROW_MARGIN: u16 = 1;
pub(super) const TRAD_ROW_STRIDE: u16 = TRAD_ROW_HEIGHT + TRAD_ROW_MARGIN;

pub(super) fn get_widths(state: &UiState) -> Vec<Constraint> {
    match state.get_mode() {
        Mode::Power | Mode::Search => match state.layout {
            LayoutStyle::Traditional => vec![
                Constraint::Ratio(3, 9),
                Constraint::Ratio(2, 9),
                Constraint::Ratio(2, 9),
                Constraint::Length(1),
                Constraint::Length(8),
            ],
            LayoutStyle::Minimal => {
                vec![
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                ]
            }
        },
        Mode::Library | Mode::Queue => match state.layout {
            LayoutStyle::Traditional => {
                vec![Constraint::Fill(1), Constraint::Length(DURATION_SPACING)]
            }
            LayoutStyle::Minimal => {
                let count = state.get_legal_songs().len();
                let digits = count.checked_ilog10().unwrap_or(0) + 2;
                vec![
                    Constraint::Length(digits as u16),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Length(DURATION_SPACING),
                ]
            }
        },
        _ => Vec::new(),
    }
}

pub fn get_keymaps(mode: &Mode, decorator: &str) -> String {
    match mode {
        Mode::Library | Mode::Queue => {
            format!(" [q]ueue {decorator} [a]dd to playlist {decorator} [x] remove ")
        }
        _ => format!(" [q]ueue {decorator} [a]dd to playlist "),
    }
}

pub fn create_standard_table<'a>(
    rows: Vec<Row<'a>>,
    state: &UiState,
    theme: &DisplayTheme,
    area: Rect,
) -> Table<'a> {
    let mode = state.get_mode();
    let pane = state.get_pane();
    let decorator = &state.theme.icons().decorator;

    let widths = get_widths(state);
    let title = get_title(state, area).centered();
    let keymaps = match pane {
        Pane::TrackList => get_keymaps(mode, decorator),
        _ => String::default(),
    }
    .fg(theme.text_muted)
    .into_centered_line();

    let ms_count = match state.get_multi_select_indices().len() {
        0 => Line::default(),
        x => format!("{x:>3} {} ", &state.theme.icons().selected)
            .fg(theme.border)
            .into(),
    };

    let block = match state.layout {
        LayoutStyle::Traditional => Block::bordered()
            .borders(theme.border_display)
            .border_type(theme.border_type)
            .border_style(theme.border)
            .title_top(title)
            .title_bottom(keymaps)
            .title_bottom(ms_count.left_aligned())
            .padding(get_padding(state, theme, area))
            .bg(theme.bg),

        LayoutStyle::Minimal => Block::bordered()
            .borders(theme.border_display)
            .border_type(theme.border_type)
            .border_style(theme.border)
            .padding(get_padding(state, theme, area))
            .bg(theme.bg_global),
    };

    let highlight_style = match state.get_pane() {
        Pane::TrackList => Style::new().fg(theme.text_selected).bg(theme.accent),
        _ => Style::new(),
    };

    Table::new(rows, widths)
        .block(block)
        .column_spacing(COLUMN_SPACING)
        .flex(Flex::SpaceBetween)
        // .highlight_symbol(state.theme.icons().selector.to_string().fg(theme.accent))
        .row_highlight_style(highlight_style)
}

pub fn create_empty_block(theme: &DisplayTheme, title: &str) -> Block<'static> {
    Block::bordered()
        .borders(theme.border_display)
        .border_type(theme.border_type)
        .border_style(theme.border)
        .title_top(format!(" {} ", title))
        .title_alignment(HorizontalAlignment::Center)
        .bg(theme.bg)
}

fn get_title(state: &UiState, area: Rect) -> Line<'static> {
    if state.layout == LayoutStyle::Minimal {
        return Line::default();
    }

    let focus = matches!(state.get_pane(), Pane::TrackList);
    let theme = state.theme.get_display_theme(focus);
    let mode = state.get_mode();
    let decorator = &state.theme.icons().decorator;
    let total = state.get_legal_songs().len();
    let third = (area.width / 3) as usize;

    if matches!(mode, Mode::Queue | Mode::Search) {
        let count_str = match total {
            1 => "[1 Song] ".to_string(),
            _ => format!("[{total} Songs] "),
        };
        Line::from_iter([
            Span::from(match mode {
                Mode::Queue => " Queue ",
                _ => " Total: ",
            })
            .fg(theme.accent),
            count_str.fg(theme.text_muted),
        ])
    } else if let Some(album) = state.get_selected_album() {
        let album_title = match album.title.is_empty() {
            true => "[Unknown Album]".to_string(),
            false => truncate_at_last_space(&album.title, third),
        };
        let year_str = album
            .year
            .filter(|y| *y != 0)
            .map_or(String::new(), |y| format!(" [{y}]"));
        Line::from_iter([
            Span::from(format!(" {album_title}"))
                .fg(theme.text_secondary)
                .italic(),
            Span::from(year_str).fg(theme.text_muted),
            Span::from(format!(" {decorator} ")).fg(theme.text_muted),
            Span::from(album.get_album_artist().to_string()).fg(theme.accent),
            Span::from(format!(" [{total} Songs] ")).fg(theme.text_muted),
        ])
    } else {
        let name = state
            .get_selected_playlist()
            .map(|p| p.name.clone())
            .or_else(|| {
                state
                    .get_selected_group_label()
                    .map(|a| format!("{a} [ALL SONGS]"))
            });

        let dur = state.get_legal_songs_dur();
        let readable = get_readable_duration(dur, DurationStyle::Clean);
        let info = match total {
            0 => String::default(),
            1 => format!("[1 Song ⫽ {readable}] "),
            _ => format!("[{total} Songs ⫽ {readable}] "),
        };

        match name {
            Some(name) => Line::from_iter([
                " ".into(),
                truncate_at_last_space(&name, third).fg(theme.text_secondary),
                format!(" {decorator} ").fg(theme.text_muted),
                info.fg(theme.text_muted),
            ]),
            None => format!(" {info}").fg(theme.text_muted).into(),
        }
    }
}

pub(super) fn scroll_offset(
    total: usize,
    capacity: usize,
    selected: usize,
    offset: usize,
) -> usize {
    if total == 0 {
        return 0;
    }
    let pad = ((capacity as f32 * SCROLL_PAD) as usize).min(capacity.saturating_sub(1) / 2);

    let mut offset = offset;
    if selected < offset + pad {
        offset = selected.saturating_sub(pad);
    }
    if selected + pad >= offset + capacity {
        offset = selected + pad + 1 - capacity;
    }
    offset.min(total.saturating_sub(capacity))
}

fn get_padding(state: &UiState, theme: &DisplayTheme, area: Rect) -> Padding {
    match state.layout {
        LayoutStyle::Traditional => {
            let ink = state
                .get_legal_songs()
                .len()
                .saturating_mul(TRAD_ROW_STRIDE as usize);
            let border_h = if theme.has_borders() { 2 } else { 0 };
            let avail = area.height.saturating_sub(border_h) as usize;
            let top = (avail.saturating_sub(ink) / 2).max(1) as u16;

            Padding {
                left: 4,
                right: 4,
                top,
                bottom: 1,
            }
        }
        LayoutStyle::Minimal => Padding {
            left: if theme.has_borders() { 2 } else { 3 },
            right: if theme.has_borders() { 2 } else { 3 },
            top: if theme.has_borders() { 0 } else { 1 },
            bottom: if theme.has_borders() { 0 } else { 1 },
        },
    }
}
