use crate::{
    Library,
    app_core::{NoctaVox, key_loop},
    key_handler::KeyBuffer,
    overwrite_line,
    player::PlayerHandle,
    tui,
    ui_state::{Mode, PopupType, SettingsMode, UiState},
};
use anyhow::Result;
use std::sync::Arc;

impl NoctaVox {
    pub fn new() -> Result<Self> {
        let lib = Arc::new({
            let mut l = Library::init();
            let _ = l.build_library();
            l
        });

        let lib_clone = Arc::clone(&lib);

        let player = PlayerHandle::spawn()?;
        let metrics = player.metrics();

        let media_controls = crate::media_controls::MediaControlsHandle::new()
            .map_err(|e| eprintln!("OS media controls unavailable: {e}"))
            .ok();

        Ok(NoctaVox {
            library: lib,
            player,
            ui: UiState::new(lib_clone, metrics),
            library_refresh_rec: None,
            key_buffer: KeyBuffer::new(),
            media_controls,
            media_sync_tick: 0,
        })
    }

    pub fn run(&mut self) {
        match ratatui::run(|t| -> anyhow::Result<()> {
            self.preload_lib();
            self.initialize_ui();
            t.draw(|f| tui::render(f, &mut self.ui))?;

            if self.library.roots.is_empty() {
                self.ui
                    .show_popup(PopupType::Settings(SettingsMode::AddRoot));
            }

            let key_rx = key_loop();

            loop {
                self.select_shortcut(&key_rx);
                t.draw(|f| tui::render(f, &mut self.ui))?;

                if self.ui.get_mode() == Mode::QUIT {
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

    pub fn preload_lib(&mut self) {
        if let Err(e) = self.ui.sync_library(Arc::clone(&self.library)) {
            self.ui.set_error(e);
        }
        let _ = self.ui.playback.load_history(self.library.get_songs_map());
    }

    pub fn initialize_ui(&mut self) {
        self.ui.soft_reset();
        let _ = self.ui.restore_state();
    }
}
