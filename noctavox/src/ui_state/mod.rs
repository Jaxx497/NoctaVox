mod display_state;
mod domain;
mod multi_select;
mod playlist;
mod popup;
mod progress_display;
mod search_state;
mod settings;
mod stats;
mod theme;
mod ui_snapshot;
mod ui_state;
mod waveform;

use std::{collections::VecDeque, sync::Arc};

pub use display_state::DisplayState;
pub use domain::{AlbumSort, LibraryView, Mode, Pane, TableSort};
pub use playlist::PlaylistAction;
pub use popup::PopupType;
pub use progress_display::ProgressDisplay;
pub use search_state::MatchField;
pub use settings::SettingsMode;
pub use stats::LibraryStats;
pub use theme::DisplayTheme;
pub use ui_snapshot::UiSnapshot;
pub use waveform::WaveformManager;

use crate::{
    Library, PlaybackSession,
    database::DbWorker,
    library::{Album, Playlist, SimpleSong},
    player::PlaybackMetrics,
    ui_state::{popup::PopupState, search_state::SearchState, stats::VoxStats},
};

pub struct UiState {
    library: Arc<Library>,
    db_worker: DbWorker,

    pub(crate) metrics: Arc<PlaybackMetrics>,
    pub(crate) playback: PlaybackSession,

    search: SearchState,
    pub(crate) popup: PopupState,
    pub(crate) theme_manager: ThemeManager,
    pub(crate) display_state: DisplayState,

    pub(crate) sample_tap: VecDeque<f32>,
    pub(crate) spectrum_bars: Vec<f32>,
    waveform: WaveformManager,
    progress_display: ProgressDisplay,
    stats: VoxStats,

    legal_songs: Vec<Arc<SimpleSong>>,
    pub(crate) albums: Vec<Album>,
    pub(crate) playlists: Vec<Playlist>,

    pub library_refresh_progress: Option<u8>,
    pub library_refresh_detail: Option<String>,
}

pub use theme::*;

fn new_textarea(placeholder: &str) -> ratatui_textarea::TextArea<'static> {
    let mut search = ratatui_textarea::TextArea::default();
    search.set_cursor_line_style(ratatui::style::Style::default());
    search.set_placeholder_text(format!(" {placeholder}: "));

    search
}
