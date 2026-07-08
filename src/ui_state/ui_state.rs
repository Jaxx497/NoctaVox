use super::{DisplayState, search_state::SearchState};
use crate::{
    Library, PlaybackSession,
    database::DbWorker,
    key_handler::{Incrementor, InputContext, KeyBuffer},
    library::{SimpleSong, SongInfo},
    theme::ThemeConfig,
    ui_state::{
        LayoutStyle, LibraryView, Mode, Pane, PlaylistAction, SettingsMode, ThemeManager, UiState,
        popup::{PopupState, PopupType},
        stats::VoxStats,
    },
    visualization::Visualizer,
};
use anyhow::anyhow;
use anyhow::{Error, Result};
use ratatui::widgets::Borders;
use std::sync::Arc;
use voxio::{TapHandle, Vox};

impl UiState {
    pub fn new(library: Arc<Library>, metrics: Arc<Vox>, tap: TapHandle) -> Self {
        UiState {
            library,
            db_worker: DbWorker::new()
                .expect("Could not establish connection to database for UiState!"),

            nav: DisplayState::new(),
            search: SearchState::new(),
            playback: PlaybackSession::init(),

            stats: VoxStats::default(),
            metrics: Arc::clone(&metrics),
            viz: Visualizer::new(metrics, tap),

            popup: PopupState::new(),
            layout: LayoutStyle::Traditional,
            theme: ThemeManager::new(),

            key_buffer: KeyBuffer::new(),

            albums: Vec::new(),
            playlists: Vec::new(),
            legal_songs: Vec::new(),

            library_refresh: None,
        }
    }
}

impl UiState {
    pub fn library(&self) -> &Arc<Library> {
        &self.library
    }

    pub fn sync_library(&mut self, library: Arc<Library>) -> Result<()> {
        self.library = library;

        self.sort_albums();
        match self.albums.is_empty() {
            true => self.nav.album_pos.select(None),
            false => {
                let album_len = self.albums.len();
                let current_selection = self.nav.album_pos.selected().unwrap_or(0);

                if current_selection > album_len {
                    self.nav.album_pos.select(Some(album_len - 1));
                } else if self.nav.album_pos.selected().is_none() {
                    self.nav.album_pos.select(Some(0));
                };
            }
        }

        self.get_playlists()?;
        self.set_legal_songs();

        Ok(())
    }

    pub fn set_error(&mut self, e: Error) {
        self.show_popup(PopupType::Error(e.to_string()));
    }

    pub fn soft_reset(&mut self) {
        if self.popup.is_open() {
            self.close_popup();
        }

        if self.get_mode() == Mode::Search {
            self.set_mode(Mode::Library(LibraryView::Albums));
        }

        self.clear_multi_select();
        self.search.input.clear();
        self.set_legal_songs();
    }

    pub fn get_error(&self) -> Option<&str> {
        match &self.popup.current {
            PopupType::Error(e) => Some(e.as_str()),
            _ => None,
        }
    }

    pub fn get_input_context(&self) -> InputContext {
        if self.popup.is_open() {
            return InputContext::Popup(self.popup.current.clone());
        }

        match (self.get_mode(), self.get_pane()) {
            (Mode::Fullscreen, _) => InputContext::Fullscreen,
            (Mode::Library(LibraryView::Albums), Pane::SideBar) => InputContext::AlbumView,
            (Mode::Library(LibraryView::Playlists), Pane::SideBar) => InputContext::PlaylistView,
            (Mode::Search, Pane::Search) => InputContext::Search,
            (mode, Pane::TrackList) => InputContext::TrackList(mode.clone()),
            (Mode::QUIT, _) => unreachable!(),
            _ => InputContext::TrackList(self.get_mode().clone()),
        }
    }

    pub fn is_text_input_active(&self) -> bool {
        matches!(
            (self.get_pane(), &self.popup.current),
            (Pane::Search, _)
                | (Pane::Popup, PopupType::Settings(SettingsMode::AddRoot))
                | (Pane::Popup, PopupType::Playlist(PlaylistAction::Create))
                | (
                    Pane::Popup,
                    PopupType::Playlist(PlaylistAction::CreateWithSongs)
                )
                | (Pane::Popup, PopupType::Playlist(PlaylistAction::Rename))
        )
    }
}

impl UiState {
    pub fn set_now_playing(&mut self, song: Option<Arc<SimpleSong>>) {
        match &song {
            Some(s) => self.db_worker.set_now_playing_db(s.get_id()),
            None => self.db_worker.clear_now_playing(),
        }
        self.playback.set_now_playing(song);
    }

    pub fn get_now_playing(&self) -> Option<&Arc<SimpleSong>> {
        self.playback.get_now_playing()
    }

    pub fn swap_layout(&mut self) {
        match self.layout {
            LayoutStyle::Traditional => self.layout = LayoutStyle::Minimal,
            LayoutStyle::Minimal => self.layout = LayoutStyle::Traditional,
        }
    }

    pub fn insert_history_entry(&mut self, song: &Arc<SimpleSong>) {
        self.db_worker.insert_song_to_history(song.id);
        self.playback.push_history(song);
    }

    pub fn delete_last_history_entry(&self) {
        self.db_worker.delete_history_latest();
    }

    pub fn restore_last_played(&self) -> Result<(u64, f32)> {
        self.db_worker.get_last_played()
    }

    pub fn update_now_playing_elapsed(&self) {
        let elapsed = self.metrics.position().as_secs_f32();
        self.db_worker.update_now_playing(elapsed);
    }

    pub fn borders_enabled(&self) -> bool {
        self.theme.active.border_display != Borders::NONE
    }

    pub fn set_theme(&mut self, theme: ThemeConfig) {
        self.theme.cached_focused = ThemeManager::set_display_theme(&theme, true);
        self.theme.cached_unfocused = ThemeManager::set_display_theme(&theme, false);
        self.viz.spectrum_mut().set_decay(theme.spectrum.decay);
        self.theme.active = theme;
    }

    pub fn refresh_current_theme(&mut self) {
        self.theme.update_themes();

        match self.theme.get_current_theme_index() {
            Some(idx) => {
                let theme = self.theme.get_theme_at_index(idx).unwrap_or_default();
                self.set_theme(theme);
            }
            _ => self.set_error(anyhow!(
                "Formatting error in theme!\n\nFalling back to last loaded"
            )),
        }
    }

    pub fn open_theme_manager(&mut self) {
        self.theme.update_themes();

        if let Some(idx) = self.theme.get_current_theme_index() {
            let theme = self.theme.get_theme_at_index(idx).unwrap_or_default();

            self.set_theme(theme);
            self.popup.selection.select(Some(idx));
        }

        self.show_popup(PopupType::ThemeManager);
    }

    pub fn cycle_theme(&mut self, dir: Incrementor) {
        let len = self.theme.theme_lib.len();
        if len < 2 {
            return;
        }

        let idx = self.theme.get_current_theme_index().unwrap_or(0);
        let new_idx = match dir {
            Incrementor::Up => (idx + len - 1) % len,
            Incrementor::Down => (idx + 1) % len,
        };

        self.set_theme(
            self.theme
                .theme_lib
                .get(new_idx)
                .cloned()
                .unwrap_or_default(),
        );
    }
}
