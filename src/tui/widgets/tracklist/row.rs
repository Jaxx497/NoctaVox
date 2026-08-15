use crate::{
    DurationStyle, SimpleSong,
    library::{Album, SongInfo},
    theme::{DisplayTheme, ThemeIcons, fade_color},
    tui::widgets::tracklist::{TRAD_ROW_HEIGHT, TRAD_ROW_MARGIN},
    ui_state::{LayoutStyle, MatchSpans, Mode, Pane, UiState},
};
use ratatui::{
    style::{Color, Stylize},
    text::{Line, Span, Text},
    widgets::{Cell, Row},
};
use std::sync::Arc;

const SEARCH_DIM: f32 = 0.7;

pub struct RowPalette {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub muted: Color,
    pub hit: Color,
    pub search_title: Color,
    pub search_meta: Color,
}

impl RowPalette {
    pub fn base(t: &DisplayTheme) -> Self {
        RowPalette {
            primary: t.text_primary,
            secondary: fade_color(t.dark, t.text_secondary, 0.8),
            accent: fade_color(t.dark, t.accent, 0.8),
            muted: t.text_muted,
            hit: t.accent,
            search_title: fade_color(t.dark, t.text_primary, SEARCH_DIM),
            search_meta: fade_color(t.dark, t.text_secondary, SEARCH_DIM),
        }
    }

    pub fn selected(c: Color) -> Self {
        RowPalette {
            primary: c,
            secondary: c,
            accent: c,
            muted: c,
            hit: c,
            search_title: c,
            search_meta: c,
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
    pub fn trad_cell_gutter(
        ctx: &RowCtx,
        song: &Arc<SimpleSong>,
        idx: usize,
        p: &RowPalette,
    ) -> Cell<'static> {
        let number =
            CellFactory::track_disc_super(ctx.layout, song, idx, ctx.selected_album.is_some());
        Cell::from(Line::from(number).fg(p.accent).right_aligned())
    }

    pub fn trad_cell_main(song: &Arc<SimpleSong>, p: &RowPalette) -> Cell<'static> {
        let title_line = Line::from_iter([
            Span::raw(song.get_title().to_string()).fg(p.primary).bold(),
            Span::raw(" "),
            Span::raw(song.filetype.as_str_label()).fg(p.muted),
        ]);

        let artist_line = Line::from(Span::raw(song.get_artist().to_string()).fg(p.secondary));

        Cell::from(Text::from(vec![title_line, artist_line]))
    }

    pub fn trad_cell_duration(
        ctx: &RowCtx,
        s: &Arc<SimpleSong>,
        style: DurationStyle,
        p: &RowPalette,
    ) -> Cell<'static> {
        let duration_str = Line::from(s.get_duration_str(style))
            .fg(p.muted)
            .right_aligned();

        let icon = CellFactory::status_icon(ctx, s).unwrap_or_default();
        let icon_line = Line::from(format!("{icon} ").fg(p.accent)).right_aligned();

        Cell::from(Text::from(vec![duration_str, icon_line]))
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
        layout: &LayoutStyle,
        song: &Arc<SimpleSong>,
        idx: usize,
        has_album: bool,
    ) -> String {
        let track = match (has_album, song.track_no) {
            (true, Some(t)) => t,
            _ => (idx + 1) as u32,
        };

        match (has_album, song.disc_no, layout) {
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
    let index = CellFactory::trad_cell_gutter(ctx, &s, idx, p);
    let left = CellFactory::trad_cell_main(&s, p);
    let right = CellFactory::trad_cell_duration(ctx, &s, DurationStyle::Clean, p);

    Row::new([index, left, right])
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
        Line::from(
            CellFactory::track_disc_super(ctx.layout, s, idx, ctx.selected_album.is_some()) + " ",
        )
        .right_aligned(),
    )
    .fg(fade_color(ctx.theme.dark, p.accent, 0.7));
    let symbol = CellFactory::status_cell(&ctx, &s).fg(p.secondary);
    let title = Cell::from(s.get_title().to_string()).fg(p.primary);
    let duration = CellFactory::duration_cell(&s, DurationStyle::Clean).fg(p.muted);

    Row::new([idx, title, symbol, duration])
}

fn search(ctx: &RowCtx, s: &Arc<SimpleSong>, p: &RowPalette) -> Row<'static> {
    let spans = ctx.state.get_match_spans(s.id);
    let hits = |f: fn(&MatchSpans) -> &Vec<u32>| spans.map_or(&[][..], |m| f(m).as_slice());

    let symbol = CellFactory::status_cell(ctx, s);
    let title_col = highlight_cell(s.get_title(), hits(|m| &m.title), p.search_title, p.hit);
    let artist_col = highlight_cell(s.get_artist(), hits(|m| &m.artist), p.search_meta, p.hit);
    let album_col = highlight_cell(s.get_album(), hits(|m| &m.album), p.search_meta, p.hit);
    let dur_col = CellFactory::duration_cell(s, DurationStyle::Clean).fg(p.muted);

    match ctx.layout {
        LayoutStyle::Traditional => Row::new([title_col, artist_col, album_col, symbol, dur_col]),
        LayoutStyle::Minimal => Row::new([title_col, artist_col, album_col]),
    }
}

fn highlight_cell(text: &str, hits: &[u32], base: Color, hit: Color) -> Cell<'static> {
    if hits.is_empty() {
        return Cell::from(text.to_string()).fg(base);
    }

    let mut spans = Vec::new();
    let mut run = String::new();
    let mut run_is_hit = false;

    for (i, c) in text.chars().enumerate() {
        let is_hit = hits.binary_search(&(i as u32)).is_ok();

        if is_hit != run_is_hit && !run.is_empty() {
            spans.push(run_span(std::mem::take(&mut run), run_is_hit, base, hit));
        }

        run_is_hit = is_hit;
        run.push(c);
    }
    spans.push(run_span(run, run_is_hit, base, hit));

    Cell::from(Line::from(spans))
}

fn run_span(text: String, is_hit: bool, base: Color, hit: Color) -> Span<'static> {
    match is_hit {
        true => Span::raw(text).fg(hit).bold(),
        false => Span::raw(text).fg(base),
    }
}
