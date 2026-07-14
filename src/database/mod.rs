use crate::{
    DB_PATH, SongMap,
    database::schema::CREATE_SCHEMA,
    library::{LongSong, SimpleSong, SongInfo},
    ui_state::LibraryStats,
    user_config,
};
use anyhow::Result;
use queries::*;
use rusqlite::{Connection, OptionalExtension, params};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::Path,
    sync::{Arc, atomic::AtomicU64},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

mod playlists;
mod queries;
mod schema;
mod snapshot;
mod worker;

pub(crate) const DB_BOUND: usize = 128;

pub use worker::DbWorker;

pub struct Database {
    conn: Connection,
    artist_map: HashMap<i64, Arc<String>>,
    album_map: HashMap<i64, Arc<String>>,
}

impl Database {
    pub fn open() -> Result<Self> {
        match Self::_open() {
            Ok(db) => Ok(db),
            Err(_) => {
                let backup = DB_PATH.with_file_name(format!(
                    "noctavox.db.corrupt-{}",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                ));
                fs::rename(&*DB_PATH, &backup)?;
                let _ = fs::remove_file(DB_PATH.with_extension("db-wal"));
                let _ = fs::remove_file(DB_PATH.with_extension("db-shm"));
                eprintln!(
                    "Database was corrupted; backup saved to {}",
                    backup.display()
                );
                Self::_open()
            }
        }
    }

    pub fn _open() -> Result<Self> {
        let conn = Connection::open(&*DB_PATH)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "cache_size", "1000")?;

        let mut db = Database {
            conn,
            artist_map: HashMap::new(),
            album_map: HashMap::new(),
        };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&mut self) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute_batch(CREATE_SCHEMA)?;
        tx.commit()?;

        Ok(())
    }

    // ===================
    //   SONG OPERATIONS
    // ===================

    pub(crate) fn insert_songs(&mut self, song_list: &[LongSong]) -> Result<()> {
        let artist_map = self.get_artist_map_name_to_id()?;
        let album_map = self.get_album_map_name_to_id()?;

        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(INSERT_SONG)?;

            for song in song_list {
                // Get artist ID for the song's artist
                let artist_id = artist_map.get(song.get_artist()).cloned();

                // Get artist ID for the album artist
                let album_artist_id = artist_map.get(song.album_artist.as_str()).cloned();

                // Look up album ID using both title and album artist ID
                let album_id = album_artist_id
                    .and_then(|aid| album_map.get(&(song.get_album().to_string(), aid)).cloned());

                if artist_id.is_none() || album_id.is_none() {
                    eprintln!(
                        "Skipping song {}: artist_id={:?}, album_id={:?}",
                        song.title, artist_id, album_id
                    );
                    continue;
                }

                stmt.execute(params![
                    song.id.to_le_bytes(),
                    &song.title,
                    &song.year,
                    &song.path.to_str(),
                    artist_id,
                    album_id,
                    &song.track_no,
                    &song.disc_no,
                    &song.duration.as_secs_f32(),
                    &song.channels,
                    &song.bitrate,
                    &song.sample_rate,
                    &song.filetype
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub(crate) fn get_all_songs(&mut self) -> Result<SongMap> {
        self.set_album_map()?;
        self.set_artist_map()?;

        let mut stmt = self.conn.prepare(GET_ALL_SONGS)?;

        let songs = stmt
            .query_map([], |row| {
                let hash = convert_from_bytes(row.get("id")?);

                let artist_id = row.get("artist_id")?;

                let artist = match self.artist_map.get(&artist_id) {
                    Some(a) => Arc::clone(a),
                    None => Arc::new("Unknown Artist".to_string()),
                };

                let duration_f32 = Duration::from_secs_f32(row.get("duration")?);
                let duration = AtomicU64::new(duration_f32.as_millis() as u64);

                let album_id = row.get("album_id")?;
                let album = match self.album_map.get(&album_id) {
                    Some(a) => Arc::clone(a),
                    None => Arc::new("Unknown Album".to_string()),
                };

                let song = SimpleSong {
                    id: hash,
                    title: row.get("title")?,
                    artist,
                    album,
                    album_id,
                    year: row.get("year")?,
                    track_no: row.get("track_no")?,
                    disc_no: row.get("disc_no")?,
                    duration,
                    filetype: row.get("format")?,
                };

                Ok((hash, Arc::new(song)))
            })?
            .filter_map(Result::ok)
            .collect();

        Ok(songs)
    }

    pub(crate) fn delete_songs(&mut self, to_delete: &[u64]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(DELETE_SONGS)?;
            for id in to_delete {
                stmt.execute([id.to_le_bytes()])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn update_play_count(&mut self, id: u64) -> Result<()> {
        let id = id.to_le_bytes();
        self.conn.execute(UPDATE_PLAY_COUNT, params![id])?;

        Ok(())
    }

    pub(crate) fn get_song_path(&mut self, id: u64) -> Result<String> {
        let output = self
            .conn
            .query_row(GET_PATH, [id.to_le_bytes()], |r| r.get(0))?;
        Ok(output)
    }

    pub(crate) fn get_hashes(&mut self) -> Result<HashSet<u64>> {
        let map = self
            .conn
            .prepare(GET_HASHES)?
            .query_map([], |row| Ok(convert_from_bytes(row.get("id")?)))?
            .filter_map(Result::ok)
            .collect::<HashSet<u64>>();

        Ok(map)
    }

    // =====================
    //   ARTIST AND ALBUMS
    // =====================

    pub(crate) fn insert_artists(&mut self, artists: &HashSet<&str>) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut insert_artists = tx.prepare(INSERT_ARTIST)?;
            for artist in artists {
                insert_artists.execute(params![artist])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn insert_albums(&mut self, aa_binding: &HashSet<(&str, &str)>) -> Result<()> {
        let artist_map = self.get_artist_map_name_to_id()?;
        let tx = self.conn.transaction()?;
        {
            let mut insert_albums = tx.prepare(INSERT_ALBUM)?;
            for (album_artist, album) in aa_binding {
                let artist_id = artist_map.get(*album_artist);
                insert_albums.execute(params![album, artist_id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn get_album_map(&mut self) -> Result<Vec<(i64, Arc<String>, Arc<String>)>> {
        let map = self
            .conn
            .prepare(ALBUM_BUILDER)?
            .query_map([], |row| {
                let album_id = row.get("id")?;
                let artist_id = row.get("artist_id")?;

                let artist = match self.artist_map.get(&artist_id) {
                    Some(a) => Arc::clone(a),
                    None => unreachable!(),
                };
                let album = match self.album_map.get(&album_id) {
                    Some(a) => Arc::clone(a),
                    None => unreachable!(),
                };

                Ok((album_id, album, artist))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(map)
    }

    /// Returns a hashmap of String: i64
    fn get_artist_map_name_to_id(&self) -> Result<HashMap<String, i64>> {
        let artist_map = self
            .conn
            .prepare(GET_ARTIST_MAP)?
            .query_map([], |row| Ok((row.get("name")?, row.get("id")?)))?
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(artist_map)
    }

    /// Get album title to ID mapping from a transaction
    fn get_album_map_name_to_id(&self) -> Result<HashMap<(String, i64), i64>> {
        let album_map = self
            .conn
            .prepare(GET_ALBUM_MAP)?
            .query_map([], |row| {
                Ok(((row.get("title")?, row.get("artist_id")?), row.get("id")?))
            })?
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(album_map)
    }

    fn set_artist_map(&mut self) -> Result<()> {
        self.artist_map = self
            .conn
            .prepare(GET_ARTIST_MAP)?
            .query_map([], |row| {
                Ok((row.get("id")?, Arc::from(row.get::<_, String>("name")?)))
            })?
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(())
    }

    fn set_album_map(&mut self) -> Result<()> {
        self.album_map = self
            .conn
            .prepare(GET_ALBUM_MAP)?
            .query_map([], |row| {
                Ok((row.get("id")?, Arc::from(row.get::<_, String>("title")?)))
            })?
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(())
    }

    pub(crate) fn update_duration(&mut self, song_id: u64, duration: f32) -> Result<()> {
        self.conn
            .execute(UPDATE_DURATION, params![duration, song_id.to_le_bytes()])?;

        Ok(())
    }

    // =============
    //   WAVEFORMS
    // =============

    pub fn get_waveform(&mut self, id: u64) -> Result<Vec<f32>> {
        let blob: Vec<u8> =
            self.conn
                .query_row(GET_WAVEFORM, params![id.to_le_bytes()], |row| row.get(0))?;

        let waveform = blob
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        Ok(waveform)
    }

    pub fn set_waveform(&mut self, id: u64, wf: &[f32]) -> Result<()> {
        let bytes: Vec<u8> = wf.iter().flat_map(|&f| f.to_le_bytes()).collect();

        self.conn
            .execute(INSERT_WAVEFORM, params![id.to_le_bytes(), bytes])?;

        Ok(())
    }

    // ============
    //   HISTORY
    // ============

    pub fn insert_to_history(&mut self, song_id: u64) -> Result<()> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        let tx = self.conn.transaction()?;

        {
            tx.execute(
                INSERT_INTO_HISTORY,
                params![song_id.to_le_bytes(), timestamp],
            )?;

            tx.execute(
                HISTORY_CLEANUP,
                params![user_config().general.history_capacity as u16],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn delete_recent_from_history(&mut self) -> Result<()> {
        self.conn.execute(HISTORY_DELETE_LATEST, params![])?;
        Ok(())
    }

    pub fn import_history(&mut self, song_map: &SongMap) -> Result<VecDeque<Arc<SimpleSong>>> {
        let mut history = VecDeque::new();

        let mut stmt = self.conn.prepare(LOAD_HISTORY)?;
        let rows = stmt.query_map(params![user_config().general.history_capacity], |row| {
            Ok(convert_from_bytes(row.get("song_id")?))
        })?;

        for song_id in rows.flatten() {
            if let Some(song) = song_map.get(&song_id) {
                history.push_back(Arc::clone(song));
            }
        }

        Ok(history)
    }

    // =================
    //   ROOTS & PATHS
    // =================

    pub(crate) fn get_roots(&mut self) -> Result<HashSet<String>> {
        let roots = self
            .conn
            .prepare(GET_ROOTS)?
            .query_map([], |row| row.get("path"))?
            .collect::<Result<HashSet<String>, _>>()?;

        Ok(roots)
    }

    pub(crate) fn set_root(&mut self, path: &Path) -> Result<()> {
        self.conn.execute(SET_ROOT, params![path.to_str()])?;
        Ok(())
    }

    pub(crate) fn delete_root(&mut self, path: &Path) -> Result<()> {
        self.conn.execute(DELETE_ROOT, params![path.to_str()])?;
        Ok(())
    }

    // ==========
    //   STATS
    // ==========

    pub(crate) fn get_stats(&mut self) -> Result<LibraryStats> {
        let x = self.conn.query_one(GET_STATS, params![], |row| {
            let total_tracks: u32 = row.get("total_tracks")?;
            let total_albums: u32 = row.get("albums")?;
            let total_artists: u32 = row.get("artists")?;
            let min_year: u32 = row.get("min_year")?;
            let max_year: u32 = row.get("max_year")?;
            let total_playlists: u32 = row.get("playlists")?;
            let unique_plays: u32 = row.get("unique_plays")?;
            let total_plays: u32 = row.get("total_plays").unwrap_or(0);
            let play_percentage: f32 = row.get("play_percentage")?;
            let total_duration: f32 = row.get("total_duration")?;

            Ok(LibraryStats {
                total_tracks,
                total_albums,
                total_artists,
                min_year,
                max_year,
                total_playlists,
                unique_plays,
                total_plays,
                play_percentage,
                total_duration,
            })
        })?;

        Ok(x)
    }

    pub fn get_most_played(&mut self, count: u16) -> Result<Vec<(u64, u16)>> {
        let mut stmt = self.conn.prepare(GET_TOP_SONGS)?;

        let rows = stmt
            .query_map(params![count], |row| {
                let hash_bytes: Vec<u8> = row.get("id")?;
                let hash = convert_from_bytes(hash_bytes);

                let plays: u16 = row.get("count")?;

                Ok((hash, plays))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn get_last_scan(&self) -> Result<Option<u64>> {
        self.conn
            .query_row(GET_LAST_SCAN, params![], |row| {
                Ok(convert_from_bytes(row.get(0)?))
            })
            .optional()
            .map_err(Into::into)
    }

    pub fn set_last_scan(&self, timestamp: u64) -> Result<()> {
        self.conn
            .execute(SET_LATEST_SCAN, params![timestamp.to_le_bytes()])?;
        Ok(())
    }

    pub fn get_last_played(&self) -> Result<(u64, f32)> {
        Ok(self.conn.query_row(GET_NOW_PLAYING, [], |row| {
            let raw_id = row.get(0)?;
            let elapsed: f32 = row.get(1)?;
            let id = convert_from_bytes(raw_id);

            Ok((id, elapsed))
        })?)
    }

    pub fn set_now_playing(&self, id: u64) -> Result<()> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        self.conn
            .execute(SET_NOW_PLAYING, params![id.to_le_bytes(), timestamp])?;

        Ok(())
    }

    pub fn update_np_elapsed(&self, elapsed_secs: f32) -> Result<()> {
        self.conn
            .execute(UPDATE_NOW_PLAYING, params![elapsed_secs])?;

        Ok(())
    }

    pub fn clear_now_playing(&self) -> Result<()> {
        self.conn.execute(CLEAR_NOW_PLAYING, params![])?;

        Ok(())
    }
}

#[inline]
fn convert_from_bytes(raw_bytes: Vec<u8>) -> u64 {
    match raw_bytes.try_into() {
        Ok(arr) => u64::from_le_bytes(arr),
        Err(a) => {
            eprintln!(
                "Warning: corrupted id in database (len {}, expected 8); skipping row",
                a.len()
            );
            0
        }
    }
}
