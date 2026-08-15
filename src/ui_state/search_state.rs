use super::{Pane, UiState, new_textarea};
use crate::{
    library::{SimpleSong, SongInfo},
    strip_diacritics,
};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::crossterm::event::KeyEvent;
use ratatui_textarea::TextArea;
use std::{collections::HashMap, sync::Arc};
use unicode_normalization::char::{decompose_canonical, is_combining_mark};

const MATCH_THRESHOLD: i64 = 80;
const MATCH_LIMIT: usize = 1024;

// Title matches are worth more than artist or album matches
const TITLE_WEIGHT: f32 = 2.0;
const ARTIST_WEIGHT: f32 = 1.7;
const ALBUM_WEIGHT: f32 = 1.7;

#[derive(Copy, Clone)]
pub enum MatchField {
    Title,
    Artist,
    Album,
}

#[derive(Default)]
pub struct MatchSpans {
    pub title: Vec<u32>,
    pub artist: Vec<u32>,
    pub album: Vec<u32>,
}

pub struct SearchState {
    pub input: TextArea<'static>,
    matcher: SkimMatcherV2,
    pub(super) match_fields: HashMap<u64, MatchField>,
    pub(super) match_spans: HashMap<u64, MatchSpans>,
}

impl SearchState {
    pub fn new() -> Self {
        SearchState {
            input: new_textarea("Enter search query"),
            matcher: SkimMatcherV2::default(),
            match_fields: HashMap::new(),
            match_spans: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.get().len()
    }

    fn get(&self) -> &str {
        &self.input.lines()[0]
    }

    pub fn get_widget_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.input
    }
}

impl UiState {
    // Algorithm looks at the title, artist, and album fields
    // and scores each attribute while applying a heavier
    // weight to the title field and returns the highest score.
    // Assuming the score is higher than the threshold, the
    // result is valid. Results are ordered by score.
    pub(crate) fn filter_songs_by_search(&mut self) {
        let raw_search_str = self.search.get();
        let query = strip_diacritics(raw_search_str);
        let matcher = &self.search.matcher;

        // [title, artist, album]
        let mut scored_songs: Vec<(Arc<SimpleSong>, i64, [i64; 3])> = self
            .library
            .get_all_songs()
            .iter()
            .filter_map(|song| {
                let weighted_score = [
                    field_score(matcher, song.get_title(), &query, TITLE_WEIGHT),
                    field_score(matcher, song.get_artist(), &query, ARTIST_WEIGHT),
                    field_score(matcher, song.get_album(), &query, ALBUM_WEIGHT),
                ];
                let best_score = weighted_score.iter().max().copied().unwrap_or(0);

                (best_score > MATCH_THRESHOLD)
                    .then(|| (Arc::clone(song), best_score, weighted_score))
            })
            .collect();

        scored_songs.sort_by(|a, b| b.1.cmp(&a.1));
        scored_songs.truncate(MATCH_LIMIT);

        self.search.match_fields.clear();
        self.search.match_spans.clear();

        for (song, best_score, [title, artist, album]) in &scored_songs {
            let match_field = match best_score {
                s if s == title => MatchField::Title,
                s if s == artist => MatchField::Artist,
                _ => MatchField::Album,
            };

            let spans = MatchSpans {
                title: field_spans(matcher, song.get_title(), &query, *title),
                artist: field_spans(matcher, song.get_artist(), &query, *artist),
                album: field_spans(matcher, song.get_album(), &query, *album),
            };

            self.search.match_fields.insert(song.get_id(), match_field);
            self.search.match_spans.insert(song.get_id(), spans);
        }

        self.legal_songs = scored_songs.into_iter().map(|i| i.0).collect();
    }

    pub fn send_search(&mut self) {
        match !self.legal_songs.is_empty() {
            true => self.set_pane(Pane::TrackList),
            false => self.soft_reset(),
        }
    }

    pub fn process_search(&mut self, k: KeyEvent) {
        self.search.input.input(k);
        self.set_legal_songs();
        match self.legal_songs.is_empty() {
            true => self.nav.table_pos.select(None),
            false => self.nav.table_pos.select(Some(0)),
        }
    }

    pub fn get_match_fields(&self, song_id: u64) -> Option<MatchField> {
        self.search.match_fields.get(&song_id).copied()
    }

    pub fn get_match_spans(&self, song_id: u64) -> Option<&MatchSpans> {
        self.search.match_spans.get(&song_id)
    }
}

fn field_score(matcher: &SkimMatcherV2, text: &str, query: &str, weight: f32) -> i64 {
    matcher
        .fuzzy_match(&strip_diacritics(text), query)
        .map_or(0, |score| (score as f32 * weight) as i64)
}

fn field_spans(matcher: &SkimMatcherV2, text: &str, query: &str, weighted_score: i64) -> Vec<u32> {
    if weighted_score <= MATCH_THRESHOLD {
        return Vec::new();
    }

    let (folded, origins) = fold_indexed(text);
    match matcher.fuzzy_indices(&folded, query) {
        Some((_, indices)) => {
            let mut offsets: Vec<u32> = indices
                .iter()
                .filter_map(|&i| origins.get(i).copied())
                .collect();

            offsets.dedup();
            offsets
        }
        None => Vec::new(),
    }
}

fn fold_indexed(s: &str) -> (String, Vec<u32>) {
    let mut folded = String::with_capacity(s.len());
    let mut origins = Vec::with_capacity(s.len());

    for (i, c) in s.chars().enumerate() {
        decompose_canonical(c, |d| {
            if !is_combining_mark(d) {
                for lower in d.to_lowercase() {
                    folded.push(lower);
                    origins.push(i as u32);
                }
            }
        });
    }

    (folded, origins)
}
