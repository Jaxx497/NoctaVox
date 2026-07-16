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
    library::{Album, SimpleSong, SongInfo},
    theme::{DisplayTheme, fade_color},
    truncate_at_last_space,
    ui_state::{LayoutStyle, Mode, Pane, Root, UiState},
};
use ratatui::{
    layout::{Constraint, Flex, HorizontalAlignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Padding, Row, Table},
};

// 7 matches the `xxM xxS` or `xxH xxM` format
const DURATION_SPACING: u16 = 7;
const COLUMN_SPACING: u16 = 2;

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
            LayoutStyle::Traditional => vec![
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Min(25),
                Constraint::Max(20),
                Constraint::Length(4),
                Constraint::Length(DURATION_SPACING),
            ],
            LayoutStyle::Minimal => vec![
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Fill(1),
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
    title: Line<'static>,
    state: &UiState,
    theme: &DisplayTheme,
    area: Rect,
) -> Table<'a> {
    let mode = state.get_mode();
    let pane = state.get_pane();
    let decorator = &state.theme.icons().decorator;

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
    pub fn status_cell(song: &Arc<SimpleSong>, state: &UiState, ms: bool) -> Cell<'static> {
        let focus = matches!(state.get_pane(), Pane::TrackList);
        let theme = state.theme.get_display_theme(focus);

        let is_playing = state.get_now_playing().as_ref().map(|s| s.id) == Some(song.id);
        let is_queued = state.playback.is_queued(song.id);

        let note = state.theme.icons().playing.to_string();
        let queued = state.theme.icons().queued.to_string();

        Cell::from(if is_playing {
            note.fg(match ms {
                true => theme.accent,
                false => theme.text_secondary,
            })
        } else if is_queued && !matches!(state.get_mode(), Mode::Queue) {
            queued.fg(match ms {
                true => theme.text_selected,
                false => theme.text_secondary,
            })
        } else {
            "".into()
        })
    }

    pub fn title_cell(theme: &DisplayTheme, title: &str, ms: bool) -> Cell<'static> {
        Cell::from(title.to_owned()).fg(match ms {
            true => theme.text_selected,
            false => theme.text_primary,
        })
    }

    pub fn artist_cell(theme: &DisplayTheme, song: &Arc<SimpleSong>, ms: bool) -> Cell<'static> {
        Cell::from(Line::from(song.get_artist().to_string())).fg(set_color_selection(ms, theme))
    }

    pub fn filetype_cell(theme: &DisplayTheme, song: &Arc<SimpleSong>, ms: bool) -> Cell<'static> {
        Cell::from(song.filetype.as_str_label()).fg(match ms {
            true => theme.text_selected,
            false => theme.text_secondary,
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

    pub fn index_cell(
        theme: &DisplayTheme,
        layout: &LayoutStyle,
        index: usize,
        ms: bool,
    ) -> Cell<'static> {
        let mut track_no = format!("{:>2}", index + 1).fg(match layout {
            LayoutStyle::Traditional => theme.accent,
            LayoutStyle::Minimal => fade_color(theme.dark, theme.accent, 0.7),
        });

        if ms {
            track_no = track_no.fg(theme.text_selected)
        };

        Cell::from(track_no)
    }

    pub fn track_disc_cell(
        theme: &DisplayTheme,
        song: &Arc<SimpleSong>,
        idx: usize,
        ms: bool,
    ) -> Cell<'static> {
        let mut track_no = match song.track_no {
            Some(t) => format!("{t:>2}").fg(theme.accent),
            None => format!("{:>2}", idx + 1).fg(theme.text_muted),
        };

        if ms {
            track_no = track_no.fg(theme.text_selected)
        };

        let disc_no = match song.disc_no {
            Some(t) => String::from("ᴰ") + SUPERSCRIPT.get(&t).unwrap_or(&"?"),
            None => "".into(),
        }
        .fg(match ms {
            true => theme.text_selected,
            false => theme.text_muted,
        });

        Cell::from(Line::from_iter([track_no, " ".into(), disc_no]))
    }
}

fn set_color_selection(selected: bool, theme: &DisplayTheme) -> Color {
    match selected {
        true => theme.text_selected,
        false => theme.text_primary,
    }
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

fn get_title(state: &UiState, album: Option<&Album>, area: Rect) -> Line<'static> {
    let focus = matches!(state.get_pane(), Pane::TrackList);
    let theme = state.theme.get_display_theme(focus);
    match state.get_mode() {
        Mode::Queue => {
            let q = state.playback.queue_len();
            let queue_len = match q {
                0 => "[0 Songs] ",
                1 => "[1 Song] ",
                _ => "[{q} Songs] ",
            };

            Line::from_iter([
                Span::from(" Queue ").fg(theme.accent),
                queue_len.fg(theme.text_muted),
            ])
        }
        Mode::Library => match album {
            Some(a) => {
                let album_title = match a.title.is_empty() {
                    true => String::from("[Unknown Album]"),
                    false => truncate_at_last_space(&a.title, (area.width / 3) as usize),
                };

                let year_str = a
                    .year
                    .filter(|y| *y != 0)
                    .map_or(String::new(), |y| format!(" [{y}]"));

                let decorator = &state.theme.icons().decorator;

                match state.layout {
                    LayoutStyle::Traditional => Line::from_iter([
                        Span::from(format!(" {album_title}"))
                            .fg(theme.text_secondary)
                            .italic(),
                        Span::from(year_str).fg(theme.text_muted),
                        Span::from(format!(" {decorator} ")).fg(theme.text_muted),
                        Span::from(a.get_album_artist().to_owned()).fg(theme.accent),
                        Span::from(format!(" [{} Songs] ", a.tracklist.len())).fg(theme.text_muted),
                    ]),
                    LayoutStyle::Minimal => Line::default(),
                }
            }
            None => {
                let name = match state.get_selected_root() {
                    Root::Library => state.get_selected_group_label().map(|a| a.to_string()),
                    Root::Playlist => state.get_selected_playlist().map(|p| p.name.clone()),
                };

                let Some(name) = name else {
                    return "".into();
                };

                let songs = state.get_legal_songs();
                let count = songs.len();
                let total: Duration = songs.iter().map(|s| s.get_duration()).sum();
                let readable = get_readable_duration(total, DurationStyle::Clean);

                let info = match count {
                    0 => {
                        return Line::from_iter([
                            " ".into(),
                            truncate_at_last_space(&name, (area.width / 3) as usize)
                                .fg(theme.text_secondary),
                            " ".into(),
                        ]);
                    }

                    1 => format!("[1 Song ⫽ {readable}] "),
                    _ => format!("[{count} Songs ⫽ {readable}] "),
                };

                Line::from_iter([
                    " ".into(),
                    truncate_at_last_space(&name, (area.width / 3) as usize)
                        .fg(theme.text_secondary),
                    format!(" {} ", state.theme.icons().decorator).fg(theme.text_muted),
                    info.fg(theme.text_muted),
                ])
            }
        },
        _ => Line::default(),
    }
}

fn get_padding(state: &UiState, theme: &DisplayTheme, area: Rect) -> Padding {
    let layout = &state.layout;
    let borders = theme.border_display;
    let song_len = (state.get_legal_songs().len()) as u16;

    let top = match song_len < area.height {
        true => (area.height.saturating_sub(song_len) / 2)
            .saturating_sub(2)
            .max(1),
        false => 1,
    };

    match layout {
        LayoutStyle::Traditional => Padding {
            left: 4,
            right: 4,
            top,
            bottom: 1,
        },
        LayoutStyle::Minimal => Padding {
            left: 3,
            right: 3,
            top: match borders {
                Borders::NONE => 1,
                _ => 0,
            },
            bottom: 0,
        },
    }
}
