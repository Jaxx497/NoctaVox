use anyhow::{Result, anyhow};
use std::{sync::Arc, time::Duration};

use crate::{
    app_core::NoctaVox,
    key_handler::SelectionType,
    library::{SimpleSong, SongDatabase, SongInfo},
    playback::ValidatedSong,
    player::{PlaybackState, PlayerEvent, VoxioTrack},
    ui_state::{LibraryView, Mode},
};

impl NoctaVox {
    pub(crate) fn play_song(&mut self, song: &ValidatedSong) -> Result<()> {
        let song = VoxioTrack::from(song);
        self.player.play(song)
    }

    pub(crate) fn play_selected_song(&mut self, count: usize) -> Result<()> {
        match count {
            0 => (),
            x => self.ui.go_to_track(x)?,
        }

        let song = self.ui.get_selected_song()?;

        if self.ui.get_mode() == &Mode::Queue {
            self.remove_song()?;
        }

        let validated = ValidatedSong::new(&song)?;

        if let Some(current) = self.ui.playback.get_now_playing() {
            self.ui.insert_history_entry(current.get_id());
            self.ui.playback.push_history(&Arc::clone(&current));
        }

        self.play_song(&validated)?;
        self.force_sync()?;

        Ok(())
    }

    pub(crate) fn play_next(&mut self) -> Result<()> {
        let (delta, next, current) = self.ui.playback.advance();

        match next {
            Some(song) => {
                self.play_song(&song)?;
                self.sync_player(&delta);
            }
            None => self.player.stop()?,
        }
        self.ui.set_legal_songs();

        if let Some(np) = current {
            self.ui.insert_history_entry(np.get_id());
        }

        Ok(())
    }

    pub(crate) fn play_prev(&mut self) -> Result<()> {
        let (delta, popped) = self
            .ui
            .playback
            .pop_previous()?
            .ok_or_else(|| anyhow!("End of history!"))?;

        self.ui.delete_last_history_entry();

        self.play_song(&popped)?;
        self.sync_player(&delta);
        self.ui.set_legal_songs();
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.ui.playback.clear_queue();
        self.player.stop()
    }

    pub fn remove_song(&mut self) -> Result<()> {
        match self.ui.get_mode() {
            Mode::Queue => match self.ui.multi_select_empty() {
                true => self.remove_from_queue()?,
                false => self.remove_from_queue_multi()?,
            },
            Mode::Library(LibraryView::Playlists) => match self.ui.multi_select_empty() {
                true => self.ui.remove_from_playlist()?,
                false => self.ui.remove_from_playlist_multi()?,
            },
            _ => {}
        }
        self.ui.set_legal_songs();
        Ok(())
    }

    pub fn queue_handler(&mut self, selection: Option<Arc<SimpleSong>>) -> Result<()> {
        if !self.ui.multi_select_empty() {
            return self.queue_selection(SelectionType::Multi, false);
        }

        let Some(ss) = selection.or_else(|| self.ui.get_selected_song().ok()) else {
            return Ok(());
        };

        match self.player.is_stopped() {
            true => {
                let validated = ValidatedSong::new(&ss)?;
                self.play_song(&validated)?;
            }
            false => self.queue_song(&ss)?,
        }

        self.ui.set_legal_songs();
        Ok(())
    }

    pub(super) fn handle_player_events(&mut self, event: PlayerEvent) -> Result<()> {
        match event {
            PlayerEvent::TrackStarted((prev_song, was_gapless)) => {
                let last_played_id = prev_song.id();
                let is_repeat = self.ui.playback.repeat_is_enabled();

                if was_gapless && !is_repeat {
                    self.advance_to_next_gapless();
                }

                let song = self.library.get_song_by_id(last_played_id).cloned();
                self.ui.set_now_playing(song);

                if is_repeat {
                    self.player.set_next(Some(prev_song.clone()))?;
                }

                let is_restore = self.restored_song_id.take() == Some(last_played_id);
                if let Some(song) = self.library.get_song_by_id(last_played_id).cloned() {
                    if !is_restore {
                        song.update_play_count()?;
                    }

                    // Update if not on repeat and not gapless
                    if !(is_repeat && was_gapless) {
                        self.ui.clear_waveform();
                        self.ui.request_waveform(&song);

                        if let Some(mc) = self.media_controls.as_mut() {
                            mc.update_metadata(
                                song.get_title(),
                                song.get_artist(),
                                song.get_album(),
                                song.get_duration(),
                            );
                            mc.set_playing(Duration::ZERO);
                        }
                    }
                }

                Ok(())
            }
            PlayerEvent::PlaybackStopped => {
                let (delta, next, current) = self.ui.playback.advance();

                if let Some(np) = current {
                    self.ui.insert_history_entry(np.get_id());
                }

                if let Some(song) = next {
                    self.play_song(&song)?;
                    self.sync_player(&delta);
                    return Ok(());
                }

                if let Some(mc) = self.media_controls.as_mut() {
                    mc.set_stopped();
                }

                if self.ui.get_mode() == Mode::Fullscreen {
                    self.ui.revert_fullscreen();
                }

                self.ui.set_now_playing(None);
                self.ui.clear_waveform();
                self.ui.set_legal_songs();

                Ok(())
            }
            PlayerEvent::Error(e) => {
                self.ui.set_error(anyhow!(e));
                Ok(())
            }
            PlayerEvent::StateChanged(state) => {
                if let Some(mc) = self.media_controls.as_mut() {
                    let elapsed = self.player.elapsed();
                    match state {
                        PlaybackState::Playing => mc.set_playing(elapsed),
                        PlaybackState::Paused => mc.set_paused(elapsed),
                        PlaybackState::Stopped => mc.set_stopped(),
                    }
                }
                Ok(())
            }
        }
    }
}
