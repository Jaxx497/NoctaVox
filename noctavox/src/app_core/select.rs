use crossbeam::channel::{Receiver, select};
use ratatui::crossterm::event::KeyEvent;

use crate::{REFRESH_RATE, app_core::NoctaVox, key_handler};

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

            recv(key_rx) -> key => {
                if let Ok(key) = key {
                    if let Some(action) = key_handler::handle_key_event(key, &self.ui) {
                        let _ = self.handle_action(action);
                        }
                    }
                }

            default(REFRESH_RATE) => {}
        }
    }
}

#[inline]
fn never<T>() -> Receiver<T> {
    crossbeam::channel::never()
}
