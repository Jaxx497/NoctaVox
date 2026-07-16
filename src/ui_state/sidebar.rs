use crate::{
    SimpleSong,
    ui_state::{AlbumSort, NodeKey, Pane, RowKind, SidebarRow, UiState, domain::Root},
};
use indexmap::IndexMap;
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use ratatui::widgets::ListState;
use std::{collections::HashSet, sync::Arc};

pub struct Sidebar {
    pub album_sort: AlbumSort,
    pub collapsed: HashSet<NodeKey>,
    pub rows: Vec<SidebarRow>,
    pub pos: ListState,
    pub width: u16, // as a percentage
}

impl Sidebar {
    pub fn new() -> Self {
        Sidebar {
            album_sort: AlbumSort::Artist,
            collapsed: HashSet::new(),
            rows: Vec::new(),
            pos: ListState::default().with_selected(Some(0)),
            width: 30,
        }
    }
}

impl UiState {
    pub(super) fn is_collapsed(&self, key: &NodeKey) -> bool {
        self.nav.sidebar.collapsed.contains(key)
    }

    pub fn get_selected_root(&self) -> Root {
        self.selected_row().map_or(Root::Library, |r| r.root())
    }

    pub fn selected_row(&self) -> Option<&SidebarRow> {
        self.nav.sidebar.rows.get(self.nav.sidebar.pos.selected()?)
    }

    pub(super) fn select_by_key(&mut self, key: &NodeKey) -> bool {
        match self.nav.sidebar.rows.iter().position(|r| r.key() == *key) {
            Some(i) => {
                self.nav.sidebar.pos.select(Some(i));
                true
            }
            None => false,
        }
    }

    pub(super) fn rebuild_rows(&mut self) {
        let keep = self.selected_row().map(|r| r.key());

        self.nav.sidebar.rows = self.project();

        if !keep.is_some_and(|k| self.select_by_key(&k)) {
            self.nav
                .sidebar
                .pos
                .select((!self.nav.sidebar.rows.is_empty()).then_some(0));
        }
    }

    fn project(&self) -> Vec<SidebarRow> {
        let mut rows = vec![SidebarRow::new(RowKind::Category(Root::Library), 0)];
        if !self.is_collapsed(&NodeKey::Root(Root::Library)) {
            rows.extend(match self.nav.sidebar.album_sort {
                AlbumSort::Artist => self.group_by_artist(),
                _ => self
                    .albums
                    .iter()
                    .map(|a| SidebarRow::new(RowKind::Album(a.id), 1))
                    .collect(),
            });
        }

        rows.push(SidebarRow::new(RowKind::Category(Root::Playlist), 0));
        if !self.is_collapsed(&NodeKey::Root(Root::Playlist)) {
            rows.extend(
                self.playlists
                    .values()
                    .map(|p| SidebarRow::new(RowKind::Playlist(p.id), 1)),
            );
        }
        rows
    }

    fn group_by_artist(&self) -> Vec<SidebarRow> {
        let mut buckets: IndexMap<Arc<String>, Vec<i64>> = IndexMap::new();
        for album in &self.albums {
            buckets
                .entry(Arc::clone(&album.artist))
                .or_default()
                .push(album.id);
        }
        buckets.sort_by(|a, _, b, _| a.to_lowercase().cmp(&b.to_lowercase()));

        let mut rows = Vec::with_capacity(self.albums.len() + buckets.len());
        for (name, mut children) in buckets {
            children.sort_by_key(|id| self.library.albums.get(id).and_then(|a| a.year));

            let collapsed = self.is_collapsed(&NodeKey::Artist(Arc::clone(&name)));
            let album_rows: Vec<SidebarRow> = match collapsed {
                true => Vec::new(),
                false => children
                    .iter()
                    .map(|&id| SidebarRow::new(RowKind::Album(id), 2))
                    .collect(),
            };

            rows.push(SidebarRow::new(RowKind::Artist { name, children }, 1));
            rows.extend(album_rows);
        }
        rows
    }

    pub(super) fn songs_for_row(&self, row: &SidebarRow) -> Vec<Arc<SimpleSong>> {
        match &row.kind {
            RowKind::Album(id) => self
                .library
                .albums
                .get(id)
                .map(|a| a.get_tracklist())
                .unwrap_or_default(),
            RowKind::Playlist(id) => self
                .playlists
                .get(id)
                .map(|p| p.get_tracklist())
                .unwrap_or_default(),
            RowKind::Artist { children, .. } => children
                .iter()
                .filter_map(|id| self.library.albums.get(id))
                .flat_map(|a| a.get_tracklist())
                .collect(),
            RowKind::Category(Root::Library) => {
                let mut songs = self.library.get_all_songs();
                let mut rng = StdRng::seed_from_u64(self.shuffle_seed);
                songs.shuffle(&mut rng);
                songs
            }
            RowKind::Category(Root::Playlist) => self
                .playlists
                .iter()
                .flat_map(|(_, p)| p.tracklist.iter().map(|s| Arc::clone(&s.song)))
                .collect(),
        }
    }

    fn parent_key_of(&self, row: &SidebarRow) -> Option<NodeKey> {
        match &row.kind {
            RowKind::Category(_) => None,
            RowKind::Artist { .. } => Some(NodeKey::Root(Root::Library)),
            RowKind::Playlist { .. } => Some(NodeKey::Root(Root::Playlist)),
            RowKind::Album(id) if row.depth == 2 => Some(NodeKey::Artist(Arc::clone(
                &self.library.albums.get(id)?.artist,
            ))),
            RowKind::Album(_) => Some(NodeKey::Root(Root::Library)),
        }
    }

    pub fn sidebar_toggle(&mut self) {
        let Some(row) = self.selected_row().cloned() else {
            return;
        };

        let Some(key) = row.collapse_key() else {
            self.set_pane(Pane::TrackList);
            return;
        };

        if !self.nav.sidebar.collapsed.remove(&key) {
            self.nav.sidebar.collapsed.insert(key);
        }

        self.rebuild_rows();
        self.set_legal_songs();
    }

    pub fn sidebar_collapse(&mut self) {
        let Some(row) = self.selected_row().cloned() else {
            return;
        };

        if let Some(key) = row.collapse_key() {
            if !self.is_collapsed(&key) {
                self.nav.sidebar.collapsed.insert(key);
                self.rebuild_rows();
                self.set_legal_songs();
                return;
            }
        }

        let Some(parent) = self.parent_key_of(&row) else {
            return;
        };

        self.nav.sidebar.collapsed.insert(parent.clone());
        self.rebuild_rows();
        self.select_by_key(&parent);
        self.set_legal_songs();
    }

    pub fn sidebar_expand(&mut self) {
        let Some(row) = self.selected_row().cloned() else {
            return;
        };

        match row.collapse_key() {
            Some(key) if self.is_collapsed(&key) => {
                self.nav.sidebar.collapsed.remove(&key);
                self.rebuild_rows();
                self.set_legal_songs();
            }
            _ => self.set_pane(Pane::TrackList),
        }
    }

    pub fn sidebar_expand_all(&mut self) {
        self.nav.sidebar.collapsed.clear();
        self.rebuild_rows();
        self.set_legal_songs();
    }

    pub fn focus_album(&mut self, album_id: i64) {
        if let Some(a) = self.library.albums.get(&album_id) {
            self.nav
                .sidebar
                .collapsed
                .remove(&NodeKey::Artist(Arc::clone(&a.artist)));
        }

        self.nav
            .sidebar
            .collapsed
            .remove(&NodeKey::Root(Root::Library));
        self.rebuild_rows();
        self.select_by_key(&NodeKey::Album(album_id));
    }

    pub fn adjust_sidebar_size(&mut self, delta: isize) {
        self.nav.sidebar.width = (self.nav.sidebar.width as isize + delta).clamp(8, 49) as u16;
    }
}
