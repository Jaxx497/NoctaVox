use crate::{
    DurationStyle,
    library::{SimpleSong, SongDatabase, SongInfo},
};
use anyhow::Result;
use std::{path::PathBuf, sync::Arc, time::Duration};

pub struct ValidatedSong {
    pub meta: Arc<SimpleSong>,
    pub path: String,
}

impl ValidatedSong {
    pub fn new(song: &Arc<SimpleSong>) -> Result<Arc<Self>> {
        let path = song.get_path()?;

        std::fs::metadata(&path)?;

        Ok(Arc::new(Self {
            meta: Arc::clone(&song),
            path,
        }))
    }

    pub fn id(&self) -> u64 {
        self.meta.get_id()
    }

    pub fn path_str(&self) -> String {
        self.path.clone()
    }

    pub fn path(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }
}

impl SongInfo for ValidatedSong {
    fn get_id(&self) -> u64 {
        self.meta.id
    }

    fn get_title(&self) -> &str {
        &self.meta.title
    }

    fn get_artist(&self) -> &str {
        &self.meta.artist
    }

    fn get_album(&self) -> &str {
        &self.meta.album
    }

    fn get_duration(&self) -> Duration {
        self.meta.get_duration()
    }

    fn get_duration_f32(&self) -> f32 {
        self.meta.get_duration_f32()
    }

    fn get_duration_str(&self, style: DurationStyle) -> String {
        self.meta.get_duration_str(style)
    }
}
