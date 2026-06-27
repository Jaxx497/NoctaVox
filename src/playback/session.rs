use crate::{Database, SongMap, library::SimpleSong, playback::ValidatedSong, user_config};
use anyhow::Result;
use rand::seq::SliceRandom;
use std::{
    collections::{HashSet, VecDeque},
    sync::Arc,
};

pub struct PlaybackSession {
    repeat: bool,

    queue: VecDeque<Arc<ValidatedSong>>,
    history: VecDeque<Arc<SimpleSong>>,
    queue_ids: HashSet<u64>,

    now_playing: Option<Arc<SimpleSong>>,
}

impl PlaybackSession {
    pub fn init() -> Self {
        PlaybackSession {
            repeat: false,

            queue: VecDeque::new(),
            history: VecDeque::with_capacity(user_config().history_capacity as usize),
            queue_ids: HashSet::new(),
            now_playing: None,
        }
    }

    pub fn peek_queue(&self) -> Option<&Arc<SimpleSong>> {
        self.queue.front().map(|s| &s.meta)
    }

    pub fn peek_queue_validated(&self) -> Option<&Arc<ValidatedSong>> {
        self.queue.front()
    }

    pub fn get_now_playing(&self) -> Option<&Arc<SimpleSong>> {
        self.now_playing.as_ref()
    }

    pub fn set_now_playing(&mut self, song: Option<Arc<SimpleSong>>) {
        self.now_playing = song
    }

    pub fn get_queue(&mut self) -> Vec<Arc<SimpleSong>> {
        self.queue
            .make_contiguous()
            .iter()
            .map(|s| Arc::clone(&s.meta))
            .collect()
    }

    // =====================
    //    QUEUE METHODS
    // =====================
    pub fn enqueue(&mut self, song: &Arc<SimpleSong>) -> Result<()> {
        let validated = ValidatedSong::new(song)?;

        self.queue_ids.insert(validated.id());
        self.queue.push_back(validated);

        Ok(())
    }

    pub fn enqueue_multi(&mut self, songs: &[Arc<SimpleSong>]) {
        for song in songs {
            if let Ok(validated) = ValidatedSong::new(song) {
                self.queue_ids.insert(validated.id());
                self.queue.push_back(validated);
            }
        }
    }

    /// Push song to front of queue
    pub fn queue_push_front(&mut self, song: &Arc<SimpleSong>) -> Result<()> {
        let validated = ValidatedSong::new(song)?;

        self.queue_ids.insert(validated.id());
        self.queue.push_front(Arc::clone(&validated));

        Ok(())
    }

    pub fn advance(&mut self) -> (Option<Arc<ValidatedSong>>, Option<Arc<SimpleSong>>) {
        let pushed = self.now_playing.take();

        let next = self.queue.pop_front().map(|song| {
            self.remove_id_if_final(song.id());
            song
        });

        (next, pushed)
    }

    pub fn remove_from_queue(&mut self, idx: usize) -> Option<Arc<ValidatedSong>> {
        self.queue.remove(idx).map(|s| {
            self.remove_id_if_final(s.id());
            s
        })
    }

    pub fn clear_queue(&mut self) {
        self.queue.clear();
        self.queue_ids.clear();
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        if a.max(b) >= self.queue.len() || a == b {
            return;
        }

        self.queue.swap(a, b);
    }

    pub fn shuffle_queue(&mut self) {
        self.queue.make_contiguous().shuffle(&mut rand::rng());
    }

    pub fn is_queued(&self, id: u64) -> bool {
        self.queue_ids.contains(&id)
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn queue_is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    // ======================
    //    HISTORY METHODS
    // ======================
    //

    pub(crate) fn load_history(&mut self, song_map: &SongMap) -> Result<()> {
        let mut db = Database::open()?;
        self.history = db.import_history(song_map)?;
        Ok(())
    }

    pub(crate) fn push_history(&mut self, song: &Arc<SimpleSong>) {
        self.history.push_front(Arc::clone(song));
        if self.history.len() > user_config().history_capacity as usize {
            self.history.pop_back();
        }
    }

    pub fn pop_previous(&mut self) -> Result<Option<Arc<ValidatedSong>>> {
        // Handle nothing in history
        let last_played = match self.history.pop_front() {
            Some(song) => song,
            None => return Ok(None),
        };

        // If something is playing, place it back in the queue
        if let Some(current) = self.now_playing.take() {
            let validated = ValidatedSong::new(&current)?;
            self.queue_ids.insert(validated.id());
            self.queue.push_front(validated);
        }

        // Validate what was popped, set as now playing
        let validated_popped = ValidatedSong::new(&last_played)?;
        self.now_playing = Some(last_played);

        Ok(Some(validated_popped))
    }

    pub fn repeat_is_enabled(&self) -> bool {
        self.repeat
    }

    pub fn set_repeat(&mut self, status: bool) {
        self.repeat = status
    }

    fn remove_id_if_final(&mut self, id: u64) {
        if !self.queue.iter().any(|s| s.id() == id) {
            self.queue_ids.remove(&id);
        }
    }
}
