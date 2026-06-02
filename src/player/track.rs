use std::path::{Path, PathBuf};

use crate::{
    library::{SimpleSong, SongDatabase},
    playback::ValidatedSong,
};

#[derive(Clone)]
pub struct VoxioTrack {
    id: u64,
    path: PathBuf,
}

impl PartialEq for VoxioTrack {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl TryFrom<&SimpleSong> for VoxioTrack {
    type Error = anyhow::Error;

    fn try_from(song: &SimpleSong) -> Result<Self, Self::Error> {
        Ok(Self {
            id: song.id,
            path: PathBuf::from(song.get_path()?),
        })
    }
}

impl From<&ValidatedSong> for VoxioTrack {
    fn from(song: &ValidatedSong) -> Self {
        VoxioTrack {
            id: song.id(),
            path: song.path(),
        }
    }
}

impl VoxioTrack {
    pub fn new<P: AsRef<Path>>(id: u64, p: P) -> Self {
        let path = PathBuf::from(p.as_ref());
        VoxioTrack { id, path }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
