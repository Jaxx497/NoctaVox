use crate::{Library, app_core::NoctaVox, library::RefreshProgress};
use anyhow::Result;
use std::{sync::Arc, thread};

impl NoctaVox {
    pub(crate) fn update_library(&mut self) -> Result<()> {
        if self.library_refresh_rec.is_some() {
            return Ok(());
        }

        let (tx, rx) = crossbeam_channel::bounded(1);
        self.library_refresh_rec = Some(rx);

        let progress = Arc::new(RefreshProgress::default());
        self.ui.library_refresh = Some(Arc::clone(&progress));

        thread::spawn(move || {
            let result = Library::init().and_then(|mut lib| {
                lib.rebuild_library(&progress)?;
                Ok(lib)
            });
            let _ = tx.send(result);
        });

        Ok(())
    }

    pub(super) fn handle_library_result(&mut self, result: Result<Library>) {
        match result {
            Ok(new_library) => {
                let cached = self.ui.display_state.album_pos.selected();
                let cached_offset = self.ui.display_state.album_pos.offset();
                let updated_len = new_library.albums.len();

                self.library = Arc::new(new_library);
                if let Err(e) = self.ui.sync_library(Arc::clone(&self.library)) {
                    self.ui.set_error(e);
                }

                if updated_len > 0 {
                    self.ui
                        .display_state
                        .album_pos
                        .select(match cached < Some(updated_len) {
                            true => cached,
                            false => Some(updated_len / 2),
                        });
                    *self.ui.display_state.album_pos.offset_mut() = cached_offset;
                }

                self.ui.set_legal_songs();
            }
            Err(e) => self.ui.set_error(e),
        }

        self.ui.library_refresh = None;
        self.library_refresh_rec = None;
    }
}
