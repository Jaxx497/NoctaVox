use crate::{
    Library, key_handler::KeyBuffer, media_controls::MediaControlsHandle, player::PlayerHandle,
    ui_state::UiState,
};
use anyhow::Result;
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
    library_refresh_rec: Option<Receiver<Result<Library>>>,
    media_controls: Option<MediaControlsHandle>,
    tick_sync: u32,
    restored_song_id: Option<u64>,
}
