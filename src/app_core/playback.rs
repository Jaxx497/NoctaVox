use crate::{
    app_core::NoctaVox,
    key_handler::{Director, Incrementor, SelectionType},
    library::{SimpleSong, SongDatabase},
    playback::ValidatedSong,
    ui_state::{LibraryView, Mode},
};
use anyhow::Result;
use rand::seq::SliceRandom;
use std::sync::Arc;

impl NoctaVox {
    pub fn advance_to_next_gapless(&mut self) {
        let (next, current) = self.ui.playback.advance();

        if let Some(np) = current {
            self.ui.insert_history_entry(&np);
        }

        if let Some(n) = &next {
            self.ui.set_now_playing(Some(Arc::clone(&n.meta)));
        }

        if self.ui.get_mode() == Mode::Queue {
            self.ui.set_legal_songs();
        }

        self.force_sync();
    }

    pub fn queue_song(&mut self, song: &Arc<SimpleSong>) -> Result<()> {
        self.ui.playback.enqueue(song)?;
        self.force_sync();
        Ok(())
    }

    pub fn queue_selection(&mut self, sel_type: SelectionType, shuffle: bool) -> Result<()> {
        let mut songs = self.ui.get_songs_by_selection(sel_type).unwrap_or_default();
        if songs.is_empty() {
            return Ok(());
        }

        if shuffle {
            songs.shuffle(&mut rand::rng());
        }

        if !self.player.is_active() {
            let first = songs.remove(0);
            let validated = ValidatedSong::new(&first)?;
            self.play_song(&validated)?;
        }

        self.ui.playback.enqueue_multi(&songs);
        self.force_sync();
        self.ui.set_legal_songs();
        Ok(())
    }

    pub fn push_queue_front(&mut self, song: &Arc<SimpleSong>) -> Result<()> {
        self.ui.playback.queue_push_front(song)?;
        self.force_sync();
        Ok(())
    }

    pub fn shuffle_queue(&mut self) {
        self.ui.playback.shuffle_queue();
        self.force_sync();
        self.ui.set_legal_songs();
    }

    pub fn remove_from_queue(&mut self) -> Result<()> {
        let idx = self.ui.get_selected_idx()?;
        self.ui.playback.remove_from_queue(idx);

        self.force_sync();
        Ok(())
    }

    pub fn remove_from_queue_multi(&mut self) -> Result<()> {
        let mut indicies = self.ui.get_multi_select_indices().clone();
        indicies.sort_unstable();

        for &idx in indicies.iter().rev() {
            self.ui.playback.remove_from_queue(idx);
        }

        self.force_sync();
        self.ui.clear_multi_select();
        Ok(())
    }

    pub fn shift_position(&mut self, dir: Incrementor) -> Result<()> {
        match self.ui.get_mode() {
            Mode::Queue => self.shift_queue_position(dir)?,
            Mode::Library(LibraryView::Playlists) => self.ui.shift_playlist_position(dir)?,
            _ => (),
        }

        self.ui.set_legal_songs();
        Ok(())
    }

    fn shift_queue_position(&mut self, dir: Incrementor) -> Result<()> {
        match self.ui.multi_select_empty() {
            true => self.shift_qposition_single(dir)?,
            false => self.shift_qposition_multi(dir),
        };

        self.force_sync();
        Ok(())
    }

    fn shift_qposition_single(&mut self, dir: Incrementor) -> Result<()> {
        let display_idx = self.ui.get_selected_idx()?;

        let target_idx = match dir {
            Incrementor::Up if display_idx > 0 => display_idx - 1,
            Incrementor::Down if display_idx < self.ui.playback.queue_len() - 1 => display_idx + 1,
            _ => return Ok(()),
        };

        self.ui.playback.swap(display_idx, target_idx);
        self.ui.scroll(match dir {
            Incrementor::Up => Director::Up(1),
            Incrementor::Down => Director::Down(1),
        });

        Ok(())
    }

    fn shift_qposition_multi(&mut self, dir: Incrementor) {
        let mut indices = self
            .ui
            .get_multi_select_indices()
            .iter()
            .copied()
            .collect::<Vec<_>>();

        indices.sort_unstable();
        let queue_len = self.ui.playback.queue_len();

        match dir {
            Incrementor::Up if indices[0] > 0 => {
                for idx in indices.iter_mut() {
                    self.ui.playback.swap(*idx, *idx - 1);
                    *idx -= 1;
                }
            }
            Incrementor::Down
                if indices[indices.len().saturating_sub(1)] < (queue_len.saturating_sub(1)) =>
            {
                for idx in indices.iter_mut().rev() {
                    self.ui.playback.swap(*idx, *idx + 1);
                    *idx += 1;
                }
            }
            _ => return,
        }

        self.ui.update_multi_select(indices);
    }

    pub fn force_sync(&self) {
        let next = match self.ui.playback.repeat_is_enabled() {
            true => self.ui.get_now_playing().and_then(|np| np.get_path().ok()),
            false => self
                .ui
                .playback
                .peek_queue_validated()
                .map(|s| s.path.to_string()),
        };

        let dr = next.as_deref();
        let _ = self.player.set_next(dr);
    }

    pub fn toggle_repeat(&mut self) {
        self.ui
            .playback
            .set_repeat(!self.ui.playback.repeat_is_enabled());
        self.force_sync();
    }
}
