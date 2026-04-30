use crossbeam::channel::{Receiver, select};
use ratatui::crossterm::event::KeyEvent;
use souvlaki::{MediaControlEvent, SeekDirection};

use crate::{DEFAULT_TICK, REFRESH_RATE, app_core::NoctaVox, key_handler};

impl NoctaVox {
    #[inline]
    pub fn select_shortcut(&mut self, key_rx: &Receiver<KeyEvent>) {
        select! {
            recv(self.player.events()) -> event => {
                if let Ok(event) = event {
                    if let Err(e) = self.handle_player_events(event) {
                        self.ui.set_error(e);
                    }
                }
            }

            recv(self.library_refresh_rec.as_ref().unwrap_or(&never())) -> progress => {
                if let Ok(progress) = progress {
                    self.handle_library_progress(progress)
                }
            }

            recv(&self.ui.wf_reciever().unwrap_or(&never())) -> result => {
                if let Ok(res) = result {
                    let now_playing = &self.ui.playback.get_now_playing().cloned();
                    self.ui.handle_wf_result(res, now_playing.as_ref());
                }
            }

            recv(self.media_controls.as_ref().map(|m| m.event_rx()).unwrap_or(&never())) -> event => {
                if let Ok(event) = event {
                    if let Err(e) = self.handle_media_control_event(event) {
                        self.ui.set_error(e);
                    }
                }
            }

            recv(key_rx) -> key => {
                if let Ok(key) = key {
                    if let Some(action) = key_handler::handle_key_event(key, &mut self.ui, &mut self.key_buffer) {
                        if let Err(e) = self.handle_action(action) {
                            self.ui.set_error(e);
                        }
                    }
                }
            }

            default(REFRESH_RATE) => {
                self.sync_media_controls_position();
            }
        }
    }

    fn handle_media_control_event(&mut self, event: MediaControlEvent) -> anyhow::Result<()> {
        match event {
            MediaControlEvent::Play => self.player.resume()?,
            MediaControlEvent::Pause => self.player.pause()?,
            MediaControlEvent::Toggle => self.player.toggle_playback()?,
            MediaControlEvent::Next => self.play_next()?,
            MediaControlEvent::Previous => self.play_prev()?,
            MediaControlEvent::Stop => self.stop()?,
            MediaControlEvent::Seek(SeekDirection::Forward) => self.player.seek_forward(5)?,
            MediaControlEvent::Seek(SeekDirection::Backward) => self.player.seek_back(5)?,
            MediaControlEvent::SeekBy(SeekDirection::Forward, dur) => {
                self.player.seek_forward(dur.as_secs())?
            }
            MediaControlEvent::SeekBy(SeekDirection::Backward, dur) => {
                self.player.seek_back(dur.as_secs())?
            }
            _ => {}
        }
        Ok(())
    }

    /// Called on every default tick (8ms), so we rate-limit with a counter.
    fn sync_media_controls_position(&mut self) {
        self.media_sync_tick = self.media_sync_tick.wrapping_add(1);
        if self.media_sync_tick % DEFAULT_TICK != 0 {
            return;
        }

        if let Some(ref mut mc) = self.media_controls {
            let elapsed = self.player.elapsed();
            if self.player.is_paused() {
                mc.set_paused(elapsed);
            } else if !self.player.is_stopped() {
                mc.set_playing(elapsed);
            }
        }
    }
}

#[inline]
fn never<T>() -> Receiver<T> {
    crossbeam::channel::never()
}
