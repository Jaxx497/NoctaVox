use crate::{
    Library, USER_CONFIG, UserConfig,
    app_core::{NoctaVox, key_loop},
    conf::{TIMING, Timing},
    key_handler::KeyBuffer,
    overwrite_line,
    player::{NoctavoxTrack, PlayerHandle},
    tui,
    ui_state::{Mode, PopupType, SettingsMode, UiState},
    user_config,
};
use anyhow::Result;
use std::sync::Arc;

impl NoctaVox {
    pub fn new() -> Result<Self> {
        let config_err = Self::load_config();
        Self::init_timings();

        let lib = Arc::new({
            let mut l = Library::init()?;
            l.build_library()?;
            l
        });

        let lib_clone = Arc::clone(&lib);

        let player = PlayerHandle::spawn()?;
        let metrics = player.metrics();

        let media_controls = crate::media_controls::MediaControlsHandle::new()
            .map_err(|e| eprintln!("OS media controls unavailable: {e}"))
            .ok();

        let mut nv = NoctaVox {
            library: lib,
            player,
            ui: UiState::new(lib_clone, metrics),
            library_refresh_rec: None,
            key_buffer: KeyBuffer::new(),
            media_controls,
            tick_sync: 0,
        };

        if let Some(e) = config_err {
            nv.ui.set_error(e);
        }

        Ok(nv)
    }

    pub fn run(&mut self) {
        match ratatui::run(|t| -> anyhow::Result<()> {
            self.preload_lib();
            self.restore_ui();
            let _ = self.restore_last_played();

            if self.library.roots.is_empty() {
                self.ui
                    .show_popup(PopupType::Settings(SettingsMode::AddRoot));
            }

            let key_rx = key_loop();

            loop {
                self.select_shortcut(&key_rx);
                t.draw(|f| tui::render(f, &mut self.ui))?;

                if self.ui.get_mode() == Mode::QUIT {
                    self.ui.update_now_playing_elapsed();
                    self.player.stop()?;
                    if let Some(mc) = self.media_controls.as_mut() {
                        mc.set_stopped();
                    }
                    break;
                }
            }
            Ok(())
        }) {
            Ok(_) => {
                let _ = overwrite_line("Shutting down... do not close terminal!");
                let _ = overwrite_line("Thank you for using NoctaVox!\n\n");
            }
            Err(e) => eprintln!("TERMINATED WITH ERROR: {e}"),
        };
    }

    fn load_config() -> Option<anyhow::Error> {
        match UserConfig::load() {
            Ok(cfg) => {
                let _ = USER_CONFIG.set(cfg);
                None
            }
            Err(e) => {
                let _ = USER_CONFIG.set(UserConfig::default());
                Some(e)
            }
        }
    }

    fn init_timings() {
        let fps = USER_CONFIG
            .get()
            .expect("Failed to read user config")
            .framerate;
        let _ = TIMING.set(Timing::from_fps(fps));
    }

    fn preload_lib(&mut self) {
        if let Err(e) = self.ui.sync_library(Arc::clone(&self.library)) {
            self.ui.set_error(e);
        }
        let _ = self.ui.playback.load_history(self.library.get_songs_map());
    }

    fn restore_ui(&mut self) {
        let _ = self.ui.restore_state();
    }

    fn restore_last_played(&mut self) -> Result<()> {
        if let Ok((song_id, elapsed_secs)) = self.ui.restore_last_played() {
            if let Some(song) = self.library.get_song_by_id(song_id) {
                let song = NoctavoxTrack::try_from(song.as_ref())?;
                self.player.play(song)?;
                self.player.seek_to(elapsed_secs)?;

                if !user_config().auto_resume {
                    self.player.pause()?;
                }
            }
        }
        Ok(())
    }
}
