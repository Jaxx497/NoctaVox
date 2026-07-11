pub struct PlayerSnapshot {
    pub volume: f32,
}

impl Default for PlayerSnapshot {
    fn default() -> Self {
        PlayerSnapshot { volume: 1.0 }
    }
}

impl PlayerSnapshot {
    pub fn to_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("player_volume", format!("{:.3}", self.volume))]
    }

    pub fn from_values(values: Vec<(String, String)>) -> Self {
        let mut snapshot = Self::default();

        for (key, value) in values {
            if key == "player_volume" {
                snapshot.volume = value.parse().unwrap_or(1.0);
            }
        }
        snapshot
    }
}
