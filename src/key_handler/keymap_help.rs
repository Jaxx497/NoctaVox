pub struct HelpKey {
    /// Shown in the guide, e.g. `"<n>, <shift+N>"`.
    pub label: &'static str,
    pub desc: &'static str,
}

pub struct HelpSection {
    pub title: &'static str,
    pub keys: &'static [HelpKey],
}

const fn key(label: &'static str, desc: &'static str) -> HelpKey {
    HelpKey { label, desc }
}

const HELP: &[HelpSection] = &[
    HelpSection {
        title: "Global",
        keys: &[
            key("<space>", "Play / pause"),
            key("<control+s>", "Stop"),
            key("<n>, <shift+N>", "Seek forward (small / large)"),
            key("<p>, <shift+P>", "Seek back (small / large)"),
            key("<control+n>, <control+p>", "Play next / previous"),
            key("<control+r>", "Toggle repeat"),
            key("< / >", "Search"),
            key("<m>", "Toggle minimal mode"),
            key("<f>", "Fullscreen visualizer"),
            key("< = >", "Jump to now playing"),
            key("<j>, <k>, <up>, <down>", "Scroll"),
            key("<d>, <u>", "Half-page down / up"),
            key("<control+a>", "Go to album view"),
            key("< ` >, < ~ >", "Access root settings"),
            key("< + >, < - >", "Adjust volume up/down"),
            key("<control+u>, <f5>", "Rescan library"),
            key("<control+o/t/q>", "Omni / Playlist / Queue view"),
            key(
                "<control+1>, <control+2>, <control+3>",
                "Albums / Playlists / Queue",
            ),
            key("<shift+D>, <shift+U>", "Page down / up"),
            key("<shift+G>", "Jump to bottom"),
            key("<[>, <]>", "Shrink / grow sidebar"),
            key("<{>, <}>", "Toggle waveform smoothness"),
            key("<w>", "Cycle progress display"),
            key(
                "<shift+W/O/S/B>",
                "Waveform / Oscilloscope / Spectrum / Bar",
            ),
            key("<shift+C>", "Theme picker"),
            key("< , >, < . > ", "Cycle theme"),
            key("<f6>", "Reload themes from disk"),
            key("< \\ >", "Statistics"),
            key("<esc>", "Clear selection / reset"),
            key("<backspace>", "Clear key count"),
            key("<control+c>", "Quit"),
            key("<?>", "Help page"),
        ],
    },
    HelpSection {
        title: "Track List",
        keys: &[
            key("<enter>", "Play Selected"),
            key("<q>", "Queue track"),
            key("<a>", "Add to playlist"),
            key("<a+a>", "Add to last appended playlist"),
            key("<control+a>", "Go to album"),
            key("<v>", "Toggle multi-select"),
            key("<control+v>", "Clear multi-select"),
            key("<shift+V>", "Select all"),
            key("<g> / <G>", "Go to <top> / <bottom>"),
            key("<# + g>", "Go to track at line # (e.g. 7g)"),
            key("<h>, <left>, <tab>", "Back to sidebar"),
            key("<x>", "Remove (playlist / queue)"),
            key(
                "<shift+K>, <shift+J>",
                "Move item up / down (playlist / queue)",
            ),
            key("<shift+Q>", "Queue all"),
            key("<s>", "Shuffle queue (queue mode)"),
            key("<control+h>, <control+l>", "Sort columns (search)"),
        ],
    },
    HelpSection {
        title: "Sidebar",
        keys: &[
            key("<enter>", "Toggle (headers) / Open (leaves)"),
            key("<l>, <right>", "Expand header / open track pane"),
            key("<h>, <left>", "Collapse (self or parent)"),
            key("<L>", "Expand everything"),
            key("<q>", "Queue selection"),
            key("<s>", "Queue and shuffle selection"),
            key("<g>, <G>", "Jump to <top> / <bottom>"),
            key("<# + g>", "Go to track at line #"),
            key("<control+h>, <control+left>", "Sort albums (prev)"),
            key("<control+l>, <control+right>", "Sort albums (next)"),
            key("<c>", "Create playlist"),
            key("<r>", "Rename playlist (playlist row)"),
            key("<control+d>", "Delete playlist (playlist row)"),
        ],
    },
    HelpSection {
        title: "Search",
        keys: &[
            key("(type)", "Filter results"),
            key("<enter>, <tab>", "Run search"),
            key("<esc>", "Exit search"),
        ],
    },
];

/// One rendered line of the guide. Sections are separated by a `Blank`.
pub enum HelpRow {
    Blank,
    Header(&'static str),
    Key(&'static HelpKey),
}

/// The flattened rows the guide renders (section headers, keys, and blank
/// separators), in display order.
pub fn help_rows() -> Vec<HelpRow> {
    let mut rows = Vec::new();
    for (i, section) in HELP.iter().enumerate() {
        if i > 0 {
            rows.push(HelpRow::Blank);
        }
        rows.push(HelpRow::Header(section.title));
        for key in section.keys {
            rows.push(HelpRow::Key(key));
        }
    }
    rows
}
