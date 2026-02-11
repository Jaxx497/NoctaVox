use std::{sync::Arc, u32};

use ratatui::{
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, StatefulWidget, Widget},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    SimpleSong,
    library::SongInfo,
    ui_state::{DisplayTheme, LibraryStats, UiState},
};

pub struct UserStats;
impl StatefulWidget for UserStats {
    type State = UiState;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let theme = state.theme_manager.get_display_theme(true);
        let stats = state.get_lib_stats();
        let most_played = state.get_most_played().to_vec();

        if most_played.is_empty() {
            return;
        }

        let block = Block::bordered()
            .title(" Library Stats ")
            .title_bottom(" Press anything to close ")
            .title_alignment(HorizontalAlignment::Center)
            .border_style(theme.border)
            .bg(theme.bg);

        let inner = block.inner(area);
        block.render(area, buf);

        let [_upper_buf, lib_stats, central_buf] = Layout::vertical([
            Constraint::Percentage(7),
            Constraint::Length(7),
            Constraint::Fill(1),
        ])
        .areas(inner);

        let [duration_buf, _, top_play_title, top_played_buf] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(central_buf);

        let [left, _buffer, right] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(2),
            Constraint::Percentage(50),
        ])
        .areas(lib_stats);

        let library_stats = library_column(stats, theme);
        let playback_stats = listening_column(stats, theme);
        let most_played_vec = get_most_played(most_played, theme, &area);

        let horiz_padding = area.width / 10;

        Paragraph::new(library_stats)
            .right_aligned()
            .render(left, buf);
        Paragraph::new(playback_stats).render(right, buf);

        Line::from_iter([
            "Total duration of library: ".fg(theme.text_muted),
            format_duration(stats.total_duration).fg(theme.accent),
        ])
        .centered()
        .render(duration_buf, buf);

        Line::from("Top Played Songs")
            .fg(theme.text_secondary)
            .centered()
            .render(top_play_title, buf);

        Paragraph::new(most_played_vec)
            .block(Block::default().padding(Padding {
                left: horiz_padding,
                right: horiz_padding,
                top: 1,
                bottom: 3,
            }))
            .centered()
            .render(top_played_buf, buf);
    }
}

fn library_column<'a>(stats: &LibraryStats, theme: &'a DisplayTheme) -> Vec<Line<'a>> {
    vec![
        Line::from("Library     ").fg(theme.text_secondary),
        Line::from("=".repeat(17)),
        stat_line_left(print_commas(stats.total_tracks), "tracks", theme),
        stat_line_left(print_commas(stats.total_artists), "artists", theme),
        stat_line_left(print_commas(stats.total_albums), "albums", theme),
        stat_line_left(print_commas(stats.total_playlists), "playlists", theme),
    ]
}

fn listening_column(stats: &LibraryStats, theme: &DisplayTheme) -> Vec<Line<'static>> {
    let play_percent = format!("Played ({:.2}%)", stats.play_percentage);

    vec![
        Line::from("    Listening").fg(theme.text_secondary),
        Line::from("=".repeat(21)),
        stat_line_right(print_commas(stats.total_plays), "Combined plays", theme),
        stat_line_right(print_commas(stats.unique_plays), &play_percent, theme),
    ]
}

fn stat_line_left<'a>(value: String, label: &'a str, theme: &DisplayTheme) -> Line<'a> {
    Line::from_iter([
        Span::from(format!("{:>7}", value)).fg(theme.text_primary),
        Span::from(" "),
        Span::from(format!("{:>9}", label)).fg(theme.text_muted),
    ])
}

fn stat_line_right(value: String, label: &str, theme: &DisplayTheme) -> Line<'static> {
    Line::from_iter([
        Span::from(format!("{:<7}", value)).fg(theme.text_primary),
        Span::from(format!("{}", label)).fg(theme.text_muted),
    ])
}

fn get_most_played(
    most_played: Vec<(Arc<SimpleSong>, u16)>,
    theme: &DisplayTheme,
    area: &Rect,
) -> Vec<Line<'static>> {
    let row_width = (area.width - ((area.width / 10) * 2)).min(70) as usize;

    let title_cutoff = most_played
        .iter()
        .map(|(s, _)| s.title.len())
        .max()
        .unwrap()
        .min(30);

    let play_count_cutoff = 6;
    let spacer_cutoff = 4;

    let remainder = row_width
        .saturating_sub(play_count_cutoff)
        .saturating_sub(spacer_cutoff)
        .saturating_sub(title_cutoff);

    let header = Line::from_iter([
        Span::from(format!("{:>5}", "#")), // right-align to match the numbers
        Span::from("    "),                // same gap as between plays and title
        Span::from(format!("{:<title_cutoff$}", "Title")),
        Span::raw(" "),
        Span::from(format!("{:>remainder$}", "Artist  ",)),
    ]);

    let mut lines = vec![header, Line::from("-".repeat(row_width))];

    lines.extend(most_played.iter().map(|(s, plays)| {
        let title = truncate_display(s.get_title(), title_cutoff);
        let artist = truncate_display(s.get_artist(), remainder);

        Line::from_iter([
            Span::from(format!("{plays:>5}")).fg(theme.text_secondary),
            Span::raw("    "),
            Span::from(pad_to_width(&title, title_cutoff)).fg(theme.text_primary),
            Span::raw(" "),
            Span::from(pad_to_width_right(&artist, remainder)).fg(theme.text_muted),
        ])
    }));

    lines
}

fn print_commas(i: u32) -> String {
    if i < 999 {
        return format!("{i}");
    }

    i.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}

fn format_duration(total_seconds: f32) -> String {
    let total_secs = total_seconds as u64;
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;

    match (days, hours, minutes) {
        (0, 0, m) => format!("{}m", m),
        (0, h, m) => format!("{}h {}m", h, m),
        (d, h, m) => format!("{}d {}h {}m", d, h, m),
    }
}

fn truncate_display(s: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(s) <= max_width {
        return s.to_string();
    }

    let mut width = 0;
    let mut end = 0;
    for (i, c) in s.char_indices() {
        let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if width + cw + 1 > max_width {
            break;
        }
        width += cw;
        end = i + c.len_utf8();
    }
    format!("{}…", &s[..end])
}

fn pad_to_width(s: &str, target: usize) -> String {
    let w = UnicodeWidthStr::width(s);
    if w >= target {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(target - w))
    }
}

fn pad_to_width_right(s: &str, target: usize) -> String {
    let w = UnicodeWidthStr::width(s);
    if w >= target {
        s.to_string()
    } else {
        format!("{}{}", " ".repeat(target - w), s)
    }
}
