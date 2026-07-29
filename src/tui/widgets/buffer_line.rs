use crate::{
    library::{RefreshStage, SongInfo},
    theme::{DisplayTheme, shimmer_line},
    truncate_at_last_space,
    ui_state::{LayoutStyle, Mode, UiState},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, StatefulWidget, Widget},
};
use unicode_width::UnicodeWidthStr;

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
        let elapsed = state.metrics.position().as_secs_f32();

        if state.layout == LayoutStyle::Traditional && state.get_mode() != Mode::Fullscreen {
            let mut vol = volume_slider(state, area);

            if let Some(count) = get_buffer_count(buffer, theme) {
                vol.push_span(" ");
                vol.push_span(count);
            }
            vol.render(left, buf);
        }
        if let Some(title) = playing_title(state, theme, center.width as usize) {
            shimmer_line(title.centered(), elapsed).render(center, buf)
        };
        queue_display(state, theme, right.width as usize).render(right, buf);
    }
}

const PADDING: usize = 2;

fn playing_title(state: &UiState, theme: &DisplayTheme, width: usize) -> Option<Line<'static>> {
    let song = state.get_now_playing()?;
    let icons = state.theme.icons();

    let decorator = match state.playback.repeat_is_enabled() {
        true => &icons.repeat,
        false => &icons.decorator,
    };

    let paused = state.metrics.is_paused();
    let (separator, sep_color) = match paused {
        true => (format!(" {} ", icons.paused), theme.text_primary),
        false => (format!(" {decorator} "), theme.text_muted),
    };

    let title = song.get_title();
    let artist = song.get_artist();

    let sep_width = separator.width();
    let budget = width.saturating_sub(PADDING);

    let mut spans = vec![" ".into()];

    match budget >= title.width() + sep_width + artist.width() {
        true => {
            spans.push(Span::from(title.to_string()).fg(theme.text_secondary));
            spans.push(Span::from(separator).fg(sep_color));
            spans.push(Span::from(artist.to_string()).fg(theme.text_muted));
        }
        false => {
            if paused {
                spans.push(Span::from(separator).fg(sep_color));
            }

            let space = budget.saturating_sub(paused as usize * sep_width);
            let title = match title.width() <= space {
                true => title.to_string(),
                false => truncate_at_last_space(title, space),
            };

            spans.push(Span::from(title).fg(theme.text_secondary));
        }
    }

    spans.push(" ".into());
    Some(Line::from_iter(spans))
}

fn volume_slider(state: &UiState, area: Rect) -> Line<'static> {
    let theme = state.theme.get_display_theme(false);
    if state.library_refresh.is_some() {
        return Line::default();
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

    Line::from_iter([
        Span::from(format!(" {left_track}")).fg(theme.text_muted),
        Span::from("○").fg(theme.accent),
        Span::from(format!("{right_track}{percent} ")).fg(theme.text_muted),
    ])
}

fn get_buffer_count(size: Option<&str>, theme: &DisplayTheme) -> Option<Span<'static>> {
    let x = size.filter(|s| !s.is_empty())?;
    Some(format!("{x} ").fg(theme.text_muted))
}

const MIN_QUEUE_TITLE: usize = 10;
fn queue_display(state: &UiState, theme: &DisplayTheme, width: usize) -> Option<Line<'static>> {
    let up_next = state.playback.peek_queue()?.get_title();

    let queue_icon = state.theme.icons().upcoming.to_string();
    let queue_total = format!(" [{}] ", state.playback.queue_len());
    let space = width.saturating_sub(queue_icon.width() + 1 + queue_total.width());

    let mut spans = vec![Span::from(queue_icon).fg(theme.text_muted)];

    if space >= MIN_QUEUE_TITLE {
        spans.push(" ".into());
        spans.push(
            Span::from(truncate_at_last_space(up_next, space))
                .fg(state.theme.active.accent_inactive),
        );
    }

    spans.push(Span::from(queue_total).fg(theme.text_muted));

    let elapsed = state.metrics.position();
    let duration = state.metrics.duration();

    match (duration.saturating_sub(elapsed)).as_secs_f32() < 5.0 {
        true => Some(shimmer_line(Line::from_iter(spans), elapsed.as_secs_f32()).right_aligned()),
        false => Some(Line::from_iter(spans).right_aligned()),
    }
}
