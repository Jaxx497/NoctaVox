mod generic_tracklist;
mod search_results;

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
    time::Duration,
};

pub use generic_tracklist::GenericView;
pub use search_results::SearchResults;

use crate::{
    DurationStyle, get_readable_duration,
    library::{SimpleSong, SongInfo},
    theme::DisplayTheme,
    truncate_at_last_space,
    ui_state::{LayoutStyle, Mode, Pane, UiState},
};
use ratatui::{
    layout::{Constraint, Flex, HorizontalAlignment, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Cell, Padding, Row, Table},
};

// 7 matches the `xxM xxS` or `xxH xxM` format
const DURATION_SPACING: u16 = 7;
const COLUMN_SPACING: u16 = 2;
const SCROLL_PAD: f32 = 0.20;

// Traditional layout stacks song info: `TRAD_ROW_HEIGHT` content lines plus a
// blank `TRAD_ROW_MARGIN` gap between entries. The two together are the vertical
// stride of one song, used for both rendering and scroll/padding math.
pub(super) const TRAD_ROW_HEIGHT: u16 = 2;
pub(super) const TRAD_ROW_MARGIN: u16 = 1;
pub(super) const TRAD_ROW_STRIDE: u16 = TRAD_ROW_HEIGHT + TRAD_ROW_MARGIN;

pub(super) fn get_widths(state: &UiState) -> Vec<Constraint> {
    let layout = &state.layout;

    match state.get_mode() {
        Mode::Power | Mode::Search => match layout {
            LayoutStyle::Traditional => vec![
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Ratio(3, 9),
                Constraint::Ratio(2, 9),
                Constraint::Ratio(2, 9),
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
        Mode::Library | Mode::Queue => match layout {
            LayoutStyle::Traditional => {
                vec![Constraint::Fill(1), Constraint::Length(DURATION_SPACING)]
            }
            LayoutStyle::Minimal => vec![
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(DURATION_SPACING),
            ],
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

    let title = get_title(state, area);
    let widths = get_widths(state);
    let keymaps = match pane {
        Pane::TrackList => get_keymaps(mode, decorator),
        _ => String::default(),
    };

    let selected = &state.theme.icons().selected;

    let ms_count = match state.get_multi_select_indices().len() {
        0 => Line::default(),
        x => format!("{x:>3} {selected} ").fg(theme.border).into(),
    };

    let block = match state.layout {
        LayoutStyle::Traditional => Block::bordered()
            .borders(theme.border_display)
            .border_type(theme.border_type)
            .border_style(theme.border)
            .title_top(title.centered())
            .title_bottom(Line::from(keymaps.fg(theme.text_muted)).centered())
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

pub struct CellFactory;

impl CellFactory {
    /// Playing/queued glyph as a span, or `None` when the song is neither.
    /// Shared by the multi-line stacked view and the single-column `status_cell`.
    pub fn status_icon(song: &Arc<SimpleSong>, state: &UiState, ms: bool) -> Option<Span<'static>> {
        let focus = matches!(state.get_pane(), Pane::TrackList);
        let theme = state.theme.get_display_theme(focus);

        let is_playing = state.get_now_playing().as_ref().map(|s| s.id) == Some(song.id);
        let is_queued = state.playback.is_queued(song.id);

        if is_playing {
            Some(state.theme.icons().playing.to_string().fg(match ms {
                true => theme.text_selected,
                false => theme.text_secondary,
            }))
        } else if is_queued && !matches!(state.get_mode(), Mode::Queue) {
            Some(state.theme.icons().queued.to_string().fg(match ms {
                true => theme.text_selected,
                false => theme.text_secondary,
            }))
        } else {
            None
        }
    }

    pub fn status_cell(song: &Arc<SimpleSong>, state: &UiState, ms: bool) -> Cell<'static> {
        Cell::from(Self::status_icon(song, state, ms).unwrap_or_else(|| "".into()))
    }

    pub fn title_cell(theme: &DisplayTheme, title: &str, ms: bool) -> Cell<'static> {
        Cell::from(title.to_owned()).fg(match ms {
            true => theme.text_selected,
            false => theme.text_primary,
        })
    }

    pub fn duration_cell(
        theme: &DisplayTheme,
        song: &Arc<SimpleSong>,
        style: DurationStyle,
        ms: bool,
    ) -> Cell<'static> {
        let duration_str = song.get_duration_str(style);
        Cell::from(Text::from(duration_str).right_aligned()).fg(match ms {
            true => theme.text_selected,
            false => theme.text_muted,
        })
    }

    pub fn track_disc_super(song: &Arc<SimpleSong>, idx: usize, has_album: bool) -> String {
        let track = match (has_album, song.track_no) {
            (true, Some(t)) => t,
            _ => (idx + 1) as u32,
        };

        match (has_album, song.disc_no) {
            (true, Some(d)) => format!("ᴰ{}·{}", superscript(d, 1), superscript(track, 2)),
            _ => superscript(track, 2),
        }
    }
}

/// Render `n`, zero-padded to `width` digits, as Unicode superscript.
fn superscript(n: u32, width: usize) -> String {
    format!("{n:0width$}")
        .chars()
        .map(|c| {
            c.to_digit(10)
                .and_then(|d| SUPERSCRIPT.get(&d).copied())
                .unwrap_or("?")
        })
        .collect()
}

static SUPERSCRIPT: LazyLock<HashMap<u32, &str>> = LazyLock::new(|| {
    HashMap::from([
        (0, "⁰"),
        (1, "¹"),
        (2, "²"),
        (3, "³"),
        (4, "⁴"),
        (5, "⁵"),
        (6, "⁶"),
        (7, "⁷"),
        (8, "⁸"),
        (9, "⁹"),
    ])
});

fn get_title(state: &UiState, area: Rect) -> Line<'static> {
    if state.layout == LayoutStyle::Minimal {
        return Line::default();
    }

    let focus = matches!(state.get_pane(), Pane::TrackList);
    let theme = state.theme.get_display_theme(focus);
    let mode = state.get_mode();
    let decorator = &state.theme.icons().decorator;
    let count = state.get_legal_songs().len();
    let third = (area.width / 3) as usize;

    if matches!(mode, Mode::Queue | Mode::Search) {
        let count_str = match count {
            1 => "[1 Song] ".to_string(),
            _ => format!("[{count} Songs] "),
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
            Span::from(format!(" [{count} Songs] ")).fg(theme.text_muted),
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

        let dur: Duration = state
            .get_legal_songs()
            .iter()
            .map(|s| s.get_duration())
            .sum();
        let readable = get_readable_duration(dur, DurationStyle::Clean);
        let info = match count {
            0 => String::default(),
            1 => format!("[1 Song ⫽ {readable}] "),
            _ => format!("[{count} Songs ⫽ {readable}] "),
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
            let ink = (state.get_legal_songs().len() as u16 * TRAD_ROW_STRIDE).wrapping_add(1);
            let border_h = if theme.has_borders() { 2 } else { 0 };
            let top = (area.height.saturating_sub(border_h + ink) / 2).max(1);

            Padding {
                left: 4,
                right: 4,
                top,
                bottom: 0,
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
