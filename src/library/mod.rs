mod domain;
mod vox_library;

pub use domain::LEGAL_EXTENSION;
pub use domain::{
    Album, FileType, LongSong, Playlist, PlaylistSong, RefreshProgress, RefreshStage, SimpleSong,
    SongDatabase, SongInfo,
};
pub use vox_library::Library;
