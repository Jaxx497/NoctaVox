use std::path::PathBuf;

use crate::{
    library::{SimpleSong, SongDatabase},
    playback::ValidatedSong,
};

#[derive(Clone)]
pub struct NoctavoxTrack {
    id: u64,
    path: PathBuf,
}

impl PartialEq for NoctavoxTrack {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl TryFrom<&SimpleSong> for NoctavoxTrack {
    type Error = anyhow::Error;

    fn try_from(song: &SimpleSong) -> Result<Self, Self::Error> {
        Ok(Self {
            id: song.id,
            path: PathBuf::from(song.get_path()?),
        })
    }
}

impl From<&ValidatedSong> for NoctavoxTrack {
    fn from(song: &ValidatedSong) -> Self {
        NoctavoxTrack {
            id: song.id(),
            path: song.path(),
        }
    }
}

impl NoctavoxTrack {
    pub fn new(id: u64, path: PathBuf) -> Self {
        NoctavoxTrack { id, path }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
