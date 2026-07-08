mod action;
mod key_buffer;

use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::LazyLock;
use std::time::Duration;
use std::time::Instant;

pub use action::handle_key_event;
pub use action::next_event;
pub use key_buffer::KeyBuffer;
use ratatui::crossterm::event::KeyEvent;
use ratatui::crossterm::event::KeyModifiers;

use crate::ui_state::Mode;
use crate::ui_state::Pane;
use crate::ui_state::PopupType;
use crate::user_config;
use crate::visualization::ProgressDisplay;

static ILLEGAL_CHARS: LazyLock<HashSet<char>> = LazyLock::new(|| HashSet::from([';']));

const X: KeyModifiers = KeyModifiers::NONE;
const S: KeyModifiers = KeyModifiers::SHIFT;
const C: KeyModifiers = KeyModifiers::CONTROL;

static SEEK_SMALL: LazyLock<f64> = LazyLock::new(|| user_config().general.seek_small);
static SEEK_LARGE: LazyLock<f64> = LazyLock::new(|| user_config().general.seek_large);

const SCROLL_MID: usize = 5;
const SCROLL_XTRA: usize = 20;
const SIDEBAR_INCREMENT: isize = 1;

pub enum Action {
    // Player Controls
    Play(usize),
    Stop,
    TogglePlayback,
    PlayNext,
    PlayPrev,
    SeekForward(f64),
    SeekBack(f64),

    // Queue & Playlist Actions
    QueueSong,
    QueueMany {
        sel_type: SelectionType,
        shuffle: bool,
    },
    RemoveSong,

    ToggleRepeat,

    AddToPlaylist,
    AddToPlaylistConfirm,

    CreatePlaylistWithSongs,
    CreatePlaylistWithSongsConfirm,

    // Updating App State
    UpdateLibrary,
    SendSearch,
    UpdateSearch(KeyEvent),
    SortColumnsNext,
    SortColumnsPrev,
    ToggleAlbumSort(bool),
    ChangeMode(Mode),
    ChangePane(Pane),
    GoToTrack(usize),
    GoToAlbum,
    GoToNowPlaying,
    Scroll(Director),

    MultiSelect(usize),
    MultiSelectAll,
    ClearMultiSelect,
    ClearKeyBuffer,

    // Playlists
    CreatePlaylist,
    CreatePlaylistConfirm,

    DeletePlaylist,
    DeletePlaylistConfirm,

    RenamePlaylist,
    RenamePlaylistConfirm,

    ShiftPosition(Incrementor),
    ShuffleElements,

    SwapLayout,

    // Display
    CycleTheme(Incrementor),
    ThemeManager,
    ThemeRefresh,

    IncrementWFSmoothness(Incrementor),
    IncrementSidebarSize(isize),

    SetProgressDisplay(ProgressDisplay),
    NextProgressDisplay,
    ToggleProgressDisplay,
    RevertFullscreen,

    PopupScrollUp,
    PopupScrollDown,
    PopupInput(KeyEvent),
    ShowStats,

    ClosePopup,

    // Errors, Convenience & Other
    ViewSettings,
    RootAdd,
    RootRemove,
    RootConfirm,

    HandleErrors,
    SoftReset,
    QUIT,
}

pub enum InputContext {
    AlbumView,
    PlaylistView,
    TrackList(Mode),
    Fullscreen,
    Search,
    Queue,
    Popup(PopupType),
}

pub enum SelectionType {
    Multi,
    Legal,
}

pub enum Director {
    Up(usize),
    Down(usize),
    Top,
    Bottom,
}

pub enum Incrementor {
    Up,
    Down,
}

thread_local! {
    static LAST_KEY_TIME: RefCell<Option<Instant>> = RefCell::new(None);
}

const PASTE_THRESHOLD: Duration = Duration::from_millis(10);

pub fn is_likely_paste() -> bool {
    LAST_KEY_TIME.with(|last_time| {
        let mut last = last_time.borrow_mut();

        let is_paste = match *last {
            Some(prev) => prev.elapsed() < PASTE_THRESHOLD,
            None => false,
        };
        *last = Some(Instant::now());
        is_paste
    })
}
