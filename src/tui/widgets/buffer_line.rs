use crate::{
    library::{RefreshStage, SongInfo},
    theme::DisplayTheme,
    truncate_at_last_space,
    ui_state::{LayoutStyle, UiState},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, StatefulWidget, Widget},
};

pub struct BufferLine;

impl StatefulWidget for BufferLine {
    type State = UiState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let theme = state.theme.get_display_theme(true);

        if let Some(refresh) = &state.library_refresh {
            let percent = refresh.percent();

            let label = match refresh.stage() {
                RefreshStage::Parsing => {
                    let (c, t) = refresh.counts();
                    format!("Processing {c}/{t} | {percent}%")
                }
                stage => format!("{} | {percent}%", stage.label()),
            }
            .fg(theme.text_muted);

            let guage = Gauge::default()
                .block(Block::new().borders(Borders::NONE))
                .gauge_style(theme.accent)
                .label(label)
                .percent(percent.min(100) as u16);

            guage.render(area, buf);
            return;
        }

        let [_, left, center, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(2),
                Constraint::Percentage(18),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .areas(area);

        let buffer = state.key_buffer.pending();

        if state.layout == LayoutStyle::Traditional {
            let mut vol = volume_slider(state, theme, area);
            if let Some(v) = vol.as_mut()
                && let Some(count) = get_buffer_count(buffer, theme)
            {
                v.push_span(" ");
                v.push_span(count);
            }
            vol.render(left, buf);
        }
        playing_title(state, theme, center.width as usize).render(center, buf);
        queue_display(state, theme, right.width as usize).render(right, buf);
    }
}

const SEPARATOR_LEN: usize = 3;
const MIN_TITLE_LEN: usize = 20;
const MIN_ARTIST_LEN: usize = 15;

fn playing_title(state: &UiState, theme: &DisplayTheme, width: usize) -> Option<Line<'static>> {
    let song = state.get_now_playing()?;
    let decorator = match state.playback.repeat_is_enabled() {
        true => &state.theme.icons().repeat,
        false => &state.theme.icons().decorator,
    };

    let paused = &state.theme.icons().paused;

    let separator = match state.metrics.is_paused() {
        true => Span::from(format!(" {paused} "))
            .fg(theme.text_primary)
            .rapid_blink(),
        false => Span::from(format!(" {decorator} ")).fg(theme.text_muted),
    };

    let title = song.get_title();
    let artist = song.get_artist();

    let title_len = title.chars().count();
    let artist_len = artist.chars().count();

    if width >= title_len + SEPARATOR_LEN + artist_len {
        Some(
            Line::from_iter([
                " ".into(),
                Span::from(title.to_string()).fg(theme.text_secondary),
                separator,
                Span::from(artist.to_string()).fg(theme.text_muted),
                " ".into(),
            ])
            .centered(),
        )
    } else if width >= MIN_TITLE_LEN + SEPARATOR_LEN + MIN_ARTIST_LEN {
        let available_space = width.saturating_sub(SEPARATOR_LEN);
        let title_space = (available_space * 2) / 3;
        let artist_space = available_space.saturating_sub(title_space);

        let truncated_title = truncate_at_last_space(title, title_space);
        let truncated_artist = truncate_at_last_space(artist, artist_space);

        Some(
            Line::from_iter([
                " ".into(),
                Span::from(truncated_title).fg(theme.text_secondary),
                separator,
                Span::from(truncated_artist).fg(theme.text_muted),
                " ".into(),
            ])
            .centered(),
        )
    } else {
        match state.metrics.is_paused() {
            true => {
                let truncated_title = truncate_at_last_space(title, title_len - SEPARATOR_LEN);
                Some(
                    Line::from_iter([
                        " ".into(),
                        separator,
                        Span::from(truncated_title).fg(theme.text_secondary),
                        " ".into(),
                    ])
                    .centered(),
                )
            }
            false => {
                let truncated_title = truncate_at_last_space(title, width);
                Some(Line::from(Span::from(truncated_title).fg(theme.text_secondary)).centered())
            }
        }
    }
}

fn volume_slider(state: &UiState, theme: &DisplayTheme, area: Rect) -> Option<Line<'static>> {
    if state.library_refresh.is_some() {
        return None;
    }

    let width = (area.width / 10).clamp(4, 11) as usize;
    let ratio = (state.metrics.volume() / 1.0).clamp(0.0, 1.0);
    let pos = (ratio * (width - 1) as f32).round() as usize;
    let pct = (state.metrics.volume() * 100.0).round() as usize;
    let percent = match area.width >= 80 {
        true => format!(" {pct}%"),
        false => String::default(),
    };

    let left_track = "─".repeat(pos);
    let right_track = "─".repeat(width - 1 - pos);

    Some(Line::from_iter([
        Span::from(format!(" {left_track}")).fg(theme.text_muted),
        Span::from("○").fg(theme.accent),
        Span::from(format!("{right_track}{percent} ")).fg(theme.text_muted),
    ]))
}

fn _volume_meter(state: &UiState, theme: &DisplayTheme) -> Line<'static> {
    const MAX: f32 = 1.5; // voxio clamps perceptual volume to 0.0..=1.5
    const CELLS: usize = 12; // one bar cell per 12.5%
    const BOOST_CELL: usize = 8; // unity (100%) lands here: 1.0 / 1.5 * 12 = 8
    const BLOCKS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
    const TRACK: char = '░';

    let vol = state.metrics.volume().clamp(0.0, MAX);
    let eighths = (vol / MAX * (CELLS * 8) as f32).round() as usize; // filled 1/8ths, 0..=96

    let mut spans: Vec<Span<'static>> = (0..CELLS)
        .map(|cell| {
            let fill = eighths.saturating_sub(cell * 8).min(8);
            let (ch, color) = match fill {
                0 => (TRACK, theme.text_muted), // unfilled track
                f if cell >= BOOST_CELL => (BLOCKS[f], theme.bg_error), // boost zone (>100%)
                f => (BLOCKS[f], theme.accent), // normal zone
            };
            Span::from(ch.to_string()).fg(color)
        })
        .collect();

    let pct = (vol * 100.0).round() as usize;
    let pct_color = if vol > 1.0 {
        theme.bg_error
    } else {
        theme.text_muted
    };
    spans.push(Span::from(format!(" {pct}%")).fg(pct_color));

    Line::from_iter(spans)
}

fn get_buffer_count(size: Option<&str>, theme: &DisplayTheme) -> Option<Span<'static>> {
    if let Some(x) = size {
        if x.is_empty() {
            return None;
        }

        return Some(format!("{x} ").fg(theme.text_muted));
    }
    None
}

const BAD_WIDTH: usize = 22;
fn queue_display(state: &UiState, theme: &DisplayTheme, width: usize) -> Option<Line<'static>> {
    let up_next_str = state.playback.peek_queue()?.get_title();

    let truncated = truncate_at_last_space(up_next_str, width - 5);

    let up_next_line = Span::from(truncated).fg(state.theme.active.accent_inactive);

    let total = state.playback.queue_len();
    let queue_total = format!(" [{total}] ").fg(theme.text_muted);
    let queue_icon = state.theme.icons().upcoming.to_string();

    match width < BAD_WIDTH {
        true => Some(
            Line::from_iter([Span::from(queue_icon).fg(theme.text_muted), queue_total])
                .right_aligned(),
        ),

        false => Some(
            Line::from_iter([
                Span::from(queue_icon).fg(theme.text_muted),
                " ".into(),
                up_next_line,
                queue_total,
            ])
            .right_aligned(),
        ),
    }
}
