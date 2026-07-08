use crate::{
    library::{RefreshStage, SongInfo},
    theme::DisplayTheme,
    truncate_at_last_space,
    ui_state::UiState,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
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

        let [left, center, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .areas(area);

        let buffer = state.key_buffer.pending();

        get_buffer_count(buffer, &theme).render(left, buf);
        playing_title(state, &theme, center.width as usize).render(center, buf);
        queue_display(state, &theme, right.width as usize).render(right, buf);
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
                Span::from(separator),
                Span::from(artist.to_string()).fg(theme.text_muted),
                " ".into(),
            ])
            .centered(),
        )
    } else if width >= MIN_TITLE_LEN + SEPARATOR_LEN + MIN_ARTIST_LEN {
        let available_space = width.saturating_sub(SEPARATOR_LEN);
        let title_space = (available_space * 2) / 3;
        let artist_space = available_space.saturating_sub(title_space);

        let truncated_title = truncate_at_last_space(&title, title_space);
        let truncated_artist = truncate_at_last_space(&artist, artist_space);

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
                let truncated_title = truncate_at_last_space(&title, title_len - SEPARATOR_LEN);
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
                let truncated_title = truncate_at_last_space(&title, width);
                Some(Line::from(Span::from(truncated_title).fg(theme.text_secondary)).centered())
            }
        }
    }
}

fn get_buffer_count(size: Option<&str>, theme: &DisplayTheme) -> Option<Line<'static>> {
    if let Some(x) = size {
        if x == "" {
            return None;
        }

        return Some(
            format!("{x:>3}")
                .fg(theme.text_muted)
                .into_left_aligned_line(),
        );
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
    let queue_icon = state.theme.icons().queued.to_string();

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
