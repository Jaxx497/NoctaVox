use crate::{
    Library, media_controls::MediaControlsHandle, player::PlayerHandle, ui_state::UiState,
};
use anyhow::Result;
use crossbeam_channel::Receiver;

mod app;
mod key_events;
mod library;
mod playback;
mod player;
mod select;

pub use key_events::key_loop;

pub struct NoctaVox {
    pub(crate) ui: UiState,
    player: PlayerHandle,
    library_refresh_rec: Option<Receiver<Result<Library>>>,
    media_controls: Option<MediaControlsHandle>,
    tick_sync: u32,
    restored_song_id: Option<u64>,
}

/// Two application event types:
/// - `Key` events are received from the `crossterm` event stream
/// - `Paste` events
pub enum AppEvent {
    Key(crossterm::event::KeyEvent),
    Paste(String),
}