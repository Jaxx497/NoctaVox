mod display_state;
mod domain;
mod multi_select;
mod playlist;
mod popup;
mod search_state;
mod settings;
mod stats;
mod ui_snapshot;
mod ui_state;

use std::sync::Arc;

pub use display_state::DisplayState;
pub use domain::{AlbumSort, LibraryView, Mode, Pane, TableSort};
pub use playlist::PlaylistAction;
pub use popup::PopupType;
pub use search_state::MatchField;
pub use settings::SettingsMode;
pub use stats::LibraryStats;
pub use ui_snapshot::UiSnapshot;
use voxio::Vox;

use crate::{
    Library, PlaybackSession,
    database::DbWorker,
    key_handler::KeyBuffer,
    library::{Album, Playlist, RefreshProgress, SimpleSong},
    theme::ThemeManager,
    ui_state::{popup::PopupState, search_state::SearchState, stats::VoxStats},
    visualization::Visualizer,
};

#[derive(PartialEq)]
pub enum LayoutStyle {
    Traditional,
    Minimal,
}

impl std::fmt::Display for LayoutStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutStyle::Minimal => write!(f, "mini"),
            _ => write!(f, "trad"),
        }
    }
}

impl LayoutStyle {
    pub fn from_str(s: &str) -> Self {
        match s {
            "mini" => LayoutStyle::Minimal,
            _ => LayoutStyle::Traditional,
        }
    }
}

pub struct UiState {
    library: Arc<Library>,
    db_worker: DbWorker,

    pub(crate) metrics: Arc<Vox>,
    pub(crate) playback: PlaybackSession,
    pub(crate) nav: DisplayState,

    pub(crate) search: SearchState,
    pub(crate) popup: PopupState,
    pub(crate) theme: ThemeManager,

    pub(crate) layout: LayoutStyle,
    pub(crate) stats: VoxStats,
    pub(crate) viz: Visualizer,

    pub(crate) albums: Vec<Album>,
    pub(crate) playlists: Vec<Playlist>,
    legal_songs: Vec<Arc<SimpleSong>>,

    pub library_refresh: Option<Arc<RefreshProgress>>,
    pub key_buffer: KeyBuffer,
}

fn new_textarea(placeholder: &str) -> ratatui_textarea::TextArea<'static> {
    let mut search = ratatui_textarea::TextArea::default();
    search.set_cursor_line_style(ratatui::style::Style::default());
    search.set_placeholder_text(format!(" {placeholder}: "));

    search
}
