use crate::{
    SimpleSong,
    ui_state::{AlbumSort, LibraryView, NodeKey, Pane, RowKind, SidebarRow, UiState},
};
use indexmap::IndexMap;
use ratatui::widgets::ListState;
use std::{collections::HashSet, sync::Arc};

pub struct Sidebar {
    pub view: LibraryView,
    pub album_sort: AlbumSort,
    pub collapsed: HashSet<NodeKey>,
    pub rows: Vec<SidebarRow>,
    pub pos: ListState,
    pub width: u16, // as a percentage
}

impl Sidebar {
    pub fn new() -> Self {
        Sidebar {
            view: LibraryView::Albums,
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

    pub(super) fn selected_row(&self) -> Option<&SidebarRow> {
        self.nav.sidebar.rows.get(self.nav.sidebar.pos.selected()?)
    }

    pub fn get_selected_row(&self) -> Option<&SidebarRow> {
        self.selected_row()
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

        self.nav.sidebar.rows = match self.nav.sidebar.view {
            LibraryView::Omni => self.project_omni(),
            LibraryView::Albums => self.project_albums(),
            LibraryView::Playlists => self.project_playlists(),
        };

        if !keep.is_some_and(|k| self.select_by_key(&k)) {
            self.nav
                .sidebar
                .pos
                .select((!self.nav.sidebar.rows.is_empty()).then_some(0));
        }
    }

    fn project_omni(&self) -> Vec<SidebarRow> {
        let mut rows = vec![SidebarRow::new(RowKind::Category(NodeKey::MusicRoot), 0)];
        if !self.is_collapsed(&NodeKey::MusicRoot) {
            rows.extend(self.group_by_artist(1));
        }

        rows.push(SidebarRow::new(
            RowKind::Category(NodeKey::PlaylistsRoot),
            0,
        ));
        if !self.is_collapsed(&NodeKey::PlaylistsRoot) {
            rows.extend(
                self.playlists
                    .iter()
                    .map(|p| SidebarRow::new(RowKind::Playlist(p.id), 1)),
            );
        }
        rows
    }

    fn project_albums(&self) -> Vec<SidebarRow> {
        match self.nav.sidebar.album_sort {
            AlbumSort::Artist => self.group_by_artist(0),
            _ => self
                .albums
                .iter()
                .map(|a| SidebarRow::new(RowKind::Album(a.id), 0))
                .collect(),
        }
    }

    fn project_playlists(&self) -> Vec<SidebarRow> {
        self.playlists
            .iter()
            .map(|p| SidebarRow::new(RowKind::Playlist(p.id), 0))
            .collect()
    }

    fn group_by_artist(&self, base_depth: u8) -> Vec<SidebarRow> {
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
                    .map(|&id| SidebarRow::new(RowKind::Album(id), base_depth + 1))
                    .collect(),
            };

            rows.push(SidebarRow::new(
                RowKind::Artist { name, children },
                base_depth,
            ));
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
                .iter()
                .find(|p| p.id == *id)
                .map(|p| p.get_tracklist())
                .unwrap_or_default(),
            RowKind::Artist { children, .. } => children
                .iter()
                .filter_map(|id| self.library.albums.get(id))
                .flat_map(|a| a.get_tracklist())
                .collect(),
            RowKind::Category(_) => Vec::new(),
        }
    }

    fn parent_key_of(&self, row: &SidebarRow) -> Option<NodeKey> {
        match &row.kind {
            RowKind::Album(id) => Some(NodeKey::Artist(Arc::clone(
                &self.library.albums.get(id)?.artist,
            ))),
            RowKind::Artist { .. } if row.depth > 0 => Some(NodeKey::MusicRoot),
            RowKind::Playlist { .. } if row.depth > 0 => Some(NodeKey::PlaylistsRoot),
            _ => None,
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

        self.nav.sidebar.collapsed.remove(&NodeKey::MusicRoot);
        self.rebuild_rows();
        self.select_by_key(&NodeKey::Album(album_id));
    }
}
