use std::sync::Arc;

use crate::{
    DurationStyle, SimpleSong,
    library::{Album, SongInfo},
    theme::{DisplayTheme, ThemeIcons, fade_color},
    tui::widgets::tracklist::{TRAD_ROW_HEIGHT, TRAD_ROW_MARGIN},
    ui_state::{LayoutStyle, MatchField, Mode, Pane, UiState},
};
use ratatui::{
    style::{Color, Stylize},
    text::{Line, Span, Text},
    widgets::{Cell, Row},
};

pub struct RowPalette {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub muted: Color,
}

impl RowPalette {
    pub fn base(t: &DisplayTheme) -> Self {
        RowPalette {
            primary: t.text_primary,
            secondary: fade_color(t.dark, t.text_secondary, 0.8),
            accent: fade_color(t.dark, t.accent, 0.8),
            muted: t.text_muted,
        }
    }

    pub fn selected(c: Color) -> Self {
        RowPalette {
            primary: c,
            secondary: c,
            accent: c,
            muted: c,
        }
    }
}

pub struct RowCtx<'a> {
    pub state: &'a UiState,
    pub theme: &'a DisplayTheme,
    pub selected_album: Option<&'a Album>,
    pub mode: Mode,
    pub layout: &'a LayoutStyle,
    pub icons: &'a ThemeIcons,
    pub palette_base: RowPalette,
    pub palette_selected: RowPalette,
}

impl<'a> RowCtx<'a> {
    pub fn new(state: &'a UiState) -> Self {
        let mode = state.get_mode().clone();
        let focus = matches!(state.get_pane(), Pane::TrackList | Pane::Search);
        let theme = state.theme.get_display_theme(focus);

        RowCtx {
            state,
            theme,
            mode,
            selected_album: state.get_selected_album(),
            layout: &state.layout,
            icons: state.theme.icons(),
            palette_base: RowPalette::base(theme),
            palette_selected: RowPalette::selected(theme.text_selected),
        }
    }
}

pub struct CellFactory;

impl CellFactory {
    pub fn trad_left_cell(
        ctx: &RowCtx,
        song: &Arc<SimpleSong>,
        idx: usize,
        p: &RowPalette,
    ) -> Cell<'static> {
        let mut title_line = Vec::new();
        match CellFactory::status_icon(&ctx, song) {
            Some(icon) => {
                title_line.push(Span::raw(" "));
                title_line.push(icon.fg(p.accent));
                title_line.push(Span::raw("  "));
            }
            None => title_line.push(Span::raw("    ")),
        }
        title_line.push(Span::raw(song.get_title().to_string()).fg(p.primary).bold());
        title_line.push(Span::raw("  "));
        title_line.push(Span::raw(song.filetype.as_str_label()).fg(p.muted));

        let number = CellFactory::track_disc_super(ctx, song, idx, ctx.selected_album.is_some());
        let artist_line = Line::from(vec![
            Span::raw("    "),
            Span::raw(number).fg(p.accent),
            Span::raw(format!(" {} ", ctx.icons.decorator)).fg(p.muted),
            Span::raw(song.get_artist().to_string()).fg(p.secondary),
        ]);

        Cell::from(Text::from(vec![Line::from(title_line), artist_line]))
    }

    pub fn status_icon(ctx: &RowCtx, song: &Arc<SimpleSong>) -> Option<Span<'static>> {
        let is_playing = ctx.state.get_now_playing().as_ref().map(|s| s.id) == Some(song.id);
        let is_queued = ctx.state.playback.is_queued(song.id);

        if is_playing {
            Some(ctx.icons.playing.to_string().into())
        } else if is_queued && !matches!(ctx.mode, Mode::Queue) {
            Some(ctx.icons.queued.to_string().into())
        } else {
            None
        }
    }

    pub fn status_cell(ctx: &RowCtx, song: &Arc<SimpleSong>) -> Cell<'static> {
        Cell::from(Self::status_icon(ctx, song).unwrap_or_else(|| "".into()))
    }

    pub fn duration_cell(s: &Arc<SimpleSong>, style: DurationStyle) -> Cell<'static> {
        let duration_str = s.get_duration_str(style);
        Cell::from(Text::from(duration_str).right_aligned())
    }

    pub fn track_disc_super(
        ctx: &RowCtx,
        song: &Arc<SimpleSong>,
        idx: usize,
        has_album: bool,
    ) -> String {
        let track = match (has_album, song.track_no) {
            (true, Some(t)) => t,
            _ => (idx + 1) as u32,
        };

        match (has_album, song.disc_no, ctx.layout) {
            (true, Some(d), LayoutStyle::Traditional) => {
                format!("ᴰ{}⁻{}", superscript(d, 1), superscript(track, 2))
            }
            _ => superscript(track, 2),
        }
    }
}

const SUPERSCRIPT: [&str; 10] = ["⁰", "¹", "²", "³", "⁴", "⁵", "⁶", "⁷", "⁸", "⁹"];
fn superscript(n: u32, width: usize) -> String {
    format!("{n:0width$}")
        .chars()
        .map(|c| SUPERSCRIPT[c.to_digit(10).unwrap() as usize])
        .collect()
}

pub(crate) fn build_row(ctx: &RowCtx, song: &Arc<SimpleSong>, idx: usize) -> Row<'static> {
    let ms = ctx.state.get_multi_select_indices().contains(&idx);
    let p = if ms {
        &ctx.palette_selected
    } else {
        &ctx.palette_base
    };
    let row = match (&ctx.mode, ctx.layout) {
        (Mode::Search | Mode::Power, _) => search(&ctx, song, p),
        (Mode::Library | Mode::Queue, LayoutStyle::Traditional) => {
            standard_tracklist(ctx, song, idx, p)
        }
        (Mode::Library | Mode::Queue, LayoutStyle::Minimal) => minimal_tracklist(ctx, song, idx, p),
        _ => Row::default(),
    };

    match ms {
        true => row
            .fg(ctx.theme.text_selected)
            .bg(ctx.state.theme.active.accent_inactive),
        false => row,
    }
}

fn standard_tracklist(
    ctx: &RowCtx,
    s: &Arc<SimpleSong>,
    idx: usize,
    p: &RowPalette,
) -> Row<'static> {
    let left = CellFactory::trad_left_cell(&ctx, &s, idx, p);
    let right = CellFactory::duration_cell(&s, DurationStyle::Clean).fg(p.muted);

    Row::new([left, right])
        .height(TRAD_ROW_HEIGHT)
        .bottom_margin(TRAD_ROW_MARGIN)
}

fn minimal_tracklist(
    ctx: &RowCtx,
    s: &Arc<SimpleSong>,
    idx: usize,
    p: &RowPalette,
) -> Row<'static> {
    let idx = Cell::from(
        Line::from(CellFactory::track_disc_super(ctx, s, idx, ctx.selected_album.is_some()) + " ")
            .right_aligned(),
    )
    .fg(p.accent);
    let symbol = CellFactory::status_cell(&ctx, &s).fg(p.secondary);
    let title = Cell::from(s.get_title().to_string()).fg(p.primary);
    let duration = CellFactory::duration_cell(&s, DurationStyle::Clean).fg(p.muted);

    Row::new([idx, title, symbol, duration])
}

fn search(ctx: &RowCtx, s: &Arc<SimpleSong>, p: &RowPalette) -> Row<'static> {
    let symbol = CellFactory::status_cell(&ctx, &s);
    let mut title_col = Cell::from(s.get_title().to_string()).fg(p.muted);
    let mut artist_col = Cell::from(s.get_artist().to_string()).fg(p.muted);
    let mut album_col = Cell::from(s.get_album().to_string()).fg(p.muted);
    let dur_col = CellFactory::duration_cell(&s, DurationStyle::Clean).fg(p.muted);

    if let Some(field) = ctx.state.get_match_fields(s.id) {
        match field {
            MatchField::Title => title_col = title_col.fg(p.secondary),
            MatchField::Artist => artist_col = artist_col.fg(p.secondary),
            MatchField::Album => album_col = album_col.fg(p.secondary),
        }
    }

    match ctx.layout {
        LayoutStyle::Traditional => Row::new([title_col, artist_col, album_col, symbol, dur_col]),
        LayoutStyle::Minimal => Row::new([title_col, artist_col, album_col]),
    }
}
