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
                if let Err(e) = self.ui.sync_library(Arc::new(new_library)) {
                    self.ui.set_error(e);
                }
                self.ui.set_legal_songs();
            }
            Err(e) => self.ui.set_error(e),
        }

        self.ui.library_refresh = None;
        self.library_refresh_rec = None;
    }
}
