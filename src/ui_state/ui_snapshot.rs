use super::{AlbumSort, Mode, Pane, UiState};
use crate::{
    ui_state::{LayoutStyle, NodeKey, PlayerSnapshot},
    visualization::ProgressDisplay,
};
use anyhow::Result;

#[derive(Default)]
pub struct UiSnapshot {
    pub mode: String,
    pub pane: String,
    pub album_sort: String,
    pub sidebar_percentage: u16,

    pub layout: String,
    pub theme_name: String,

    pub song_selection: Option<usize>,
    pub song_sel_offset: usize,

    pub sidebar_key: String,
    pub sidebar_offset: usize,
    pub sidebar_collapsed: String,

    pub progress_display: String,
    pub smoothing_factor: f32,
}

impl UiSnapshot {
    pub fn to_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("ui_mode", self.mode.clone()),
            ("ui_pane", self.pane.clone()),
            ("ui_album_sort", self.album_sort.clone()),
            ("ui_theme", self.theme_name.clone()),
            ("ui_layout", self.layout.clone()),
            ("ui_smooth", format!("{:.1}", self.smoothing_factor)),
            ("ui_sidebar_percent", self.sidebar_percentage.to_string()),
            ("ui_progress_display", self.progress_display.clone()),
            ("ui_sidebar_key", self.sidebar_key.clone()),
            ("ui_sidebar_offset", self.sidebar_offset.to_string()),
            ("ui_sidebar_collapsed", self.sidebar_collapsed.clone()),
        ];

        if let Some(pos) = self.song_selection {
            pairs.push(("ui_song_pos", pos.to_string()));
            pairs.push(("ui_song_offset", self.song_sel_offset.to_string()))
        }

        pairs
    }

    pub fn from_values(values: Vec<(String, String)>) -> Self {
        let mut snapshot = UiSnapshot::default();

        for (key, value) in values {
            match key.as_str() {
                "ui_mode" => snapshot.mode = value,
                "ui_pane" => snapshot.pane = value,
                "ui_progress_display" => snapshot.progress_display = value,
                "ui_theme" => snapshot.theme_name = value,
                "ui_layout" => snapshot.layout = value,
                "ui_album_sort" => snapshot.album_sort = value,
                "ui_sidebar_key" => snapshot.sidebar_key = value,
                "ui_sidebar_offset" => {
                    snapshot.sidebar_offset = value.parse::<usize>().unwrap_or(0)
                }
                "ui_sidebar_collapsed" => snapshot.sidebar_collapsed = value,
                "ui_song_pos" => snapshot.song_selection = value.parse().ok(),
                "ui_song_offset" => snapshot.song_sel_offset = value.parse::<usize>().unwrap_or(0),
                "ui_smooth" => snapshot.smoothing_factor = value.parse::<f32>().unwrap_or(1.0),
                "ui_sidebar_percent" => {
                    snapshot.sidebar_percentage = value.parse::<u16>().unwrap_or(30)
                }
                _ => {}
            }
        }

        snapshot
    }
}

impl UiState {
    pub fn create_ui_snapshot(&self) -> UiSnapshot {
        let orig_pane = self.get_pane();
        let pane = match orig_pane {
            Pane::Popup => &self.popup.cached,
            _ => orig_pane,
        };

        UiSnapshot {
            mode: self.get_mode().to_string(),
            pane: pane.to_string(),
            album_sort: self.nav.sidebar.album_sort.to_string(),
            sidebar_percentage: self.nav.sidebar.width,

            theme_name: self.theme.active.name.to_owned(),
            layout: self.layout.to_string(),

            song_selection: self.nav.table_pos.selected(),
            song_sel_offset: self.nav.table_pos.offset(),

            sidebar_key: self
                .selected_row()
                .map(|r| r.key().serialize())
                .unwrap_or_default(),

            sidebar_offset: self.nav.sidebar.pos.offset(),

            sidebar_collapsed: self
                .nav
                .sidebar
                .collapsed
                .iter()
                .map(|k| k.serialize())
                .collect::<Vec<_>>()
                .join("\x1f"),

            progress_display: self.viz.get_progress_display().to_string(),
            smoothing_factor: self.viz.get_smoothing_factor(),
        }
    }

    pub fn create_player_snapshot(&self) -> PlayerSnapshot {
        PlayerSnapshot {
            volume: self.metrics.volume(),
        }
    }

    pub fn save_state(&self) -> Result<()> {
        let mut snapshot = self.create_ui_snapshot().to_pairs();
        snapshot.extend(self.create_player_snapshot().to_pairs());
        self.db_worker.save_snapshot(snapshot)?;
        Ok(())
    }

    pub fn restore_last_state(&mut self) -> Result<()> {
        let player_snap = PlayerSnapshot::from_values(self.db_worker.load_snapshot("player_%")?);

        let vol = player_snap.volume;
        self.metrics.set_volume(vol);

        let ui_pairs = self.db_worker.load_snapshot("ui_%")?;
        if ui_pairs.is_empty() {
            return Ok(());
        }

        let ui_snapshot = UiSnapshot::from_values(ui_pairs);
        self.layout = LayoutStyle::from_str(&ui_snapshot.layout);

        if !ui_snapshot.theme_name.is_empty()
            && let Some(theme) = self.theme.find_theme_by_name(&ui_snapshot.theme_name)
        {
            self.set_theme(theme.clone());
        }

        self.nav.sidebar.collapsed = ui_snapshot
            .sidebar_collapsed
            .split('\x1f')
            .filter_map(NodeKey::deserialize)
            .collect();
        self.nav.sidebar.album_sort = AlbumSort::from_str(&ui_snapshot.album_sort);
        self.sort_albums();
        *self.nav.sidebar.pos.offset_mut() = ui_snapshot.sidebar_offset;

        let mode_to_restore = match ui_snapshot.mode.as_str() {
            "search" | "queue" => "library",
            _ => &ui_snapshot.mode,
        };

        let pane_to_restore = match ui_snapshot.pane.as_str() {
            "search" => "tracklist",
            _ => &ui_snapshot.pane,
        };

        self.set_mode(Mode::from_str(mode_to_restore));
        self.set_pane(Pane::from_str(pane_to_restore));

        if let Some(k) = NodeKey::deserialize(&ui_snapshot.sidebar_key) {
            self.select_by_key(&k);
        }

        self.set_legal_songs();

        self.viz.set_smoothing_factor(ui_snapshot.smoothing_factor);

        self.viz
            .set_progress_display(ProgressDisplay::from_str(&ui_snapshot.progress_display));

        self.nav.sidebar.width = ui_snapshot.sidebar_percentage;

        if let Some(pos) = ui_snapshot.song_selection
            && pos < self.legal_songs.len()
        {
            self.nav.table_pos.select(Some(pos));
            *self.nav.table_pos.offset_mut() = ui_snapshot.song_sel_offset
        }

        Ok(())
    }
}
