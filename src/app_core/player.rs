use anyhow::{Result, anyhow, bail};
use std::{sync::Arc, time::Duration};
use voxio::{EndReason, StartReason, VoxEvent};

use crate::{
    app_core::NoctaVox,
    key_handler::SelectionType,
    library::{SimpleSong, SongDatabase, SongInfo},
    playback::ValidatedSong,
    ui_state::{LibraryView, Mode},
};

impl NoctaVox {
    pub(crate) fn play_song(&mut self, song: &ValidatedSong) -> Result<()> {
        self.player.play(&song.path)?;
        self.ui.set_now_playing(Some(Arc::clone(&song.meta)));
        Ok(())
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
            let song = Arc::clone(&current);
            self.ui.insert_history_entry(&song);
        }

        self.play_song(&validated)?;
        self.force_sync();

        Ok(())
    }

    pub(crate) fn play_next(&mut self) -> Result<()> {
        let (next, current) = self.ui.playback.advance();

        match next {
            Some(song) => {
                self.play_song(&song)?;
                self.force_sync();
            }
            None => self.player.stop(),
        }
        self.ui.set_legal_songs();

        if let Some(np) = current {
            self.ui.insert_history_entry(&np);
        }

        Ok(())
    }

    pub(crate) fn play_prev(&mut self) -> Result<()> {
        let popped = self
            .ui
            .playback
            .pop_previous()?
            .ok_or_else(|| anyhow!("End of history!"))?;

        self.ui.delete_last_history_entry();

        self.play_song(&popped)?;
        self.force_sync();
        self.ui.set_legal_songs();
        Ok(())
    }

    pub fn stop(&mut self) {
        self.ui.playback.clear_queue();
        self.player.stop();
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

        let Some(song) = selection.or_else(|| self.ui.get_selected_song().ok()) else {
            return Ok(());
        };

        match !self.player.is_active() {
            true => {
                let validated = ValidatedSong::new(&song)?;
                self.play_song(&validated)?;
            }
            false => self.queue_song(&song)?,
        }

        self.ui.set_legal_songs();
        Ok(())
    }

    pub(super) fn handle_player_events(&mut self, event: VoxEvent) -> Result<()> {
        match event {
            VoxEvent::TrackStarted { reason, path, .. } => {
                let is_repeat = self.ui.playback.repeat_is_enabled();
                let gapless = matches!(reason, StartReason::Gapless);

                if gapless && !is_repeat {
                    self.advance_to_next_gapless();
                }

                let Some(song) = self.ui.playback.get_now_playing().cloned() else {
                    return Ok(());
                };

                if is_repeat {
                    let _ = self.player.set_next(path.to_str());
                }

                let is_restore = self.restored_song_id.take() == Some(song.get_id());

                if !is_restore {
                    song.update_play_count()?;
                }

                // Update if not on repeat and not gapless
                if !(is_repeat && gapless) {
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
                Ok(())
            }

            VoxEvent::DurationResolved { duration, .. } => {
                if let Some(np) = self.ui.get_now_playing() {
                    let _ = np.update_duration_db(duration);
                }
                Ok(())
            }

            VoxEvent::TrackEnded { path, reason } => {
                if matches!(reason, EndReason::Failed) {
                    bail!(
                        "Track failed with no decodeable packets.\n\nPath: {}",
                        path.display()
                    )
                }
                Ok(())
            }

            VoxEvent::Stopped => {
                if let Some(np) = self.ui.playback.get_now_playing() {
                    let song = Arc::clone(&np);
                    self.ui.insert_history_entry(&song);
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

            VoxEvent::StateChanged { paused } => {
                if let Some(mc) = self.media_controls.as_mut() {
                    let elapsed = self.player.elapsed();
                    match paused {
                        true => mc.set_paused(elapsed),
                        false => mc.set_playing(elapsed),
                    }
                }
                Ok(())
            }

            VoxEvent::Error { error: e, .. } => {
                self.ui.set_error(anyhow!(e));
                Ok(())
            }

            _ => Ok(()),
        }
    }
}
