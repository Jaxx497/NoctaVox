use crate::{
    Library, key_handler::KeyBuffer, media_controls::MediaControlsHandle, player::PlayerHandle,
    ui_state::UiState,
};
use crossbeam_channel::Receiver;
use std::sync::Arc;

mod app;
mod key_events;
mod library;
mod playback;
mod player;
mod select;

pub use key_events::key_loop;

pub struct NoctaVox {
    library: Arc<Library>,
    pub(crate) ui: UiState,
    player: PlayerHandle,
    key_buffer: KeyBuffer,
    library_refresh_rec: Option<Receiver<LibraryRefreshProgress>>,
    media_controls: Option<MediaControlsHandle>,
    tick_sync: u32,
    restored_song_id: Option<u64>,
}

pub enum LibraryRefreshProgress {
    Scanning {
        progress: u8,
    },
    Processing {
        progress: u8,
        current: usize,
        total: usize,
    },
    UpdatingDatabase {
        progress: u8,
    },
    Rebuilding {
        progress: u8,
    },
    Complete(crate::Library),
    Error(String),
}
