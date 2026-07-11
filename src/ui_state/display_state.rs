use super::{AlbumSort, LibraryView, Mode, Pane, TableSort, UiState};
use crate::{
    key_handler::{Director, Incrementor},
    library::{Album, Playlist, SimpleSong, SongInfo},
    ui_state::PopupType,
};
use anyhow::{Context, Result, anyhow, bail};
use indexmap::IndexSet;
use ratatui::widgets::{ListState, TableState};
use std::sync::Arc;

pub struct DisplayState {
    mode: Mode,
    mode_cached: Option<Mode>,
    pane: Pane,

    table_sort: TableSort,
    pub(super) album_sort: AlbumSort,

    pub sidebar_percent: u16,
    pub sidebar_view: LibraryView,
    pub album_pos: ListState,
    pub playlist_pos: ListState,

    pub table_pos: TableState,
    table_pos_cached: usize,

    pub multi_select: IndexSet<usize>,
}

impl DisplayState {
    pub fn new() -> Self {
        DisplayState {
            mode: Mode::Library(LibraryView::Albums),
            mode_cached: None,
            pane: Pane::TrackList,

            table_sort: TableSort::Title,
            album_sort: AlbumSort::Artist,

            sidebar_percent: 30,
            sidebar_view: LibraryView::Albums,
            album_pos: ListState::default().with_selected(Some(0)),
            playlist_pos: ListState::default().with_selected(Some(0)),

            table_pos: TableState::default().with_selected(0),
            table_pos_cached: 0,

            multi_select: IndexSet::default(),
        }
    }

    pub fn get_table_idx(&self) -> Result<usize> {
        self.table_pos
            .selected()
            .ok_or_else(|| anyhow!("No song selected"))
    }

    pub fn get_sidebar_view(&self) -> &LibraryView {
        &self.sidebar_view
    }

    pub fn get_album_sort(&self) -> &AlbumSort {
        &self.album_sort
    }

    pub fn get_table_sort(&self) -> &TableSort {
        &self.table_sort
    }
}

impl UiState {
    pub fn get_pane(&self) -> &Pane {
        &self.nav.pane
    }

    pub fn set_pane(&mut self, pane: Pane) {
        self.nav.pane = pane;
    }

    pub fn get_mode(&self) -> &Mode {
        &self.nav.mode
    }

    pub fn get_sidebar_details(&self) -> (LibraryView, usize) {
        let sidebar_type = self.nav.get_sidebar_view();
        let count = match sidebar_type {
            LibraryView::Albums => self.albums.len(),
            LibraryView::Playlists => self.playlists.len(),
        };

        (*sidebar_type, count)
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.clear_multi_select();
        match self.nav.mode {
            Mode::Power => {
                self.nav.table_pos_cached = self
                    .nav
                    .table_pos
                    .selected()
                    .unwrap_or(self.nav.table_pos_cached)
            }
            _ => (),
        }

        match mode {
            Mode::Power => {
                self.nav.pane = Pane::TrackList;
                self.nav.mode = Mode::Power;
                self.nav.table_sort = TableSort::Title;
                self.set_legal_songs();
                self.nav.table_pos.select(Some(self.nav.table_pos_cached));
            }

            Mode::Library(view) => {
                self.nav.sidebar_view = view;
                self.nav.mode = Mode::Library(view);
                self.nav.pane = Pane::SideBar;

                // Ensure we have a valid selection for the view we're entering
                match view {
                    LibraryView::Albums => {
                        if self.albums.is_empty() {
                            self.nav.album_pos.select(None);
                        } else if self.nav.album_pos.selected().is_none() {
                            self.nav.album_pos.select(Some(0));
                        }
                    }
                    LibraryView::Playlists => {
                        if self.playlists.is_empty() {
                            self.nav.playlist_pos.select(None);
                        } else if self.nav.playlist_pos.selected().is_none() {
                            self.nav.playlist_pos.select(Some(0));
                        }
                    }
                }

                *self.nav.table_pos.offset_mut() = 0;
                self.set_legal_songs();
            }
            Mode::Fullscreen => {
                if self.metrics.is_active() {
                    self.nav.mode_cached = Some(self.nav.mode.to_owned());
                    self.nav.mode = Mode::Fullscreen
                }
            }
            Mode::Queue => {
                if !self.playback.queue_is_empty() {
                    *self.nav.table_pos.offset_mut() = 0;
                    self.nav.mode = Mode::Queue;
                    self.nav.pane = Pane::TrackList;
                    self.set_legal_songs()
                } else {
                    self.set_error(anyhow!("Queue is empty!"));
                }
            }
            Mode::Search => {
                self.nav.table_sort = TableSort::Title;
                self.search.input.clear();
                self.nav.mode = Mode::Search;
                self.nav.pane = Pane::Search;
            }
            Mode::QUIT => {
                let _ = self.save_state();
                self.nav.mode = Mode::QUIT;
            }
        }
    }

    pub fn get_selected_song(&mut self) -> Result<Arc<SimpleSong>> {
        if self.legal_songs.is_empty() {
            self.nav.table_pos.select(None);
            bail!("No songs to select!");
        }

        match self.nav.mode {
            Mode::Power | Mode::Library(_) | Mode::Search | Mode::Queue => {
                let idx = self.nav.get_table_idx()?;
                Ok(Arc::clone(&self.legal_songs[idx]))
            }
            _ => Err(anyhow!("Should not be reachable")),
        }
    }

    pub fn get_selected_album(&self) -> Option<&Album> {
        self.nav
            .album_pos
            .selected()
            .and_then(|idx| self.albums.get(idx))
    }

    pub fn get_selected_playlist(&self) -> Option<&Playlist> {
        self.nav
            .playlist_pos
            .selected()
            .and_then(|idx| self.playlists.get(idx))
    }

    pub fn toggle_album_sort(&mut self, next: bool) {
        self.nav.album_sort = match next {
            true => self.nav.album_sort.next(),
            false => self.nav.album_sort.prev(),
        };
        self.sort_albums();
        self.set_legal_songs();
    }

    pub(super) fn sort_albums(&mut self) {
        self.albums = self
            .library
            .albums
            .values()
            .cloned()
            .collect::<Vec<Album>>();

        match self.nav.album_sort {
            AlbumSort::Artist => self.albums.sort_by(|a, b| {
                a.artist
                    .to_lowercase()
                    .cmp(&b.artist.to_lowercase())
                    .then(a.year.cmp(&b.year))
            }),
            AlbumSort::Title => self
                .albums
                .sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase())),
            AlbumSort::Year => self.albums.sort_by(|a, b| a.year.cmp(&b.year)),
        }
    }

    pub(crate) fn next_song_column(&mut self) {
        if self.search.len() < 1 {
            self.nav.table_sort = self.nav.table_sort.next();
            self.set_legal_songs();
        }
    }

    pub(crate) fn prev_song_column(&mut self) {
        if self.search.len() < 1 {
            self.nav.table_sort = self.nav.table_sort.prev();
            self.set_legal_songs();
        }
    }

    fn sort_by_table_column(&mut self) {
        match self.nav.table_sort {
            TableSort::Title => {
                self.legal_songs.sort_by(|a, b| a.title.cmp(&b.title));
            }

            TableSort::Artist => self.legal_songs.sort_by(|a, b| {
                let artist_a = a.get_artist().to_lowercase();
                let artist_b = b.get_artist().to_lowercase();
                artist_a.cmp(&artist_b)
            }),
            TableSort::Album => self.legal_songs.sort_by(|a, b| {
                let album_a = a.get_album().to_lowercase();
                let album_b = b.get_album().to_lowercase();

                album_a.cmp(&album_b)
            }),
            TableSort::Duration => self.legal_songs.sort_by_key(|s| s.get_duration()),
        };
    }

    pub(crate) fn go_to_now_playing(&mut self) -> Result<()> {
        if let Some(np) = &self.get_now_playing() {
            let np = Arc::clone(&np);
            let album_id = np.album_id;

            let album_idx = self.albums.iter().position(|a| a.id == album_id);

            self.nav.album_pos.select(album_idx);
            self.set_mode(Mode::Library(LibraryView::Albums));
            self.set_pane(Pane::TrackList);
            self.set_legal_songs();

            let idx = self.legal_songs.iter().position(|s| s.id == np.id);

            self.nav.table_pos.select(idx);
        }

        Ok(())
    }

    pub(crate) fn go_to_track(&mut self, count: usize) -> Result<()> {
        let range = self.legal_songs.len();
        if (count > range) || (count < 1) {
            bail!("OUT OF RANGE")
        }

        self.nav.table_pos.select(Some(count - 1));

        Ok(())
    }

    pub(crate) fn go_to_album(&mut self) -> Result<()> {
        if let Ok(this_song) = self.get_selected_song() {
            let album_id = this_song.album_id;

            self.set_mode(Mode::Library(LibraryView::Albums));
            self.set_pane(Pane::TrackList);

            let album = self
                .library
                .albums
                .get(&album_id)
                .context("Invalid album index")?;

            let track_pos = album
                .tracklist
                .iter()
                .position(|s| s.id == this_song.id)
                .unwrap_or(0);

            let album_pos = self
                .albums
                .iter()
                .position(|a| a.id == album_id)
                .ok_or_else(|| anyhow!("Could not identify album!"))?;

            self.legal_songs = album.get_tracklist();

            self.nav.album_pos.select(Some(album_pos));
            self.nav.table_pos.select(Some(track_pos));
            *self.nav.table_pos.offset_mut() = 0;
        } else {
            self.set_mode(Mode::Library(LibraryView::Albums));
            self.set_pane(Pane::SideBar);
        }

        Ok(())
    }

    pub fn get_legal_songs(&self) -> &[Arc<SimpleSong>] {
        &self.legal_songs.as_slice()
    }

    pub(crate) fn set_legal_songs(&mut self) {
        match &self.nav.mode {
            Mode::Power => {
                self.legal_songs = self.library.get_all_songs().to_vec();
                self.sort_by_table_column();
            }
            Mode::Library(view) => match view {
                LibraryView::Albums => {
                    if let Some(idx) = self.nav.album_pos.selected() {
                        if let Some(album) = self.albums.get(idx) {
                            self.legal_songs = album.get_tracklist();
                        }
                    }
                }
                LibraryView::Playlists => {
                    if let Some(idx) = self.nav.playlist_pos.selected() {
                        if let Some(playlist) = self.playlists.get(idx) {
                            self.legal_songs = playlist.get_tracklist()
                        }
                    } else {
                        self.legal_songs.clear()
                    }
                }
            },
            Mode::Queue => self.legal_songs = self.playback.get_queue(),

            Mode::Search => match self.search.len() > 1 {
                true => self.filter_songs_by_search(),
                false => self.sort_by_table_column(),
            },
            _ => (),
        }

        // Autoselect first entry if table_pos selection is none
        if !self.legal_songs.is_empty() && self.nav.table_pos.selected().is_none() {
            self.nav.table_pos.select(Some(0));
        }
    }

    pub fn revert_fullscreen(&mut self) {
        if matches!(self.get_mode(), Mode::Fullscreen) {
            if let Some(mode) = &self.nav.mode_cached {
                self.set_mode(mode.to_owned());
                self.nav.mode_cached = None;
            }
        }
    }
}

impl UiState {
    pub fn scroll(&mut self, director: Director) {
        match self.nav.pane {
            Pane::SideBar => self.scroll_sidebar(&director),
            Pane::TrackList => match director {
                Director::Top => self.scroll_to_top(),
                Director::Bottom => self.scroll_to_bottom(),
                _ => self.scroll_tracklist(&director),
            },
            _ => (),
        }
    }

    fn scroll_tracklist(&mut self, director: &Director) {
        if !self.legal_songs.is_empty() {
            let len = self.legal_songs.len();
            let selected_idx = self.nav.table_pos.selected();

            let new_pos = match director {
                Director::Up(x) => selected_idx
                    .map(|idx| (idx + len - (x % len)) % len)
                    .unwrap_or(0),
                Director::Down(x) => selected_idx.map(|idx| (idx + x) % len).unwrap_or(0),
                _ => unreachable!(),
            };
            self.nav.table_pos.select(Some(new_pos));
        }
    }

    fn scroll_sidebar(&mut self, director: &Director) {
        let (items_len, state) = match self.nav.sidebar_view {
            LibraryView::Albums => (self.albums.len(), &mut self.nav.album_pos),
            LibraryView::Playlists => (self.playlists.len(), &mut self.nav.playlist_pos),
        };

        if items_len == 0 {
            return;
        }

        let current = state.selected().unwrap_or(0);
        let new_pos = match director {
            Director::Up(x) => (current + items_len - x) % items_len,
            Director::Down(x) => (current + x) % items_len,
            Director::Top => 0,
            Director::Bottom => items_len - 1,
        };

        state.select(Some(new_pos));
        *self.nav.table_pos.offset_mut() = 0;
        self.set_legal_songs();
    }

    fn scroll_to_top(&mut self) {
        match &self.nav.pane {
            Pane::TrackList => self.nav.table_pos.select_first(),
            _ => (),
        }
    }

    fn scroll_to_bottom(&mut self) {
        match self.nav.pane {
            Pane::TrackList => self.nav.table_pos.select_last(),
            _ => (),
        }
    }

    pub(crate) fn popup_scroll(&mut self, i: Incrementor) {
        match i {
            Incrementor::Up => self.popup_scroll_up(),
            Incrementor::Down => self.popup_scroll_down(),
        }
    }

    fn popup_scroll_up(&mut self) {
        let popup_type = &self.popup.current;

        // The keymap guide scrolls by a raw line offset (no selection cursor,
        // no wrap); the widget clamps the top end at render.
        if matches!(popup_type, PopupType::KeymapGuide) {
            let current = self.popup.selection.selected().unwrap_or(0);
            self.popup.selection.select(Some(current.saturating_sub(1)));
            return;
        }

        let list_len = match popup_type {
            PopupType::Settings(_) => self.get_roots().len(),
            PopupType::Playlist(_) => self.playlists.len(),
            PopupType::ThemeManager => self.theme.theme_lib.len(),
            _ => return,
        };

        if list_len > 0 {
            let current = self.popup.selection.selected().unwrap_or(0);
            let new_selection = match current > 0 {
                true => current - 1,
                false => list_len - 1, // Wrap to bottom
            };
            self.popup.selection.select(Some(new_selection));

            if matches!(popup_type, PopupType::ThemeManager) {
                self.switch_theme();
            }
        }
    }

    fn popup_scroll_down(&mut self) {
        let popup_type = &self.popup.current;

        // The keymap guide scrolls by a raw line offset (no selection cursor,
        // no wrap); the widget clamps the bottom end at render.
        if matches!(popup_type, PopupType::KeymapGuide) {
            let current = self.popup.selection.selected().unwrap_or(0);
            self.popup.selection.select(Some(current + 1));
            return;
        }

        let list_len = match popup_type {
            PopupType::Settings(_) => self.get_roots().len(),
            PopupType::Playlist(_) => self.playlists.len(),
            PopupType::ThemeManager => self.theme.theme_lib.len(),
            _ => return,
        };

        if list_len > 0 {
            let current = self.popup.selection.selected().unwrap_or(0);
            let new_selection = (current + 1) % list_len; // Wrap to top
            self.popup.selection.select(Some(new_selection));
        }

        if matches!(popup_type, PopupType::ThemeManager) {
            self.switch_theme();
        }
    }

    fn switch_theme(&mut self) {
        if let Some(idx) = self.popup.selection.selected() {
            if let Some(theme) = self.theme.theme_lib.get(idx) {
                self.set_theme(theme.clone());
            }
        }
    }

    pub fn adjust_sidebar_size(&mut self, x: isize) {
        match x > 0 {
            true => {
                if self.nav.sidebar_percent < 49 {
                    self.nav.sidebar_percent += x as u16;
                }
            }
            false => {
                if self.nav.sidebar_percent >= 9 {
                    self.nav.sidebar_percent -= -x as u16;
                }
            }
        }
    }
}
