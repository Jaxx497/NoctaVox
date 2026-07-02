use super::LEGAL_EXTENSION;
use crate::{
    SongMap, calculate_signature,
    database::Database,
    expand_tilde,
    library::{Album, LongSong, RefreshProgress, RefreshStage, SimpleSong, SongInfo},
    user_config,
};
use anyhow::{Result, anyhow};
use indexmap::IndexMap;
use rayon::prelude::*;
use std::{
    collections::{HashSet, VecDeque},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
use walkdir::WalkDir;

const SCANNING_PRE: u8 = 5;
const PARSING_BASE: u8 = 20;
const DB_OPEN: u8 = 70;
const DB_INSERT_BASE: u8 = 72;
const DB_BASE: u8 = 85;

pub struct Library {
    db: Database,
    pub roots: HashSet<PathBuf>,
    pub songs: SongMap,
    pub albums: IndexMap<i64, Album>,
}

impl Library {
    fn new() -> Result<Self> {
        let db = Database::open()?;
        Ok(Library {
            db,
            roots: HashSet::new(),
            songs: SongMap::default(),
            albums: IndexMap::new(),
        })
    }

    pub fn init() -> Result<Self> {
        let mut lib = Self::new()?;

        if let Ok(db_roots) = lib.db.get_roots() {
            lib.roots.extend(
                db_roots
                    .into_iter()
                    .filter_map(|r| PathBuf::from(r).canonicalize().ok()),
            );
        };

        Ok(lib)
    }

    pub fn init_and_build() -> Result<Self> {
        let mut lib = Self::init()?;

        match user_config().update_on_start {
            true => lib.build_library()?,
            false => {
                lib.collect_songs()?;
                lib.build_albums()?;
            }
        }
        Ok(lib)
    }

    /// Collect valid files from a root directory
    ///
    /// Function collects valid files with vetted extensions
    /// Currently, proper extensions are MP3, FLAC, and M4A
    ///
    /// Folders with a `.nomedia` file will be ignored
    fn collect_valid_files(dir: impl AsRef<Path>) -> impl ParallelIterator<Item = PathBuf> {
        WalkDir::new(dir)
            .into_iter()
            .filter_entry(|e| {
                !e.path().join(".nomedia").exists()
                    && !e.path().to_string_lossy().contains("$RECYCLE.BIN")
            })
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .collect::<Vec<_>>()
            .into_par_iter()
            .filter(move |entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| LEGAL_EXTENSION.contains(ext.to_lowercase().as_str()))
                    .unwrap_or(false)
            })
            .filter_map(|e| e.path().canonicalize().ok())
    }

    /// Push everything to db
    fn commit_to_db(
        db: &mut Database,
        songs: &[LongSong],
        progress: &RefreshProgress,
    ) -> Result<()> {
        let mut artist_cache = HashSet::new();
        let mut aa_binding = HashSet::new();

        // Fill maps
        for song in songs {
            artist_cache.insert(song.get_artist());
            artist_cache.insert(song.album_artist.as_str());
            aa_binding.insert((song.album_artist.as_str(), song.get_album()));
        }

        progress.set(RefreshStage::Database, DB_OPEN);

        // ORDER IS IMPORTANT HERE
        db.insert_artists(&artist_cache)?;
        db.insert_albums(&aa_binding)?;

        let total = songs.len();
        let mut done = 0;
        for chunk in songs.chunks(500) {
            db.insert_songs(chunk)?;
            done += chunk.len();
            progress.set(
                RefreshStage::Database,
                DB_INSERT_BASE + (done * 13 / total.max(1)) as u8,
            );
        }

        Ok(())
    }

    fn collect_songs(&mut self) -> Result<()> {
        self.songs = self.db.get_all_songs()?;
        Ok(())
    }

    fn build_albums(&mut self) -> Result<()> {
        let aa_cache = self.db.get_album_map()?;
        self.albums = IndexMap::with_capacity(aa_cache.len());

        // Create album instances from album_artist/album_title combination
        for (album_id, album_name, artist_name) in aa_cache {
            let album = Album::from_aa(album_id, album_name, artist_name);
            self.albums.insert(album_id, album);
        }

        let mut album_songs: IndexMap<i64, Vec<Arc<SimpleSong>>> =
            IndexMap::with_capacity(self.albums.len());

        for song in self.songs.values() {
            album_songs
                .entry(song.album_id)
                .or_insert_with(Vec::new)
                .push(Arc::clone(&song));
        }

        for (album_id, mut songs) in album_songs {
            if let Some(album) = self.albums.get_mut(&album_id) {
                if !songs.is_empty() {
                    if album.year.is_none() {
                        album.year = songs[0].year
                    }

                    songs.sort_by_key(|s| (s.disc_no.unwrap_or(0), s.track_no.unwrap_or(0)));
                    album.tracklist = songs.into()
                }
            }
        }

        self.albums.retain(|_id, album| !album.tracklist.is_empty());

        Ok(())
    }

    fn any_root_modified(&self) -> Result<bool> {
        let last_scan = match self.db.get_last_scan()? {
            None => return Ok(true),
            Some(t) => t,
        };

        for root in &self.roots {
            let modified = WalkDir::new(root)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_dir())
                .any(|e| {
                    let check_modified = || -> Option<bool> {
                        let m = e.metadata().ok()?;
                        let t = m.modified().ok()?;
                        let d = t.duration_since(UNIX_EPOCH).ok()?;
                        Some(d.as_secs() > last_scan)
                    };

                    check_modified().unwrap_or(true)
                });

            if modified {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl Library {
    pub fn add_root(&mut self, root: impl AsRef<Path>) -> Result<()> {
        let expanded_path = expand_tilde(root.as_ref())?;
        let canon = PathBuf::from(expanded_path)
            .canonicalize()
            .map_err(|_| anyhow!("Path does not exist! {}", root.as_ref().display()))?;

        if self.roots.insert(canon.clone()) {
            self.db.set_root(&canon)?;
        }

        Ok(())
    }

    pub fn delete_root(&mut self, root: &str) -> Result<()> {
        let bad_root = PathBuf::from(root);
        self.roots.remove(&bad_root);
        self.db.delete_root(&bad_root)
    }

    /// Build the library based on the current state of the database.
    pub fn build_library(&mut self) -> Result<()> {
        if self.roots.is_empty() {
            return Ok(());
        }

        match !self.any_root_modified()? {
            true => {
                self.collect_songs()?;
                self.build_albums()?;
            }
            false => self.rebuild_library(&RefreshProgress::default())?,
        }

        Ok(())
    }

    pub fn get_songs_map(&self) -> &SongMap {
        &self.songs
    }

    pub fn get_song_by_id(&self, id: u64) -> Option<&Arc<SimpleSong>> {
        self.songs.get(&id)
    }

    pub fn load_history(&mut self, songs: &SongMap) -> Result<VecDeque<Arc<SimpleSong>>> {
        self.db.import_history(songs)
    }

    pub fn get_all_songs(&self) -> Vec<Arc<SimpleSong>> {
        self.songs.values().cloned().collect()
    }
}

impl Library {
    pub fn rebuild_library(&mut self, progress: &RefreshProgress) -> Result<()> {
        if self.roots.is_empty() {
            return Ok(());
        }

        progress.set(RefreshStage::Scanning, 0);

        let mut existing_hashes = self.db.get_hashes()?;
        let mut all_files = Vec::new();

        // First pass: collect all files from all roots
        for root in &self.roots {
            all_files.extend(Self::collect_valid_files(root).collect::<Vec<_>>());
        }

        progress.set(RefreshStage::Scanning, SCANNING_PRE);

        // Second pass: Filter files
        let total_files = all_files.len();
        let mut new_files = Vec::new();

        for (i, path) in all_files.into_iter().enumerate() {
            progress.set(
                RefreshStage::Scanning,
                SCANNING_PRE + ((i + 1) * 15 / total_files.max(1)) as u8,
            );

            match calculate_signature(&path) {
                Ok(hash) => {
                    if !existing_hashes.remove(&hash) {
                        new_files.push(path);
                    }
                }
                Err(_) => {}
            }
        }

        // Phase 2: Processing song metadata
        let removed_ids = existing_hashes.into_iter().collect::<Vec<u64>>();

        // 2.1 Inserting songs
        match new_files.is_empty() {
            true => progress.set(RefreshStage::Database, DB_BASE),
            false => Self::process_new_files(&mut self.db, new_files, progress)?,
        }

        // 2.1 Deleting songs
        // Delete in batches for progress reporting
        let total_removed = removed_ids.len();
        for (i, chunk) in removed_ids.chunks(100).enumerate() {
            progress.set(
                RefreshStage::Database,
                (DB_BASE + (i * 100 * 5 / total_removed.max(1)) as u8).min(90),
            );
            self.db.delete_songs(chunk)?;
        }

        // Phase 3: Collecting & Rebuilding
        progress.set(RefreshStage::Rebuilding, 90);

        self.collect_songs()?;
        progress.set(RefreshStage::Rebuilding, 95);

        self.build_albums()?;
        progress.set(RefreshStage::Rebuilding, 100);

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        self.db.set_last_scan(timestamp)
    }

    fn process_new_files(
        db: &mut Database,
        new_files: Vec<PathBuf>,
        progress: &RefreshProgress,
    ) -> Result<()> {
        let total = new_files.len();
        let processed = AtomicUsize::new(0);

        let songs: Vec<LongSong> = new_files
            .into_par_iter()
            .filter_map(|path| {
                let song = LongSong::build_song_symphonia(path).ok();
                let count = processed.fetch_add(1, Ordering::Relaxed) + 1;

                progress.set(
                    RefreshStage::Parsing,
                    PARSING_BASE + (count * 50 / total.max(1)) as u8,
                );
                progress.set_counts(count, total);

                song
            })
            .collect();

        Self::commit_to_db(db, &songs, progress)?;
        Ok(())
    }
}
