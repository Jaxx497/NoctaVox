use super::{FileType, SongInfo};
use crate::{Database, DurationStyle, get_readable_duration};
use anyhow::Result;
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

pub struct SimpleSong {
    pub(crate) id: u64,
    pub(crate) title: String,
    pub(crate) artist: Arc<String>,
    pub(crate) year: Option<u32>,
    pub(crate) album: Arc<String>,
    pub(crate) album_id: i64,
    pub(crate) track_no: Option<u32>,
    pub(crate) disc_no: Option<u32>,
    pub(crate) duration: AtomicU64,
    pub(crate) filetype: FileType,
}

/// DATABASE RELATED METHODS
impl super::SongDatabase for SimpleSong {
    /// Returns the path of a song as a String
    fn get_path(&self) -> Result<String> {
        let mut db = Database::open()?;
        db.get_song_path(self.id)
    }

    /// Update the play_count of the song
    fn update_play_count(&self) -> Result<()> {
        let mut db = Database::open()?;
        db.update_play_count(self.id)
    }

    /// Retrieve the waveform of a song
    /// returns Result<Vec<f32>>
    fn get_waveform_db(&self) -> Result<Vec<f32>> {
        let mut db = Database::open()?;
        db.get_waveform(self.id)
    }

    /// Store the waveform of a song in the databse
    fn set_waveform_db(&self, wf: &[f32]) -> Result<()> {
        let mut db = Database::open()?;
        db.set_waveform(self.id, wf)
    }

    fn update_duration_db(&self, cand: Duration) -> anyhow::Result<()> {
        let cand_f32 = cand.as_secs_f32();
        let current = self.get_duration_f32();

        let changed = cand_f32 > 0.5 && (cand_f32 - current).abs() > 0.5;

        if changed {
            self.duration
                .store(cand.as_millis() as u64, Ordering::Release);
            let mut db = Database::open()?;
            db.update_duration(self.id, cand_f32)?
        }

        Ok(())
    }
}

/// Generic getter methods
impl SongInfo for SimpleSong {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn get_title(&self) -> &str {
        &self.title
    }

    fn get_artist(&self) -> &str {
        &self.artist
    }

    fn get_album(&self) -> &str {
        &self.album
    }

    fn get_duration(&self) -> Duration {
        let ms = self.duration.load(Ordering::Acquire);
        Duration::from_millis(ms)
    }

    fn get_duration_f32(&self) -> f32 {
        let ms = self.duration.load(Ordering::Acquire);
        Duration::from_millis(ms).as_secs_f32()
    }

    fn get_duration_str(&self, style: DurationStyle) -> String {
        get_readable_duration(self.get_duration(), style)
    }
}

impl SongInfo for Arc<SimpleSong> {
    fn get_id(&self) -> u64 {
        self.as_ref().get_id()
    }

    fn get_title(&self) -> &str {
        self.as_ref().get_title()
    }

    fn get_artist(&self) -> &str {
        self.as_ref().get_artist()
    }

    fn get_album(&self) -> &str {
        self.as_ref().get_album()
    }

    fn get_duration(&self) -> Duration {
        self.as_ref().get_duration()
    }

    fn get_duration_f32(&self) -> f32 {
        self.as_ref().get_duration_f32()
    }

    fn get_duration_str(&self, style: DurationStyle) -> String {
        self.as_ref().get_duration_str(style)
    }
}
