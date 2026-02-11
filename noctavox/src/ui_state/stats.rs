use std::sync::Arc;

use crate::{SimpleSong, ui_state::UiState};

#[derive(Default)]
pub struct VoxStats {
    pub lib_stats: LibraryStats,
    pub top_played: Vec<(Arc<SimpleSong>, u16)>,
}

#[derive(Default)]
pub struct LibraryStats {
    pub total_tracks: u32,
    pub total_albums: u32,
    pub total_artists: u32,
    pub min_year: u32,
    pub max_year: u32,
    pub total_playlists: u32,
    pub unique_plays: u32,
    pub total_plays: u32,
    pub total_duration: f32,
    pub play_percentage: f32,
}

impl UiState {
    pub fn show_stats_popup(&mut self) -> anyhow::Result<()> {
        self.update_stats()?;
        self.show_popup(super::PopupType::Stats);

        Ok(())
    }

    fn update_stats(&mut self) -> anyhow::Result<()> {
        self.stats.lib_stats = self.db_worker.get_stats()?;
        self.stats.top_played = self
            .db_worker
            .get_most_played(20)?
            .into_iter()
            .filter_map(|(id, plays)| self.library.get_song_by_id(id).cloned().map(|s| (s, plays)))
            .collect::<Vec<_>>();

        Ok(())
    }

    pub fn get_lib_stats(&self) -> &LibraryStats {
        &self.stats.lib_stats
    }

    pub fn get_most_played(&self) -> &[(Arc<SimpleSong>, u16)] {
        &self.stats.top_played
    }
}
